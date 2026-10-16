//! References into another workbook whose data the engine does not hold (no cached values; that
//! workbook is never opened, directive 5): their shape, and what their cells read through INDEX,
//! the reference operators, aggregates and spill. Excel: docs/excel-semantics/references.md
//! "External reference with no cached data is `#BLOCKED!`".

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    external_links::CachedSheet,
    functions::Function,
    limits::check_array_size,
    model::Model,
    reference_operators::{
        empty_intersection, names_other_workbook, several_areas, several_areas_ref, too_many_areas,
        Picked, MAX_AREAS,
    },
};

/// `#BLOCKED!`, what a cell of another workbook reads when the engine holds no cache for it.
pub(crate) fn external_blocked(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::BLOCKED, cell, "External reference".into())
}

/// The rows and columns a reference into another workbook covers, absolute and in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExternalArea {
    pub(crate) first_row: i32,
    pub(crate) first_column: i32,
    pub(crate) last_row: i32,
    pub(crate) last_column: i32,
    /// The externalLink cache of the sheet the area is on, `None` when the workbook holds none
    /// (its cells read `#BLOCKED!`).
    pub(crate) cache: Option<CachedSheet>,
}

impl ExternalArea {
    /// The area's top-left cell, the value an element of a lifted INDEX keeps.
    pub(crate) fn top_left(self) -> Self {
        Self {
            last_row: self.first_row,
            last_column: self.first_column,
            ..self
        }
    }

    /// `(rows, columns)` of the area.
    pub(crate) fn extent(self) -> (usize, usize) {
        let span = |first: i32, last: i32| usize::try_from(last - first + 1).unwrap_or(0);
        (
            span(self.first_row, self.last_row),
            span(self.first_column, self.last_column),
        )
    }
}

/// The area of `node` when it is a reference on one sheet of another workbook whose data the
/// engine does not hold: still a reference of the written shape (its cells read `#BLOCKED!`), so
/// `ISREF('[other.xlsx]Sheet1'!A1)` is TRUE, `ROWS('[other.xlsx]Sheet1'!A1:B3)` 3, `COLUMNS`,
/// `ROW`, `COLUMN` and `AREAS` read its shape (probes/external-ref-no-link A4, A10;
/// external-ref-no-link-shapes A1, A2, A6:A10, A13). A 3-D prefix naming another workbook
/// (`'[other.xlsx]S1:S2'!A1`) is not a reference: ISREF is FALSE (external-ref-no-link-shapes
/// A12). `None` also for an area off the grid.
pub(crate) fn external_area(node: &Node, cell: CellReferenceIndex) -> Option<ExternalArea> {
    let (sheet_name, first, last) = match node.ungrouped() {
        Node::WrongReferenceKind {
            sheet_name,
            absolute_row,
            absolute_column,
            row,
            column,
        } => {
            let corner = (*row, *column, *absolute_row, *absolute_column);
            (sheet_name, corner, corner)
        }
        Node::WrongRangeKind {
            sheet_name,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } => (
            sheet_name,
            (*row1, *column1, *absolute_row1, *absolute_column1),
            (*row2, *column2, *absolute_row2, *absolute_column2),
        ),
        _ => return None,
    };
    let name = sheet_name.as_deref();
    if !names_other_workbook(name) || name.is_some_and(|name| name.contains(':')) {
        return None;
    }
    // Relative coordinates are offsets from the formula's cell, as for a reference on a sheet.
    let absolute = |(row, column, absolute_row, absolute_column): (i32, i32, bool, bool)| {
        (
            row + if absolute_row { 0 } else { cell.row },
            column + if absolute_column { 0 } else { cell.column },
        )
    };
    let ((row1, column1), (row2, column2)) = (absolute(first), absolute(last));
    let area = ExternalArea {
        first_row: row1.min(row2),
        first_column: column1.min(column2),
        last_row: row1.max(row2),
        last_column: column1.max(column2),
        cache: None,
    };
    let on_grid = area.first_row >= 1
        && area.first_column >= 1
        && area.last_row <= LAST_ROW
        && area.last_column <= LAST_COLUMN;
    on_grid.then_some(area)
}

impl ExternalArea {
    /// The area's cells in a value context, where the area was reached through INDEX or an
    /// intersection: one cell reads as empty (`INDEX(…!A1:C4,2,2)+1` is 1, `&"x"` is "x",
    /// ISBLANK TRUE, ERROR.TYPE `#N/A`), and several are `#BLOCKED!` each, spilled
    /// (`=INDEX(…!A1:C4,2,0)` spills three; probes/external-ref-no-link-index A1:A6, A19, A36,
    /// C7:E7).
    pub(crate) fn picked_value(self, cell: CellReferenceIndex) -> CalcResult {
        if self.extent() == (1, 1) {
            CalcResult::EmptyCell
        } else {
            self.blocked_value(cell)
        }
    }

