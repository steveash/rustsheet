//! PERCENTILE.INC/.EXC and PERCENTRANK.INC/.EXC (and their compatibility names). The data
//! argument is read once and sorted once per call (`sorted_data.rs`); `k`, `x` and
//! `significance` lift over arrays and ranges (probes/stat-argument-modes,
//! probes/stat-whole-lift W21-W47).

use super::sorted_data::{DataArg, DataOrder};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The numbers of the data argument of PERCENTILE*, QUARTILE* and PERCENTRANK*: a range's
/// numbers (text, booleans and blanks skipped), an array's, a number, or nothing for an empty
/// argument; anything else is `#VALUE!`.
fn percentile_numbers(
    model: &mut Model,
    arg: &Node,
    cell: CellReferenceIndex,
) -> Result<Vec<f64>, CalcResult> {
    match model.evaluate_node_in_context(arg, cell) {
        CalcResult::Range { left, right } => {
            let v = model.values_from_range(left, right)?;
            Ok(v.into_iter().flatten().collect::<Vec<_>>())
        }
        CalcResult::Array(arr) => match model.values_from_array(arr) {
            Ok(v) => Ok(v.into_iter().flatten().collect::<Vec<_>>()),
            Err(e) => Err(CalcResult::new_error(Error::VALUE, cell, format!("{e:?}"))),
        },
        CalcResult::Number(n) => Ok(vec![n]),
        CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(vec![]),
        _ => Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "Expected numeric range".to_string(),
        )),
    }
}

/// The data of PERCENTILE*, QUARTILE* and PERCENTRANK*: smallest first, each element O(1) (a
/// binary search for PERCENTRANK) once sorted.
pub(super) const PERCENTILE_DATA: DataArg = DataArg {
    read: percentile_numbers,
    order: DataOrder::Ascending,
    cost: LiftCost::Prepared,
};

/// The `#NUM!` of an element out of its function's domain; an element error allocates nothing.
pub(super) fn num_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, String::new())
}

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
            &mut |model, sorted, values| {
                if sorted.is_empty() {
                    return num_error(cell);
                }
                match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(k) if (0.0..=1.0).contains(&k) => {
                        CalcResult::Number(percentile_inc_impl(sorted, k))
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
            &mut |model, sorted, values| {
                if sorted.is_empty() {
                    return num_error(cell);
                }
                match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(k) => percentile_exc_impl(sorted, k)
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
            &mut |model, sorted, values| {
                model.percentrank_of(sorted, values, percentrank_inc_impl, cell)
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
            &mut |model, sorted, values| {
                model.percentrank_of(sorted, values, percentrank_exc_impl, cell)
            },
        )
    }

    /// PERCENTRANK.INC or .EXC (`rank_of`) of one element: an empty array is `#N/A`, then x,
    /// then the significance (3 digits when omitted; below 1 `#NUM!` unless the data is one
    /// value), then an x outside the data is `#N/A`.
    fn percentrank_of(
        &self,
        sorted: &[f64],
        values: &[CalcResult],
        rank_of: fn(&[f64], f64) -> f64,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (Some(first), Some(last)) = (sorted.first(), sorted.last()) else {
            return CalcResult::new_error(Error::NA, cell, String::new());
        };
        let x = match self.cast_number_ref(value_at(values, 1), cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let sig = match values.get(2) {
            Some(significance) => match self.cast_number_ref(significance, cell) {
                Ok(v) => {
                    let s = v.floor() as i32;
                    if s < 1 && sorted.len() > 1 {
                        return num_error(cell);
                    }
                    s
                }
                Err(e) => return e,
            },
            None => 3,
        };
        // Unlike the documentation, x at either end of the array is in range (EXC too).
        if x < *first || x > *last {
            return CalcResult::new_error(Error::NA, cell, String::new());
        }
        if sorted.len() == 1 {
            // One value equal to x ranks 1 (EXC too, not 0.5); any numeric significance leaves 1 (probes/percentrank-single-value B9, B14-B19).
            return CalcResult::Number(1.0);
        }
        let factor = 10f64.powi(sig);
        CalcResult::Number((rank_of(sorted, x) * factor).floor() / factor)
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

    if k == n || (k < n && sorted[k] == x) {
        // x exactly matches sorted[k] or x >= all values
        return k as f64 / (n - 1) as f64;
    }

    // x is between sorted[k-1] and sorted[k]
    let frac = (x - sorted[k - 1]) / (sorted[k] - sorted[k - 1]);
    (k as f64 - 1.0 + frac) / (n - 1) as f64
}

/// PERCENTRANK.EXC: rank = (`virtual_position` + 1) / (n + 1).
pub(crate) fn percentrank_exc_impl(sorted: &[f64], x: f64) -> f64 {
    let n = sorted.len();
    let k = sorted.partition_point(|&v| v < x);

    let virtual_pos = if k < n && sorted[k] == x {
        k as f64
    } else if k > 0 && k < n {
        let frac = (x - sorted[k - 1]) / (sorted[k] - sorted[k - 1]);
        k as f64 - 1.0 + frac
    } else {
        k as f64
    };

    (virtual_pos + 1.0) / (n + 1) as f64
}
