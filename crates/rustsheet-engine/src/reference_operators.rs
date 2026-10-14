//! Evaluation of the reference operators: the range (`:`), the intersection (a space) and
//! the union (a comma inside parentheses).

use std::{ops::Deref, sync::Arc};

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{util::is_at_if, Function},
    model::Model,
};

/// Most areas a union or an intersection may stand for. Unions nest in balanced trees
/// (`((A1,A2),(A3,A4))`) whose depth grows only with the logarithm of their leaves, so the
/// parser's depth cap does not bound them, and intersecting unions multiplies their areas.
/// Every area list the evaluator builds is checked against this cap before it grows, so a
/// formula cannot allocate without bound (CLAUDE.md directive 5; DEVIATION, decision 0014:
/// docs/excel-semantics/references.md "Union area cap").
pub(crate) const MAX_AREAS: usize = 1 << 16;

/// Message of the `#VALUE!` for a union or intersection of more than [`MAX_AREAS`] areas.
pub(crate) const TOO_MANY_AREAS: &str = "Too many areas in a reference";

/// The areas [`Model::evaluate_areas`] finds: one area, held without an allocation (the
/// common `LET(r,A1:C3,r B2)`), a list built for this expression, or a list shared with the
/// binding or name it came from, read without a copy.
enum AreaList {
    One(Range),
    Owned(Vec<Range>),
    Shared(Arc<[Range]>),
}

impl Deref for AreaList {
    type Target = [Range];

    fn deref(&self) -> &[Range] {
        match self {
            AreaList::One(range) => std::slice::from_ref(range),
            AreaList::Owned(areas) => areas,
            AreaList::Shared(areas) => areas,
        }
    }
}

impl AreaList {
    fn into_vec(self) -> Vec<Range> {
        match self {
            AreaList::One(range) => vec![range],
            AreaList::Owned(areas) => areas,
            AreaList::Shared(areas) => areas.to_vec(),
        }
    }
}

/// The areas of a reference that has more than one, in formula order, duplicates kept
/// (`SUM((A1:B2,B2))` counts B2 twice, probes/multiarea-agg H3): 2..=[`MAX_AREAS`] ranges, all
/// on one sheet, each with its corners in order. Shared (`Arc`, so a `Model` holding one in a LET
/// binding stays `Send`), so a copy is never made on read.
#[derive(Clone)]
pub(crate) struct Areas(Arc<[Range]>);

impl Areas {
    /// How many areas the reference has: 2..=[`MAX_AREAS`].
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

/// The ranges a reference argument stands for: one range (a 3-D reference included, whose
/// corners sit on two sheets), or the areas of a multi-area reference. Derefs to the ranges in
/// formula order; a single range is borrowed, never copied.
#[derive(Clone)]
pub(crate) enum Ranges {
    One(Range),
    Areas(Areas),
}

impl Ranges {
    /// The ranges of an area list built by [`Model::evaluate_areas`]: one area is a plain
    /// range; several must sit on one sheet, as a reference across sheets is `#VALUE!` where a
    /// function takes a union (`SUM((Data!A1,Other!A1))`, probes/multiarea-xsheet).
    fn from_list(areas: AreaList, cell: CellReferenceIndex) -> Result<Ranges, CalcResult> {
        let mut areas = match areas {
            AreaList::One(area) => return Ok(Ranges::One(area)),
            // The areas of an `Areas`, which hold to its rules already.
            AreaList::Shared(areas) => return Ok(Ranges::Areas(Areas(areas))),
            AreaList::Owned(areas) => areas,
        };
        if areas.len() > MAX_AREAS {
            return Err(too_many_areas(cell));
        }
        let Some(first) = areas.first() else {
            return Err(empty_intersection(cell));
        };
        let sheet = first.left.sheet;
        if areas
            .iter()
            .any(|area| area.left.sheet != sheet || area.right.sheet != sheet)
        {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Reference of areas on several sheets".to_string(),
            ));
        }
        if areas.len() == 1 {
            if let Some(area) = areas.pop() {
                return Ok(Ranges::One(area));
            }
        }
        Ok(Ranges::Areas(Areas(Arc::from(areas))))
    }
}

