//! A pivot's layout: in [`lines`], its layout lines and location (SPEC-0032 stage 7, T-4147),
//! in [`render`], the cells they give (T-4149), and in [`records_behind`], the records behind one
//! value cell (T-4184); and the aggregation GETPIVOTDATA reads until
//! it moves onto the `PivotResult` (T-4154).
//!
//! There is one layout path. A grouped spec pivot (T-4235), which has no rendered cells because
//! the case builder cannot know Excel's groups, is computed, laid out and rendered by the same
//! stages as any other pivot (the facade's `lay_out_grouped`); oracle reads of Excel's own
//! layout hold it in step (probes/t4228-pivot-range-groups, probes/t4235-pivot-range-rounding,
//! probes/t4311-pivot-showall-column-and-groups).

mod lines;
mod records;
mod render;

pub use lines::{lay_out, AxisLine, PivotLayout, MAX_PIVOT_CELLS};
pub use records::records_behind;
pub(super) use render::item_text;
pub use render::{
    render, CellFormat, PivotCell, RenderedCell, ALL_ITEMS, BLANK_ITEM, COLUMN_LABELS, GRAND_TOTAL,
    MAX_PIVOT_TEXT_BYTES, MULTIPLE_ITEMS, ROW_LABELS, VALUES,
};

use crate::expressions::token::Error;
use crate::functions::statistical::variance::{excel_variance, VarianceKind};

use super::{Aggregate, CacheField, CacheItem};

/// The data field `field`'s numbers in `records` (cache order). Every record must hold a number
/// there: an error, blank, text, boolean or date is `#REF!` (SPEC-0015 unknown 6).
pub(crate) fn record_numbers(field: &CacheField, records: &[u32]) -> Result<Vec<f64>, Error> {
    records
        .iter()
        .map(|&record| {
            match field
                .records
                .get(record as usize)
                .and_then(|&i| field.items.get(i as usize))
            {
                Some(CacheItem::Number(n)) => Ok(*n),
                _ => Err(Error::REF),
            }
        })
        .collect()
}

/// The data field's aggregate over `values` (cache order). A Sum, Count, Average, Max or Min over
/// nothing is 0 (probes/getpivotdata-real-pivot H13, probes/getpivotdata-empty-intersection);
/// any other aggregate with no defined value is `#REF!` (SPEC-0015 unknown 2), and a non-finite
/// result `#NUM!` (unknown 16). Var and `StdDev` reuse the worksheet functions' algorithm
/// (`excel_variance`); that the pivot computes them alike is UNVERIFIED (unknown 15). The pivot
/// result (`pivot::result`, T-4142) uses the textbook one-pass formula instead
/// (probes/t4616-pivot-total-order-3), so until T-4154 moves GETPIVOTDATA onto the
/// `PivotResult` the two may differ in the last bits for Var and `StdDev`.
pub(crate) fn aggregate(function: Aggregate, values: &[f64]) -> Result<f64, Error> {
    let n = values.len();
    let minimum = match function {
        Aggregate::Sum
        | Aggregate::Count
        | Aggregate::Average
        | Aggregate::Max
        | Aggregate::Min => 0,
        Aggregate::StdDev | Aggregate::Var => 2,
        _ => 1,
    };
    if n < minimum {
        return Err(Error::REF);
    }
    if n == 0 {
        return Ok(0.0);
    }
    let sum = || values.iter().fold(0.0, |acc, x| acc + x);
    let variance = |kind| excel_variance(values, kind);
    let value = match function {
        Aggregate::Sum => sum(),
        Aggregate::Count | Aggregate::CountNums => n as f64,
        Aggregate::Average => sum() / n as f64,
        Aggregate::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        Aggregate::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        Aggregate::Product => values.iter().fold(1.0, |acc, x| acc * x),
        Aggregate::Var => variance(VarianceKind::Sample),
        Aggregate::VarP => variance(VarianceKind::Population),
        Aggregate::StdDev => variance(VarianceKind::Sample).sqrt(),
        Aggregate::StdDevP => variance(VarianceKind::Population).sqrt(),
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NUM)
    }
}