    /// The written area's cells in a value context: `#BLOCKED!` each, one cell a scalar and
    /// several spilled (`='[other.xlsx]Sheet1'!A1:B2` spills four, `…!A1:B2*1` too;
    /// probes/external-ref-no-link A2, external-ref-no-link-index C1:D5). An area past the
    /// array bound is that bound's error (CLAUDE.md directive 5).
    pub(crate) fn blocked_value(self, cell: CellReferenceIndex) -> CalcResult {
        let (rows, columns) = self.extent();
        if (rows, columns) == (1, 1) {
            return external_blocked(cell);
        }
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        CalcResult::Array(vec![vec![ArrayNode::Error(Error::BLOCKED); columns]; rows])
    }

    /// Whether the cell (`row`, `column`) is in the area.
    pub(crate) fn contains(self, row: i32, column: i32) -> bool {
        (self.first_row..=self.last_row).contains(&row)
            && (self.first_column..=self.last_column).contains(&column)
    }

    /// The cells this area and `other` share, `None` if none.
    pub(crate) fn intersect(self, other: ExternalArea) -> Option<ExternalArea> {
        let area = ExternalArea {
            first_row: self.first_row.max(other.first_row),
            first_column: self.first_column.max(other.first_column),
            last_row: self.last_row.min(other.last_row),
            last_column: self.last_column.min(other.last_column),
            // Areas on two cached sheets share no cached cell.
            cache: self.cache.filter(|_| self.cache == other.cache),
        };
        (area.first_row <= area.last_row && area.first_column <= area.last_column).then_some(area)
    }
}

/// Whether `node` is an IF or CHOOSE one of whose value arguments may stand for a reference into
/// another workbook ([`may_be_external`]): only then does [`Model::pick_branch`] read its
/// condition or index ahead of the caller.
fn may_pick_external(node: &Node) -> bool {
    match node.ungrouped() {
        Node::FunctionKind {
            kind: Function::If | Function::Choose,
            args,
        } => args.iter().skip(1).any(may_be_external),
        _ => false,
    }
}

/// Whether `node` may stand for a reference into another workbook, from its form alone: such a
/// reference written, an external name, or INDEX, a reference operator, IF or CHOOSE over one.
/// Bounded by the parser's depth cap.
fn may_be_external(node: &Node) -> bool {
    match node.ungrouped() {
        Node::WrongReferenceKind { sheet_name, .. } | Node::WrongRangeKind { sheet_name, .. } => {
            names_other_workbook(sheet_name.as_deref())
        }
        Node::DefinedNameKind((name, ..)) => name.starts_with('['),
        Node::FunctionKind {
            kind: Function::Index,
            args,
        } => args.first().is_some_and(may_be_external),
        Node::OpUnionKind { left, right, .. } | Node::OpIntersectKind { left, right, .. } => {
            may_be_external(left) || may_be_external(right)
        }
        // A LET whose body picks only such references stands for the one it picks
        // (`ROWS(IF(TRUE,LET(c,1,…!A1:B3)))` is 3; probes/external-ref-no-link-let A13).
        Node::FunctionKind {
            kind: Function::Let,
            args,
        } => let_picks_external(args),
        node => may_pick_external(node),
    }
}

/// Whether `node` can only stand for a reference into another workbook or a value an IF or
/// CHOOSE gives in its place, never a reference into this workbook: every argument IF or CHOOSE
/// may pick is such a reference ([`may_be_external`]). Only such a LET body is read while its
/// names are bound ([`Model::pick_reference`]), so a LET that may pick a reference into this
/// workbook stays the reference it evaluates to. Bounded by the parser's depth cap.
fn picks_only_external(node: &Node) -> bool {
    match node.ungrouped() {
        Node::FunctionKind {
            kind: Function::If | Function::Choose,
            args,
        } => args.len() > 1 && args.iter().skip(1).all(picks_only_external),
        node => may_be_external(node),
    }
}

/// Whether a LET with arguments `args` has a body that picks only references into another
/// workbook ([`picks_only_external`]).
fn let_picks_external(args: &[Node]) -> bool {
    args.last().is_some_and(picks_only_external)
}

