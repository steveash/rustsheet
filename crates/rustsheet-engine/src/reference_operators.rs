//! Evaluation of the reference operators: the range (`:`), the intersection (a space) and
//! the union (a comma inside parentheses).

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Most areas an intersection of unions may produce. A union has at most about
/// [`crate::expressions::parser::MAX_FORMULA_DEPTH`] areas, but intersecting unions multiplies
/// them, so without a cap a short formula could allocate without bound (CLAUDE.md directive 5).
const MAX_AREAS: usize = 1 << 16;

/// `#VALUE!` for a reference of several areas where one area is needed.
fn several_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "Reference of several areas".to_string())
}

impl Model {
    /// `left:right`, the range operator between two operands that are not a plain `A1:B2`
    /// token (the lexer builds those): only two single cells are joined.
    pub(crate) fn get_range(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let left_result = self.evaluate_node_with_reference(left, cell);
        let right_result = self.evaluate_node_with_reference(right, cell);
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
            ) if left1 == right1 && left2 == right2 => {
                // Corners may be written in either order (`Data!B2:Data!A1` is `Data!A1:B2`).
                // UNVERIFIED against the oracle (references.md).
                CalcResult::Range {
                    left: CellReferenceIndex {
                        sheet: left1.sheet,
                        row: left1.row.min(right2.row),
                        column: left1.column.min(right2.column),
                    },
                    right: CellReferenceIndex {
                        sheet: right2.sheet,
                        row: left1.row.max(right2.row),
                        column: left1.column.max(right2.column),
                    },
                }
            }
            _ => CalcResult::new_error(Error::VALUE, cell, "Invalid range".to_string()),
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
        match self.intersect_areas(left, right, cell) {
            Ok(areas) => match areas.as_slice() {
                [area] => CalcResult::Range {
                    left: area.left,
                    right: area.right,
                },
                _ => several_areas(cell),
            },
            Err(error) => error,
        }
    }

    /// `(left,right)`, the union operator, where a value is needed: `#VALUE!`, even for
    /// `(E4,E4)` (gen-m1/c940d3cfb0d3b083). The engine has no multi-area value yet (T-0133),
    /// so this is also what a function argument gets. An operand's own error comes first
    /// (UNVERIFIED, references.md).
    pub(crate) fn get_union(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if let Err(error) = self.union_areas(left, right, cell) {
            return error;
        }
        several_areas(cell)
    }

    /// The areas a reference expression stands for: several for a union, and for an
    /// intersection with one; one for any other reference. `Err` holds the result to give
    /// instead (see [`Model::evaluate_intersection_operand`]).
    fn evaluate_areas(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        match node {
            Node::OpUnionKind { left, right } => self.union_areas(left, right, cell),
            Node::OpIntersectKind { left, right } => self.intersect_areas(left, right, cell),
            _ => Ok(vec![self.evaluate_intersection_operand(node, cell)?]),
        }
    }

    fn union_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        let mut areas = self.evaluate_areas(left, cell)?;
        areas.extend(self.evaluate_areas(right, cell)?);
        Ok(areas)
    }

    /// The pairwise intersections of the areas of `left` and `right` that hold a cell. A pair
    /// on different sheets is `#VALUE!`, and no shared cell at all is `#NULL!`. Unions of
    /// unions multiply the areas, so more than [`MAX_AREAS`] is `#VALUE!` (DEVIATION,
    /// decision 0014: references.md "Union area cap").
    fn intersect_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        let left = self.evaluate_areas(left, cell)?;
        let right = self.evaluate_areas(right, cell)?;
        if left.len().saturating_mul(right.len()) > MAX_AREAS {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Too many areas in an intersection".to_string(),
            ));
        }
        let mut areas = Vec::new();
        for a in &left {
            for b in &right {
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
                if top <= bottom && first <= last {
                    let sheet = a.left.sheet;
                    areas.push(Range {
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
                    });
                }
            }
        }
        if areas.is_empty() {
            return Err(CalcResult::new_error(
                Error::NULL,
                cell,
                "Empty intersection".to_string(),
            ));
        }
        Ok(areas)
    }

    /// An operand of the intersection as a single-sheet area with its corners in order, or
    /// the result to give instead: the operand's own error, or `#VALUE!` for anything that
    /// is not a reference (`(1+0) A1`) or spans sheets.
    fn evaluate_intersection_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        match self.evaluate_node_with_reference(node, cell) {
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
}
