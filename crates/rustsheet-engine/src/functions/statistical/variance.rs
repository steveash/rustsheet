use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::aggregate_args::{
    AggregatePolicy, ArgReading, ArrayNumbers, CellNumbers, DirectNumbers, ErrorRule, NumberRules,
};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// VARA, VARPA, STDEVA and STDEVPA return the first error in argument order at once, as before
/// they moved onto the walker (an error in a range, an array or a direct argument; the order is
/// UNVERIFIED, T-1439).
const VARA_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::First,
    skip_empty_args: false,
};

/// A referenced cell coerces (text 0, booleans 1/0); a direct value or an array element counts
/// only when it is a number ("A semantics to be added"). Excel coerces a direct "2" and TRUE
/// (probes/stat-argument-modes Sheet1!H69, H72, H75, H78, known failures), which T-1291 fixes.
const VARA_NUMBERS: NumberRules = NumberRules {
    direct: DirectNumbers::NumbersOnly,
    cells: CellNumbers::CoercedA,
    array: ArrayNumbers::Numbers,
};

/// Which divisor a variance uses: `n` (VAR.P family) or `n - 1` (VAR.S family).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VarianceKind {
    Population,
    Sample,
}

/// Excel's VAR.P / VAR.S of `values` (STDEV.P / STDEV.S are its square root).
///
/// Excel first computes the one-pass sum-of-squares form, `(n*Σx² - (Σx)²)/(n*n)` for the
/// population and `(Σx² - (Σx)²/n)/(n-1)` for the sample, and keeps it unless the data cancel
/// (the one-pass sum of squared deviations is below 1% of `Σx²`) or the variance is small next
/// to the mean (below `0.001*|mean|`); then it recomputes with the two-pass form
/// `Σ(x - mean)²/(n-d)`. Both forms use plain left-to-right double sums, so the result is often
/// not the correctly rounded variance (`VAR.S({1,2,4})` = 2.333333333333334).
/// Excel: see docs/excel-semantics/statistics.md "VAR/STDEV switch between a one-pass and a
/// two-pass formula" (probes/variance-*, 1500+ cells).
///
/// `values` must hold at least one value (two for `Sample`); an overflowing `Σx²` gives NaN,
/// which callers turn into `#NUM!` like Excel (numbers.md "Overflow"). The two-pass form needs
/// the values twice, so callers collect them into a buffer (linear in the numeric values found).
pub(crate) fn excel_variance(values: &[f64], kind: VarianceKind) -> f64 {
    // Every product and quotient flushes a subnormal result to 0 like Excel's `*` and `/`:
    // `STDEV.S({0,2.2E-154})` is 0 because each squared deviation (1.21E-308) is subnormal,
    // although their sum is not (probes/variance-underflow; numbers.md "Subnormal results become 0").
    let mul = |a: f64, b: f64| flush_subnormal(a * b);
    let div = |a: f64, b: f64| flush_subnormal(a / b);
    let n = values.len() as f64;
    let mut sum = 0.0;
    let mut sumsq = 0.0;
    for &x in values {
        sum += x;
        sumsq += mul(x, x);
    }
    let (one_pass, divisor) = match kind {
        VarianceKind::Population => (div(mul(n, sumsq) - mul(sum, sum), mul(n, n)), n),
        VarianceKind::Sample => (div(sumsq - div(mul(sum, sum), n), n - 1.0), n - 1.0),
    };
    let mean = div(sum, n);
    if mul(one_pass, divisor) < mul(0.01, sumsq) || one_pass < mul(0.001, mean.abs()) {
        let mut squares = 0.0;
        for &x in values {
            squares += mul(x - mean, x - mean);
        }
        return div(squares, divisor);
    }
    one_pass
}

impl Model {
    /// VAR.P / VAR.S / STDEV.P / STDEV.S (and AGGREGATE 7, 8, 10, 11) of the numbers
    /// `for_each_value` yields.
    pub(crate) fn variance_of_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        kind: VarianceKind,
        std_dev: bool,
        name: &str,
    ) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut values = Vec::new();
        if let Err(error) = self.for_each_value(args, cell, |value| values.push(value)) {
            return error;
        }
        variance_result(&values, kind, std_dev, name, cell)
    }

    /// VARA / VARPA / STDEVA / STDEVPA: like `variance_of_values`, but the cells of a reference
    /// coerce (text 0, booleans 1/0) while direct values and array elements count only as numbers
    /// (SPEC-0009's walker with [`VARA_ARGS`] and [`VARA_NUMBERS`]).
    pub(crate) fn variance_of_values_a(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        kind: VarianceKind,
        std_dev: bool,
        name: &str,
    ) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut values = Vec::new();
        if let Err(error) =
            self.for_each_aggregate_number(args, cell, VARA_ARGS, VARA_NUMBERS, |value| {
                values.push(value);
            })
        {
            return error;
        }
        variance_result(&values, kind, std_dev, name, cell)
    }

    pub(crate) fn fn_var_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values(args, cell, VarianceKind::Population, false, "VAR.P")
    }

    pub(crate) fn fn_var_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values(args, cell, VarianceKind::Sample, false, "VAR.S")
    }

    pub(crate) fn fn_vara(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values_a(args, cell, VarianceKind::Sample, false, "VARA")
    }

    pub(crate) fn fn_varpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values_a(args, cell, VarianceKind::Population, false, "VARPA")
    }
}

