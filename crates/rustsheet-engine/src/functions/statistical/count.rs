use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    pub(crate) fn fn_count(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            // References (also via IF/INDEX) arrive as a Range; the scalar
            // arms below only see values.
            match self.evaluate_aggregate_arg(arg, cell) {
                CalcResult::Number(_) => {
                    result += 1.0;
                }
                CalcResult::Boolean(_) if !matches!(arg, Node::ReferenceKind { .. }) => {
                    result += 1.0;
                }
                // COUNT(5,) is 2 (probes/average-value-args F16).
                CalcResult::EmptyArg => result += 1.0,
                // Direct text counts when it coerces to a number as SUM's does, clock, date,
                // currency and percent text included (probes/count-direct-text; T-0252). Only
                // number-ness matters, so the 15-digit cut cannot change the answer
                // (docs/excel-semantics/numbers.md "15-digit literal parsing").
                CalcResult::String(s)
                    if !matches!(arg, Node::ReferenceKind { .. })
                        && self.cast_number(&s).is_some_and(f64::is_finite) =>
                {
                    result += 1.0;
                }
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    let (row2, column2) = match self.clipped_corner(left, right, cell) {
                        Ok(corner) => corner,
                        Err(error) => return error,
                    };
                    for row in left.row..=row2 {
                        for column in left.column..=column2 {
                            if let CalcResult::Number(_) = self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                result += 1.0;
                            }
                        }
                    }
                }
                // Only an array's numbers count (probes/average-value-args F32).
                CalcResult::Array(array) => {
                    let numbers = array.iter().flatten();
                    result += numbers
                        .filter(|v| matches!(v, ArrayNode::Number(_)))
                        .count() as f64;
                }
                _ => {
                    // Ignore everything else
                }
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_counta(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    let (row2, column2) = match self.clipped_corner(left, right, cell) {
                        Ok(corner) => corner,
                        Err(error) => return error,
                    };
                    for row in left.row..=row2 {
                        for column in left.column..=column2 {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                _ => {
                                    result += 1.0;
                                }
                            }
                        }
                    }
                }
                CalcResult::Array(a) => {
                    for row in a {
                        for node in row {
                            if !matches!(node, crate::expressions::parser::ArrayNode::Empty) {
                                result += 1.0;
                            }
                        }
                    }
                }
                _ => {
                    result += 1.0;
                }
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_countblank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // COUNTBLANK requires only one argument
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::EmptyCell | CalcResult::EmptyArg => result += 1.0,
                CalcResult::String(s) if s.is_empty() => result += 1.0,
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    let (row2, column2) = match self.clipped_corner(left, right, cell) {
                        Ok(corner) => corner,
                        Err(error) => return error,
                    };
                    // The clipped-off cells are empty and each one is blank.
                    let rows = |last: i32| f64::from((last - left.row + 1).max(0));
                    let columns = |last: i32| f64::from((last - left.column + 1).max(0));
                    result +=
                        rows(right.row) * columns(right.column) - rows(row2) * columns(column2);
                    for row in left.row..=row2 {
                        for column in left.column..=column2 {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::EmptyCell | CalcResult::EmptyArg => result += 1.0,
                                CalcResult::String(s) if s.is_empty() => result += 1.0,
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            };
        }
        CalcResult::Number(result)
    }
}
