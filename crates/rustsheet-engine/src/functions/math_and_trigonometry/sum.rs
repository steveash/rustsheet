use crate::expressions::types::CellReferenceIndex;
use crate::reference_operators::RefArg;

use crate::{
    arithmetic::snap_near_zero,
    calc_result::CalcResult,
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    model::Model,
};

type TwoMatricesResult = (i32, i32, Vec<Option<f64>>, Vec<Option<f64>>);

// Helper to check if two shapes are the same or compatible 1D shapes
fn is_same_shape_or_1d(rows1: i32, cols1: i32, rows2: i32, cols2: i32) -> bool {
    (rows1 == rows2 && cols1 == cols2)
        || (rows1 == 1 && cols2 == 1 && cols1 == rows2)
        || (rows2 == 1 && cols1 == 1 && cols2 == rows1)
}

impl Model {
    // SUMX2MY2(array_x, array_y) - Returns the sum of the difference of squares
    pub(crate) fn fn_sumx2my2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.sumx_fold(args, cell, |x, y| x * x - y * y)
    }

    // SUMX2PY2(array_x, array_y) - Returns the sum of the sum of squares
    pub(crate) fn fn_sumx2py2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.sumx_fold(args, cell, |x, y| x * x + y * y)
    }

    // SUMXMY2(array_x, array_y) - Returns the sum of squares of differences
    pub(crate) fn fn_sumxmy2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.sumx_fold(args, cell, |x, y| (x - y) * (x - y))
    }

    // Excel pairs the arrays row-major and only needs equal element counts (not shapes); pairs where
    // either side is not a number are skipped, none left is #DIV/0!, unequal counts are #N/A
    // (docs/excel-semantics/coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing").
    fn sumx_fold(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        term: impl Fn(f64, f64) -> f64,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let left = match self.sumx_values(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let right = match self.sumx_values(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if left.len() != right.len() {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "Arrays must have the same number of elements".to_string(),
            );
        }
        let mut sum = 0.0;
        let mut count = 0;
        for (x, y) in left.into_iter().zip(right) {
            if let (Some(x), Some(y)) = (x, y) {
                sum += term(x, y);
                count += 1;
            }
        }
        if count == 0 {
            return CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string());
        }
        CalcResult::finite_number_or_num(sum, cell)
    }

    // Flattens one SUMX argument (range, array or scalar number) row-major; non-numbers are `None`.
    fn sumx_values(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Range { left, right } => self.values_from_range(left, right),
            CalcResult::Array(array) => self
                .values_from_array(array)
                .map_err(|error| CalcResult::new_error(error, cell, "Error in array".to_string())),
            CalcResult::Number(n) => Ok(vec![Some(n)]),
            error @ CalcResult::Error { .. } => Err(error),
            _ => Ok(vec![None]),
        }
    }

    // Helper function to extract and validate two matrices (ranges or arrays) with compatible shapes.
    // Returns (rows, cols, values_left, values_right) or an error.
    pub(crate) fn fn_get_two_matrices(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<TwoMatricesResult, CalcResult> {
        if args.len() != 2 {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let x_range = self.evaluate_node_in_context(&args[0], cell);
        let y_range = self.evaluate_node_in_context(&args[1], cell);

        let result = match (x_range, y_range) {
            (
                CalcResult::Range {
                    left: l1,
                    right: r1,
                },
                CalcResult::Range {
                    left: l2,
                    right: r2,
                },
            ) => {
                if l1.sheet != l2.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                let rows1 = r1.row - l1.row + 1;
                let cols1 = r1.column - l1.column + 1;
                let rows2 = r2.row - l2.row + 1;
                let cols2 = r2.column - l2.column + 1;
                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges must be of the same shape".to_string(),
                    ));
                }
                let values_left = self.values_from_range(l1, r1)?;
                let values_right = self.values_from_range(l2, r2)?;
                (rows1, cols1, values_left, values_right)
            }
            (
                CalcResult::Array(left),
                CalcResult::Range {
                    left: l2,
                    right: r2,
                },
            ) => {
                let rows2 = r2.row - l2.row + 1;
                let cols2 = r2.column - l2.column + 1;

                let rows1 = left.len() as i32;
                let cols1 = if rows1 > 0 { left[0].len() as i32 } else { 0 };
                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Array and range must be of the same shape".to_string(),
                    ));
                }
                let values_left = match self.values_from_array(left) {
                    Err(error) => {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in first array: {error:?}"),
                        ));
                    }
                    Ok(v) => v,
                };
                let values_right = self.values_from_range(l2, r2)?;
                (rows2, cols2, values_left, values_right)
            }
            (
                CalcResult::Range {
                    left: l1,
                    right: r1,
                },
                CalcResult::Array(right),
            ) => {
                let rows1 = r1.row - l1.row + 1;
                let cols1 = r1.column - l1.column + 1;

                let rows2 = right.len() as i32;
                let cols2 = if rows2 > 0 { right[0].len() as i32 } else { 0 };
                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Range and array must be of the same shape".to_string(),
                    ));
                }
                let values_left = self.values_from_range(l1, r1)?;
                let values_right = match self.values_from_array(right) {
                    Err(error) => {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in second array: {error:?}"),
                        ));
                    }
                    Ok(v) => v,
                };
                (rows1, cols1, values_left, values_right)
            }
            (CalcResult::Array(left), CalcResult::Array(right)) => {
                let rows1 = left.len() as i32;
                let rows2 = right.len() as i32;
                let cols1 = if rows1 > 0 { left[0].len() as i32 } else { 0 };
                let cols2 = if rows2 > 0 { right[0].len() as i32 } else { 0 };

                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Arrays must be of the same shape".to_string(),
                    ));
                }
                let values_left = match self.values_from_array(left) {
                    Err(error) => {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in first array: {error:?}"),
                        ));
                    }
                    Ok(v) => v,
                };
                let values_right = match self.values_from_array(right) {
                    Err(error) => {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in second array: {error:?}"),
                        ));
                    }
                    Ok(v) => v,
                };
                (rows1, cols1, values_left, values_right)
            }
            _ => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Both arguments must be ranges or arrays".to_string(),
                ));
            }
        };
        Ok(result)
    }
}

