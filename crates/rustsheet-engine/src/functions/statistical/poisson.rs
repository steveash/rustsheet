use statrs::distribution::{Discrete, Poisson};

use super::incomplete_gamma::gamma_ur;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `POISSON.DIST(x, mean, cumulative)`, and POISSON: every argument lifts over an array and a
    /// range (SPEC-0011; probes/stat-argument-modes H476-H485, probes/stat-quantile-lift
    /// H47-H50). The cdf is statrs' Q(k + 1, mean), the capped port of `incomplete_gamma.rs`,
    /// which the lifting table charges (lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT`).
    pub(crate) fn fn_poisson_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::PoissonDist, args, cell, &mut |model, values| {
            number_or_error(model.poisson_dist_of(values, cell))
        })
    }

    fn poisson_dist_of(
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

        if x < 0.0 {
            return Err(num_error(cell));
        }
        // An x past u64 (the count truncates to one) is a cdf of 1 for a mean of 0 or more
        // (`POISSON.DIST(2E+29,7,-1)` is 1, probes/t2475-stat-subexpressions P!A216); the
        // pmf there, and a negative mean, stay #NUM! (unprobed).
        if x > (u64::MAX as f64) {
            return if cumulative && lambda >= 0.0 {
                Ok(1.0)
            } else {
                Err(num_error(cell))
            };
        }

        let k = x.trunc() as u64;

        // A negative mean is #NUM! unless x truncates to 0: then both the pmf and the cdf are
        // EXP(-mean), which only P(X = 0) = e^-mean gives (`POISSON(0,-1,0)` = 2.718281828459045,
        // `POISSON(0.3,-0.5,TRUE)` = 1.6487212707001282; gen-m9 607e85cfee83a689 A189, b9a32ce9a1fd0d71
        // A221; statistics.md "Scalar distributions at extreme arguments, part 2").
        if lambda < 0.0 {
            let prob = (-lambda).exp();
            return if k == 0 && prob.is_finite() {
                Ok(prob)
            } else {
                Err(num_error(cell))
            };
        }

        // A mean of 0 is the distribution at 0: P(X <= x) = 1, P(X = 0) = 1, P(X = k > 0) = 0.
        if lambda == 0.0 {
            return Ok(if cumulative || k == 0 { 1.0 } else { 0.0 });
        }

        let dist = Poisson::new(lambda).map_err(|_| num_error(cell))?;
        let prob = if cumulative {
            // statrs' `Poisson::cdf`.
            gamma_ur(k as f64 + 1.0, lambda)
        } else {
            dist.pmf(k)
        };
        if !prob.is_finite() {
            return Err(num_error(cell));
        }
        Ok(prob)
    }
}
