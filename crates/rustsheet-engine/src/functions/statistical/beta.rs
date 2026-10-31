use statrs::distribution::{Beta, Continuous};

use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error, value_error};
use crate::functions::statistical::beta_density::beta_density_small_shape;
use crate::functions::statistical::bratio::{beta_dist_lower_tail, beta_dist_tails};
use crate::functions::statistical::incomplete_beta::beta_density;
use crate::functions::statistical::inverse_beta::{beta_inverse_cdf, beta_quantile_crossing};
use crate::functions::statistical::loader::excel_beta_density;
use crate::functions::statistical::special::refine_increasing;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// `statrs`' `inverse_cdf` panics from p about 1E-200, does not finish for BETA.INV(1E-100,1000,1)
/// nor for a shape of 1E8 or more, and for the rest of the sweep (T-1543, 9x9 shapes up to 1E15 by
/// 8 probabilities) is bounded and last-digit exact for shapes up to 1E5 and p of 1E-15 or more.
/// Inside that range its search (the capped port of `inverse_beta.rs`, T-1848, which also ends the
/// one hang found there, `BETA.INV(1-2^-53,1.0024,12177)`) is the start of the refinement. Above
/// that shape the quantile is the crossing of our own incomplete beta in Excel's AS 109 structure
/// (`inverse_beta::beta_quantile_crossing`, T-1538; statrs' cdf misses by 13 ulp at
/// `BETAINV(1E-20,1E10,0.5)`, Excel 0.9999999956419133, gen-m9/0a9226016e8f5f33 A358).
const BETA_QUANTILE_STATRS_SHAPE_MAX: f64 = 1e5;
const BETA_QUANTILE_STATRS_P_MIN: f64 = 1e-15;

/// The quantile of `p` in (0, 1): the crossing of BETA.DIST's own cdf (`special::refine_increasing`
/// on `bratio::beta_dist_tails`, the kernel BETA.DIST returns), from statrs' search as a start
/// where it is reliable. Excel's BETA.INV solves its own cdf to the crossing
/// (docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing
/// (T-1745)"); until T-3398 the search crossed the older Cephes-based tails, which BETA.DIST no
/// longer returns. Of the final bracket the even-significand end fits Excel better than the end
/// whose cdf is nearer p (T-3398: 11 visibly-off cells fixed and none new, against 16 fixed and
/// 1 new). Above p = 0.5 the target is the exact complement 1 - p on the upper tail,
/// as for F.INV (T-0974). At most 512 Newton steps and 129 cdfs, or 65 cdfs without a start. A
/// shape above 1E5 takes T-1538's crossing instead (above).
fn beta_quantile(alpha: f64, beta: f64, p: f64) -> f64 {
    if alpha > BETA_QUANTILE_STATRS_SHAPE_MAX || beta > BETA_QUANTILE_STATRS_SHAPE_MAX {
        return beta_quantile_crossing(alpha, beta, p);
    }
    let estimate = if p >= BETA_QUANTILE_STATRS_P_MIN {
        beta_inverse_cdf(alpha, beta, p)
    } else {
        f64::NAN
    };
    // A subnormal crossing is the underflow the caller reports as #NUM!, as the old search's 0 was
    // (`BETAINV(0.5,1E-5,1)`, probes/t2435-dist-extremes S!A71).
    let t = if p > 0.5 {
        refine_increasing(
            |t| -beta_dist_tails(t, 1.0 - t, alpha, beta).1,
            -(1.0 - p),
            estimate,
            0.0,
            1.0,
        )
    } else {
        refine_increasing(
            |t| beta_dist_tails(t, 1.0 - t, alpha, beta).0,
            p,
            estimate,
            0.0,
            1.0,
        )
    };
    flush_subnormal(t)
}

/// BETA.DIST's `cumulative`: a boolean, a number (TRUE unless 0), "TRUE" or "FALSE" in any case,
/// empty as FALSE, an error as itself; anything else is `#VALUE!`. A one-cell range is its value
/// (probes/stat-later-argument-lift L11, L12). This is `Model::cast_bool_ref` but for a LAMBDA
/// value, which is `#VALUE!` here as in BETA.DIST's scalar code before the lift, where
/// `cast_bool_ref` gives `#N/IMPL!`; it stays separate so the scalar result does not change.
fn beta_cumulative(value: &CalcResult, cell: CellReferenceIndex) -> Result<bool, CalcResult> {
    match value {
        CalcResult::Boolean(b) => Ok(*b),
        CalcResult::Number(n) => Ok(*n != 0.0),
        CalcResult::EmptyArg | CalcResult::EmptyCell => Ok(false),
        CalcResult::String(s) => match s.to_uppercase().as_str() {
            "TRUE" => Ok(true),
            "FALSE" => Ok(false),
            _ => Err(value_error(cell)),
        },
        error @ CalcResult::Error { .. } => Err(error.clone()),
        _ => Err(value_error(cell)),
    }
}

