//! PERCENTILE.INC/.EXC and PERCENTRANK.INC/.EXC (and their compatibility names). The data
//! argument is read once and sorted once per call (`sorted_data.rs`); `k`, `x` and
//! `significance` lift over arrays and ranges (probes/stat-argument-modes,
//! probes/stat-whole-lift W21-W47).

use super::sorted_data::{DataArg, DataOrder, SortedNumbers};
use crate::decimal_round::{cut_to_15_digits, decimal_to_f64};
use crate::expressions::types::CellReferenceIndex;
pub(super) use crate::functions::lift::num_error;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::nested_array::nested_array_error;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The numbers of the data argument of PERCENTILE*, QUARTILE* and PERCENTRANK*: a range's
/// numbers (text, booleans and blanks skipped), an array's, a number, or nothing for an empty
/// argument; a typed boolean or text is a one-element array (see below); an error value is that error, which comes before a bad `k` (`PERCENTILE(NA(),2)` and
/// `QUARTILE.EXC(NA(),9)` are `#N/A`, probes/stat-test-lift P81, P83; T-1982); anything else is
/// `#VALUE!`.
fn percentile_numbers(
    model: &mut Model,
    arg: &Node,
    cell: CellReferenceIndex,
) -> Result<Vec<f64>, CalcResult> {
    // A name, INDEX, OFFSET, IF or CHOOSE result that is a reference reads as one (T-2344).
    let typed_in_place = !model.is_reference(arg, cell);
    match model.evaluate_node_in_context(arg, cell) {
        CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(vec![]),
        error @ CalcResult::Error { .. } => Err(error),
        // A typed boolean is a one-element array (`QUARTILE(TRUE,1)` is 1, probes-numeric/t1748-
        // typed-boolean-data A1-A8 for the whole family, probes/stat-order-floor-batch A102, A128),
        // and typed text a number (A11).
        CalcResult::Boolean(b) if typed_in_place => Ok(vec![f64::from(u8::from(b))]),
        text @ CalcResult::String(_) if typed_in_place => {
            model.cast_number_ref(&text, cell).map(|n| vec![n])
        }
        // A single cell holding text or a boolean (a reference, whether written, named or picked by
        // INDEX, OFFSET, IF or CHOOSE; not the typed values above, probes/t2344-percentile-name-
        // index-data)
        // is no number, like text inside a range: the data is empty and every function reports
        // `#NUM!`, not `#VALUE!`, after k's or quart's own error (probes/quartile-single-text-cell
        // B1-B11, quartile-numeric-text-cell C2-C5; t2429-quartile-exc-no-number A1-A3, A9-A12,
        // A19-A22 for a TRUE cell).
        CalcResult::String(_) | CalcResult::Boolean(_) => Ok(vec![]),
        CalcResult::Array(array) => match nested_array_error(&array, cell) {
            Some(error) => Err(error),
            None => model.numeric_flat_with_element_errors(CalcResult::Array(array), cell),
        },
        value => model.numeric_flat_with_element_errors(value, cell),
    }
}

/// The data of PERCENTILE*, QUARTILE* and PERCENTRANK*: smallest first, each element O(1) (a
/// binary search for PERCENTRANK) once sorted.
pub(super) const PERCENTILE_DATA: DataArg = DataArg {
    read: percentile_numbers,
    order: DataOrder::Ascending,
    cost: LiftCost::Prepared,
};

/// Argument `index` of one element's values, `EmptyArg` past the end.
pub(super) fn value_at(values: &[CalcResult], index: usize) -> &CalcResult {
    values.get(index).unwrap_or(&CalcResult::EmptyArg)
}

/// The value at the 0-based fractional `rank` of `sorted`, interpolated linearly. `rank` is in
/// `0..=len-1` (the callers check their k first).
fn interpolate(sorted: &[f64], rank: f64) -> f64 {
    let low = rank.floor() as usize;
    let high = (low + 1).min(sorted.len().saturating_sub(1));
    let frac = rank - rank.floor();
    let at = |index: usize| sorted.get(index).copied().unwrap_or(0.0);
    at(low) + frac * (at(high) - at(low))
}

