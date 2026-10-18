use statrs::distribution::{Continuous, ContinuousCDF, Weibull};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `WEIBULL.DIST(x, alpha, beta, cumulative)`, and WEIBULL: every argument lifts over an
    /// array and a range (SPEC-0011; probes/stat-argument-modes H552-H563,
    /// probes/stat-later-argument-lift H45, H47). The kernel is closed form.
    pub(crate) fn fn_weibull_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::WeibullDist, args, cell, &mut |model, values| {
            number_or_error(model.weibull_dist_of(values, cell))
        })
    }

    fn weibull_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, alpha, beta, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_no_bools_ref(x, cell)?;
        let alpha = self.cast_number_no_bools_ref(alpha, cell)?;
        let beta = self.cast_number_no_bools_ref(beta, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if x < 0.0 || alpha <= 0.0 || beta <= 0.0 {
            return Err(num_error(cell));
        }
        // statrs::Weibull: shape = k (alpha), scale = lambda (beta)
        let dist = Weibull::new(alpha, beta).map_err(|_| num_error(cell))?;
        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}
