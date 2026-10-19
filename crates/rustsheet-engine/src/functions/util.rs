use crate::external_reference::PickedReference;
use crate::{
    area_budget::SLOTS_PER_WALK_STEP,
    calc_result::{CalcResult, Range},
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    external_links::CachedBlanks,
    functions::{logical::IfBranchMode, Function},
    model::Model,
    number_format::cmp_excel_precision,
    reference_operators::{Picked, RefArg},
};

pub(crate) mod aggregate_args;
pub(crate) mod collation;

use collation::cmp_text;

/// The number a referenced cell gives an aggregate: numbers count, booleans, text and empty cells
/// are skipped (SPEC-0009 `CellNumbers::Numbers`).
pub(crate) fn cell_number(value: &CalcResult) -> Option<f64> {
    match value {
        CalcResult::Number(value) => Some(*value),
        _ => None,
    }
}

/// The number a referenced cell gives the A functions (VARA, AVERAGEA, MAXA, ...): text counts
/// as 0, TRUE/FALSE as 1/0, empty cells are skipped (SPEC-0009 `CellNumbers::CoercedA`).
pub(crate) fn coerced_cell_number(value: &CalcResult) -> Option<f64> {
    match value {
        CalcResult::Number(value) => Some(*value),
        CalcResult::String(_) => Some(0.0),
        CalcResult::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Converts a single array element into a scalar [`CalcResult`].
pub(crate) fn array_node_to_calc_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(n) => CalcResult::Number(*n),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::new_error(e.clone(), cell, String::new()),
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

/// The top left element of an array, which is what the implicit intersection
/// operator makes of it.
pub(crate) fn top_left_of_array(array: &[Vec<ArrayNode>], cell: CellReferenceIndex) -> CalcResult {
    match array.first().and_then(|row| row.first()) {
        Some(node) => array_node_to_calc_result(node, cell),
        None => CalcResult::new_error(Error::CALC, cell, "Empty array".to_string()),
    }
}

fn error_sort_rank(e: &Error) -> u8 {
    match e {
        Error::NULL => 1,
        Error::DIV => 2,
        Error::VALUE => 3,
        Error::REF => 4,
        Error::NAME => 5,
        Error::NUM => 6,
        Error::NA => 7,
        _ => 8,
    }
}

// In Excel there are two ways of comparing cell values.
// The old school comparison valid in formulas like D3 < D4 or HLOOKUP,... cast empty cells into empty strings or 0
// For the new formulas like XLOOKUP or SORT an empty cell is always larger than anything else.

// ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE;
pub(crate) fn compare_values(left: &CalcResult, right: &CalcResult) -> i32 {
    // An empty value (`IFERROR(<blank>,1)`, `CalcResult::EmptyArg`) compares as a blank cell:
    // `IFERROR(B1,1)=0`, `=""` and `=FALSE` are TRUE and SWITCH matches it to 0
    // (probes/iferror-blank-value D2:D5, D18, D19; T-0291).
    fn as_blank(value: &CalcResult) -> &CalcResult {
        static BLANK: CalcResult = CalcResult::EmptyCell;
        if matches!(value, CalcResult::EmptyArg) {
            &BLANK
        } else {
            value
        }
    }
    let (left, right) = (as_blank(left), as_blank(right));
    match (left, right) {
        (CalcResult::Number(value1), CalcResult::Number(value2)) => {
            // Equality is decided on the 15-digit values alone: an absolute tolerance would equate
            // every tiny value with 0, but Excel says `EXP(-708)=0` is FALSE (docs/excel-semantics/numbers.md,
            // probes/subnormal-function-results C4-C6).
            match cmp_excel_precision(*value1, *value2) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (CalcResult::Number(_value1), CalcResult::String(_value2)) => -1,
        (CalcResult::Number(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::String(value1), CalcResult::String(value2)) => {
            // Case-insensitive, accent- and hyphen-aware (T-0220).
            match cmp_text(value1, value2) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (CalcResult::String(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => {
            if value1 == value2 {
                return 0;
            }
            if *value1 {
                return 1;
            }
            -1
        }
        (CalcResult::EmptyCell, CalcResult::String(_value2)) => {
            compare_values(&CalcResult::String("".into()), right)
        }
        (CalcResult::String(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::String("".into()))
        }
        (CalcResult::EmptyCell, CalcResult::Number(_value2)) => {
            compare_values(&CalcResult::Number(0.0), right)
        }
        (CalcResult::Number(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Number(0.0))
        }
        (CalcResult::EmptyCell, CalcResult::Boolean(_)) => {
            compare_values(&CalcResult::Boolean(false), right)
        }
        (CalcResult::Boolean(_), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Boolean(false))
        }
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => 0,
        // Errors sort after everything else, ordered by Excel's canonical rank
        (CalcResult::Error { error: e1, .. }, CalcResult::Error { error: e2, .. }) => {
            let r1 = error_sort_rank(e1);
            let r2 = error_sort_rank(e2);
            if r1 < r2 {
                -1
            } else if r1 > r2 {
                1
            } else {
                0
            }
        }
        (CalcResult::Error { .. }, _) => 1,
        (_, CalcResult::Error { .. }) => -1,
        (_, _) => 1,
    }
}

/// Whether `node` is `@IF(...)`: the `@` of an IF keeps the reference the IF picks.
pub(crate) fn is_at_if(node: &Node) -> bool {
    matches!(
        node,
        Node::ImplicitIntersection { child, .. }
            if matches!(child.ungrouped(), Node::FunctionKind { kind: Function::If, .. })
    )
}

/// Whether `node` is `@XLOOKUP(...)` or `@TAKE(...)`: the `@` keeps the reference they pick.
pub(crate) fn is_at_xlookup_take(node: &Node) -> bool {
    matches!(
        node,
        Node::ImplicitIntersection { child, .. }
            if matches!(
                child.ungrouped(),
                Node::FunctionKind { kind: Function::Xlookup | Function::Take, .. }
            )
    )
}

/// Whether `node` is a written multi-sheet 3-D span (`Sheet1:Sheet2!A1`, a one-cell one included).
pub(crate) fn is_multi_sheet_span(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::SheetSpanKind { first_index, last_index, .. } if first_index != last_index
    )
}

/// Whether `node` is a multi-sheet 3-D span, possibly as a branch of IF (which hands its branch on
/// as a reference). Recursion is bounded by the parser's depth cap.
pub(crate) fn is_span_or_if_of_span(node: &Node) -> bool {
    let node = node.ungrouped();
    match node {
        Node::FunctionKind {
            kind: Function::If,
            args,
        } => args.iter().skip(1).any(is_span_or_if_of_span),
        _ => is_multi_sheet_span(node),
    }
}

/// Most columns holding cells that a union area may have to be walked through the column index
/// ([`Model::for_each_indexed_position`]); a wider one is walked by rows. The merge compares the
/// next row of every column still being merged for each row it hands on, and a row is charged at
/// least a step per four of them, so with this few columns the comparisons stay a small part of
/// a row's cost.
const MAX_INDEXED_COLUMNS: usize = 16;

/// Fewest rows of the sheet a union area must span to be walked through the column index
/// ([`Model::for_each_existing_position`]): one chunk of row storage.
const MIN_INDEXED_ROWS: i32 = 1024;

/// Rows of one column an indexed walk reads from the index at a time: one search, then a step per
/// row, so a dense column costs a read step per this many cells.
const INDEX_BATCH_ROWS: usize = 64;

/// Which cells of a sheet a range walk hands on ([`Model::walk_area`]). The two dense walks
/// differ only for an explicit block reaching past the used area: `ClipBlocks` stops at the
/// used area ([`Model::clip_block_to_used_area`]), `ClipWholeLines` walks the block whole and
/// clips only whole rows and columns ([`Model::clip_to_used_area`]).
#[derive(Clone, Copy)]
enum AreaWalk {
    /// Every position of the range within the sheet's used area: the cells past it are blank.
    ClipBlocks,
    /// Every position of the range, but a whole-row or whole-column range only within the
    /// used area, for walkers that read a blank cell as something.
    ClipWholeLines,
    /// The cells that exist only, for the areas of a union.
    Existing,
}

/// The rows of one column of an indexed walk that were read from the index, from `next` on not
/// yet handed on.
struct ColumnBatch {
    column: i32,
    rows: Vec<i32>,
    next: usize,
}

impl ColumnBatch {
    fn head(&self) -> Option<i32> {
        self.rows.get(self.next).copied()
    }

    /// Whether the index may hold rows past the ones read: the read was full and is used up.
    fn needs_read(&self) -> bool {
        self.next == self.rows.len() && self.rows.len() == INDEX_BATCH_ROWS
    }
}

impl Model {
    /// Evaluates an argument that must be a single-sheet reference (INDEX, AREAS, VLOOKUP,
    /// MATCH, COUNTBLANK): a 3-D reference is `#VALUE!` there, not the `#REF!` it is as a
    /// value (probes/3d-ref-functions; docs/excel-semantics/references.md). A one-cell reference,
    /// written or picked by IF or INDEX, stays a one-cell range (T-1862).
    pub(crate) fn evaluate_single_sheet_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let node = node.ungrouped();
        match node {
            Node::SheetSpanKind {
                first_index,
                last_index,
                reference,
                ..
            } if first_index != last_index => {
                self.get_sheet_span_reference(*first_index, *last_index, reference, cell)
            }
            // IF hands its chosen branch on as a reference: a 3-D span inside it is
            // `#VALUE!` here, as it is inside CHOOSE (T-1026).
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => self
                .if_impl(args, cell, IfBranchMode::SingleSheet)
                .into_single_area(cell),
            // One cell is a one-cell range, searched as one: `MATCH("b",A2,0)` is 1 and
            // `VLOOKUP(2,B2,1,FALSE)` 2 where B2 is 2, also picked by IF or INDEX
            // (probes/match-single-value-lookup-array U2, V2, W2, Z2, AA2; T-1862).
            Node::ReferenceKind { .. } => self.evaluate_node_with_reference(node, cell),
            Node::FunctionKind {
                kind: Function::Index,
                ..
            } => self.eval_ref_or_index(node, cell),
            _ => self.evaluate_node_in_context(node, cell),
        }
    }

    /// An argument that reads one cell of a reference (MATCH's `match_type`, IFS and SWITCH's chosen
    /// value): IF, IFS, SWITCH, LET and CHOOSE hand back the cell they pick, however deeply they
    /// nest (probes/match-type-text-passthrough K1:K9, match-type-nested-passthrough K1:K17;
    /// T-1399, T-1452). Aggregates keep [`Self::evaluate_reference_arg`], where IFS and SWITCH are
    /// values (`SUM(IFS(TRUE,A1:B2))`, docs/excel-semantics/logic.md).
    pub(crate) fn evaluate_chosen_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node.ungrouped() {
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => self
                .if_impl(args, cell, IfBranchMode::Chosen)
                .into_single_area(cell),
            Node::FunctionKind {
                kind: Function::Ifs,
                args,
            } => self.ifs_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Switch,
                args,
            } => self.switch_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Let,
                args,
            } => self.let_impl(args, cell, true).into_single_area(cell),
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } => self.choose_impl(args, cell, true).into_single_area(cell),
            other => self.eval_ref_or_index(other, cell),
        }
    }

    /// Evaluates one argument of an aggregate (SUM, COUNT, AVERAGE, MIN, ...).
    /// A cell reference, and an IF or INDEX that selects one, comes back as a
    /// range, so the caller skips its text, booleans and blanks like any range
    /// cell; everything else is a value (T-0216; docs/excel-semantics/references.md).
    /// A union, and an intersection that leaves several areas, is its areas, walked one after
    /// the other with each one counted as written (probes/multiarea-agg, multiarea-intersect;
    /// SPEC-0004). A function that takes only one area turns those into `#VALUE!` with
    /// [`RefArg::into_single_area`].
    pub(crate) fn evaluate_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.reference_arg_reading(node, CachedBlanks::Kept, cell)
    }

    /// [`Self::evaluate_reference_arg`] with a cached area of a linked workbook read as `blanks`
    /// says ([`Model::external_aggregate_value`]).
    fn reference_arg_reading(
        &mut self,
        node: &Node,
        blanks: CachedBlanks,
        cell: CellReferenceIndex,
    ) -> RefArg {
        // An IF or CHOOSE that may pick a reference into another workbook is the argument it
        // picks (`SUM(IF(TRUE,…!A1))` is `#BLOCKED!`; probes/external-ref-no-link-ifchoose A3).
        // An aggregate reading cells of another workbook reads what its externalLink cache
        // holds, as reference cells (fixtures/external-link-cached-ranges; T-1460), and is
        // `#BLOCKED!` when there is no cache, however the reference was reached
        // (`SUM(INDEX(…!A1:C4,2,2))`, `SUM((…!A1,…!B2))`, `SUMPRODUCT(…!A1:B2)`;
        // probes/external-ref-no-link-index A7, A11, A15, A22, A27). COUNTA, COUNTBLANK and the
        // reference functions read [`Model::pick_reference`] first.
        let node = match self.pick_reference(node, cell) {
            PickedReference::Node(node) => node,
            PickedReference::Value(value) => return RefArg::Value(value),
            PickedReference::External(areas) => {
                return RefArg::Value(match areas {
                    Ok(areas) => self.external_aggregate_value(&areas, blanks, cell),
                    Err(error) => error,
                })
            }
        };
        let result = match node {
            Node::OpUnionKind { .. } | Node::OpIntersectKind { .. } if self.is_multi_area(node) => {
                return match self.reference_ranges(node, cell) {
                    Ok(ranges) => RefArg::Ranges(ranges),
                    Err(error) => RefArg::Value(error),
                };
            }
            // `@A1:A3` is the intersected cell's reference, so text there is ignored
            // (probes/min-max-reference-forms C1-C4).
            Node::ReferenceKind { .. }
            | Node::ImplicitIntersection { .. }
            | Node::OpRangeKind { .. } => self.evaluate_node_with_reference(node, cell),
            // A name whose body is a formula, a union included, stands for what the body does
            // in reference context: `SUM(mu)` with `mu` = `Sheet1!$A$1,Sheet1!$B$2` is 6
            // (probes/union-defined-name H1).
            Node::DefinedNameKind(_) => return self.defined_name_reference(node, cell),
            Node::SheetSpanKind {
                first_index,
                last_index,
                reference,
                ..
            } => self.get_sheet_span_range(*first_index, *last_index, reference, cell),
            // IF and CHOOSE hand back the chosen branch, LET its body, in reference context,
            // a union included (probes/multiarea-ifchoose H2:H3, multiarea-letbind H8).
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => return self.if_impl(args, cell, IfBranchMode::Aggregate),
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } => return self.choose_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Let,
                args,
            } => return self.let_impl(args, cell, true),
            // A LET variable bound to a union is its areas (probes/multiarea-let H1).
            Node::NamedVariableKind { .. } => return self.variable_reference(node, cell),
            Node::LambdaCallKind { lambda, args } => {
                return self.call_callee_as_reference(lambda, args, cell)
            }
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } => self.index_reference(args, cell),
            Node::FunctionKind {
                kind: Function::Hlookup,
                args,
            } => self.hlookup_impl(args, cell, true),
            Node::FunctionKind {
                kind: Function::Vlookup,
                args,
            } => self.vlookup_impl(args, cell, true),
            _ => self.evaluate_node_in_context(node, cell),
        };
        RefArg::from(result)
    }

    /// [`Self::evaluate_reference_arg`] for the arguments SUBTOTAL, AREAS, SUM, COUNT, AVERAGE, MIN, MAX
    /// and AVERAGEA take (and a LET or LAMBDA binding, T-1232): XLOOKUP, TAKE, DROP and TRIMRANGE over a
    /// range, also as the arm a scalar IF, CHOOSE, IFS or SWITCH picks or a LET body (T-2009,
    /// T-2014), are the cells
    /// they pick (probes/subtotal-reference-operands-b H12, H15:H17, T-1175; -c H4:H14, T-1206;
    /// -d H1:H27, L1:L7, T-1211). The aggregates sharing `for_each_value` follow the
    /// same rule. The call list is [`Function::picks_reference`].
    pub(crate) fn evaluate_picked_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.picked_reference_arg_reading(node, CachedBlanks::Kept, cell)
    }

    /// [`Self::evaluate_picked_reference_arg`] with a cached area of a linked workbook read as
    /// `blanks` says.
    fn picked_reference_arg_reading(
        &mut self,
        node: &Node,
        blanks: CachedBlanks,
        cell: CellReferenceIndex,
    ) -> RefArg {
        // The arm a scalar IF, CHOOSE, IFS or SWITCH picks and a LET body follow the same rule,
        // however deeply they nest: `SUBTOTAL(9,IF(TRUE,XLOOKUP(1,J1:J2,H1:I2)))` is 9 and
        // `AREAS(LET(v,1,XLOOKUP(..)))` 1 (probes/subtotal-if-choose-over-xlookup A10-A42,
        // T-2009); `SUBTOTAL(9,IFS(TRUE,XLOOKUP(..)))` is 9 and `SUBTOTAL(9,SWITCH(1,1,(H1,I2)))`
        // 11 (probes/subtotal-ifs-switch-reference-picks, T-2014). Each condition, index, case
        // and binding is read once, as `reference_arg_reading` reads them.
        if let Node::FunctionKind { kind, args } = node.ungrouped() {
            let picked = match kind {
                Function::If => self.if_pick(args, cell),
                Function::Choose => self.choose_pick(args, cell),
                Function::Ifs => self.ifs_pick(args, cell),
                Function::Switch => self.switch_pick(args, cell),
                Function::Let => {
                    return self
                        .let_scope(args, cell, |model, body| {
                            model.picked_reference_arg_reading(body, blanks, cell)
                        })
                        .unwrap_or_else(RefArg::Value);
                }
                _ if kind.picks_reference() => {
                    if let Some(result) = self.evaluate_reference_form(kind, args, cell) {
                        return RefArg::from(result);
                    }
                    return self.reference_arg_reading(node, blanks, cell);
                }
                _ => return self.reference_arg_reading(node, blanks, cell),
            };
            return match picked {
                Picked::Node(arm) => self.picked_reference_arg_reading(arm, blanks, cell),
                Picked::Value(value) => RefArg::Value(value),
            };
        }
        self.reference_arg_reading(node, blanks, cell)
    }

    /// [`Self::evaluate_picked_reference_arg`] for an aggregate that skips the blanks of a range
    /// (SUM, COUNT, AVERAGE, AVERAGEA, MIN, MAX and the like): a cached area of a linked
    /// workbook reads only the cells its cache spans ([`CachedBlanks::Skipped`]), so
    /// `SUM([1]Uses!$A$2:$B$1048576)` reads the cache instead of tripping the array bound
    /// (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"; T-1550).
    pub(crate) fn evaluate_aggregate_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.picked_reference_arg_reading(node, CachedBlanks::Skipped, cell)
    }

    /// The bottom-right corner of the range `row1:column1..=row2:column2` once a whole-column
    /// or whole-row range is clipped to the sheet's used area; any other range is unchanged.
    /// Reads nothing and records nothing (see [`Self::clip_to_used_area`]).
    pub(crate) fn used_area_corner(
        &self,
        sheet: u32,
        row1: i32,
        column1: i32,
        row2: i32,
        column2: i32,
    ) -> Result<(i32, i32), String> {
        let whole_rows = row1 == 1 && row2 == LAST_ROW;
        let whole_columns = column1 == 1 && column2 == LAST_COLUMN;
        if !whole_rows && !whole_columns {
            return Ok((row2, column2));
        }
        let (max_row, max_column) = self.used_area_end_of(sheet)?;
        let last_row = if whole_rows { max_row } else { row2 };
        let last_column = if whole_columns { max_column } else { column2 };
        Ok((last_row, last_column))
    }

    /// Calls `f` with the value of every cell of `areas`, area by area in formula
    /// order, each in row-major order.
    ///
    /// The one range walk behind every aggregate function. A range whose corners
    /// sit on two sheets is a 3-D reference (only [`Self::evaluate_reference_arg`]
    /// builds one) and is walked sheet by sheet in tab order; every range is
    /// clipped to each sheet's used area so the walk stays proportional to the
    /// data, and `f` is not handed the blank cells past it (T-0405). A walker
    /// that reads those blanks uses
    /// [`Self::for_each_cell_or_error_in_whole_range`]. Several areas (a union) are walked over
    /// their existing cells only, so `f` is never handed a blank cell of a union
    /// ([`Self::for_each_position_in_area`]). The first error cell aborts the walk
    /// and is returned as `Err`, as does an `Err` from `f`. `f` never sees
    /// `CalcResult::Error`.
    pub(crate) fn for_each_cell_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.for_each_cell_or_error_in_areas(areas, cell, |model, value| match value {
            error @ CalcResult::Error { .. } => Err(error),
            value => f(model, value),
        })
    }

    /// [`Self::for_each_cell_in_areas`] that also hands `f` the error cells
    /// (COUNTA counts them, COUNT skips them).
    pub(crate) fn for_each_cell_or_error_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.for_each_position_in_areas(areas, cell, |model, position| {
            let value = model.evaluate_cell(position);
            f(model, value)
        })
    }

    /// [`Self::for_each_cell_or_error_in_areas`] over one range that also hands `f` the blank
    /// cells of an explicit block past the used area, for the walkers that read a blank as
    /// something: TEXTJOIN with `ignore_empty` FALSE adds a delimiter for it. (GCD and LCM read
    /// it as 0, `LCM(F1:F3)` with 4, blank, 6 is 0, docs/excel-semantics/coercion.md, but any
    /// number of blanks as one: [`Self::for_each_cell_or_error_in_range_noting_blanks`].)
    /// Whole-row and whole-column ranges are still clipped to the used area.
    pub(crate) fn for_each_cell_or_error_in_whole_range<F>(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.walk_area(
            range,
            AreaWalk::ClipWholeLines,
            cell,
            &mut |model, position| {
                let value = model.evaluate_cell(position);
                f(model, value)
            },
        )
        .map(|_| ())
    }

    /// [`Self::for_each_cell_or_error_in_whole_range`] for a walker to which any number of blank
    /// cells reads as one: an explicit block is cut at the used area as
    /// [`Self::for_each_cell_or_error_in_areas`] cuts it, and the result says whether the cut
    /// left out blank cells that the whole-range walk hands on. GCD and LCM read them as one 0,
    /// so `LCM(A1:Z1048575)` costs the data, not 27 million cells (T-0438).
    pub(crate) fn for_each_cell_or_error_in_range_noting_blanks<F>(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<bool, CalcResult>
    where
        F: FnMut(&mut Self, CalcResult) -> Result<(), CalcResult>,
    {
        self.walk_area(range, AreaWalk::ClipBlocks, cell, &mut |model, position| {
            let value = model.evaluate_cell(position);
            f(model, value)
        })
    }

    /// Calls `f` with the position of every cell of `areas`, as
    /// [`Self::for_each_cell_in_areas`] orders them, without evaluating it: for walks that
    /// skip cells by position (AGGREGATE's hidden rows and nested SUBTOTAL/AGGREGATE cells).
    /// Several areas are walked by [`Self::for_each_position_in_area`] as areas of a union: only
    /// their cells that exist, held to the formula's walk budget. An `Err` from `f` stops the
    /// walk.
    pub(crate) fn for_each_position_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let of_union = areas.len() > 1;
        for area in areas {
            self.for_each_position_in_area(area, of_union, cell, &mut f)?;
        }
        Ok(())
    }

    /// Calls `f` with the position of the cells of one area, row by row. A range that is the
    /// whole argument (`of_union` false) is walked densely: every position of the range within
    /// each sheet's used area, for an explicit block as for whole rows and columns (T-0405). An
    /// area of a union is walked over the cells that exist only: every caller of a union walk
    /// skips blank cells, and a union can repeat a whole column
    /// many times over (`MAX_AREAS` copies, or a LAMBDA walking one per row), so the walk must
    /// cost what the data costs (T-1497). Its work is drawn from the formula's walk budget
    /// ([`Model::spend_area_walk`]) as it goes, and the area is recorded as seen empty in one
    /// piece rather than cell by cell.
    pub(crate) fn for_each_position_in_area<F>(
        &mut self,
        area: &Range,
        of_union: bool,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let walk = if of_union {
            AreaWalk::Existing
        } else {
            AreaWalk::ClipBlocks
        };
        self.walk_area(area, walk, cell, f).map(|_| ())
    }

    /// [`Self::for_each_position_in_area`] with the cells of each sheet chosen as `walk` says.
    /// Returns whether `ClipBlocks` cut an explicit block at the used area: whether it left out
    /// blank cells that `ClipWholeLines` would have handed on.
    fn walk_area<F>(
        &mut self,
        area: &Range,
        walk: AreaWalk,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<bool, CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let Range { left, right } = area;
        // The columns of a row found by a union walk, reused from row to row and sheet to sheet.
        let mut columns = Vec::new();
        let mut blanks_left_out = false;
        // A 3-D reference spans the sheets `left.sheet..=right.sheet` (tab order); each one
        // is clipped to its own used area, so a span of whole columns, or of a tall block,
        // walks only data (T-0405).
        debug_assert!(left.sheet <= right.sheet, "3-D span corners out of order");
        for sheet in left.sheet..=right.sheet {
            let clipped = match walk {
                AreaWalk::Existing => {
                    self.for_each_existing_position(sheet, (left, right), cell, &mut columns, f)?;
                    continue;
                }
                AreaWalk::ClipBlocks => self
                    .clip_block_to_used_area(sheet, left.row, left.column, right.row, right.column)
                    .and_then(|corner| {
                        let whole_lines = self.used_area_corner(
                            sheet,
                            left.row,
                            left.column,
                            right.row,
                            right.column,
                        )?;
                        blanks_left_out |= corner != whole_lines;
                        Ok(corner)
                    }),
                AreaWalk::ClipWholeLines => {
                    self.clip_to_used_area(sheet, left.row, left.column, right.row, right.column)
                }
            };
            let (row2, column2) =
                clipped.map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
            for row in left.row..=row2 {
                for column in left.column..=column2 {
                    f(self, CellReferenceIndex { sheet, row, column })?;
                }
            }
        }
        Ok(blanks_left_out)
    }

    /// The union-area walk of [`Self::for_each_position_in_area`] on one sheet: the existing
    /// cells of the rectangle `left..=right`, in row-major order like the dense walk, so the
    /// first error cell is the same one. The area, every chunk and row slot the search looks at
    /// and every cell it hands to `f` are paid for before `f` runs. An area whose cells lie in at
    /// most [`MAX_INDEXED_COLUMNS`] columns is walked through the column index instead
    /// ([`Self::for_each_indexed_position`]), so the rows other columns fill cost it nothing.
    fn for_each_existing_position<F>(
        &mut self,
        sheet: u32,
        (left, right): (&CellReferenceIndex, &CellReferenceIndex),
        cell: CellReferenceIndex,
        columns: &mut Vec<i32>,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let sheet_error = |message| CalcResult::new_error(Error::ERROR, cell, message);
        // The positions not handed to `f` are empty. Recording the whole rectangle covers the
        // existing cells too, which is harmless: a spill can only write where nothing but its
        // own spill cells stands, and those read as empty until it has run.
        // The area itself is a step: its record must be paid for even when it finds nothing, or
        // a LAMBDA repeating a union over an empty sheet would grow the record without bound.
        self.spend_area_walk(1, cell)?;
        self.record_seen_empty_range(sheet, left.row, left.column, right.row, right.column);
        let data = &mut self
            .workbook
            .worksheet_mut(sheet)
            .map_err(sheet_error)?
            .sheet_data;
        // Only an area over more rows of the sheet than one chunk gains from the index: the row
        // walk of a shorter one is a chunk's slots at most. Such an area builds the index, once
        // per sheet: a pass over its cells, after which every write keeps it in step. Finding
        // the area's columns is then a search of the index per column found (one when none).
        if right.row.min(data.last_row()) - left.row >= MIN_INDEXED_ROWS {
            data.build_column_index();
            let few = data.columns_between(left.column, right.column, MAX_INDEXED_COLUMNS, columns);
            self.spend_area_walk((columns.len() as u64).max(1), cell)?;
            if few {
                return self.for_each_indexed_position(sheet, (left, right), cell, columns, f);
            }
        }
        // Slots are paid for in whole steps as they add up, so a dense column costs a step per
        // cell and one more per `SLOTS_PER_WALK_STEP` rows.
        let mut slots: u64 = 0;
        let mut from = left.row;
        while from <= right.row {
            let search = self
                .workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data
                .next_row_between(from, right.row);
            let paid = slots.div_ceil(SLOTS_PER_WALK_STEP);
            slots += search.slots;
            let steps = search.absent_chunks + (slots.div_ceil(SLOTS_PER_WALK_STEP) - paid);
            self.spend_area_walk(steps, cell)?;
            let Some(row) = search.row else {
                break;
            };
            self.workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data
                .columns_in_row_between(row, left.column, right.column, columns);
            // A row costs a step even with no cell in the area's columns: finding them is a
            // search of the row.
            self.spend_area_walk((columns.len() as u64).max(1), cell)?;
            for &column in columns.iter() {
                f(self, CellReferenceIndex { sheet, row, column })?;
            }
            from = row + 1;
        }
        Ok(())
    }

    /// The walk of [`Self::for_each_existing_position`] through the column index, for an area
    /// whose cells lie in `columns` (ascending, at most [`MAX_INDEXED_COLUMNS`] of them): each
    /// column's rows are read from the index [`INDEX_BATCH_ROWS`] at a time and merged in
    /// row-major order, so only the area's own cells are visited. A read is a step and every
    /// cell handed to `f` is a step (a row at least one per four columns still being merged), paid
    /// before `f` runs. `f` may write to the sheet (a spill
    /// anchored in the area); each of `columns` is then read again from the next row on, a step
    /// each, as the row walk finds each next row afresh. The set of columns is fixed when the walk
    /// starts: a spill into a column of the area that held no cell is not walked, and is caught
    /// by the seen-empty record of the area, which restarts the pass.
    fn for_each_indexed_position<F>(
        &mut self,
        sheet: u32,
        (left, right): (&CellReferenceIndex, &CellReferenceIndex),
        cell: CellReferenceIndex,
        columns: &[i32],
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let sheet_error = |message| CalcResult::new_error(Error::ERROR, cell, message);
        let mut batches: Vec<ColumnBatch> = columns
            .iter()
            .map(|&column| ColumnBatch {
                column,
                rows: Vec::new(),
                next: 0,
            })
            .collect();
        let mut row_columns = Vec::with_capacity(columns.len());
        let mut version = None;
        let mut from = left.row;
        loop {
            let data = &self
                .workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data;
            let stale = version != Some(data.layout_version());
            version = Some(data.layout_version());
            let mut reads = 0;
            for batch in &mut batches {
                if stale || batch.needs_read() {
                    data.rows_in_column_between(
                        batch.column,
                        (from, right.row),
                        INDEX_BATCH_ROWS,
                        &mut batch.rows,
                    );
                    batch.next = 0;
                    reads += 1;
                }
            }
            self.spend_area_walk(reads, cell)?;
            // A column with no rows left is done (a write by `f` into it is caught by the
            // area's read-empty record, as for a column that held none at the start).
            batches.retain(|batch| batch.head().is_some());
            let Some(row) = batches.iter().filter_map(ColumnBatch::head).min() else {
                break;
            };
            row_columns.clear();
            for batch in &mut batches {
                if batch.head() == Some(row) {
                    row_columns.push(batch.column);
                    batch.next += 1;
                }
            }
            // A row is a step per cell handed on, and at least a step per four columns still
            // being merged, which each row compares.
            let merge = batches.len().div_ceil(4);
            self.spend_area_walk(row_columns.len().max(merge) as u64, cell)?;
            for &column in &row_columns {
                f(self, CellReferenceIndex { sheet, row, column })?;
            }
            from = row + 1;
        }
        Ok(())
    }

    /// Calls `f` with every number held by the cells of `areas`, in the order of
    /// [`Self::for_each_cell_in_areas`].
    ///
    /// This is how aggregate functions (SUM, NPV, VAR, ...) read a
    /// *reference*: numbers count, booleans, text and empty cells are skipped
    /// (unlike direct arguments, which the callers coerce themselves). See
    /// [`Self::for_each_cell_in_areas`] for errors and range clipping.
    pub(crate) fn for_each_number_in_areas<F>(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_cell_in_areas(areas, cell, |_, value| {
            if let Some(value) = cell_number(&value) {
                f(value);
            }
            Ok(())
        })
    }
}

