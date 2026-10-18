use statrs::distribution::{Beta, Continuous, ContinuousCDF};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error, value_error};
use crate::functions::statistical::inverse_beta::beta_inverse_cdf;
use crate::functions::statistical::special::bisect_unit_interval;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// `statrs`' `inverse_cdf` panics from p about 1E-200, does not finish for BETA.INV(1E-100,1000,1)
/// nor for a shape of 1E8 or more, and for the rest of the sweep (T-1543, 9x9 shapes up to 1E15 by
/// 8 probabilities) is bounded and last-digit exact for shapes up to 1E5 and p of 1E-15 or more.
/// Outside that range the quantile comes from bisection on the cdf (at most 1100 evaluations).
/// statrs' search is the capped port of `inverse_beta.rs` (T-1848), which also ends the one hang
/// found inside the range (`BETA.INV(1-2^-53,1.0024,12177)`, `#NUM!`).
const BETA_QUANTILE_STATRS_SHAPE_MAX: f64 = 1e5;
const BETA_QUANTILE_STATRS_P_MIN: f64 = 1e-15;

fn beta_quantile(dist: &Beta, alpha: f64, beta: f64, p: f64) -> f64 {
    if alpha <= BETA_QUANTILE_STATRS_SHAPE_MAX
        && beta <= BETA_QUANTILE_STATRS_SHAPE_MAX
        && p >= BETA_QUANTILE_STATRS_P_MIN
    {
        return beta_inverse_cdf(alpha, beta, p);
    }
    bisect_unit_interval(|x| dist.cdf(x), p)
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
    /// H1-H5, K3-K6). The cdf is statrs' incomplete beta, a continued fraction of at most 140
    /// steps.
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
        // TODO(T-1849): Excel reads a TRUE as 1 in x (probes/stat-later-argument-lift L1).
        let x = self.cast_number_no_bools_ref(x, cell)?;
        let alpha = self.cast_number_no_bools_ref(alpha, cell)?;
        let beta_param = self.cast_number_no_bools_ref(beta, cell)?;
        let cumulative = match cumulative {
            Some(value) => beta_cumulative(value, cell)?,
            None => true,
        };
        let a = self.cast_number_no_bools_or(bounds.first(), 0.0, cell)?;
        let b = self.cast_number_no_bools_or(bounds.get(1), 1.0, cell)?;

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
        let t = (x - a) / width;
        let dist = Beta::new(alpha, beta_param).map_err(|_| num_error(cell))?;
        let result = if cumulative {
            dist.cdf(t)
        } else {
            // general-interval beta pdf: f_X(x) = f_T(t) / (B - A), t=(x-A)/(B-A)
            dist.pdf(t) / width
        };
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `BETA.INV(probability, alpha, beta, [A], [B])`, and BETAINV: every argument lifts over an
    /// array and a range (SPEC-0011; probes/stat-argument-modes H90-H99,
    /// probes/stat-quantile-lift). The quantile is the capped port of statrs' inversion or a
    /// bisection of at most 1,100 cdfs (`beta_quantile`), which the lifting table charges
    /// (kinds.rs `QUANTILE_WEIGHT`).
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
        let p = self.cast_number_no_bools_ref(p, cell)?;
        let alpha = self.cast_number_no_bools_ref(alpha, cell)?;
        let beta_param = self.cast_number_no_bools_ref(beta, cell)?;
        let a = self.cast_number_no_bools_or(bounds.first(), 0.0, cell)?;
        let b = self.cast_number_no_bools_or(bounds.get(1), 1.0, cell)?;
        // alpha or beta <= 0, probability outside (0, 1) and A >= B are #NUM!.
        if alpha <= 0.0 || beta_param <= 0.0 || p <= 0.0 || p >= 1.0 || b <= a {
            return Err(num_error(cell));
        }
        let dist = Beta::new(alpha, beta_param).map_err(|_| num_error(cell))?;
        let t = beta_quantile(&dist, alpha, beta_param, p);
        if t.is_nan() || t.is_infinite() {
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
