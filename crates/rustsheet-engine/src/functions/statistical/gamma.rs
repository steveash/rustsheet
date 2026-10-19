use statrs::distribution::{Continuous, Gamma};
use statrs::function::gamma::gamma;

use super::erf::gamma_half_lower;
use super::incomplete_gamma::{gamma_cdf, gamma_inverse_cdf};
use super::special::ln_gamma;

use crate::arithmetic::extended_quotient;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The one shape whose cdf Excel takes from its ERF kernel, on exact equality: `a = 0.5 +- 1 ulp`
/// already takes the generic path (docs/excel-semantics/statistics.md, T-1778,
/// `kmyst-erf-ashape`).
const HALF_SHAPE: f64 = 0.5;

/// GAMMA.DIST's cdf for the gamma `dist` of scale `beta`, through Excel's identities
/// (docs/excel-semantics/statistics.md, T-1778): GAMMA.DIST(x,a,b,TRUE) = GAMMA.DIST(x/b,a,1,TRUE),
/// the quotient rounded twice like the worksheet `/` (statrs scales by a rounded 1/b instead), and
/// at shape 0.5 that is ERF.PRECISE(SQRT(x/b)). An x of 0, and an x/b that underflows or
/// overflows, keep the port's own branches: `GAMMA.DIST(1E300,1,1E-10,TRUE)` is #NUM! and
/// `GAMMA.DIST(1E-300,2,1E100,TRUE)` is 0 (probes/incomplete-gamma-cap E6, E7).
fn gamma_dist_cdf(dist: &Gamma, x: f64, beta: f64) -> f64 {
    let scaled = extended_quotient(x, beta);
    if !(scaled > 0.0 && scaled.is_finite()) {
        return gamma_cdf(dist, x);
    }
    if dist.shape() == HALF_SHAPE {
        return gamma_half_lower(scaled);
    }
    Gamma::new(dist.shape(), 1.0).map_or(f64::NAN, |unit| gamma_cdf(&unit, scaled))
}

impl Model {
    /// `GAMMA(x)`: lifts over an array and a range (SPEC-0011; probes/stat-argument-modes
    /// H284-H286). The Lanczos sum is closed form.
    pub(crate) fn fn_gamma(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Gamma, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                if x < 0.0 && x.floor() == x {
                    return Err(num_error(cell));
                }
                let result = gamma(x);
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }

    /// `GAMMA.DIST(x, alpha, beta, cumulative)`, and GAMMADIST: every argument lifts over an
    /// array and a range (SPEC-0011; probes/stat-argument-modes H293-H304,
    /// probes/stat-quantile-lift H39-H42). The cdf is the capped port of statrs' incomplete gamma
    /// (`incomplete_gamma.rs`), which the lifting table charges (lift/kinds/statistical.rs
    /// `INCOMPLETE_GAMMA_WEIGHT`).
    pub(crate) fn fn_gamma_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::GammaDist, args, cell, &mut |model, values| {
            number_or_error(model.gamma_dist_of(values, cell))
        })
    }

    fn gamma_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, alpha, beta_scale, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta_scale = self.cast_number_ref(beta_scale, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        // x < 0, alpha <= 0 and beta <= 0 are #NUM!.
        if x < 0.0 || alpha <= 0.0 || beta_scale <= 0.0 {
            return Err(num_error(cell));
        }
        let dist = Gamma::new(alpha, 1.0 / beta_scale).map_err(|_| num_error(cell))?;
        let result = if cumulative {
            gamma_dist_cdf(&dist, x, beta_scale)
        } else {
            dist.pdf(x)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `GAMMA.INV(probability, alpha, beta)`, and GAMMAINV: every argument lifts over an array
    /// and a range (SPEC-0011; probes/stat-argument-modes H305-H314, probes/stat-quantile-lift
    /// H43-H46). The quantile is the capped port of statrs' search (`incomplete_gamma.rs`), which
    /// the lifting table charges (lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT`).
    pub(crate) fn fn_gamma_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::GammaInv, args, cell, &mut |model, values| {
            number_or_error(model.gamma_inv_of(values, cell))
        })
    }

    fn gamma_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, alpha, beta_scale] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let p = self.cast_number_ref(p, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta_scale = self.cast_number_ref(beta_scale, cell)?;
        // probability outside [0, 1], alpha <= 0 and beta <= 0 are #NUM!.
        if !(0.0..=1.0).contains(&p) || alpha <= 0.0 || beta_scale <= 0.0 {
            return Err(num_error(cell));
        }
        let dist = Gamma::new(alpha, 1.0 / beta_scale).map_err(|_| num_error(cell))?;
        let x = gamma_inverse_cdf(&dist, p);
        if !x.is_finite() || x < 0.0 {
            return Err(num_error(cell));
        }
        Ok(x)
    }

    /// `GAMMALN(x)` and GAMMALN.PRECISE: lift over an array and a range (SPEC-0011;
    /// probes/stat-argument-modes H287-H292).
    pub(crate) fn fn_gamma_ln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::GammaLn, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                if x < 0.0 {
                    return Err(num_error(cell));
                }
                let result = ln_gamma(x);
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }
}