impl Deref for Ranges {
    type Target = [Range];

    fn deref(&self) -> &[Range] {
        match self {
            Ranges::One(range) => std::slice::from_ref(range),
            Ranges::Areas(Areas(areas)) => areas,
        }
    }
}

/// What a reference-accepting argument evaluates to ([`Model::evaluate_reference_arg`]). Kept
/// apart from [`CalcResult`] so that a reference of several areas reaches only the code that
/// asks for one; everywhere a value is needed it stays `#VALUE!` (SPEC-0004).
pub(crate) enum RefArg {
    /// A reference: one range or several areas.
    Ranges(Ranges),
    /// Anything else: a value, an array or an error. Never a `CalcResult::Range`.
    Value(CalcResult),
}

/// What an array stands for where a function takes a reference (SUBTOTAL, AREAS, COUNTBLANK):
/// `#VALUE!` per element, an element that is an error kept (`SUBTOTAL(9,IF(TRUE,{1,#N/A}))` is
/// `#VALUE!`, `#N/A`; probes/subtotal-reference-operands, T-1163; probes/countblank-array-values).
pub(crate) fn not_a_reference_array(rows: Vec<Vec<ArrayNode>>) -> CalcResult {
    CalcResult::Array(
        rows.into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| match value {
                        ArrayNode::Error(error) => ArrayNode::Error(error),
                        _ => ArrayNode::Error(Error::VALUE),
                    })
                    .collect()
            })
            .collect(),
    )
}

impl From<CalcResult> for RefArg {
    fn from(result: CalcResult) -> RefArg {
        match result {
            CalcResult::Range { left, right } => RefArg::Ranges(Ranges::One(Range { left, right })),
            other => RefArg::Value(other),
        }
    }
}

impl RefArg {
    /// The areas of a reference that sits on one sheet: borrowed, never copied. `None` for a
    /// 3-D reference (corners on two sheets) and for anything that is not a reference, where
    /// AREAS, INDEX and ISREF keep their `#VALUE!` (probes/3d-ref-functions).
    pub(crate) fn single_sheet_areas(&self) -> Option<&[Range]> {
        match self {
            RefArg::Ranges(ranges)
                if ranges
                    .iter()
                    .all(|area| area.left.sheet == area.right.sheet) =>
            {
                Some(ranges)
            }
            _ => None,
        }
    }

    /// [`RefArg::into_single_area`] where the area must sit on one sheet: a 3-D range (a span
    /// bound by LET, from IF or CHOOSE) is `#VALUE!`, as the written span is there
    /// (`ROWS(x)`, `COUNTBLANK(x)` for `x` = `Data1:Data3!A1`: probes/3d-ref-let-bound-span C7,
    /// 3d-ref-let-bound-span-readers C11, 3d-ref-let-values C5; T-1158). Not every caller of
    /// `into_single_area` may refuse one: making that refuse 3-D ranges regressed probe cells.
    pub(crate) fn into_single_sheet_area(self, cell: CellReferenceIndex) -> CalcResult {
        match self {
            RefArg::Ranges(Ranges::One(range)) if range.left.sheet != range.right.sheet => {
                three_d_needs_one_sheet(cell)
            }
            other => other.into_single_area(cell),
        }
    }

    /// The argument where only one area is taken: a reference of several areas is `#VALUE!`,
    /// as in a value context, and one range is a `CalcResult::Range`.
    pub(crate) fn into_single_area(self, cell: CellReferenceIndex) -> CalcResult {
        match self {
            RefArg::Ranges(Ranges::One(Range { left, right })) => CalcResult::Range { left, right },
            RefArg::Ranges(Ranges::Areas(_)) => several_areas(cell),
            RefArg::Value(value) => value,
        }
    }
}

/// `#VALUE!` for a 3-D reference where a single-sheet reference is needed (INDEX, AREAS,
/// VLOOKUP, MATCH, ..., probes/3d-ref-functions).
pub(crate) fn three_d_needs_one_sheet(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "3-D reference where one sheet is needed".to_string(),
    )
}