/// Evaluates `c[0] + c[1]*y + c[2]*y^2 + ...` by Horner's rule in plain doubles, from the top
/// coefficient.
pub(crate) fn poly(c: &[f64], y: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, &a| acc * y + a)
}

/// Excel's `MOD`: the exact remainder (`fmod`) shifted into the divisor's sign. Not
/// `n - d*floor(n/d)`, which loses precision for tiny divisors; evidence: `MOD_QUOTIENT.xlsx`
/// C20/C21/C25/C26 (docs/excel-semantics/numbers.md). `divisor` must be non-zero.
pub(crate) fn excel_mod(value: f64, divisor: f64) -> f64 {
    let remainder = value % divisor;
    if remainder == 0.0 {
        // Drop the sign of -0.0 (fmod keeps the dividend's sign).
        0.0
    } else if (remainder < 0.0) != (divisor < 0.0) {
        remainder + divisor
    } else {
        remainder
    }
}

/// How far below the next integer a whole-number argument may fall and still count as that
/// integer: 2^-22 + 2^-33, the same at 1, 500, 1000, 30000 and 100000 (an absolute bound, not
/// relative). LEFT, RIGHT, REPLACE counts and INDEX's row and column indexes share it.
pub(crate) const NEAR_INTEGER_TOLERANCE: f64 = 1.0 / 4_194_304.0 + 1.0 / 8_589_934_592.0;

