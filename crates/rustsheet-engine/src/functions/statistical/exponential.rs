use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `EXPON.DIST(x, lambda, cumulative)`, and EXPONDIST: every argument lifts over an array
    /// and a range (SPEC-0011; probes/stat-argument-modes H205-H214,
    /// probes/stat-later-argument-lift H41, H43).
    pub(crate) fn fn_expon_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::ExponDist, args, cell, &mut |model, values| {
            number_or_error(model.expon_dist_of(values, cell))
        })
    }

    fn expon_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, lambda, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let lambda = self.cast_number_ref(lambda, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if x < 0.0 || lambda <= 0.0 {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            // CDF: expm1 avoids 1 - exp(-t) cancellation for tiny lambda*x (EXPONDIST.xlsx D15/D16)
            -(-lambda * x).exp_m1()
        } else {
            lambda * (-lambda * x).exp()
        };
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}
