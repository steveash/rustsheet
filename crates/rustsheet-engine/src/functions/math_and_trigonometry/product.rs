use crate::arithmetic::{extended_product, flush_subnormal};
use crate::expressions::types::CellReferenceIndex;
use crate::reference_operators::RefArg;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// PRODUCT(number1, [number2], ...)
    ///
    /// The PRODUCT function multiplies values. You can add individual values, cell references or ranges or a mix of all
    /// three.
    ///
    /// # Details
    /// Generally each argument is evaluated and the result is converted to a number. However there are some special
    /// cases:
    /// - **Args evaluating to `CalcResult::Range`**: The product of all the numbers in the range is calculated. No
    ///   conversions are performed, non-numeric values are ignored and errors are propagated.
    /// - **`Node::ReferenceKind` args**: Similar to ranges, conversions are not performed on the evaluated value of the
    ///   referenced cell and errors are propagated.
    /// - **Args evaluating to `CalcResult::Array`**: Only the numbers are multiplied; text and booleans are skipped and
    ///   the first error element is propagated
    ///   after any direct scalar error (it ranks like a referenced-cell error).
    /// - **Args evaluating to `CalcResult::Error`**: Errors are propagated.
    pub(crate) fn fn_product(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 1.0;
        let mut seen_value = false;
        // A referenced-cell or in-array error ranks below a direct scalar error; the leftmost
        // wins (probes/aggregate-error-precedence C1-C8, probes/product-array-2d-and-scalar
        // A8-A17).
        let mut deferred_error = None;

        for arg in args {
            // Handle reference special case. Behaviour of a reference is equivalent to a range of one cell.
            let arg = match arg.ungrouped() {
                Node::ReferenceKind {
                    column,
                    absolute_column,
                    row,
                    absolute_row,
                    sheet_index,
                    sheet_name,
                } => &Node::RangeKind {
                    sheet_name: sheet_name.clone(),
                    sheet_index: *sheet_index,
                    absolute_row1: *absolute_row,
                    absolute_column1: *absolute_column,
                    row1: *row,
                    column1: *column,
                    absolute_row2: *absolute_row,
                    absolute_column2: *absolute_column,
                    row2: *row,
                    column2: *column,
                },
                _ => arg,
            };
            match self.evaluate_aggregate_arg(arg, cell) {
                RefArg::Ranges(areas) => {
                    if deferred_error.is_none() {
                        if let Err(error) = self.for_each_number_in_areas(&areas, cell, |value| {
                            seen_value = true;
                            result = flush_subnormal(extended_product(result, value));
                        }) {
                            deferred_error = Some(error);
                        }
                    }
                }
                RefArg::Value(error @ CalcResult::Error { .. }) => return error,
                // Like a range, an array argument contributes only its numbers; text and
                // booleans inside it are skipped (same rule as `for_each_value`).
                RefArg::Value(CalcResult::Array(array)) => {
                    let values = match self.values_from_array(array) {
                        Ok(values) => values,
                        Err(error) => {
                            if deferred_error.is_none() {
                                deferred_error = Some(CalcResult::new_error(
                                    error,
                                    cell,
                                    "Error in array".to_string(),
                                ));
                            }
                            continue;
                        }
                    };
                    for value in values.into_iter().flatten() {
                        seen_value = true;
                        result = flush_subnormal(extended_product(result, value));
                    }
                }
                RefArg::Value(calc_result) => {
                    seen_value = true;
                    let cast_result = self.cast_aggregate_arg_to_number(calc_result, cell);
                    match cast_result {
                        Ok(f) => result = flush_subnormal(extended_product(result, f)),
                        Err(s) => return s,
                    }
                }
            };
        }
        if let Some(error) = deferred_error {
            return error;
        }
        if !seen_value {
            return CalcResult::Number(0.0);
        }
        CalcResult::finite_number_or_num(flush_subnormal(result), cell)
    }
}