/// The smallest range holding every area of a union (all on one sheet, corners in order).
fn bounding_area(areas: &[Range]) -> CalcResult {
    let mut bounds = areas.iter();
    // Callers pass at least one area (a union has two or more); the empty arm is never taken.
    let Some(first) = bounds.next() else {
        return CalcResult::EmptyCell;
    };
    let (mut left, mut right) = (first.left, first.right);
    for area in bounds {
        left.row = left.row.min(area.left.row);
        left.column = left.column.min(area.left.column);
        right.row = right.row.max(area.right.row);
        right.column = right.column.max(area.right.column);
    }
    CalcResult::Range { left, right }
}

/// `#VALUE!` for a reference of several areas where one area is needed.
pub(crate) fn several_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "Reference of several areas".to_string())
}

/// `#REF!` for a reference of several areas in the contexts where Excel gives `#REF!` rather than
/// the `#VALUE!` of [`several_areas`] (ROW/COLUMN/ROWS/COLUMNS, a VLOOKUP/HLOOKUP index).
pub(crate) fn several_areas_ref(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "Reference of several areas".to_string())
}

pub(crate) fn too_many_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREAS.to_string())
}

/// The cells `a` and `b` share: `None` if none, `#VALUE!` if they are on different sheets.
fn intersect_pair(
    a: &Range,
    b: &Range,
    cell: CellReferenceIndex,
) -> Result<Option<Range>, CalcResult> {
    if a.left.sheet != b.left.sheet {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "Intersection of references on different sheets".to_string(),
        ));
    }
    let top = a.left.row.max(b.left.row);
    let bottom = a.right.row.min(b.right.row);
    let first = a.left.column.max(b.left.column);
    let last = a.right.column.min(b.right.column);
    if top > bottom || first > last {
        return Ok(None);
    }
    let sheet = a.left.sheet;
    Ok(Some(Range {
        left: CellReferenceIndex {
            sheet,
            row: top,
            column: first,
        },
        right: CellReferenceIndex {
            sheet,
            row: bottom,
            column: last,
        },
    }))
}

pub(crate) fn empty_intersection(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NULL, cell, "Empty intersection".to_string())
}

/// Whether a sheet prefix names a sheet of another workbook: `[book]Sheet` (a sheet name cannot
/// hold `[`). That workbook is never opened (directive 5).
pub(crate) fn names_other_workbook(sheet_name: Option<&str>) -> bool {
    sheet_name.is_some_and(|name| name.contains('['))
}

impl Model {
    /// The error of a spill range operator whose anchor is not a single reference. The `@` of a
    /// value (`=ANCHORARRAY(@1)`) is `#VALUE!` in Excel (probes/colon-hash-value-operands D3,
    /// T-1226).
    pub(crate) fn spill_anchor_error(child: &Node, cell: CellReferenceIndex) -> CalcResult {
        let error = if matches!(child.ungrouped(), Node::ImplicitIntersection { .. }) {
            Error::VALUE
        } else {
            Error::ERROR
        };
        CalcResult::new_error(
            error,
            cell,
            format!("Error with Spill Range Operator in cell {cell:?}"),
        )
    }

    /// The error of a spill range operator whose anchor is a multi-cell range: `#REF!`, plain or
    /// picked by an IF (`ANCHORARRAY(D1:D3)`, `ANCHORARRAY(IF(TRUE,A1:B2))`,
    /// probes/anchorarray-if-operands G1, P1, S1, T1; T-1369).
    pub(crate) fn spill_anchor_range_error(cell: CellReferenceIndex) -> CalcResult {
        CalcResult::new_error(
            Error::REF,
            cell,
            format!("Error with Spill Range Operator in cell {cell:?}"),
        )
    }

    /// The error of a reference on a sheet that does not exist: `#REF!`, but `#NAME?` when the prefix
    /// is a 3-D span `First:Last` (a `:` cannot occur in a sheet name) whose end is not a sheet:
    /// `=SUM(Data1:Nope!A1)` and `=SUM(Nope:Data3!A1)` (probes/3d-ref-span-edges A1, A2; T-1170).
    /// A prefix with a `[` (which a sheet name cannot hold) names another workbook, which is never
    /// opened (directive 5): `#BLOCKED!`, as Excel gives one it holds no cached data for
    /// (docs/excel-semantics/references.md "External reference with no cached data"; T-0124).
    pub(crate) fn wrong_sheet_error(
        sheet_name: Option<&str>,
        cell: CellReferenceIndex,
        message: &str,
    ) -> CalcResult {
        let error = if names_other_workbook(sheet_name) {
            Error::BLOCKED
        } else if sheet_name.is_some_and(|name| name.contains(':')) {
            Error::NAME
        } else {
            Error::REF
        };
        CalcResult::new_error(error, cell, message.to_string())
    }