/// PERCENTILE.EXC's value at `k`, `None` when k is outside `[1/(n+1), n/(n+1)]` or not in
/// (0, 1). QUARTILE.EXC uses it for quart/4.
pub(super) fn percentile_exc_impl(sorted: &[f64], k: f64) -> Option<f64> {
    let n = sorted.len() as f64;
    if k <= 0.0 || k >= 1.0 || k < 1.0 / (n + 1.0) || k > n / (n + 1.0) {
        return None;
    }
    // 0-indexed rank = k*(n+1) - 1
    Some(interpolate(sorted, k * (n + 1.0) - 1.0))
}

impl Model {
    /// `PERCENTILE.INC(array, k)`, k in [0, 1]; also PERCENTILE.
    pub(crate) fn fn_percentile_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_with_sorted_data(
            Function::PercentileInc,
            args,
            &PERCENTILE_DATA,
            cell,
            &mut |model, data, values| {
                // k's error comes before the empty-data `#NUM!`, as PERCENTILE.EXC's:
                // `PERCENTILE(<no number>,1/0)` is `#DIV/0!` (probes/quartile-numeric-text-cell C9).
                match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(_) if data.is_empty() => num_error(cell),
                    Ok(k) if (0.0..=1.0).contains(&k) => {
                        CalcResult::Number(percentile_inc_impl(data.sorted(), k))
                    }
                    Ok(_) => num_error(cell),
                    Err(e) => e,
                }
            },
        )
    }

    /// `PERCENTILE.EXC(array, k)`, k in (0, 1) and within `[1/(n+1), n/(n+1)]`.
    pub(crate) fn fn_percentile_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_with_sorted_data(
            Function::PercentileExc,
            args,
            &PERCENTILE_DATA,
            cell,
            &mut |model, data, values| {
                // k's error comes before the empty-data `#NUM!` (gen-m9/d5bd4272655c5211 A31).
                match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(_) if data.is_empty() => num_error(cell),
                    Ok(k) => percentile_exc_impl(data.sorted(), k)
                        .map_or_else(|| num_error(cell), CalcResult::Number),
                    Err(e) => e,
                }
            },
        )
    }

    /// `PERCENTRANK.INC(array, x, [significance])`; also PERCENTRANK.
    pub(crate) fn fn_percentrank_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_with_sorted_data(
            Function::PercentrankInc,
            args,
            &PERCENTILE_DATA,
            cell,
            &mut |model, data, values| {
                model.percentrank_of(data, values, percentrank_inc_impl, cell)
            },
        )
    }

    /// `PERCENTRANK.EXC(array, x, [significance])`.
    pub(crate) fn fn_percentrank_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_with_sorted_data(
            Function::PercentrankExc,
            args,
            &PERCENTILE_DATA,
            cell,
            &mut |model, data, values| {
                model.percentrank_of(data, values, percentrank_exc_impl, cell)
            },
        )
    }

    /// PERCENTRANK.INC or .EXC (`rank_of`) of one element: an error x, then an empty array `#N/A`,
    /// then an x outside the data `#N/A`, then the significance (3 digits when omitted; below 1
    /// `#NUM!` unless the data is one value).
    fn percentrank_of(
        &self,
        data: &mut SortedNumbers,
        values: &[CalcResult],
        rank_of: fn(&[f64], f64) -> f64,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // An error or text x comes before the empty-data `#N/A` (probes/quartile-single-text-cell
        // B12-B18, T-2066).
        let x = match self.cast_number_ref(value_at(values, 1), cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if data.is_empty() {
            return CalcResult::new_error(Error::NA, cell, String::new());
        }
        // No cell or coercion gives NaN; refuse one before it reaches the rank search.
        if x.is_nan() {
            return num_error(cell);
        }
        let sig = match values.get(2) {
            Some(significance) => match self.cast_number_ref(significance, cell) {
                Ok(v) => v.floor() as i32,
                Err(e) => return e,
            },
            None => 3,
        };
        let sorted = data.sorted();
        let (Some(first), Some(last)) = (sorted.first(), sorted.last()) else {
            return CalcResult::new_error(Error::NA, cell, String::new());
        };
        // Unlike the documentation, x at either end of the array is in range (EXC too). An x
        // outside the data is `#N/A` before a significance below 1 is `#NUM!`
        // (`PERCENTRANK.EXC(C4:C10,-1,0)`, gen-m9/097af1ca33638670 A312; statistics.md "Order
        // statistics: the scalar argument's error before the no-number result").
        if x < *first || x > *last {
            return CalcResult::new_error(Error::NA, cell, String::new());
        }
        if sig < 1 && sorted.len() > 1 {
            return num_error(cell);
        }
        if sorted.len() == 1 {
            // One value equal to x ranks 1 (EXC too, not 0.5); any numeric significance leaves 1 (probes/percentrank-single-value B9, B14-B19).
            return CalcResult::Number(1.0);
        }
        // A significance whose 10^sig overflows (309 or more) is `#DIV/0!` (308 still computes:
        // probes/t2473-percentrank-significance A6-A15, B6-B15, C6-C15).
        // 10^sig correctly rounded: `powi` drifts by ulps past 10^22 and moves the result
        // (`PERCENTRANK({1,2,3,4},2.5,100)` is 0.5, t2473 A5). `None` is a power past the double
        // range.
        let factor = decimal_to_f64(1, sig.into()).unwrap_or(f64::INFINITY);
        if !factor.is_finite() {
            return CalcResult::new_error(Error::DIV, cell, String::new());
        }
        // The scaled rank is cut to 15 significant digits before the floor, so a rank within
        // rounding below a cut keeps its digit: `PERCENTRANK.EXC({1E+15,1,-1E+15},0.3)` (the double
        // 0.49999999999999983) is 0.5, and 2/3 at 15 digits 0.666666666666667 (t2473 D1-D4, D19-D23;
        // statistics.md "PERCENTRANK: a huge significance is #DIV/0!").
        let scaled = cut_to_15_digits(rank_of(sorted, x) * factor, true);
        CalcResult::Number(scaled.floor() / factor)
    }
}