/// `#DIV/0!` for too few values, otherwise the variance (or its square root), with `#NUM!`
/// for a non-finite result.
pub(crate) fn variance_result(
    values: &[f64],
    kind: VarianceKind,
    std_dev: bool,
    name: &str,
    cell: CellReferenceIndex,
) -> CalcResult {
    let minimum = match kind {
        VarianceKind::Population => 1,
        VarianceKind::Sample => 2,
    };
    if values.len() < minimum {
        let message = match kind {
            VarianceKind::Population => format!("{name} with no numeric data"),
            VarianceKind::Sample => format!("{name} requires at least two numeric values"),
        };
        return CalcResult::new_error(Error::DIV, cell, message);
    }
    let variance = excel_variance(values, kind);
    let result = if std_dev { variance.sqrt() } else { variance };
    CalcResult::finite_number_or_num(result, cell)
}

#[cfg(test)]
mod tests {
    use super::{excel_variance, VarianceKind};

    // Expected values are Excel's (oracle: excel-graph, probes/variance-*).
    #[test]
    fn one_pass_by_default() {
        let pi = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0];
        // probes/aggregate-grid B31; the two-pass value is 7.472222222222221.
        assert_eq!(
            excel_variance(&pi, VarianceKind::Population),
            7.472222222222222
        );
        // probes/variance-switch B13; 7/3 correctly rounded is 2.3333333333333335.
        assert_eq!(
            excel_variance(&[1.0, 2.0, 4.0], VarianceKind::Sample),
            2.333333333333334
        );
        // probes/variance-switch B16: just above the 1% cancellation limit.
        assert_eq!(
            excel_variance(&[11.0, 12.0, 14.0], VarianceKind::Sample),
            2.333333333333343
        );
    }

    #[test]
    fn two_pass_when_the_data_cancel() {
        let offset = [
            1e9 + 3.0,
            1e9 + 1.0,
            1e9 + 4.0,
            1e9 + 1.0,
            1e9 + 5.0,
            1e9 + 9.0,
        ];
        // probes/variance-digits A37; the one-pass form gives -113.78.
        assert_eq!(
            excel_variance(&offset, VarianceKind::Population),
            7.472222222222224
        );
        // probes/variance-switch B17.
        assert_eq!(
            excel_variance(&[101.0, 102.0, 104.0], VarianceKind::Sample),
            2.3333333333333335
        );
    }

    #[test]
    fn two_pass_when_the_variance_is_small_next_to_the_mean() {
        // probes/variance-switch A10, B10 (one-pass: 1.555555555555555E-06, 2.333333333333332E-06).
        let small = [0.001, 0.002, 0.004];
        assert_eq!(
            excel_variance(&small, VarianceKind::Population),
            1.5555555555555558e-06
        );
        assert_eq!(
            excel_variance(&small, VarianceKind::Sample),
            2.3333333333333336e-06
        );
        // probes/variance-fit G30, H30: the same data, VAR.P two-pass (one-pass: 1.225E-05) but
        // VAR.S one-pass (two-pass: 2.4499999999999996E-05), because the test uses each
        // function's own variance.
        let pair = [0.02, 0.027];
        assert_eq!(
            excel_variance(&pair, VarianceKind::Population),
            1.2249999999999998e-05
        );
        assert_eq!(excel_variance(&pair, VarianceKind::Sample), 2.45e-05);
    }

    #[test]
    fn subnormal_squares_flush_to_zero() {
        // probes/variance-underflow A6:D7: each squared deviation of {0,2.2E-154} is subnormal.
        assert_eq!(excel_variance(&[0.0, 2.2e-154], VarianceKind::Sample), 0.0);
        assert_eq!(
            excel_variance(&[0.0, 3e-154], VarianceKind::Sample),
            4.500000000000001e-308
        );
    }

    #[test]
    fn overflowing_squares_stay_non_finite() {
        // numbers.md "Overflow": VAR.P of 1E+200 twice is #NUM!, not 0.
        assert!(excel_variance(&[1e200, 1e200], VarianceKind::Population).is_nan());
    }
}
