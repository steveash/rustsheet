use statrs::distribution::{Continuous, ContinuousCDF, Weibull};

use crate::arithmetic::flush_subnormal;
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
        // A TRUE x is 1: WEIBULL(TRUE,1,1,TRUE) = 0.6321205588285577 (probes-numeric/t1048-weibull
        // A25, t1049-weibulldist A33).
        let x = self.cast_number_ref(x, cell)?;
        // A boolean alpha or beta reads as 1 or 0 (gen-m9/d37313406b9f6e55 A380, 928d08e3e1eecb5f A303).
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta = self.cast_number_ref(beta, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if x < 0.0 || alpha <= 0.0 || beta <= 0.0 {
            return Err(num_error(cell));
        }
        // The density and the cdf at x = 0 are 0 for every shape (t1048-weibull A16, A17;
        // t1049-weibulldist A27, A28; `WEIBULL.DIST(0,1E+8,4.6E-09,TRUE)` is 0 where statrs'
        // beta^-alpha overflows, probes/t2475-stat-subexpressions P!A78).
        if x == 0.0 {
            return Ok(0.0);
        }
        // Excel reports the overflow of (x/beta)^alpha, though the cdf's limit is 1:
        // `WEIBULL(1E+15,1E+15,TRUE,1)` and `WEIBULL.DIST(7,1E+8,0.3,"TRUE")` are #NUM!
        // (probes/t2475-stat-subexpressions P!A407, probes/t2496-fin-subexpressions P!A90).
        let t = (x / beta).powf(alpha);
        if !t.is_finite() {
            return Err(num_error(cell));
        }
        // statrs::Weibull: shape = k (alpha), scale = lambda (beta)
        let dist = Weibull::new(alpha, beta).map_err(|_| num_error(cell))?;
        // statrs forms x^alpha * beta^-alpha, which is inf * 0 = NaN where one power overflows
        // and the other underflows. Excel's cdf is a number there, as 1 - EXP(-(x/beta)^alpha):
        // `WEIBULL(0.2,1E+15,0.2,TRUE)` is 0.6321205588285577 (probes/t2436/abc2939ebd1e2365
        // A196) and `WEIBULL(1E+15,42,10000000001,TRUE)` a number, read through
        // `VLOOKUP(..,TAN(3.14159))`'s `#VALUE!` (gen-m9 7f7ca726c3f68554 A308). Its density
        // there stays #NUM! (`WEIBULL(3,3.14159E+15,<about 1.5E+6>,FALSE)`, 9e2225ead8a6a07b A340).
        let result = if cumulative {
            let p = dist.cdf(x);
            if p.is_nan() {
                -(-t).exp_m1()
            } else {
                p
            }
        } else {
            // A subnormal density is 0, as in Excel (`(x)=0` TRUE: probes/t2693-subnormal-weibull
            // A2; docs/excel-semantics/numbers.md "Distribution densities").
            flush_subnormal(dist.pdf(x))
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}