impl Model {
    /// `BETA.DIST(x, alpha, beta, cumulative, [A], [B])`: every argument lifts over an array and
    /// a range (SPEC-0011; probes/stat-argument-modes H84-H89, probes/stat-later-argument-lift
    /// H1-H5, K3-K6). The cdf is `bratio::beta_dist_lower_tail`: Excel's closed forms on bratio's
    /// oriented arguments at a shape of 1 (T-2502), the ported bpser branches of TOMS 708 bratio
    /// (T-2505), and the T-0970 incomplete beta (T-2119) for the branches not ported yet;
    /// the density is `beta_density::beta_density_small_shape` for min(alpha, beta) <= 1 (T-2503)
    /// and `incomplete_beta::beta_density` where it applies above.
    pub(crate) fn fn_beta_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::BetaDist, args, cell, &mut |model, values| {
            let [x, alpha, beta, cumulative, bounds @ ..] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.beta_dist_of(x, alpha, beta, Some(cumulative), bounds, cell))
        })
    }

    /// The BETA.DIST kernel; `cumulative` is `None` for BETADIST, always cumulative, and
    /// `bounds` holds the given `[A]` and `[B]`.
    fn beta_dist_of(
        &self,
        x: &CalcResult,
        alpha: &CalcResult,
        beta: &CalcResult,
        cumulative: Option<&CalcResult>,
        bounds: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        // Excel reads a TRUE as 1 in x (probes/stat-later-argument-lift L1).
        let x = self.cast_number_ref(x, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta_param = self.cast_number_ref(beta, cell)?;
        let cumulative = match cumulative {
            Some(value) => beta_cumulative(value, cell)?,
            None => true,
        };
        let a = self.cast_number_or(bounds.first(), 0.0, cell)?;
        let b = self.cast_number_or(bounds.get(1), 1.0, cell)?;

        // Excel: alpha <= 0 or beta <= 0 → #NUM!; x < A, x > B, or A = B → #NUM!
        if alpha <= 0.0 || beta_param <= 0.0 || b == a || x < a || x > b {
            return Err(num_error(cell));
        }

        // Transform to standard Beta(0,1)
        let width = b - a;
        // B - A past f64::MAX: Excel's cdf is 0 at x = A and 1 at x = B, and everything else is
        // #DIV/0! (`BETA.DIST(0,2,3,TRUE,-1E308,1E308)`, the pdf at x = B; probes/
        // stat-overflow-arguments A7-A10, E9). statrs' incomplete beta would unwrap a NaN `t`.
        if !width.is_finite() {
            return match (cumulative, x == a, x == b) {
                (true, true, _) => Ok(0.0),
                (true, _, true) => Ok(1.0),
                _ => Err(div_error(cell)),
            };
        }
        // The density is #NUM! at an x of exactly 0 with alpha <= 1, and at an x of exactly 1 with
        // beta <= 1: Excel checks the given x against 0 and 1 rather than the interval's own ends,
        // so `BETA.DIST(0,0.5,2,FALSE,-1,1)` and `BETA.DIST(1,2,0.5,FALSE,0,5)` are #NUM! at an
        // interior point while `BETA.DIST(2,0.5,2,FALSE,2,5)` is 0 at the interval's own lower end
        // (statistics.md "BETA.DIST's density is #NUM! at an x of 0 or 1 (T-2435)";
        // probes/t2435-dist-extremes A59:A67, A103:A106, A126:A128, A132:A140).
        if !cumulative && ((x == 0.0 && alpha <= 1.0) || (x == 1.0 && beta_param <= 1.0)) {
            return Err(num_error(cell));
        }
        let t = (x - a) / width;
        let dist = Beta::new(alpha, beta_param).map_err(|_| num_error(cell))?;
        // At the interval's own ends Excel gives 0 rather than the #NUM! an x of 0 or 1 takes
        // above, at a shape of exactly 1 as well as a diverging one, so the density steps down to
        // 0 at the end: `BETA.DIST(2,0.5,2,FALSE,2,5)`, `BETA.DIST(5,2,0.5,FALSE,2,5)`,
        // `BETA.DIST(2,1,2,FALSE,2,5)` and `BETA.DIST(5,2,1,FALSE,2,5)` are 0 where
        // `BETA.DIST(2.0000001,1,2,FALSE,2,5)` is 0.6666666444444445
        // (probes/t2435-dist-extremes A126, A127, A143:A146).
        if !cumulative && ((t == 0.0 && alpha <= 1.0) || (t == 1.0 && beta_param <= 1.0)) {
            return Ok(0.0);
        }
        let result = if cumulative {
            // A subnormal cdf is kept only at beta = 1, the closed form t^alpha, and is 0 at any
            // other beta, 1.0000001 included (`(x)=0` FALSE at B2, B3, B6, B7, B11, B12 of
            // probes/t2737-subnormal-cdf-shapes, TRUE at B1, B4, B5, B8:B10; T-2737).
            let cdf = beta_dist_lower_tail(t, 1.0 - t, alpha, beta_param);
            if beta_param == 1.0 {
                cdf
            } else {
                flush_subnormal(cdf)
            }
        } else {
            // general-interval beta pdf: f_X(x) = f_T(t) / (B - A), t=(x-A)/(B-A). A subnormal
            // density is 0, as in Excel (`(x)=0` TRUE: probes-numeric/t2513-r3-m1-residual-hold E863,
            // E875, E957; docs/excel-semantics/numbers.md "Distribution densities"). Both shapes
            // above 1: Excel's Loader saddle point (T-2507; decision 0181 accepts the cells it
            // leaves visibly off, which T-3220 owns).
            flush_subnormal(
                beta_density_small_shape(t, alpha, beta_param)
                    .or_else(|| {
                        (alpha > 1.0 && beta_param > 1.0)
                            .then(|| excel_beta_density(t, alpha, beta_param))
                            .flatten()
                    })
                    .or_else(|| beta_density(t, alpha, beta_param))
                    .unwrap_or_else(|| dist.pdf(t))
                    / width,
            )
        };
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `BETA.INV(probability, alpha, beta, [A], [B])`, and BETAINV: every argument lifts over an
    /// array and a range (SPEC-0011; probes/stat-argument-modes H90-H99,
    /// probes/stat-quantile-lift). The quantile is the crossing of BETA.DIST's cdf
    /// (`beta_quantile`), which the lifting table charges (lift/kinds/statistical.rs
    /// `QUANTILE_WEIGHT`).
    pub(crate) fn fn_beta_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::BetaInv, args, cell, &mut |model, values| {
            number_or_error(model.beta_inv_of(values, cell))
        })
    }

    fn beta_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, alpha, beta, bounds @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // A boolean reads as 1 or 0 in every argument (probes/stat-quantile-lift L3, gen-m9/165db58d1540f014 A257).
        let p = self.cast_number_ref(p, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta_param = self.cast_number_ref(beta, cell)?;
        let a = self.cast_number_or(bounds.first(), 0.0, cell)?;
        let b = self.cast_number_or(bounds.get(1), 1.0, cell)?;
        // alpha or beta <= 0, probability outside (0, 1) and A >= B are #NUM!.
        if alpha <= 0.0 || beta_param <= 0.0 || p <= 0.0 || p >= 1.0 || b <= a {
            return Err(num_error(cell));
        }
        // A quantile that underflows to 0 or rounds to 1 is #NUM! (`BETAINV(0.025,1E-15,2.5)`,
        // `BETAINV(0.05,0.002,1)`, `BETAINV(1-1E-16,2,0.01)`; statistics.md "Scalar distributions
        // at extreme arguments (T-2255)", probes/t2475-stat-subexpressions P!A127).
        let t = beta_quantile(alpha, beta_param, p);
        if !(t > 0.0 && t < 1.0) {
            return Err(num_error(cell));
        }
        // Map back from [0,1] to [A,B]
        Ok(a + t * (b - a))
    }

    /// `BETADIST(x, alpha, beta, [A], [B])`: BETA.DIST with `cumulative` TRUE; every argument
    /// lifts (probes/stat-argument-modes H100-H104, probes/stat-later-argument-lift H7, H9, K7-K10).
    pub(crate) fn fn_betadist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::BetaDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [x, alpha, beta, bounds @ ..] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(model.beta_dist_of(x, alpha, beta, None, bounds, cell))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::beta_quantile;

    #[test]
    fn quantile_is_the_crossing_of_beta_dist_own_cdf() {
        // ironcalc-calc STATISTICAL/BETADIST_BETAINV.xlsx (Excel-cached), each off Excel before
        // T-3398 and exact on BETA.DIST's own kernel: `BETA.INV(G,alpha,beta,A,B)` = A + t (B - A).
        // H15: BETA.INV(0.4999999999999999,2,2,0.2,1.2) = 0.7.
        assert_eq!(
            0.2 + beta_quantile(2.0, 2.0, 0.499_999_999_999_999_9) * 1.0,
            0.7
        );
        // H32: BETA.INV(0.5136696247356396,5,5,0.1,1) = 0.555.
        assert_eq!(
            0.1 + beta_quantile(5.0, 5.0, 0.513_669_624_735_639_6) * 0.9,
            0.555
        );
        // H43: BETA.INV(0.5882886668596937,2,2.5,0.15,1.2) = 0.6655623437846115.
        assert_eq!(
            0.15 + beta_quantile(2.0, 2.5, 0.588_288_666_859_693_7) * (1.2 - 0.15),
            0.665_562_343_784_611_5
        );
    }
}