/// What a reference argument stands for once IF, CHOOSE and LET have picked it
/// ([`Model::pick_reference`]).
pub(crate) enum PickedReference<'a> {
    /// The areas of a reference into another workbook, or the result it gives instead
    /// ([`Model::external_reference`]).
    External(ExternalAreas),
    /// Any other argument picked, still unevaluated.
    Node(&'a Node),
    /// The value an IF, CHOOSE or LET gives in place of a reference ([`Picked::Value`]).
    Value(CalcResult),
}

/// The one area of a reference into another workbook, as ROWS, COLUMNS, ROW and COLUMN read
/// it: several are `#REF!`, as for a union (probes/multiarea-count H2, multiarea-cellfn H3).
pub(crate) fn one_external_area(
    areas: ExternalAreas,
    cell: CellReferenceIndex,
) -> Result<ExternalArea, CalcResult> {
    areas.and_then(|areas| match areas[..] {
        [area] => Ok(area),
        _ => Err(several_areas_ref(cell)),
    })
}

/// The areas a reference into another workbook stands for, or the result it gives instead
/// ([`Model::external_reference`]).
pub(crate) type ExternalAreas = Result<Vec<ExternalArea>, CalcResult>;

/// The areas of both operands of a union or intersection into another workbook, or the result it
/// gives instead ([`Model::external_operands`]).
type ExternalOperands = Result<(Vec<ExternalArea>, Vec<ExternalArea>), CalcResult>;

/// `#VALUE!` for a union or intersection of a reference into another workbook with one into this
/// workbook (`AREAS((…!A1,E1))`, `ROWS(…!A1:B3 E1:F3)`; probes/external-ref-no-link-index A13,
/// A14).
fn two_workbooks(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "Reference into two workbooks".to_string(),
    )
}