/// PERCENTILE.INC interpolation: rank = k*(n-1) in 0-indexed sorted array.
pub(crate) fn percentile_inc_impl(sorted: &[f64], k: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let rank = k * (n - 1) as f64;
    let low = rank.floor() as usize;
    let high = (low + 1).min(n - 1);
    let frac = rank - rank.floor();
    sorted[low] + frac * (sorted[high] - sorted[low])
}

/// PERCENTRANK.INC: virtual position in sorted array, divided by (n-1).
pub(crate) fn percentrank_inc_impl(sorted: &[f64], x: f64) -> f64 {
    let n = sorted.len();
    // Number of values strictly less than x
    let k = sorted.partition_point(|&v| v < x);

    let (Some(below), Some(at)) = (k.checked_sub(1).and_then(|i| sorted.get(i)), sorted.get(k))
    else {
        // x matches the smallest value (k = 0) or is at least every value (k = n).
        return k as f64 / n.saturating_sub(1) as f64;
    };
    if *at == x {
        return k as f64 / (n - 1) as f64;
    }

    // x is between sorted[k-1] and sorted[k]
    let frac = (x - below) / (at - below);
    (k as f64 - 1.0 + frac) / (n - 1) as f64
}

/// PERCENTRANK.EXC: rank = (`virtual_position` + 1) / (n + 1).
pub(crate) fn percentrank_exc_impl(sorted: &[f64], x: f64) -> f64 {
    let n = sorted.len();
    let k = sorted.partition_point(|&v| v < x);

    let below = k.checked_sub(1).and_then(|i| sorted.get(i));
    let virtual_pos = match (below, sorted.get(k)) {
        (_, Some(at)) if *at == x => k as f64,
        (Some(below), Some(at)) => {
            let frac = (x - below) / (at - below);
            k as f64 - 1.0 + frac
        }
        _ => k as f64,
    };

    (virtual_pos + 1.0) / (n + 1) as f64
}
