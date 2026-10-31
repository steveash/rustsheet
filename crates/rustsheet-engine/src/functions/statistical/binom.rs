//! BINOM.DIST, BINOM.DIST.RANGE, BINOM.INV and NEGBINOM.DIST (and BINOMDIST, CRITBINOM,
//! NEGBINOMDIST): every argument lifts over an array and a range (SPEC-0011;
//! probes/stat-argument-modes H105-H131, H374-H384, probes/stat-later-argument-lift H11-H23,
//! K15-K16). BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST sum their pmf terms in double-double
//! (`discrete.rs`, T-0927), and past that walk's budget fall back to statrs' incomplete beta, a
//! continued fraction of at most 140 steps, as BINOM.INV's search does throughout. BINOM.DIST at
//! k = 0, and its pmf at k = n, take Excel's closed forms instead (T-2645).

use statrs::distribution::{Binomial, Discrete, DiscreteCDF, NegativeBinomial};

use super::bratio::{closed_or_ported_tails, dispatch, ported_tails, Branch};
use super::discrete::{binomial_cdf, binomial_pmf, binomial_range, odds, times_ratio};
use super::incomplete_beta::closed_form_tails;

use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;
use crate::x87_log1p::excel_ln_one_minus;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The trial count (after truncation) from which BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE,
/// BINOM.INV and CRITBINOM are `#NUM!` whatever the other arguments: 2^31 - 1, so 2^31 - 2 trials
/// is the most Excel evaluates (statistics.md "BINOM.DIST is #NUM! from 2^31 - 1 trials";
/// probes-numeric t2645-binom-trials-limit, excel-graph: every k, both cumulative modes, p = 0 and
/// 1 too; the same limit holds for the inverses, `BINOM.INV(2147483647,0.5,0.5)` and
/// `CRITBINOM(2147483647,0.5,0.5)` #NUM! against a count at 2^31 - 2, probes/t2435-dist-extremes
/// A16, A17, A19; the count itself at such a trial count is still ours to fix, A15 and A18,
/// T-2809).
const TRIALS_LIMIT: f64 = crate::limits::COUNT_LIMIT_EXCLUSIVE;

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
        // A count in (-1, 0) is `#NUM!`: the sign is checked before truncation (gen-m9 85f48d5cdfb24555
        // A35, caa1eed48b1883fe A384, 7bd219ec0c0c5e22 A320; statistics.md "Scalar distributions
        // at extreme arguments (T-2255)").
        let number_s = self.cast_number_ref(number_s, cell)?;
        let negative = number_s < 0.0;
        let number_s = number_s.trunc();
        let trials = self.cast_number_ref(trials, cell)?.trunc();
        let p = self.cast_number_ref(p, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if !(0.0..TRIALS_LIMIT).contains(&trials)
            || negative
            || number_s > trials
            || p.is_nan()
            || !(0.0..=1.0).contains(&p)
        {
            return Err(num_error(cell));
        }
        let n = trials as u64;
        let k = number_s as u64;
        // A certain outcome: the walks need 0 < p < 1.
        if p == 0.0 || p == 1.0 {
            // The pmf at k = 0 is EXP(n.LN(1 - p)) (below), so p = 1 takes Excel's LN(0) and is
            // #NUM! from one trial on, where the cdf there is 0 and n = 0 trials is 1
            // (`BINOM.DIST(0,2,1,FALSE)` and `BINOMDIST(0,2,1,FALSE)` #NUM!,
            // `BINOM.DIST(0,2,1,TRUE)` 0, `BINOM.DIST(0,0,1,FALSE)` 1,
            // `BINOM.DIST(1,2,1,FALSE)` 0; probes/t2435-dist-extremes A1:A13, A113, A114;
            // gen-m9 05d44f6dd6cfc567 A344, a013a45a8fbcab4f A206).
            if p == 1.0 && !cumulative && k == 0 && n >= 1 {
                return Err(num_error(cell));
            }
            let certain = if p == 0.0 { 0 } else { n };
            let hit = if cumulative {
                k >= certain
            } else {
                k == certain
            };
            return Ok(if hit { 1.0 } else { 0.0 });
        }
        // Loader's (2000) end branches, Excel's own closed forms (statistics.md "BINOM.DIST pmf:
        // closed forms at k = 0 and k = n"; T-2464 r2-log1p, T-2513 r3-binom-pmf): at k = 0 the
        // pmf, and so the cdf, is EXP(fl(n·l)) with l Excel's ln(1 - p), bit-exact on all 345
        // BINOM.DIST(0,n,p) cells of the T-2464 research set; at k = n the pmf is
        // EXP(fl(n·LN(p))), 172 of 172.
        if k == 0 {
            return Ok(excel_exp(trials * excel_ln_one_minus(p)));
        }
        if k == n && !cumulative {
            return Ok(excel_exp(trials * excel_ln(p)));
        }
        // Excel's cdf is the upper tail of the incomplete beta I_p(k+1, n-k) at x = p, y = fl(1 - p),
        // which at k = n - 1 takes Excel's closed form (T-2464 w5-triangulation, T-2502). k = 0 is
        // BINOM's own shortcut, not the kernel's (T-2645).
        if cumulative && k >= 1 && k < n {
            // Then the bratio port's branches (T-2847).
            if let Some((_, upper)) =
                closed_or_ported_tails(p, 1.0 - p, number_s + 1.0, trials - number_s)
            {
                return Ok(flush_subnormal(upper));
            }
        }
        // Off k = 0 a subnormal cdf is 0, as in Excel (`(x)=0` TRUE: probes/
        // t2706-subnormal-density-paths B5:B7), while the k = 0 one is kept (B4).
        let walked = if cumulative {
            binomial_cdf(n, odds(p), k).map(flush_subnormal)
        } else {
            // Off the k = 0 and k = n closed forms a subnormal pmf is 0, as in Excel (`(x)=0` TRUE:
            // probes-numeric/t2513-r3-binom-pmf-n E1729:E1792), while the k = 0 one is kept
            // (t2513-verify-r3-binom-pmf-h I229, I238; docs/excel-semantics/numbers.md "Distribution densities").
            binomial_pmf(n, odds(p), k).map(|pmf| flush_subnormal(pmf.to_f64()))
        };
        if let Some(prob) = walked {
            return Ok(prob);
        }
        let dist = Binomial::new(p, n).map_err(|_| num_error(cell))?;
        let prob = flush_subnormal(if cumulative { dist.cdf(k) } else { dist.pmf(k) });
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
        if !(0.0..TRIALS_LIMIT).contains(&trials)
            || number_s < 0.0
            || number_s2 < 0.0
            || number_s > number_s2
            || number_s2 > trials
            || p.is_nan()
            || !(0.0..=1.0).contains(&p)
        {
            return Err(num_error(cell));
        }
        let n = trials as u64;
        let lower = number_s as u64;
        let upper = number_s2 as u64;
        if p == 0.0 || p == 1.0 {
            let certain = if p == 0.0 { 0 } else { n };
            return Ok(if (lower..=upper).contains(&certain) {
                1.0
            } else {
                0.0
            });
        }
        if let Some(prob) = binomial_range(n, odds(p), lower, upper) {
            return Ok(prob);
        }
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
    /// `[0, trials]` with `trials` under [`TRIALS_LIMIT`], so it stops after at most 31 steps of
    /// one cdf each,
    /// its own per-call cap: up to about 65 us an element in release, which the lifting table
    /// charges as `Weighted(4096)` (SPEC-0011 security rule; lift/kinds/statistical.rs
    /// `BINOM_INV`).
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
        if !(0.0..TRIALS_LIMIT).contains(&trials)
            || p.is_nan()
            // Probability 0 is #NUM! too, unlike BINOMDIST (probes-numeric/m9-legacy-critbinom A23, A24).
            || p <= 0.0
            || p >= 1.0
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
        // number_f in (-1, 0) is `#NUM!` before truncation (gen-m9 d208bbafc3bc1997 A231,
        // 3b7990bd4e3307ba A312, 1db18890916ee643 A124; statistics.md "Scalar distributions at
        // extreme arguments, part 2 (T-2256)").
        let number_f = self.cast_number_ref(number_f, cell)?;
        let negative = number_f < 0.0;
        let number_f = number_f.trunc();
        let number_s = self.cast_number_ref(number_s, cell)?.trunc();
        let probability_s = self.cast_number_ref(probability_s, cell)?;
        let cumulative = match cumulative {
            Some(value) => Self::cast_bool_ref(value, cell)?,
            None => false,
        };
        if negative || number_s < 1.0 || probability_s <= 0.0 || probability_s >= 1.0 {
            return Err(num_error(cell));
        }
        // Guard against absurdly large failures that won't fit in u64. Not TRIALS_LIMIT:
        // NEGBINOM.DIST has no 2^31 - 1 limit (probes-numeric/t2645-binom-trials-limit A34, A46).
        if number_f > (u64::MAX as f64) {
            return Err(num_error(cell));
        }
        let f_u = number_f as u64;
        // Excel's cdf is the incomplete beta I_p(r, f+1) at x = p, y = fl(1 - p), which at r = 1 or
        // f = 0 takes Excel's closed form (T-2464 w5-triangulation, T-2502).
        let walk_first = cumulative && takes_bfrac(number_s, number_f + 1.0, probability_s);
        if cumulative {
            // Then the bratio port's branches (T-2847), whose subnormal cdf is 0 in Excel
            // (`NEGBINOM.DIST(1,1000,0.48,TRUE)` = 0: probes/t2706-subnormal-density-paths A9:B9).
            // Not [`closed_or_ported_tails`]: the flush is backed for the port only, so the closed
            // form's result is returned as it was.
            let (x, y, a, b) = (probability_s, 1.0 - probability_s, number_s, number_f + 1.0);
            if let Some((lower, _)) = closed_form_tails(x, y, a, b) {
                return Ok(lower);
            }
            // bfrac's points take the double-double walk below first, and bfrac only past its
            // budget: Excel's `NEGBINOM.DIST(1,100,0.99,TRUE)` (ironcalc-calc BINOM.xlsx
            // NEGBINOM.DIST!E39) is the correctly rounded I_0.99(100, 2), which the walk hits and
            // bfrac misses by one ulp (its w is correctly rounded, but 1 - w is then an exact tie
            // that rounds the other way; T-2508).
            if !walk_first {
                if let Some((lower, _)) = ported_tails(a, b, x, y) {
                    return Ok(flush_subnormal(lower));
                }
            }
        }
        // F failures before the r-th success: P(F <= f) is P(at most f failures in f + r trials),
        // a binomial cdf whose "success" is a failure, and P(F = f) = r / (f + r) times that
        // binomial's pmf at f.
        if number_f + number_s < 2f64.powi(53) {
            let r_u = number_s as u64;
            let trials = f_u + r_u;
            let failure_odds = {
                let (p, q) = odds(probability_s);
                (q, p)
            };
            // Off the closed forms a subnormal cdf or pmf is 0, as in Excel (`(x)=0` TRUE: probes/
            // t2706-subnormal-density-paths B9:B11, probes-numeric/t2513-verify-r3-binom-pmf-h J229,
            // J238), but the pmf at f = 0, p^r, is kept (t2706-subnormal-density-paths B12:B13).
            let walked = if cumulative {
                binomial_cdf(trials, failure_odds, f_u).map(flush_subnormal)
            } else {
                binomial_pmf(trials, failure_odds, f_u).map(|pmf| {
                    let pmf = times_ratio(pmf, r_u, trials);
                    if f_u == 0 {
                        pmf
                    } else {
                        flush_subnormal(pmf)
                    }
                })
            };
            if let Some(result) = walked {
                return Ok(result);
            }
        }
        if walk_first {
            if let Some((lower, _)) =
                ported_tails(number_s, number_f + 1.0, probability_s, 1.0 - probability_s)
            {
                return Ok(flush_subnormal(lower));
            }
        }
        let dist = NegativeBinomial::new(number_s, probability_s).map_err(|_| num_error(cell))?;
        let raw = if cumulative {
            dist.cdf(f_u)
        } else {
            dist.pmf(f_u)
        };
        let keeps_subnormal = !cumulative && f_u == 0;
        let result = if keeps_subnormal {
            raw
        } else {
            flush_subnormal(raw)
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

/// Whether bratio takes bfrac for NEGBINOM.DIST's cdf, `I_p(r, f + 1)` at y = fl(1 - p).
fn takes_bfrac(r: f64, f_plus_one: f64, p: f64) -> bool {
    dispatch(r, f_plus_one, p, 1.0 - p).is_some_and(|d| d.branch == Branch::Bfrac)
}