    /// `@XLOOKUP(..)` / `@TAKE(..)`: the one cell they pick stays a reference, `None` for any
    /// other node. Only ISREF and LET binding use it (probes/isref-implicit-xlookup-take H1, H3,
    /// H5, H6, excel-graph; T-1509): the spill-anchor and LOOKUP callers of `eval_ref_or_index`
    /// have no evidence for these forms and keep their value reading.
    pub(crate) fn eval_at_picked_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let Node::ImplicitIntersection { child, .. } = node.ungrouped() else {
            return None;
        };
        let Node::FunctionKind {
            kind: kind @ (Function::Xlookup | Function::Take),
            args,
        } = child.ungrouped()
        else {
            return None;
        };
        let picked = self.evaluate_reference_form(kind, args, cell)?;
        Some(Self::intersect_reference(picked, cell))
    }

    /// A reference-context operand: a reference, or an INDEX that selects one cell, which stays
    /// a reference: the anchor of `INDEX(D1:D2,1)#` (probes/at-operand-formulatext K30, T-1109),
    /// a lookup index argument and LOOKUP's `result_vector` (probes/lookup-vector-edges A9, A12).
    pub(crate) fn eval_ref_or_index(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let node = node.ungrouped();
        match node {
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } => self.index_impl(
                args,
                cell,
                crate::functions::lookup_and_reference::IndexForm::Reference,
            ),
            // `ANCHORARRAY(@INDEX(D1:D2,1))` spills D1#: the `@` of a one-cell reference keeps
            // the reference (probes/anchorarray-at-operand-kinds H81, T-1247). The operand is
            // evaluated once and intersected, so nested `@INDEX` stays linear (T-1321).
            Node::ImplicitIntersection { child, .. }
                if matches!(
                    child.ungrouped(),
                    Node::FunctionKind {
                        kind: Function::Index,
                        ..
                    }
                ) =>
            {
                let picked = self.eval_ref_or_index(child, cell);
                Self::intersect_reference(picked, cell)
            }
            _ => self.evaluate_node_with_reference(node, cell),
        }
    }

    /// The operand of `#` and ANCHORARRAY: [`Model::eval_ref_or_index`], and the `@` of an IF
    /// that picks one cell keeps the reference, so `ANCHORARRAY(@IF(TRUE,D1))` spills D1#
    /// (probes/at-signed-operand H71:H73, T-1321). Also LOOKUP's `result_vector`, for `@IF`
    /// only (probes/at-if-spill-anchor-operand H50, T-1368); the other lookup callers of
    /// `eval_ref_or_index` have no evidence for `@IF`.
    pub(crate) fn spill_anchor_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let node = node.ungrouped();
        match node {
            Node::ImplicitIntersection { child, .. } if is_at_if(node) => {
                let picked = self
                    .evaluate_reference_arg(child, cell)
                    .into_single_area(cell);
                Self::intersect_reference(picked, cell)
            }
            // A plain IF is the reference it picks: `IF(TRUE,D1)#` spills D1#
            // (probes/written-parens-reference-context E20:E22, T-1363). Evaluated once, so
            // nested IFs stay linear.
            Node::FunctionKind {
                kind: Function::If, ..
            } => self
                .evaluate_reference_arg(node, cell)
                .into_single_area(cell),
            _ => self.eval_ref_or_index(node, cell),
        }
    }

    /// An operand of `:`. An INDEX operand is the cell it selects, not its value
    /// (probes/text-result-limit-errors-refs C2, D2).
    pub(crate) fn range_operand(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        let node = node.ungrouped();
        match node {
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } => self.index_impl(
                args,
                cell,
                crate::functions::lookup_and_reference::IndexForm::Reference,
            ),
            // An IF corner is the reference it picks, on either side: `A1:IF(TRUE,B2)` is A1:B2
            // and `IF(TRUE,B2):A1` too; a picked value is `#VALUE!` (probes/isref-range-operands
            // D3, D7, D10, D11, E3, E7, E10-E13; T-1177). A picked union is its bounding box
            // (`SUM(A1:IF(TRUE,(B3,B1)))` = A1:B3, probes/isref-if-range-corners D1, D2, D15-D18;
            // T-1227).
            Node::FunctionKind {
                kind: Function::If, ..
            } => match self.evaluate_reference_arg(node, cell) {
                RefArg::Ranges(ranges @ Ranges::Areas(_)) => bounding_area(&ranges),
                other => other.into_single_area(cell),
            },
            // A name whose body is a formula, parenthesized or not, joins as what the body is
            // in reference context (`ig:B3`, probes/defined-name-grouped-reference A10, T-1173).
            Node::DefinedNameKind(..) if self.defined_name_formula(node).is_some() => {
                self.defined_name_range_operand(node, cell)
            }
            _ => self.evaluate_node_with_reference(node, cell),
        }
    }

    /// `left:right`, the range operator between two operands that are not a plain `A1:B2`
    /// token (the lexer builds those): two references on one sheet are joined into their
    /// bounding box, so `Data!A1:Other!B2` is `#VALUE!` (UNVERIFIED, references.md) and a `Range` across sheets
    /// is only ever a 3-D reference.
    pub(crate) fn get_range(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let left_result = self.range_operand(left, cell);
        let right_result = self.range_operand(right, cell);
        match (left_result, right_result) {
            (
                CalcResult::Range {
                    left: left1,
                    right: right1,
                },
                CalcResult::Range {
                    left: left2,
                    right: right2,
                },
            ) if left1.sheet == left2.sheet
                && left1.sheet == right1.sheet
                && left2.sheet == right2.sheet =>
            {
                // The result is the bounding box of both operands, whichever order the corners
                // are written in (`Data!B2:Data!A1` is `Data!A1:B2`). Multi-cell operands join
                // too: with `rg` = `(Sheet1!$B$1:$B$2)`, `rg:B3` is B1:B3 and `ig:B3` (ig =
                // `(INDEX(Sheet1!$B$1:$B$2,2))`) is B2:B3 (probes/defined-name-grouped-reference
                // A10, A11, T-1173). Corner order: see references.md "Sheet-qualified range corners".
                bounding_area(&[
                    Range {
                        left: left1,
                        right: right1,
                    },
                    Range {
                        left: left2,
                        right: right2,
                    },
                ])
            }
            // An error operand's own error comes before the `#VALUE!`: `=(@(A1 B1)):C3` is
            // `#NULL!` (probes/at-operand-formulatext H12, T-1109). Excel passes the right
            // operand's error on too (`=A1:INDEX(A1:A3,5)` is `#REF!`, `=A1:(A1 B2)` is `#NULL!`;
            // probes/range-right-operand-errors D11, D14, T-1115).
            (error @ CalcResult::Error { .. }, _) | (_, error @ CalcResult::Error { .. }) => error,
            _ => CalcResult::new_error(Error::VALUE, cell, "Invalid range".to_string()),
        }
    }

    /// A 3-D reference (`First:Last!A1`) where a value is needed: Excel gives `#REF!` for the
    /// bare reference, in an operator and in value functions such as ABS
    /// (probes/3d-ref-values C1:C3). A span of one sheet is that sheet's reference
    /// (`=Data1:Data1!A1`, C9).
    pub(crate) fn get_sheet_span_value(
        &mut self,
        first_index: u32,
        last_index: u32,
        reference: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if first_index == last_index {
            return self.evaluate_node_in_context(reference, cell);
        }
        CalcResult::new_error(
            Error::REF,
            cell,
            "3-D reference where a value is needed".to_string(),
        )
    }

    /// A 3-D reference where a single-sheet reference is needed: `#VALUE!` (INDEX, ROWS, ROW,
    /// AREAS, VLOOKUP, MATCH, SUMIF, COUNTIF, probes/3d-ref-functions). A span of one sheet is
    /// that sheet's reference.
    pub(crate) fn get_sheet_span_reference(
        &mut self,
        first_index: u32,
        last_index: u32,
        reference: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if first_index == last_index {
            return self.evaluate_node_with_reference(reference, cell);
        }
        three_d_needs_one_sheet(cell)
    }

    /// A 3-D reference as an aggregate argument: one `Range` whose corners sit on the first
    /// and the last sheet of the span in tab order, whichever way it was written
    /// (`Data3:Data1!A1` is `Data1:Data3!A1`, probes/3d-ref-aggregates C2). Only
    /// [`Model::evaluate_reference_arg`] builds a `Range` across sheets, and
    /// [`Model::for_each_cell_or_error_in_areas`] walks it sheet by sheet.
    pub(crate) fn get_sheet_span_range(
        &mut self,
        first_index: u32,
        last_index: u32,
        reference: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self.evaluate_node_with_reference(reference, cell) {
            CalcResult::Range { left, right } => CalcResult::Range {
                left: CellReferenceIndex {
                    sheet: first_index.min(last_index),
                    ..left
                },
                right: CellReferenceIndex {
                    sheet: first_index.max(last_index),
                    ..right
                },
            },
            other => other,
        }
    }

    /// Whether `node` may stand for several areas, so evaluating it as an intersection
    /// operand or reference argument needs an area list.
    /// An intersection is multi-area only if a union sits somewhere under it: a chain of
    /// plain references (`A1:C3 B1:D4 C1:C9`) stays on the allocation-free pair path. A name
    /// is multi-area when its formula body is: a name for a cell, a range, an array or a
    /// scalar formula keeps the path it had (`box A1`, `FREQUENCY(arr,2)`). Recursion is
    /// bounded by the parser's depth cap.
    pub(crate) fn is_multi_area(&self, node: &Node) -> bool {
        self.may_be_multi_area(node, true)
    }

    /// [`Model::is_multi_area`]; `look_through_names` is false inside a name's body, where a
    /// name with a formula body counts as multi-area without being opened, so names that
    /// refer to each other cannot recurse here (they are bounded when evaluated, by
    /// [`Model::within_defined_name`]).
    fn may_be_multi_area(&self, node: &Node, look_through_names: bool) -> bool {
        let node = node.ungrouped();
        match node {
            // A LET variable, IF, CHOOSE and LET may stand for a union (probes/multiarea-let
            // H2, multiarea-ifchoose H1, multiarea-letbind H13:H14 and H16).
            Node::OpUnionKind { .. }
            | Node::NamedVariableKind { .. }
            | Node::FunctionKind {
                kind: Function::If | Function::Choose | Function::Let,
                ..
            } => true,
            // So may a name whose body does (probes/union-defined-name H4).
            Node::DefinedNameKind(_) => self
                .defined_name_formula(node)
                .is_some_and(|body| !look_through_names || self.may_be_multi_area(&body, false)),
            Node::OpIntersectKind { left, right, .. } => {
                self.may_be_multi_area(left, look_through_names)
                    || self.may_be_multi_area(right, look_through_names)
            }
            _ => false,
        }
    }

    /// `left right`, the intersection operator: the reference to the cells both operands
    /// contain. Excel (docs/excel-semantics/references.md, gen-m1/4dc8ed1de05a61be): an
    /// operand's error propagates, areas that share no cell give `#NULL!`, and areas on two
    /// different sheets give `#VALUE!` even where their coordinates overlap. An operand may be
    /// a union: a result of one area is that area (`(Data!A1:A12,Data!C1:C12) Data!A7` is
    /// Data!A7, gen-m1/c940d3cfb0d3b083), and one of several areas is `#VALUE!`, even when
    /// they are the same cell (`=A1:A3 (A2:B2,A2)`,
    /// probes/intersection-function-operands H5).
    pub(crate) fn get_intersection(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // Into another workbook, the one area's cells are read at once (`ERROR.TYPE(…!A1
        // …!A1:B2)` is `#N/A`, probes/external-ref-no-link-index A36).
        if let Some(areas) = self.external_intersection(left, right, cell) {
            return match areas.as_deref() {
                Ok([area]) => area.picked_value(cell),
                Ok(_) => several_areas(cell),
                Err(error) => error.clone(),
            };
        }
        match self.intersect_to_one_area(left, right, cell) {
            Ok(area) => CalcResult::Range {
                left: area.left,
                right: area.right,
            },
            Err(error) => error,
        }
    }

    /// The one area `left right` stands for, or the result to give instead.
    fn intersect_to_one_area(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        if !self.is_multi_area(left) && !self.is_multi_area(right) {
            // Two single areas, the common case: no area list.
            let a = self.evaluate_intersection_operand(left, cell)?;
            let b = self.evaluate_intersection_operand(right, cell)?;
            return intersect_pair(&a, &b, cell)?.ok_or_else(|| empty_intersection(cell));
        }
        match self.intersect_areas(left, right, cell)? {
            AreaList::One(area) => Ok(area),
            areas => match &*areas {
                [area] => Ok(area.clone()),
                _ => Err(several_areas(cell)),
            },
        }
    }

    /// `(left,right)`, the union operator, where a value is needed: `#VALUE!`, even for
    /// `(E4,E4)` (gen-m1/c940d3cfb0d3b083). A function that takes a union reads it through
    /// [`Model::evaluate_reference_arg`] instead. An operand's own error comes first
    /// (UNVERIFIED, references.md). The operands are checked in the order
    /// [`Model::union_areas`] evaluates them, with the same errors, but only their area counts
    /// are kept.
    pub(crate) fn get_union(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self.count_union_areas(left, right, cell) {
            Ok(_) => several_areas(cell),
            Err(error) => error,
        }
    }

    /// The ranges of a reference expression that may stand for several areas (a union, or an
    /// intersection with one under it), or the result to give instead: an operand's error, in
    /// formula order, or `#VALUE!` (see [`Ranges::from_list`]). One area left is a plain range
    /// (`SUM((A1:A3,B1:B3) A1:A2)`); two are both walked (probes/multiarea-intersect H1).
    pub(crate) fn reference_ranges(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Ranges, CalcResult> {
        let areas = self.evaluate_areas(node, cell)?;
        Ranges::from_list(areas, cell)
    }

    /// The areas of a reference argument: several for a union, one for any other reference.
    /// `Err` holds the error the argument evaluates to, or `#VALUE!` if it is not a reference
    /// (AGGREGATE's reference form; probes/aggregate-args).
    pub(crate) fn reference_areas(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        self.evaluate_areas(node, cell).map(AreaList::into_vec)
    }

    /// The number of areas `(left,right)` stands for, or the result to give instead, exactly
    /// as [`Model::union_areas`] finds them but without building the union's area list.
    fn count_union_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<usize, CalcResult> {
        let mut count = 0;
        for node in [left, right] {
            let node = node.ungrouped();
            count += match node {
                Node::OpUnionKind { left, right, .. } => {
                    self.count_union_areas(left, right, cell)?
                }
                _ => self.evaluate_union_operand(node, cell)?.len(),
            };
            if count > MAX_AREAS {
                return Err(too_many_areas(cell));
            }
        }
        Ok(count)
    }

    /// The areas a reference expression stands for: several for a union, and for an
    /// intersection with one; one for any other reference. Never more than [`MAX_AREAS`].
    /// `Err` holds the result to give instead (see [`Model::evaluate_intersection_operand`]).
    fn evaluate_areas(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<AreaList, CalcResult> {
        let node = node.ungrouped();
        match node {
            Node::OpUnionKind { left, right, .. } => {
                self.union_areas(left, right, cell).map(AreaList::Owned)
            }
            Node::OpIntersectKind { left, right, .. } => self.intersect_areas(left, right, cell),
            Node::NamedVariableKind { .. }
            | Node::DefinedNameKind(_)
            | Node::FunctionKind {
                kind: Function::If | Function::Choose | Function::Let,
                ..
            } => match self.evaluate_reference_arg(node, cell) {
                RefArg::Ranges(Ranges::Areas(Areas(areas))) => Ok(AreaList::Shared(areas)),
                other => Ok(AreaList::One(single_sheet_area(
                    other.into_single_area(cell),
                    cell,
                )?)),
            },
            _ => Ok(AreaList::One(
                self.evaluate_intersection_operand(node, cell)?,
            )),
        }
    }

    /// The areas of `left` followed by those of `right`. More than [`MAX_AREAS`] is `#VALUE!`,
    /// checked before the list grows (DEVIATION, decision 0014: references.md "Union area
    /// cap").
    fn union_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        let left = self.evaluate_union_operand(left, cell)?;
        let right = self.evaluate_union_operand(right, cell)?;
        if left.len() + right.len() > MAX_AREAS {
            return Err(too_many_areas(cell));
        }
        let mut areas = left.into_vec();
        areas.extend_from_slice(&right);
        Ok(areas)
    }

    /// An operand of a union. A 3-D reference cannot be one: Excel answers `#NAME?`, not the
    /// `#VALUE!` of other non-reference operands (`SUM((Sheet1:Sheet2!A1,B2))`,
    /// probes/multiarea-3d-ref).
    fn evaluate_union_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<AreaList, CalcResult> {
        if let Node::SheetSpanKind {
            first_index,
            last_index,
            ..
        } = node
        {
            if first_index != last_index {
                return Err(CalcResult::new_error(
                    Error::NAME,
                    cell,
                    "3-D reference inside a union".to_string(),
                ));
            }
        }
        self.evaluate_areas(node, cell)
    }

    /// The pairwise intersections of the areas of `left` and `right` that hold a cell. A pair
    /// on different sheets is `#VALUE!`, and no shared cell at all is `#NULL!`. Unions of
    /// unions multiply the areas, so more than [`MAX_AREAS`] pairs is `#VALUE!`, and so is
    /// going over what is left of the formula's budget of
    /// [`MAX_AREA_PAIRS`](crate::area_budget::MAX_AREA_PAIRS) (DEVIATION, decision 0014:
    /// references.md "Union area cap"). Two single areas give one, with no list.
    fn intersect_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<AreaList, CalcResult> {
        let left = self.evaluate_areas(left, cell)?;
        let right = self.evaluate_areas(right, cell)?;
        let pairs = left.len().saturating_mul(right.len());
        if pairs > MAX_AREAS {
            return Err(too_many_areas(cell));
        }
        self.spend_area_pairs(pairs, cell)?;
        if let ([a], [b]) = (&*left, &*right) {
            return intersect_pair(a, b, cell)?
                .map(AreaList::One)
                .ok_or_else(|| empty_intersection(cell));
        }
        let mut areas = Vec::new();
        for a in left.iter() {
            for b in right.iter() {
                areas.extend(intersect_pair(a, b, cell)?);
            }
        }
        if areas.is_empty() {
            return Err(empty_intersection(cell));
        }
        Ok(AreaList::Owned(areas))
    }

    /// An operand of the intersection as a single-sheet area with its corners in order, or
    /// the result to give instead: the operand's own error, or `#VALUE!` for anything that
    /// is not a reference (`(1+0) A1`) or spans sheets.
    fn evaluate_intersection_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        let node = node.ungrouped();
        let reference_form = match node {
            Node::FunctionKind { kind, args } => self.evaluate_reference_form(kind, args, cell),
            _ => None,
        };
        let operand =
            reference_form.unwrap_or_else(|| self.evaluate_node_with_reference(node, cell));
        single_sheet_area(operand, cell)
    }
}

/// An intersection operand as a single-sheet area with its corners in order, or the result to
/// give instead: the operand's own error, or `#VALUE!` for anything that is not a reference
/// (`(1+0) A1`) or spans sheets.
fn single_sheet_area(operand: CalcResult, cell: CellReferenceIndex) -> Result<Range, CalcResult> {
    match operand {
        CalcResult::Range { left, right } if left.sheet == right.sheet => Ok(Range {
            left: CellReferenceIndex {
                sheet: left.sheet,
                row: left.row.min(right.row),
                column: left.column.min(right.column),
            },
            right: CellReferenceIndex {
                sheet: left.sheet,
                row: left.row.max(right.row),
                column: left.column.max(right.column),
            },
        }),
        error @ CalcResult::Error { .. } => Err(error),
        _ => Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "Intersection of a value that is not a reference".to_string(),
        )),
    }
}