/// A running SUM: the total and the two addends of the latest addition, which
/// [`snap_near_zero`] needs.
#[derive(Default)]
pub(crate) struct SumParts {
    total: f64,
    before_last: f64,
    last: f64,
}

impl SumParts {
    pub(crate) fn add(&mut self, value: f64) {
        self.before_last = self.total;
        self.last = value;
        self.total += value;
    }

    /// The total with its final addition snapped by [`snap_near_zero`].
    pub(crate) fn snapped(&self) -> f64 {
        snap_near_zero(self.before_last, self.last, self.total)
    }
}

impl Model {
    /// SUM(number1, [number2], ...)
    ///
    /// The SUM function adds values. You can add individual values, cell references or ranges or a mix of all three.
    ///
    /// # Details
    /// Generally each argument is evaluated and the result is converted to a number. However there are some special
    /// cases:
    /// - **Args evaluating to `CalcResult::Range`**: The sum of all the numbers in the range is calculated. No
    ///   conversions are performed, non-numeric values are ignored and errors are propagated.
    /// - **`Node::ReferenceKind` args**: Similar to ranges, conversions are not performed on the evaluated value of the
    ///   referenced cell and errors are propagated.
    /// - **Args evaluating to `CalcResult::Error`**: Errors are propagated.
    ///
    /// SUM's final addition snaps a near-zero result to 0 like `+`/`-` do, wherever the call sits
    /// (`=SUM(0.1,0.2,-0.3)`, `=(SUM(...))`, `=SUM(...)+0` and `=SUM(...)*1` all give 0:
    /// probes/near-zero-sum-forms; docs/excel-semantics/numbers.md "Near-zero snapping").
    pub(crate) fn fn_sum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        match self.sum_with_last_addition(args, cell) {
            Ok(sum) => CalcResult::finite_number_or_num(sum.snapped(), cell),
            Err(error) => error,
        }
    }

    /// The sum of SUM's arguments, with the two addends of its final addition.
    fn sum_with_last_addition(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<SumParts, CalcResult> {
        if args.is_empty() {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let mut sum = SumParts::default();
        // Excel: an error in a referenced cell or an array ranks below an error in a direct argument, whichever
        // comes first, and the leftmost reference error wins (docs/excel-semantics/syntax.md,
        // "Stored #BUSY! behaviors", probes/sum-error-precedence). So a reference error is held
        // back until the direct arguments are known to be clean.
        let mut reference_error = None;
        for arg in args {
            // A reference (also via IF/INDEX) is a one-cell range: text is skipped.
            match self.evaluate_picked_reference_arg(arg, cell) {
                RefArg::Ranges(areas) => {
                    if reference_error.is_none() {
                        if let Err(error) =
                            self.for_each_number_in_areas(&areas, cell, |value| sum.add(value))
                        {
                            reference_error = Some(error);
                        }
                    }
                }
                RefArg::Value(CalcResult::Array(array)) => {
                    for value in array.into_iter().flatten() {
                        match value {
                            ArrayNode::Number(value) => sum.add(value),
                            // An error inside an array is not a scalar direct error: it ranks with
                            // the reference errors (leftmost wins, a direct scalar error beats it;
                            // probes/aggregate-error-precedence C50, C53, C61, C77).
                            ArrayNode::Error(error) if reference_error.is_none() => {
                                reference_error = Some(CalcResult::Error {
                                    error,
                                    origin: cell,
                                    message: "Error in array".to_string(),
                                });
                            }
                            // Later errors, booleans and text in an array are ignored.
                            _ => {}
                        }
                    }
                }
                RefArg::Value(error @ CalcResult::Error { .. }) => return Err(error),
                RefArg::Value(calc_result) => {
                    sum.add(self.cast_aggregate_arg_to_number(calc_result, cell)?);
                }
            };
        }
        match reference_error {
            Some(error) => Err(error),
            None => Ok(sum),
        }
    }
}
