use crate::functions::util::aggregate_args::{
    AggregatePolicy, AggregateWalk, ArrayErrorRank, ArrayNumbers, CellNumbers, DirectNumbers,
    ErrorRule, ItemSource, NumberRules,
};
use std::cmp::Ordering;

use crate::arithmetic::{extended_sum, flush_subnormal};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The AVERAGE family: a referenced error ranks below a direct scalar error whichever comes
/// first, the leftmost reference error wins, and an array error returns at once
/// (docs/excel-semantics/syntax.md "Aggregate error precedence"; probes/aggregate-error-precedence
/// C25-C31; the array rank is UNVERIFIED, T-1439).
const AVERAGE_ARGS: AggregatePolicy = AggregatePolicy {
    errors: ErrorRule::Ranked {
        array_errors: ArrayErrorRank::Direct,
    },
    skip_empty_args: false,
};

/// DEVSQ, AVEDEV and SUMSQ: an empty argument is skipped and the first error returns at once,
/// as when each argument was walked on its own (TODO(T-0236); T-1439).
const SKIP_EMPTY_ARGS: AggregatePolicy = AggregatePolicy {
    errors: ErrorRule::First,
    skip_empty_args: true,
};

/// AVERAGEA, MAXA and MINA: the first error returns at once (their error order is unprobed,
/// T-1566). The A rules' visitor errs on a referenced cell holding a range ("Unexpected Range").
const A_ARGS: AggregatePolicy = AggregatePolicy {
    errors: ErrorRule::First,
    skip_empty_args: false,
};

/// The AVERAGE family's coercion: a direct number, boolean, text or empty argument counts, and
/// only numbers in references and arrays (coercion.md "AVERAGE, MEDIAN, GEOMEAN, …";
/// probes/average-value-args).
const AVERAGE_NUMBERS: NumberRules = NumberRules {
    direct: DirectNumbers::CastScalars,
    cells: CellNumbers::Numbers,
    array: ArrayNumbers::Numbers,
};

