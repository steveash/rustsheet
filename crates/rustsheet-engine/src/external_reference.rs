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
        empty_intersection, names_other_workbook, several_areas_ref, too_many_areas, MAX_AREAS,
    },
};

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
        let blocked = || CalcResult::new_error(Error::BLOCKED, cell, "External reference".into());
        let (rows, columns) = self.extent();
        if (rows, columns) == (1, 1) {
            return blocked();
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

    /// [`Model::external_reference`] where one area is read: several are `#REF!`, as ROWS,
    /// COLUMNS, ROW and COLUMN give for a union (probes/multiarea-count H2, multiarea-cellfn H3).
    pub(crate) fn one_external_area(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<Result<ExternalArea, CalcResult>> {
        Some(
            self.external_reference(node, cell)?
                .and_then(|areas| match areas[..] {
                    [area] => Ok(area),
                    _ => Err(several_areas_ref(cell)),
                }),
        )
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
