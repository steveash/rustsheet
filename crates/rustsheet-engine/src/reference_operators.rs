//! Evaluation of the reference operators: the range (`:`) and the intersection (a space).

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

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
            ) if left1 == right1 && left2 == right2 => CalcResult::Range {
                left: left1,
                right: right2,
            },
            _ => CalcResult::new_error(Error::VALUE, cell, "Invalid range".to_string()),
        }
    }

    /// `left right`, the intersection operator: the reference to the cells both operands
    /// contain. Excel (docs/excel-semantics/references.md, gen-m1/4dc8ed1de05a61be): an
    /// operand's error propagates, areas that share no cell give `#NULL!`, and areas on two
    /// different sheets give `#VALUE!` even where their coordinates overlap.
    pub(crate) fn get_intersection(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let left = match self.evaluate_intersection_operand(left, cell) {
            Ok(range) => range,
            Err(error) => return error,
        };
        let right = match self.evaluate_intersection_operand(right, cell) {
            Ok(range) => range,
            Err(error) => return error,
        };
        if left.left.sheet != right.left.sheet {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Intersection of references on different sheets".to_string(),
            );
        }
        let top = left.left.row.max(right.left.row);
        let bottom = left.right.row.min(right.right.row);
        let first = left.left.column.max(right.left.column);
        let last = left.right.column.min(right.right.column);
        if top > bottom || first > last {
            return CalcResult::new_error(Error::NULL, cell, "Empty intersection".to_string());
        }
        let sheet = left.left.sheet;
        CalcResult::Range {
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
        }
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
