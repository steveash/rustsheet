//! QUARTILE.INC/.EXC (and QUARTILE). The data argument is read and sorted once per call
//! (`sorted_data.rs`); `quart` lifts over arrays and ranges, floored per element
//! (probes/stat-argument-modes H437-H448, probes/stat-whole-lift W24-W26, W46).

use super::percentile::{
    num_error, percentile_exc_impl, percentile_inc_impl, value_at, PERCENTILE_DATA,
};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `QUARTILE.INC(array, quart)`, quart 0..=4 for 0%, 25%, 50%, 75%, 100%; also QUARTILE.
    pub(crate) fn fn_quartile_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_with_sorted_data(
            Function::QuartileInc,
            args,
            &PERCENTILE_DATA,
            cell,
            &mut |model, data, values| {
                if data.is_empty() {
                    return num_error(cell);
                }
                match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(v) => {
                        let quart = v.floor() as i64;
                        if !(0..=4).contains(&quart) {
                            return num_error(cell);
                        }
                        CalcResult::Number(percentile_inc_impl(data.sorted(), quart as f64 / 4.0))
                    }
                    Err(e) => e,
                }
            },
        )
    }

    /// `QUARTILE.EXC(array, quart)`, quart 1..=3, and PERCENTILE.EXC's range rule for quart/4.
    pub(crate) fn fn_quartile_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_with_sorted_data(
            Function::QuartileExc,
            args,
            &PERCENTILE_DATA,
            cell,
            &mut |model, data, values| {
                if data.is_empty() {
                    return num_error(cell);
                }
                match model.cast_number_ref(value_at(values, 1), cell) {
                    Ok(v) => {
                        let quart = v.floor() as i64;
                        if !(1..=3).contains(&quart) {
                            return num_error(cell);
                        }
                        percentile_exc_impl(data.sorted(), quart as f64 / 4.0)
                            .map_or_else(|| num_error(cell), CalcResult::Number)
                    }
                    Err(e) => e,
                }
            },
        )
    }
}
