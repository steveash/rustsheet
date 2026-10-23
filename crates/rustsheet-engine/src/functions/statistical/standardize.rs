use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `STANDARDIZE(x, mean, standard_dev)`: every argument lifts over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes H500-H504, probes/stat-later-argument-lift H39).
    pub(crate) fn fn_standardize(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Standardize, args, cell, &mut |model, values| {
            number_or_error(model.standardize_of(values, cell))
        })
    }

    fn standardize_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, mean, std_dev] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // Excel reads a TRUE as 1 for x (probes/stats-evidence-floor-t1623 Q58,
        // probes/stat-later-argument-lift L14); a boolean mean or standard_dev reads as 1 or 0 too
        // (gen-m9/928d08e3e1eecb5f A249, gen-m9/334f3dc9ca937f10 A219).
        let x = self.cast_number_ref(x, cell)?;
        let mean = self.cast_number_ref(mean, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        if std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        Ok((x - mean) / std_dev)
    }
}
