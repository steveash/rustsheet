use crate::calc_result::Range;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::element_errors::array_element_error;
use crate::functions::statistical::paired::{marked_number_error, PairedArg};
use crate::functions::util::aggregate_args::{
    AggregatePolicy, ArgReading, ErrorRule, GridReading, NestedReading, NumberRules,
};

use crate::{
    arithmetic::{extended_sum, snap_near_zero},
    calc_result::CalcResult,
    expressions::parser::Node,
    expressions::token::Error,
    model::Model,
};

type TwoMatricesResult = (i64, i64, Vec<Option<f64>>, Vec<Option<f64>>);

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

    // Excel pairs the arrays row-major and only needs equal element counts (not shapes). The
    // checks run in this order (docs/excel-semantics/coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2
    // pairing", probes/sumx-blank-text-error-order): each argument in turn, an error value or a
    // one-cell reference holding one is that error, and a blank one-cell reference or an omitted
    // argument is #VALUE!; then unequal counts are #N/A, ahead of any error inside a range or
    // array; then that error; then an argument with no number at all is #DIV/0!. Otherwise the
    // result is the sum over the pairs where both sides are numbers, which may be none (0).
    fn sumx_fold(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        term: impl Fn(f64, f64) -> f64,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let first = match self.sumx_arg(&args[0], cell) {
            Ok(arg) => arg,
            Err(e) => return e,
        };
        let second = match self.sumx_arg(&args[1], cell) {
            Ok(arg) => arg,
            Err(e) => return e,
        };
        let unequal_counts = || {
            CalcResult::new_error(
                Error::NA,
                cell,
                "Arrays must have the same number of elements".to_string(),
            )
        };
        if let (Some(m), Some(n)) = (first.cell_count(), second.cell_count()) {
            if m != n {
                return unequal_counts();
            }
        }
        let array_error = |_, error| array_element_error(error, cell);
        let paired = match self.paired_values(first, second, cell, array_error) {
            Ok(paired) => paired,
            Err(e) => return e,
        };
        // A 3-D reference has no count before it is walked.
        if !paired.same_size {
            return unequal_counts();
        }
        let has_number = |values: &[Option<f64>]| values.iter().any(Option::is_some);
        if !has_number(&paired.first) || !has_number(&paired.second) {
            return CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string());
        }
        let mut sum = 0.0;
        for (x, y) in paired.first.into_iter().zip(paired.second) {
            if let (Some(x), Some(y)) = (x, y) {
                sum += term(x, y);
            }
        }
        CalcResult::finite_number_or_num(sum, cell)
    }

    // One SUMX argument: a range, an array or a single value (a non-number is `None`). An error
    // value, or a one-cell reference holding an error, is returned at once, and so is #VALUE! for
    // a blank one-cell reference or an omitted argument; errors inside a longer range or an array
    // wait for the count check.
    fn sumx_arg(&mut self, arg: &Node, cell: CellReferenceIndex) -> Result<PairedArg, CalcResult> {
        let value = match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Range { left, right } if left == right => self.evaluate_cell(left),
            value => value,
        };
        match value {
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Blank argument".to_string(),
            )),
            CalcResult::Number(n) => Ok(PairedArg::Values(vec![Some(n)])),
            CalcResult::Range { left, right } => Ok(PairedArg::Range(Range { left, right })),
            CalcResult::Array(array) => Ok(PairedArg::Array(array)),
            _ => Ok(PairedArg::Values(vec![None])),
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
        // A marked first argument (a number straight from ERROR.TYPE or CELL, an array of
        // references) is `#NUM!` once the sizes match (`INTERCEPT(ERROR.TYPE(#REF!),5)`,
        // `SLOPE(INDIRECT("B1",{1;1}),{1;2})`, while `RSQ(ERROR.TYPE(#NAME?),Data!C8:C11)` is
        // `#N/A`; T-2576, T-2577). The second reads as the number it is
        // (`INTERCEPT(5,ERROR.TYPE(#REF!))` is `#DIV/0!` like (5,4)).
        let (x_range, first_marked) = self.evaluate_sample_arg(&args[0], cell);
        let y_range = self.evaluate_node_in_context(&args[1], cell);
        let x_range = self.blank_one_cell_as_error(x_range, cell);
        let y_range = self.blank_one_cell_as_error(y_range, cell);
        if let (CalcResult::Range { left: l1, .. }, CalcResult::Range { left: l2, .. }) =
            (&x_range, &y_range)
        {
            if l1.sheet != l2.sheet {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Ranges are in different sheets".to_string(),
                ));
            }
        }
        // The first argument's error comes before anything about the second.
        let first = matrix_arg(&args[0], x_range, cell.row)?;
        let second = matrix_arg(&args[1], y_range, cell.row)?;
        let (Some((rows1, cols1, first)), Some((rows2, cols2, second))) = (first, second) else {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Both arguments must be ranges or arrays".to_string(),
            ));
        };
        if rows1 * cols1 != rows2 * cols2 {
            // Excel: unequal cell counts are #N/A, with or without text in the ranges (T-1292, T-1620);
            // any two shapes with the same count pair in reading order (T-2431).
            return Err(CalcResult::new_error(
                Error::NA,
                cell,
                "Arguments must have the same number of cells".to_string(),
            ));
        }
        if first_marked {
            return Err(marked_number_error(cell));
        }
        let array_error = |_, error| array_element_error(error, cell);
        let paired = self.paired_values(first, second, cell, array_error)?;
        Ok((rows1, cols1, paired.first, paired.second))
    }
}

