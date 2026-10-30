use statrs::distribution::{Continuous, Gamma};

use super::erf::gamma_half_lower;
use super::gamma_x87::excel_gamma;
use super::gammaln::excel_gammaln;
use super::incomplete_gamma::{
    excel_small_shape_lower, excel_small_shape_quantile, gamma_cdf, gamma_density, gamma_quantile,
    wilson_hilferty_quantile, Tail, DIRECT_SHAPE_MAX, WILSON_HILFERTY_SHAPE_MIN,
};
use super::normal::normal_quantile;

use crate::arithmetic::{extended_quotient, flush_subnormal};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::functions::Function;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The one shape whose cdf Excel takes from its ERF kernel, on exact equality: `a = 0.5 +- 1 ulp`
/// already takes the generic path (docs/excel-semantics/statistics.md, T-1778,
/// `kmyst-erf-ashape`).
const HALF_SHAPE: f64 = 0.5;

/// The x / beta up to which GAMMA.DIST's density is Excel's log-space form at a shape from
/// [`LOG_FORM_SHAPE_MAX`] to 20 (probes/t2753-gamma-density-switch: 0 to 3 ulp from Excel up to
/// 3E-16 at shapes 0.5, 1 and 2, 4 to 7 at 1E-15, where the direct form is 4 to 12 off; T-2753).
const LOG_FORM_RATIO: f64 = 1E-15;

/// The shape below which GAMMA.DIST's density is EXP(a LN(y)) / GAMMA(a) * EXP(-y) / x at every
/// x / beta ([`log_form_density`]). A fit inside its evidence bracket (2.5, 5] (T-2852): at shapes
/// 0.5 to 2.5 that form is 0 to 1 ulp from Excel at 208 of 222 probed densities (the direct form
/// 74), at 5, 9 and 10 it is 10 to 18 off at x / beta from 1 up (probes-numeric m9-chisq-dist A48,
/// m9-gamma-dist-a9 A8), where the direct form is 0 to 1; T-3147 finds Excel's large-shape form
/// above the mode from 3.
const LOG_FORM_SHAPE_MAX: f64 = 3.0;

/// GAMMA.DIST's (and CHISQ.DIST's, at beta = 2) density in Excel's log-space forms, with Excel's
/// own LN, EXP and GAMMA (x87_log10, x87_exp, gamma_x87), or `None` where the caller's forms hold.
/// - Below [`LOG_FORM_SHAPE_MAX`], wherever y^a and e^-y are normal: EXP(a LN(y)) / GAMMA(a) *
///   EXP(-y) / x. Excel's error there is a times that of LN(y), nothing more (probes/
///   t2753-gamma-density-switch and -ratio-sweep at shapes 0.5, 1 and 2, x / beta 1E-300 to 0.5,
///   both columns: Excel's relative error minus a times LN(y)'s rounding error is within about 1 ulp at
///   every cell from 1E-16 to 1E-10), so `GAMMA.DIST(1E-14,2,1,FALSE)` is 11.8 ulp below the true
///   value, where the direct form was 17 from Excel; 0 to 2 ulp from Excel at every probed cell
///   below 1E-10 (T-2852).
/// - Otherwise, below shape 20 and at y at most [`LOG_FORM_RATIO`]: EXP(a LN(y) - y) / GAMMA(a) / x,
///   whose log carries about |LN(y)| ulp of error, so `GAMMA.DIST(1E-300,1,1,FALSE)` is
///   1.0000000000000235, not 1 (probes/t2753-gamma-density-ratio-sweep A1, B1; t2738-gamma-dist-
///   tiny-ratio A2, A4, A15, A19; T-2753). Unprobed above shape 2; it keeps to the prefactor's
///   direct shapes.
///
/// `None` at x = 0, which keeps its own value (`GAMMA.DIST(0,9,2,FALSE)` is 0, GAMMA.xlsx E13).
pub(super) fn log_form_density(alpha: f64, x: f64, beta: f64) -> Option<f64> {
    let y = x / beta;
    if !(x > 0.0) {
        return None;
    }
    if alpha < LOG_FORM_SHAPE_MAX && y.is_normal() {
        let power = excel_exp(alpha * excel_ln(y));
        let decay = excel_exp(-y);
        // Where either factor is not normal (y^a underflows below about 1E-150 at shape 2, e^-y
        // past 708) its digits are gone; the single exponential below keeps them.
        if power.is_normal() && decay.is_normal() {
            return Some(power / excel_gamma(alpha) * decay / x);
        }
    }
    (alpha < DIRECT_SHAPE_MAX && y <= LOG_FORM_RATIO)
        .then(|| excel_exp(alpha * excel_ln(y) - y) / excel_gamma(alpha) / x)
}

