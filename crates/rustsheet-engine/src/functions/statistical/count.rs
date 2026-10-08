use crate::calc_result::Range;
use crate::expressions::parser::ArrayNode;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::is_span_or_if_of_span;
use crate::reference_operators::{is_multi_area, RefArg};
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    pub(crate) fn fn_count(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            // References (also via IF/INDEX) arrive as a Range; the scalar
            // arms below only see values.
            match self.evaluate_reference_arg(arg, cell) {
                RefArg::Value(CalcResult::Number(_)) => {
                    result += 1.0;
                }
                RefArg::Value(CalcResult::Boolean(_))
                    if !matches!(arg, Node::ReferenceKind { .. }) =>
                {
                    result += 1.0;
                }
                // COUNT(5,) is 2 (probes/average-value-args F16).
                RefArg::Value(CalcResult::EmptyArg) => result += 1.0,
                // Direct text counts when it coerces to a number as SUM's does, clock, date,
                // currency and percent text included (probes/count-direct-text; T-0252). Only
                // number-ness matters, so the 15-digit cut cannot change the answer
                // (docs/excel-semantics/numbers.md "15-digit literal parsing").
                RefArg::Value(CalcResult::String(s))
                    if !matches!(arg, Node::ReferenceKind { .. })
                        && self.cast_number(&s).is_some_and(f64::is_finite) =>
                {
                    result += 1.0;
                }
                RefArg::Ranges(areas) => {
                    if let Err(error) =
                        self.for_each_cell_or_error_in_areas(&areas, cell, |_, value| {
                            if let CalcResult::Number(_) = value {
                                result += 1.0;
                            }
                            Ok(())
                        })
                    {
                        return error;
                    }
                }
                // Only an array's numbers count (probes/average-value-args F32).
                RefArg::Value(CalcResult::Array(array)) => {
                    let numbers = array.iter().flatten();
                    result += numbers
                        .filter(|v| matches!(v, ArrayNode::Number(_)))
                        .count() as f64;
                }
                RefArg::Value(_) => {
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
            // A reference, 3-D ones included, arrives as a Range: its non-empty cells count.
            match self.evaluate_reference_arg(arg, cell) {
                RefArg::Value(CalcResult::EmptyCell | CalcResult::EmptyArg) => {}
                RefArg::Ranges(areas) => {
                    if let Err(error) =
                        self.for_each_cell_or_error_in_areas(&areas, cell, |_, value| {
                            if !matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                                result += 1.0;
                            }
                            Ok(())
                        })
                    {
                        return error;
                    }
                }
                RefArg::Value(CalcResult::Array(a)) => {
                    for row in a {
                        for node in row {
                            if !matches!(node, ArrayNode::Empty) {
                                result += 1.0;
                            }
                        }
                    }
                }
                RefArg::Value(_) => {
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
            let value = if is_multi_area(arg) && !is_span_or_if_of_span(arg) {
                // A union is #VALUE! here (probes/multiarea-count H1); one area is a range.
                self.evaluate_reference_arg(arg, cell)
                    .into_single_area(cell)
            } else {
                self.evaluate_single_sheet_arg(arg, cell)
            };
            match value {
                CalcResult::EmptyCell | CalcResult::EmptyArg => result += 1.0,
                CalcResult::String(s) if s.is_empty() => result += 1.0,
                CalcResult::Range { left, right } => {
                    // Only the used area is walked: the cells past it are empty,
                    // so every cell that is not blank is found in the walk.
                    let mut filled = 0.0;
                    if let Err(error) = self.for_each_cell_or_error_in_areas(
                        &[Range { left, right }],
                        cell,
                        |_, value| {
                            match value {
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                CalcResult::String(s) if s.is_empty() => {}
                                _ => filled += 1.0,
                            }
                            Ok(())
                        },
                    ) {
                        return error;
                    }
                    let side = |first: i32, last: i32| f64::from(last - first + 1);
                    result += side(left.row, right.row) * side(left.column, right.column) - filled;
                }
                // Only the 3-D reference and union errors; other errors count nothing, as before.
                error @ CalcResult::Error {
                    error: Error::VALUE,
                    ..
                } if is_span_or_if_of_span(arg)
                    || matches!(arg, Node::SheetSpanKind { .. })
                    || is_multi_area(arg) =>
                {
                    return error;
                }
                _ => {}
            };
        }
        CalcResult::Number(result)
    }
}