/// A range, array, number, text or boolean argument of [`Model::fn_get_two_matrices`] with its
/// rows and columns; an error value is returned as the error, anything else is `None`.
fn matrix_arg(
    node: &Node,
    value: CalcResult,
    row: i32,
) -> Result<Option<(i64, i64, PairedArg)>, CalcResult> {
    let arg = match PairedArg::from_node_result(node, value, row) {
        Ok(arg) => arg,
        // Excel: a typed text or boolean is one value with no number,
        // like a single cell holding one: `CORREL("a",5)` is #DIV/0! and `CORREL(A1:A3,TRUE)`
        // #N/A (probes/regression-pairs-text-scalar).
        Err(CalcResult::String(_) | CalcResult::Boolean(_)) => PairedArg::Values(vec![None]),
        Err(_) => return Ok(None),
    };
    let (rows, columns) = match &arg {
        PairedArg::Range(Range { left, right }) => (
            i64::from(right.row - left.row + 1),
            i64::from(right.column - left.column + 1),
        ),
        PairedArg::Array(array) => (
            i64::try_from(array.len()).unwrap_or(i64::MAX),
            array
                .first()
                .map_or(0, |row| i64::try_from(row.len()).unwrap_or(i64::MAX)),
        ),
        PairedArg::Values(_) => (1, 1),
        PairedArg::Error(error) => return Err(error.clone()),
    };
    Ok(Some((rows, columns, arg)))
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
        self.total = extended_sum(self.total, value);
    }

    /// The total with its final addition snapped by [`snap_near_zero`].
    pub(crate) fn snapped(&self) -> f64 {
        snap_near_zero(self.before_last, self.last, self.total)
    }
}

/// SUM: an error in a referenced cell or an array ranks below an error in a direct argument,
/// whichever comes first, and the leftmost reference error wins (docs/excel-semantics/syntax.md
/// "Aggregate error precedence", probes/sum-error-precedence; arrays: probes/aggregate-error-precedence
/// C50, C53, C61, C77).
const SUM_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::Ranked,
    skip_empty_args: false,
    grids: GridReading::TopLeft,
    nested: NestedReading::Flatten,
};

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
        self.for_each_aggregate_number(args, cell, SUM_ARGS, NumberRules::SUM_LIKE, |value| {
            sum.add(value);
        })?;
        Ok(sum)
    }
}
