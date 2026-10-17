//! BINOM.DIST, BINOM.DIST.RANGE, BINOM.INV and NEGBINOM.DIST (and BINOMDIST, CRITBINOM,
//! NEGBINOMDIST): every argument lifts over an array and a range (SPEC-0011;
//! probes/stat-argument-modes H105-H131, H374-H384, probes/stat-later-argument-lift H11-H23,
//! K15-K16). The cdfs are statrs' incomplete beta, a continued fraction of at most 140 steps.

use statrs::distribution::{Binomial, Discrete, DiscreteCDF, NegativeBinomial};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `BINOM.DIST(number_s, trials, probability_s, cumulative)`, and BINOMDIST.
    pub(crate) fn fn_binom_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::BinomDist, args, cell, &mut |model, values| {
            number_or_error(model.binom_dist_of(values, cell))
        })
    }

    fn binom_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [number_s, trials, p, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let number_s = self.cast_number_ref(number_s, cell)?.trunc();
        let trials = self.cast_number_ref(trials, cell)?.trunc();
        let p = self.cast_number_ref(p, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if trials < 0.0
            || number_s < 0.0
            || number_s > trials
            || p.is_nan()
            || !(0.0..=1.0).contains(&p)
        {
            return Err(num_error(cell));
        }
        // Limit to u64
        if trials > u64::MAX as f64 {
            return Err(num_error(cell));
        }
        let n = trials as u64;
        let k = number_s as u64;
        let dist = Binomial::new(p, n).map_err(|_| num_error(cell))?;
        let prob = if cumulative { dist.cdf(k) } else { dist.pmf(k) };
        if prob.is_nan() || prob.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(prob)
    }

    /// `BINOM.DIST.RANGE(trials, probability_s, number_s, [number_s2])`.
    pub(crate) fn fn_binom_dist_range(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::BinomDistRange,
            args,
            cell,
            &mut |model, values| number_or_error(model.binom_dist_range_of(values, cell)),
        )
    }

    fn binom_dist_range_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [trials, p, number_s, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let trials = self.cast_number_ref(trials, cell)?.trunc();
        let p = self.cast_number_ref(p, cell)?;
        let number_s = self.cast_number_ref(number_s, cell)?.trunc();
        // number_s2 (upper, optional)
        let number_s2 = match rest.first() {
            Some(value) => self.cast_number_ref(value, cell)?.trunc(),
            None => number_s,
        };
        if trials < 0.0
            || number_s < 0.0
            || number_s2 < 0.0
            || number_s > number_s2
            || number_s2 > trials
            || p.is_nan()
            || !(0.0..=1.0).contains(&p)
        {
            return Err(num_error(cell));
        }
        if trials > u64::MAX as f64 {
            return Err(num_error(cell));
        }
        let n = trials as u64;
        let lower = number_s as u64;
        let upper = number_s2 as u64;
        let dist = Binomial::new(p, n).map_err(|_| num_error(cell))?;
        let prob = if lower == 0 {
            dist.cdf(upper)
        } else {
            dist.cdf(upper) - dist.cdf(lower - 1)
        };
        if prob.is_nan() || prob.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(prob)
    }

    /// `BINOM.INV(trials, probability_s, alpha)`, and CRITBINOM. The search halves
    /// `[0, trials]` with `trials` a `u64`, so it stops after at most 64 steps of one cdf each,
    /// its own per-call cap; the lifting table charges each element that weight (SPEC-0011
    /// security rule).
    pub(crate) fn fn_binom_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::BinomInv, args, cell, &mut |model, values| {
            number_or_error(model.binom_inv_of(values, cell))
        })
    }

    fn binom_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [trials, p, alpha] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let trials = self.cast_number_ref(trials, cell)?.trunc();
        let p = self.cast_number_ref(p, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        if trials < 0.0
            || trials > u64::MAX as f64
            || p.is_nan()
            || !(0.0..1.0).contains(&p)
            || alpha.is_nan()
            || alpha <= 0.0
            || alpha >= 1.0
        {
            return Err(num_error(cell));
        }
        let n = trials as u64;
        let dist = Binomial::new(p, n).map_err(|_| num_error(cell))?;
        // Smallest k with CDF(k) >= alpha. statrs' generic `inverse_cdf` can panic
        // (an unwrap inside its bisection), e.g. for BINOM.INV(1, 0.001, 0.1).
        let (mut lo, mut hi) = (0u64, n);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if DiscreteCDF::cdf(&dist, mid) >= alpha {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        Ok(lo as f64)
    }

    /// `NEGBINOM.DIST(number_f, number_s, probability_s, cumulative)`.
    pub(crate) fn fn_negbinom_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::NegbinomDist, args, cell, &mut |model, values| {
            let [number_f, number_s, probability_s, cumulative] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.negbinom_dist_of(
                number_f,
                number_s,
                probability_s,
                Some(cumulative),
                cell,
            ))
        })
    }

    /// The NEGBINOM.DIST kernel; `cumulative` is `None` for NEGBINOMDIST, always the pmf.
    fn negbinom_dist_of(
        &self,
        number_f: &CalcResult,
        number_s: &CalcResult,
        probability_s: &CalcResult,
        cumulative: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let number_f = self.cast_number_ref(number_f, cell)?.trunc();
        let number_s = self.cast_number_ref(number_s, cell)?.trunc();
        let probability_s = self.cast_number_ref(probability_s, cell)?;
        let cumulative = match cumulative {
            Some(value) => Self::cast_bool_ref(value, cell)?,
            None => false,
        };
        if number_f < 0.0 || number_s < 1.0 || probability_s <= 0.0 || probability_s >= 1.0 {
            return Err(num_error(cell));
        }
        // Guard against absurdly large failures that won't fit in u64
        if number_f > (u64::MAX as f64) {
            return Err(num_error(cell));
        }
        let dist = NegativeBinomial::new(number_s, probability_s).map_err(|_| num_error(cell))?;
        let f_u = number_f as u64;
        let result = if cumulative {
            dist.cdf(f_u)
        } else {
            dist.pmf(f_u)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `NEGBINOMDIST(number_f, number_s, probability_s)`: NEGBINOM.DIST with `cumulative` FALSE.
    pub(crate) fn fn_negbinomdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::NegbinomDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [number_f, number_s, probability_s] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(model.negbinom_dist_of(
                    number_f,
                    number_s,
                    probability_s,
                    None,
                    cell,
                ))
            },
        )
    }
}