/// The whole number LEFT, RIGHT, REPLACE and INDEX read from a count, position or index: `x`
/// truncated, except that a fraction within [`NEAR_INTEGER_TOLERANCE`] of the next integer counts
/// as that integer, so `LEFT("abc",1-2^-22)` is "a". MID truncates exactly and does not use this.
// Excel: docs/excel-semantics/text.md "LEFT, RIGHT and REPLACE round counts up near an
// integer" (probes/text-count-argument-bounds) and lookup.md "INDEX rounds an index up near an
// integer" (probes/index-snap-tolerance).
pub(crate) fn near_integer_count(x: f64) -> f64 {
    let whole = x.floor();
    // `x - whole` is exact for every finite double, so the bound is not blurred at large x.
    if x - whole >= 1.0 - NEAR_INTEGER_TOLERANCE {
        whole + 1.0
    } else {
        whole
    }
}

impl Function {
    /// XLOOKUP, TAKE, DROP and TRIMRANGE: over a range they evaluate to the cells they pick, so
    /// SUBTOTAL, AREAS and a LET or LAMBDA binding read a reference
    /// (`Model::evaluate_picked_reference_arg`, probes/subtotal-reference-operands-b H12,
    /// H15:H17; probes/let-name-uncalled-picked-refs). The one list of these calls, with the
    /// index of the argument each picks from: XLOOKUP's `return_array` (2), the array (0) of
    /// the others.
    pub(crate) fn picked_reference_arg(&self) -> Option<usize> {
        match self {
            Function::Xlookup => Some(2),
            Function::Take | Function::Drop | Function::Trimrange => Some(0),
            _ => None,
        }
    }

    /// Whether the call is one of [`Function::picked_reference_arg`]'s.
    pub(crate) fn picks_reference(&self) -> bool {
        self.picked_reference_arg().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::near_integer_count;

    #[test]
    fn near_integer_count_rounds_up_within_the_tolerance() {
        // probes/text-count-argument-bounds Z10, Z9, AA1, AA2: 1 - (2^-22 + 2^-33) is 1, a hair
        // further down is 0; the bound is absolute (AA13, AA14 below 30000).
        let at = 2f64.powi(-22) + 2f64.powi(-33);
        let past = 2f64.powi(-22) + 9.0 * 2f64.powi(-36);
        assert_eq!(near_integer_count(1.0 - at), 1.0);
        assert_eq!(near_integer_count(1.0 - past), 0.0);
        assert_eq!(near_integer_count(30000.0 - at), 30000.0);
        assert_eq!(near_integer_count(30000.0 - past), 29999.0);
        assert_eq!(near_integer_count(2.5), 2.0);
        assert_eq!(near_integer_count(1e-300), 0.0);
        assert_eq!(near_integer_count(1e100), 1e100);
    }
}