impl Model {
    /// The areas of `node` when it is a reference into another workbook: written
    /// ([`external_area`]), an INDEX into one, or a union or intersection of them; several
    /// areas in formula order, never more than [`MAX_AREAS`]. `Err` holds what `node` gives
    /// instead: an operand's error, `#NULL!` for an empty intersection, `#VALUE!` for an
    /// operator joining two workbooks, INDEX's error. `None` when `node` is none of these: then
    /// nothing in it was evaluated, so the caller evaluates it as usual. Recursion is bounded by
    /// the parser's depth cap.
    /// Excel: docs/excel-semantics/references.md "External reference with no cached data"
    /// (probes/external-ref-no-link-shapes, external-ref-no-link-index).
    pub(crate) fn external_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<ExternalAreas> {
        let node = node.ungrouped();
        match node {
            Node::WrongReferenceKind { .. } | Node::WrongRangeKind { .. } => self
                .cached_external_area(node, cell)
                .map(|area| Ok(vec![area])),
            Node::DefinedNameKind((name, ..)) => {
                self.external_name_area(name).map(|area| Ok(vec![area]))
            }
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } => self
                .external_index(args, cell)
                .map(|picked| picked.map(|area| vec![area])),
            Node::OpUnionKind { left, right, .. } => Some(
                self.external_operands(left, right, cell)?
                    .and_then(|(left, right)| {
                        if left.len() + right.len() > MAX_AREAS {
                            return Err(too_many_areas(cell));
                        }
                        let mut areas = left;
                        areas.extend(right);
                        Ok(areas)
                    }),
            ),
            Node::OpIntersectKind { left, right, .. } => {
                self.external_intersection(left, right, cell)
            }
            _ => None,
        }
    }

    /// `node` with every IF and CHOOSE at its root that may pick a reference into another
    /// workbook ([`may_pick_external`]) replaced by the argument it picks, each condition and
    /// index read once: an IF or CHOOSE picking such a reference is that reference
    /// (`ROWS(IF(TRUE,…!A1:B3))` is 3, `ISREF(CHOOSE(1,…!A1))` TRUE, `COUNTA(IF(TRUE,…!A1:B2))`
    /// 0; probes/external-ref-no-link-index A25, external-ref-no-link-ifchoose). A caller reads
    /// the picked node in place of `node`, so nothing is evaluated twice; any other `node` comes
    /// back unread. Bounded by the parser's depth cap.
    pub(crate) fn pick_branch<'a>(
        &mut self,
        node: &'a Node,
        cell: CellReferenceIndex,
    ) -> Picked<'a> {
        let mut node = node.ungrouped();
        loop {
            let picked = match node {
                Node::FunctionKind {
                    kind: Function::If,
                    args,
                } if may_pick_external(node) => self.if_pick(args, cell),
                Node::FunctionKind {
                    kind: Function::Choose,
                    args,
                } if may_pick_external(node) => self.choose_pick(args, cell),
                _ => return Picked::Node(node),
            };
            match picked {
                Picked::Node(branch) => node = branch.ungrouped(),
                value @ Picked::Value(_) => return value,
            }
        }
    }

    /// What `node` reads as in a value context when IF or CHOOSE picked it and it is a reference
    /// into another workbook ([`Model::external_reference`]): one area as the cells INDEX picks
    /// ([`Model::external_picked_value`]), several `#VALUE!`. `None` for any other `node`, unread.
    pub(crate) fn external_reference_picked_value(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let areas = self.external_reference(node, cell)?;
        Some(self.external_areas_value(areas, cell))
    }

    /// The value of the areas a reference into another workbook stands for, picked by INDEX, an
    /// intersection, IF or CHOOSE: one area's cells read at once (`ERROR.TYPE(…!A1 …!A1:B2)` is
    /// `#N/A`, probes/external-ref-no-link-index A36), several `#VALUE!`.
    pub(crate) fn external_areas_value(
        &self,
        areas: ExternalAreas,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match areas.as_deref() {
            Ok([area]) => self.external_picked_value(*area, cell),
            Ok(_) => several_areas(cell),
            Err(error) => error.clone(),
        }
    }

    /// `left right` when either operand is a reference into another workbook
    /// ([`Model::external_reference`]): the cells both share, pairwise as for areas of this
    /// workbook (`ROWS(…!A1:B3 …!A2:B5)` is 2, disjoint areas `#NULL!`;
    /// probes/external-ref-no-link-shapes A17, external-ref-no-link-index A18).
    pub(crate) fn external_intersection(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Option<ExternalAreas> {
        let operands = self.external_operands(left, right, cell)?;
        Some(operands.and_then(|(left, right)| {
            let pairs = left.len().saturating_mul(right.len());
            if pairs > MAX_AREAS {
                return Err(too_many_areas(cell));
            }
            self.spend_area_pairs(pairs, cell)?;
            let areas: Vec<ExternalArea> = left
                .iter()
                .flat_map(|a| right.iter().filter_map(|b| a.intersect(*b)))
                .collect();
            if areas.is_empty() {
                return Err(empty_intersection(cell));
            }
            Ok(areas)
        }))
    }

    /// `node` once IF, CHOOSE ([`Model::pick_branch`]) and LET have picked the argument it
    /// stands for, with a reference into another workbook read as its areas
    /// ([`Model::external_reference`]). A LET whose body picks only such references
    /// ([`picks_only_external`]) is read while its names are bound, so it is the reference its
    /// body picks: `ROWS(LET(c,TRUE,IF(c,…!A1:B3)))` is 3, `ISREF` TRUE, `COUNTA` 0
    /// (probes/external-ref-no-link-ifchoose A26, external-ref-no-link-let). Each condition,
    /// index and LET value is read once. Bounded by the parser's depth cap.
    pub(crate) fn pick_reference<'a>(
        &mut self,
        node: &'a Node,
        cell: CellReferenceIndex,
    ) -> PickedReference<'a> {
        let node = match self.pick_branch(node, cell) {
            Picked::Node(node) => node,
            Picked::Value(value) => return PickedReference::Value(value),
        };
        if let Node::FunctionKind {
            kind: Function::Let,
            args,
        } = node
        {
            if let_picks_external(args) {
                let picked = self.let_scope(args, cell, |model, body| {
                    match model.pick_reference(body, cell) {
                        PickedReference::External(areas) => PickedReference::External(areas),
                        PickedReference::Value(value) => PickedReference::Value(value),
                        // A form that names another workbook but is no reference of it (an
                        // area off the grid, an external name with no area): its value.
                        PickedReference::Node(body) => {
                            PickedReference::Value(model.evaluate_node_with_reference(body, cell))
                        }
                    }
                });
                return picked.unwrap_or_else(PickedReference::Value);
            }
        }
        match self.external_reference(node, cell) {
            Some(areas) => PickedReference::External(areas),
            None => PickedReference::Node(node),
        }
    }

    /// The areas of both operands of a union or intersection when either is a reference into
    /// another workbook ([`Model::external_reference`]); `#VALUE!` when only one is, and the
    /// left operand's error first.
    fn external_operands(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Option<ExternalOperands> {
        let left = self.external_reference(left, cell);
        if let Some(Err(error)) = left {
            return Some(Err(error));
        }
        match (left, self.external_reference(right, cell)) {
            (None, None) => None,
            (Some(Ok(left)), Some(Ok(right))) => Some(Ok((left, right))),
            (_, Some(Err(error))) => Some(Err(error)),
            _ => Some(Err(two_workbooks(cell))),
        }
    }
}
