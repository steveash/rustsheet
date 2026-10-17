//! TRIMMEAN. The data argument is read and sorted once per call (`sorted_data.rs`); `percent`
//! lifts over arrays and ranges (probes/stat-argument-modes H549-H550, probes/stat-whole-lift
//! W35-W37).

use super::percentile::{num_error, value_at};
use super::sorted_data::{DataArg, DataOrder};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// TRIMMEAN's data, smallest first. Each element averages the values it keeps, so a call is
/// charged elements × numbers (`LiftCost::Cells`), not the sort alone.
const TRIMMEAN_DATA: DataArg = DataArg {
    read: |model, node, cell| model.collect_numeric_flat(node, cell),
    order: DataOrder::Ascending,
    cost: LiftCost::Cells,
};

impl Model {
    /// `TRIMMEAN(array, percent)`: the mean of the data after removing `floor(n*percent/2)`
    /// values from each end.
    pub(crate) fn fn_trimmean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_with_sorted_data(
            Function::Trimmean,
            args,
            &TRIMMEAN_DATA,
            cell,
            &mut |model, sorted, values| {
                if sorted.is_empty() {
                    return num_error(cell);
                }
                let percent = match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                if !(0.0..1.0).contains(&percent) {
                    return num_error(cell);
                }
                let n = sorted.len();
                // Number of values to remove from each end (floored)
                let trim = (n as f64 * percent / 2.0).floor() as usize;
                match sorted.get(trim..n.saturating_sub(trim)) {
                    Some(trimmed) if !trimmed.is_empty() => {
                        CalcResult::Number(trimmed.iter().sum::<f64>() / trimmed.len() as f64)
                    }
                    _ => num_error(cell),
                }
            },
        )
    }
}