impl Model {
    /// The numbers of the AVERAGE family's arguments (AVERAGE, GEOMEAN, HARMEAN, MEDIAN, SKEW,
    /// KURT, VAR/STDEV): SPEC-0009's walker with [`AVERAGE_ARGS`] and [`AVERAGE_NUMBERS`].
    pub(crate) fn for_each_value<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_aggregate_number(args, cell, AVERAGE_ARGS, AVERAGE_NUMBERS, f)
    }

    /// `for_each_value` that skips empty arguments (`DEVSQ(1, )`) instead of counting them as 0,
    /// and returns the first error at once. DEVSQ, AVEDEV and SUMSQ ignored them before they
    /// moved onto `for_each_value` and there is no Excel evidence either way yet. TODO(T-0236):
    /// probe empty arguments and fold this into `for_each_value`.
    pub(crate) fn for_each_value_skip_empty<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        self.for_each_aggregate_number(args, cell, SKIP_EMPTY_ARGS, AVERAGE_NUMBERS, f)
    }

    /// The values AVERAGEA, MAXA and MINA read from `args`: a reference's cells and a direct
    /// argument coerce text to 0 and booleans to 1/0 (coercion.md "MAXA / MINA"), while an array
    /// value's elements follow `array_text` (statistics.md "AVERAGEA, MAXA and MINA over an array").
    fn for_each_value_a<F>(
        &mut self,
        args: &[Node],
        array_text: ArrayNumbers,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        let rules = NumberRules {
            direct: DirectNumbers::CastA,
            cells: CellNumbers::CoercedA,
            array: array_text,
        };
        let mut walk = AggregateWalk::new(A_ARGS);
        let mut visit = |model: &mut Self, source, value| {
            if let Some(number) = model.aggregate_number(rules, source, value, cell)? {
                f(number);
            }
            Ok(())
        };
        for arg in args {
            // A cached range of a linked workbook is a reference, not an array: its text counts 0
            // and its booleans 1/0 (`AVERAGEA([1]Uses!$A$1:$B$3)` is 3.2;
            // fixtures/external-link-cached-ranges Refs!B53).
            if let Some(cells) = self.external_reference_cells(arg, cell) {
                for value in cells? {
                    let value = match value {
                        ArrayNode::Error(error) => {
                            return Err(CalcResult::new_error(
                                error,
                                cell,
                                "Error in external reference".to_string(),
                            ))
                        }
                        ArrayNode::Empty => continue,
                        ArrayNode::Number(value) => CalcResult::Number(value),
                        ArrayNode::Boolean(b) => CalcResult::Boolean(b),
                        ArrayNode::String(s) => CalcResult::String(s),
                    };
                    visit(self, ItemSource::ReferencedCell, value)?;
                }
                continue;
            }
            // An XLOOKUP or INDEX that picks a cell is that cell: text there counts 0
            // (`AVERAGEA(XLOOKUP(1,A1:A2,B1:B2),5,7)` over "x" is 4;
            // probes/subtotal-reference-operands-d H22).
            self.walk_aggregate_arg(arg, cell, &mut walk, &mut visit)?;
        }
        walk.finish()
    }

    pub(crate) fn fn_average(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut sum = 0.0;
        if let Err(e) = self.for_each_value(args, cell, |f| {
            count += 1.0;
            sum = extended_sum(sum, f);
        }) {
            return e;
        }

        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::finite_number_or_num(flush_subnormal(sum / count), cell)
    }

    pub(crate) fn fn_averagea(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut sum = 0.0;
        // Coerced as MAXA coerces, a cached range of a linked workbook included
        // (`AVERAGEA([1]Uses!$A$1:$B$3)` is 3.2; fixtures/external-link-cached-ranges Refs!B53).
        if let Err(error) = self.for_each_value_a(args, ArrayNumbers::TextZero, cell, |value| {
            count += 1.0;
            sum += value;
        }) {
            return error;
        }
        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::Number(flush_subnormal(sum / count))
    }

    pub(crate) fn fn_avedev(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        let mut sum = 0.0;
        let mut count: u64 = 0;

        #[inline]
        fn accumulate(values: &mut Vec<f64>, sum: &mut f64, count: &mut u64, value: f64) {
            values.push(value);
            *sum += value;
            *count += 1;
        }

        if let Err(error) = self.for_each_value_skip_empty(args, cell, |value| {
            accumulate(&mut values, &mut sum, &mut count, value);
        }) {
            return error;
        }

        if count == 0 {
            // Excel: #NUM!, not #DIV/0! (docs/excel-semantics/coercion.md, poi EverythingTests!F176).
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "AVEDEV with no numeric data".to_string(),
            );
        }

        let n = count as f64;
        let mean = sum / n;

        let mut sum_abs_dev = 0.0;
        for v in &values {
            sum_abs_dev += (v - mean).abs();
        }

        CalcResult::Number(sum_abs_dev / n)
    }

    pub(crate) fn fn_median(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        if values.is_empty() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "No numeric values for MEDIAN".to_string(),
            };
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let n = values.len();
        let median = if n % 2 == 1 {
            // odd
            values[n / 2]
        } else {
            // even: average of the two middle values
            let a = values[(n / 2) - 1];
            let b = values[n / 2];
            // Halve first when the sum would overflow: Excel gives 9E+307 for two 9E+307 values.
            let sum = a + b;
            if sum.is_finite() {
                sum / 2.0
            } else {
                a / 2.0 + b / 2.0
            }
        };

        CalcResult::Number(median)
    }

    pub(crate) fn fn_harmean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        if values.is_empty() {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }

        // Excel HARMEAN: all values must be > 0
        if values.iter().any(|&v| v <= 0.0) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "HARMEAN requires strictly positive values".to_string(),
            };
        }

        let n = values.len() as f64;
        let sum_recip: f64 = values.iter().map(|v| 1.0 / v).sum();

        if sum_recip == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }

        CalcResult::Number(n / sum_recip)
    }

    pub(crate) fn fn_mina(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut mina: Option<f64> = None;
        if let Err(e) = self.for_each_value_a(args, ArrayNumbers::Numbers, cell, |f| {
            if let Some(m) = mina {
                mina = Some(m.min(f));
            } else {
                mina = Some(f);
            }
        }) {
            return e;
        }
        // No value at all is 0, as for MAX (`MINA(Z1:Z3)` over blanks and `MINA({"a"})` are 0;
        // probes/averagea-array-constant-elements A22:A25).
        CalcResult::Number(mina.unwrap_or(0.0))
    }

    pub(crate) fn fn_maxa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut maxa: Option<f64> = None;
        if let Err(e) = self.for_each_value_a(args, ArrayNumbers::Numbers, cell, |f| {
            if let Some(m) = maxa {
                maxa = Some(m.max(f));
            } else {
                maxa = Some(f);
            }
        }) {
            return e;
        }
        // No value at all is 0, as for MAX (`MAXA(Z1:Z3)` over blanks and `MAXA({"a"})` are 0;
        // probes/averagea-array-constant-elements A22:A25).
        CalcResult::Number(maxa.unwrap_or(0.0))
    }

    pub(crate) fn fn_skew(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Sample skewness (Excel SKEW)
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        let n = values.len();
        if n < 3 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "SKEW requires at least 3 data points".to_string(),
            };
        }

        let n_f = n as f64;
        let mean = values.iter().sum::<f64>() / n_f;

        let mut m2 = 0.0;
        for &x in &values {
            let d = x - mean;
            m2 += d * d;
        }

        if m2 == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Zero variance in SKEW".to_string(),
            };
        }

        let s = (m2 / (n_f - 1.0)).sqrt();

        let mut sum_cubed = 0.0;
        for &x in &values {
            let z = (x - mean) / s;
            sum_cubed += z * z * z;
        }

        let skew = (n_f / ((n_f - 1.0) * (n_f - 2.0))) * sum_cubed;
        CalcResult::Number(skew)
    }

    pub(crate) fn fn_skew_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Population skewness (Excel SKEW.P)
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        let n = values.len();
        if n < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "SKEW.P requires at least 2 data points".to_string(),
            };
        }

        let n_f = n as f64;
        let mean = values.iter().sum::<f64>() / n_f;

        let mut m2 = 0.0;
        for &x in &values {
            let d = x - mean;
            m2 += d * d;
        }

        if m2 == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Zero variance in SKEW.P".to_string(),
            };
        }

        let sigma = (m2 / n_f).sqrt();

        let mut sum_cubed = 0.0;
        for &x in &values {
            let z = (x - mean) / sigma;
            sum_cubed += z * z * z;
        }

        let skew_p = sum_cubed / n_f;
        CalcResult::Number(skew_p)
    }

    pub(crate) fn fn_kurt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        let n = values.len();
        if n < 4 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "KURT requires at least 4 data points".to_string(),
            };
        }

        let n_f = n as f64;
        let mean = values.iter().sum::<f64>() / n_f;

        let mut m2 = 0.0;
        for &x in &values {
            let d = x - mean;
            m2 += d * d;
        }

        if m2 == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Zero variance in KURT".to_string(),
            };
        }

        let s = (m2 / (n_f - 1.0)).sqrt();

        let mut sum_fourth = 0.0;
        for &x in &values {
            let z = (x - mean) / s;
            sum_fourth += z * z * z * z;
        }

        let term1 = (n_f * (n_f + 1.0)) / ((n_f - 1.0) * (n_f - 2.0) * (n_f - 3.0)) * sum_fourth;
        let term2 = 3.0 * (n_f - 1.0) * (n_f - 1.0) / ((n_f - 2.0) * (n_f - 3.0));

        let kurt = term1 - term2;
        CalcResult::Number(kurt)
    }

    /// The `k` of LARGE/SMALL: booleans and numeric text coerce (TRUE is 1) and anything below 1
    /// is `#NUM!` before rounding. A fractional k then picks the `ceil(k)`-th largest but the
    /// `floor(k)`-th smallest, and `#NUM!` when `ceil(k)` exceeds the count, so the callers
    /// round it themselves. Evidence: probes/large-small-k (excel-graph),
    /// docs/excel-semantics/statistics.md.
    fn large_small_k(&mut self, node: &Node, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let k = self.get_number(node, cell)?;
        if k < 1.0 {
            return Err(CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "K must be >= 1".to_string(),
            });
        }
        Ok(k)
    }

    pub(crate) fn fn_large(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.numbers_from_reference_arg(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let k = match self.large_small_k(&args[1], cell) {
            Ok(k) => k,
            Err(e) => return e,
        };
        let mut numeric_values: Vec<f64> = values.into_iter().flatten().collect();
        if numeric_values.is_empty() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "No numeric values for LARGE".to_string(),
            };
        }
        numeric_values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
        let k_usize = k.ceil() as usize;
        if k_usize > numeric_values.len() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "K is larger than the number of data points".to_string(),
            };
        }
        CalcResult::Number(numeric_values[k_usize - 1])
    }

    pub(crate) fn fn_small(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let values = match self.numbers_from_reference_arg(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let k = match self.large_small_k(&args[1], cell) {
            Ok(k) => k,
            Err(e) => return e,
        };

        let mut numeric_values: Vec<f64> = values.into_iter().flatten().collect();
        if numeric_values.is_empty() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "No numeric values for SMALL".to_string(),
            };
        }

        // For SMALL, sort ascending
        numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        // Excel takes the floor for SMALL but still refuses a k whose ceiling is out of range.
        let k_usize = k.floor() as usize;
        if k.ceil() as usize > numeric_values.len() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "K is larger than the number of data points".to_string(),
            };
        }

        CalcResult::Number(numeric_values[k_usize - 1])
    }
}