/// GAMMA.DIST's cdf for the gamma `dist` of scale `beta`, through Excel's identities
/// (docs/excel-semantics/statistics.md, T-1778): GAMMA.DIST(x,a,b,TRUE) = GAMMA.DIST(x/b,a,1,TRUE),
/// the quotient rounded twice like the worksheet `/` (statrs scales by a rounded 1/b instead), and
/// at shape 0.5 that is ERF.PRECISE(SQRT(x/b)). At another shape below 1 and an x/b below 1 it is
/// Excel's own leading factor over the series ([`excel_small_shape_lower`], T-3060). An x of 0,
/// and an x/b that underflows or overflows, keep the port's own branches:
/// `GAMMA.DIST(1E300,1,1E-10,TRUE)` is #NUM! and `GAMMA.DIST(1E-300,2,1E100,TRUE)` is 0
/// (probes/incomplete-gamma-cap E6, E7).
fn gamma_dist_cdf(dist: &Gamma, x: f64, beta: f64) -> f64 {
    let scaled = extended_quotient(x, beta);
    if !(scaled > 0.0 && scaled.is_finite()) {
        return gamma_cdf(dist, x);
    }
    if dist.shape() == HALF_SHAPE {
        return gamma_half_lower(scaled);
    }
    if let Some(lower) = excel_small_shape_lower(dist.shape(), scaled) {
        return lower;
    }
    Gamma::new(dist.shape(), 1.0).map_or(f64::NAN, |unit| gamma_cdf(&unit, scaled))
}

impl Model {
    /// `GAMMA(x)`: lifts over an array and a range (SPEC-0011; probes/stat-argument-modes
    /// H284-H286).
    pub(crate) fn fn_gamma(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Gamma, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                // Excel's own x87 routine (gamma_x87.rs, T-2129): NaN at the poles, infinity past
                // the double range, both #NUM!.
                let result = excel_gamma(x);
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
        // The density of a positive x whose x / beta is subnormal is #NUM!, whatever the shape or
        // the size of the result, while the cdf there is 0 (probes/t2738-gamma-dist-tiny-ratio:
        // `GAMMA.DIST(4.4E-308,1,2,FALSE)` is #NUM!, 4.5E-308 is 0.4999...; T-2738).
        // An x / beta that overflows is #NUM! for the density as for the cdf
        // (`GAMMA.DIST(1E300,1,1E-10,FALSE)` and `TRUE`, probes/incomplete-gamma-cap E18, E6; T-2982).
        if !cumulative && x > 0.0 && !(f64::MIN_POSITIVE..=f64::MAX).contains(&(x / beta_scale)) {
            return Err(num_error(cell));
        }
        let dist = Gamma::new(alpha, 1.0 / beta_scale).map_err(|_| num_error(cell))?;
        let result = if cumulative {
            // A subnormal cdf is 0, as in Excel (`(x)=0` TRUE: probes/t2706-subnormal-density-paths
            // B19:B20, probes/t2737-subnormal-cdf-shapes B13:B14).
            flush_subnormal(gamma_dist_cdf(&dist, x, beta_scale))
        } else {
            // A subnormal density is 0, as in Excel (`(x)=0` TRUE: probes-numeric/m9-gamma-dist-a500
            // B22; docs/excel-semantics/numbers.md "Distribution densities").
            flush_subnormal(
                log_form_density(alpha, x, beta_scale)
                    .or_else(|| gamma_density(alpha, x, beta_scale))
                    .unwrap_or_else(|| dist.pdf(x)),
            )
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
        // Excel's closed form at a shape below 1 and a tiny quantile, times the scale as a plain
        // final product (T-3012: column B of probes/t3012-gamma-inv-small-shape is column A times
        // 1E+15 to the double). A subnormal quantile there is #DIV/0! (A4
        // `GAMMA.INV(1E-16,0.05,1)`, about 1E-320), one that underflows is 0 (A1-A3; T-3013).
        // A scale whose rate 1 / beta overflows stays #NUM! (`gamma_quantile`).
        let closed_form = (1.0 / beta_scale)
            .is_finite()
            .then(|| excel_small_shape_quantile(alpha, p))
            .flatten();
        if let Some(x) = closed_form {
            if x > 0.0 && x < f64::MIN_POSITIVE {
                return Err(div_error(cell));
            }
            // The product cannot overflow (x is below 1E-21 and beta a finite double). A subnormal
            // product is 0, as `gamma_quantile` flushes after its scale.
            return Ok(flush_subnormal(x * beta_scale));
        }
        let x = if alpha >= WILSON_HILFERTY_SHAPE_MIN && p > 0.0 && p < 1.0 {
            let z = normal_quantile(p, 0.0, 1.0, cell)?;
            wilson_hilferty_quantile(alpha, z) * beta_scale
        } else {
            gamma_quantile(alpha, beta_scale, p, Tail::Lower)
        };
        if !x.is_finite() || x < 0.0 {
            return Err(num_error(cell));
        }
        Ok(x)
    }

    /// `GAMMALN(x)` and GAMMALN.PRECISE (one routine, `gammaln.rs`): lift over an array and a
    /// range (SPEC-0011; probes/stat-argument-modes H287-H292).
    pub(crate) fn fn_gamma_ln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::GammaLn, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                if x < 0.0 {
                    return Err(num_error(cell));
                }
                // Excel's own x87 routine (gammaln.rs, T-2128); x = 0 is NaN there, #NUM!.
                let result = excel_gammaln(x);
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }
}
