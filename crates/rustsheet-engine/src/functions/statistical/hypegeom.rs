//! HYPGEOM.DIST and HYPGEOMDIST, and HYPGEOM.DIST's cdf with a per-call cap of its own (SPEC-0011
//! security rule, T-1873).
//!
//! [`hypergeometric_cdf`] is statrs 0.18.0's `Hypergeometric::cdf`: the sum of the pmf from 0 to
//! x, one `exp` of three `ln_binomial`s a term, in that order, so that every call that ends in
//! statrs ends here with the same double. statrs' sum is open: it takes x + 1 terms whatever the
//! distribution, so `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` took about 4 s in
//! release, and x near 2^64 would not finish. Most of those terms are exactly 0: below
//! max(0, n + K - N) a term is `exp(-inf)`, and far from the mode its exponent is below about
//! -745 and `exp` underflows to 0. Adding +0.0 to the running sum (itself +0.0 or more) leaves it
//! unchanged, so the port skips those terms and sums the rest in statrs' order. Past
//! [`MAX_TERMS`] terms that are not known to be 0 the cdf is NaN, which HYPGEOM.DIST turns into
//! `#NUM!` (docs/excel-semantics/statistics.md "HYPGEOM.DIST: per-call cap and lifting", a
//! DEVIATION).
//!
//! statrs is distributed under this notice:
//!
//! > MIT License
//! >
//! > Copyright (c) 2016 Michael Ma
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a copy of this software
//! > and associated documentation files (the "Software"), to deal in the Software without
//! > restriction, including without limitation the rights to use, copy, modify, merge, publish,
//! > distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
//! > Software is furnished to do so, subject to the following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in all copies or
//! > substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
//! > BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//! > NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! > DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! > OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use statrs::distribution::{Discrete, Hypergeometric};
// statrs' own `ln_binomial` and `ln_factorial`: bit-for-bit parity with statrs needs the same
// functions.
use statrs::function::factorial::{ln_binomial, ln_factorial};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The most terms one cdf may sum once the terms known to be 0 are skipped. The terms that do
/// not underflow lie within about 39 standard deviations of the mode (an exponent of -750 is
/// 38.7 sigma in the normal limit, and the hypergeometric tails are lighter), so 2^18 keeps every
/// cdf whose standard deviation is below about 3,400: every population up to about 1.84E8 whatever
/// the sample and x (the widest distribution has K = n = N / 2 and a deviation of sqrt(N) / 4; at
/// x past its upper tail it sums 258,239 terms at a population of 1.8E8 and 261,091 at 1.84E8),
/// and `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` (x at the mode: 96,259 terms, of
/// which 95,942 are not 0).
///
/// What the cap costs, which the lifting table charges (lift/kinds/statistical.rs
/// `HYPGEOM_DIST_WEIGHT`, 2^22 units an element): one term is three statrs `ln_binomial`s and an
/// `exp` (six Lanczos `ln_gamma`s above 170). Measured in release (T-1873, load 1 to 2 on 4 cores):
/// 167 to 193 ns a term where the terms are normal (statrs' own sum over the 25,000,001 terms of
/// `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)`, 4.8 s as on main; 32 elements of
/// `HYPGEOM.DIST(45500000,90000000,90000000,180000000,TRUE)`, the widest distribution at a
/// population of 1.8E8 with 258,239 terms each, 1.38 to 1.55 s in benches/lift.rs), and 206 to 234
/// ns where most of the window is subnormal (`exp`'s subnormal path and additions with a subnormal
/// operand: 32 elements of `HYPGEOM.DIST(99994114707,200000000000,200000000000,400000000000,TRUE)`,
/// whose window at the cap is 262,144 terms, 96,639 of them exactly 0 and 153,739 subnormal, took
/// 1.73 to 1.96 s, 54 to 61 ms an element). The bound of an element is 2^18 + 131 terms at 250 ns
/// (above the slowest measured), about 66 ms, so the weight caps a lifted call at 2^27 / 2^22 = 32
/// elements: about 2.1 s at the bound. The cdfs the suites check sum at most 501 terms.
const MAX_TERMS: u64 = 1 << 18;

/// An exponent at or below which `exp` is exactly 0. e^-750 is about 1.9E-326, far below half
/// the smallest subnormal double, so a correctly rounded `exp` gives 0 there (it does from about
/// -745.13 down); the skip relies on the platform `exp` returning exactly 0 below that, as glibc's
/// does (unit test `exp_is_zero_at_the_threshold`).
const ZERO_EXPONENT: f64 = -750.0;

/// The largest population whose terms may be skipped far from the mode: every count of the sum
/// is then a double exactly, so its `ln_factorial` is that of the count itself (above 2^53,
/// `n as f64` rounds, the computed exponent jumps by thousands where it rounds, and statrs' terms
/// are no longer the pmf's).
const SKIP_POPULATION_MAX: u64 = 1 << 52;

/// The relative error the skip allows in one computed exponent, against the true log pmf. The
/// exponent is seven `ln_factorial`s, each within a few units of the last place of
/// `ln_factorial(N)` (statrs' Lanczos `ln_gamma` above 170, the log of its exact factorial table
/// below), and six roundings of sums no larger than three of them: about 45 * 2^-52 of
/// `ln_factorial(N)` in all. 2^-40 is 90 times that.
const EXPONENT_RELATIVE_ERROR: f64 = 1.0 / (1u64 << 40) as f64;

/// The absolute error the skip allows on top of [`EXPONENT_RELATIVE_ERROR`], for the counts whose
/// `ln_factorial` is small (an error of about 1E-13 at most).
const EXPONENT_ABSOLUTE_ERROR: f64 = 1.0 / (1u64 << 20) as f64;

/// statrs' `Hypergeometric::cdf(x)` for a population N, K successes and n draws: its double
/// wherever its terms other than the skipped zeros number at most [`MAX_TERMS`], NaN past that.
pub(super) fn hypergeometric_cdf(population: u64, successes: u64, draws: u64, x: u64) -> f64 {
    cdf_within(population, successes, draws, x, MAX_TERMS)
}

/// [`hypergeometric_cdf`] with a cap of `max_terms`, so that tests can reach the cap with sums
/// statrs finishes quickly.
fn cdf_within(population: u64, successes: u64, draws: u64, x: u64, max_terms: u64) -> f64 {
    // statrs' `min()` is `(draws + successes).saturating_sub(population)`, which overflows a u64
    // near 2^64 (a panic in a debug build); in u128 it is exact.
    let min = (u128::from(draws) + u128::from(successes)).saturating_sub(u128::from(population));
    let max = successes.min(draws);
    if u128::from(x) < min {
        return 0.0;
    }
    if x >= max {
        return 1.0;
    }
    // min <= x < u64::MAX here, so it is a u64 and `x + 1` does not overflow.
    let Ok(min) = u64::try_from(min) else {
        return f64::NAN;
    };
    let ln_denom = ln_binomial(population, draws);
    // statrs' term i, as its fold writes it. For i in [min, max] neither `ln_binomial` is -inf.
    let exponent = |i: u64| {
        ln_binomial(successes, i) + ln_binomial(population - successes, draws - i) - ln_denom
    };
    // Terms below `min` are exp(-inf) = 0 in statrs' sum, so it starts at `min`.
    let (first, end) = if x - min < max_terms {
        (min, x + 1)
    } else {
        match nonzero_terms(population, successes, draws, min, x, exponent) {
            Some(range) => range,
            None => return f64::NAN,
        }
    };
    if end - first > max_terms {
        return f64::NAN;
    }
    (first..end).fold(0.0, |acc, i| acc + exponent(i).exp())
}

/// The terms `[first, end)` of the sum over `[min, x]` outside which every term is exactly 0, or
/// `None` where that cannot be shown (a population above [`SKIP_POPULATION_MAX`]).
///
/// The true log pmf rises to the mode m = floor((n + 1)(K + 1) / (N + 2)) and falls after it
/// (p(i) / p(i - 1) = (K - i + 1)(n - i + 1) / (i (N - K - n + i)), at least 1 exactly for
/// i <= (n + 1)(K + 1) / (N + 2)), and each computed exponent is within `error` of it. So where a
/// computed exponent at j <= m is below [`ZERO_EXPONENT`] by more than twice `error`, the true
/// one is below it by more than `error` there and at every i < j, so each computed exponent at
/// i < j is below it too and its term is exactly 0; the same holds after the mode for i > j.
/// Each side is a bisection over the computed exponents themselves: the index it keeps is one
/// whose exponent it evaluated and found below the threshold (so the skip holds even where the
/// computed exponents are not monotone): at most 131 exponents in all (`min`, up to 64 halvings,
/// the right side's start and x, and up to 64 more halvings).
fn nonzero_terms(
    population: u64,
    successes: u64,
    draws: u64,
    min: u64,
    x: u64,
    exponent: impl Fn(u64) -> f64,
) -> Option<(u64, u64)> {
    if population > SKIP_POPULATION_MAX {
        return None;
    }
    let error = ln_factorial(population) * EXPONENT_RELATIVE_ERROR + EXPONENT_ABSOLUTE_ERROR;
    let margin = 2.0 * error;
    // Written as a positive condition, so a NaN exponent is never skipped.
    let is_zero = |i: u64| exponent(i) + margin < ZERO_EXPONENT;
    // n and K are at most 2^52 here, so the product fits a u128, and the mode is at most n.
    let mode = (u128::from(draws) + 1) * (u128::from(successes) + 1) / (u128::from(population) + 2);
    let mode = u64::try_from(mode).ok()?;

    // The left side: the largest j in [min, min(x, m)] whose term is known 0 starts the sum at
    // j + 1.
    let left_end = x.min(mode);
    let mut first = min;
    if min <= left_end && is_zero(min) {
        // `zero` is known 0, `above` is not (or is past the side).
        let (mut zero, mut above) = (min, left_end + 1);
        for _ in 0..u64::BITS {
            if above - zero <= 1 {
                break;
            }
            let mid = zero + (above - zero) / 2;
            if is_zero(mid) {
                zero = mid;
            } else {
                above = mid;
            }
        }
        first = zero + 1;
    }

    // The right side: the smallest j in [max(first, m), x] whose term is known 0 ends the sum.
    let mut end = x + 1;
    if x > mode {
        let start = first.max(mode);
        if is_zero(start) {
            end = start;
        } else if is_zero(x) {
            // `below` is not known 0, `zero` is.
            let (mut below, mut zero) = (start, x);
            for _ in 0..u64::BITS {
                if zero - below <= 1 {
                    break;
                }
                let mid = below + (zero - below) / 2;
                if is_zero(mid) {
                    zero = mid;
                } else {
                    below = mid;
                }
            }
            end = zero;
        }
    }
    Some((first, end.max(first)))
}

impl Model {
    /// `HYPGEOM.DIST(sample_s, number_sample, population_s, number_pop, cumulative)`: every
    /// argument lifts over an array and a range (SPEC-0011; probes/stat-argument-modes
    /// H330-H336, probes/stat-quantile-lift H51-H52). The cdf is the capped port of statrs' sum
    /// (`hypergeometric_cdf`), which the lifting table charges (lift/kinds/statistical.rs
    /// `HYPGEOM_DIST_WEIGHT`).
    pub(crate) fn fn_hyp_geom_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::HypGeomDist, args, cell, &mut |model, values| {
            let [sample_s, number_sample, population_s, number_pop, cumulative] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.hyp_geom_dist_of(
                [sample_s, number_sample, population_s, number_pop],
                Some(cumulative),
                cell,
            ))
        })
    }

    /// `HYPGEOMDIST(sample_s, number_sample, population_s, number_pop)`, HYPGEOM.DIST's pmf: every
    /// argument lifts over an array and a range (probes/stat-argument-modes H337-H342,
    /// probes/stat-quantile-lift H53-H54). The pmf is closed form.
    pub(crate) fn fn_hypgeomdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::HypGeomDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [sample_s, number_sample, population_s, number_pop] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(model.hyp_geom_dist_of(
                    [sample_s, number_sample, population_s, number_pop],
                    None,
                    cell,
                ))
            },
        )
    }

    /// The HYPGEOM.DIST kernel; `cumulative` is `None` for HYPGEOMDIST, always the pmf. The
    /// arguments are read in order (the first error wins), each count truncated, a boolean count
    /// `#VALUE!`.
    fn hyp_geom_dist_of(
        &self,
        counts: [&CalcResult; 4],
        cumulative: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [sample_s, number_sample, population_s, number_pop] = counts;
        let sample_s = self.cast_number_no_bools_ref(sample_s, cell)?.trunc();
        let number_sample = self.cast_number_no_bools_ref(number_sample, cell)?.trunc();
        let population_s = self.cast_number_no_bools_ref(population_s, cell)?.trunc();
        let number_pop = self.cast_number_no_bools_ref(number_pop, cell)?.trunc();
        let cumulative = match cumulative {
            Some(value) => Self::cast_bool_ref(value, cell)?,
            None => false,
        };

        // 0 <= sample_s <= min(number_sample, population_s), sample_s >= number_sample +
        // population_s - number_pop, 0 < number_sample <= number_pop and 0 < population_s <=
        // number_pop; anything else is #NUM!.
        let valid = sample_s >= 0.0
            && sample_s <= f64::min(number_sample, population_s)
            && sample_s >= f64::max(0.0, number_sample + population_s - number_pop)
            && number_sample > 0.0
            && number_sample <= number_pop
            && population_s > 0.0
            && population_s <= number_pop;
        if !valid {
            return Err(num_error(cell));
        }

        let n_pop = number_pop as u64;
        let k_pop = population_s as u64;
        let n_sample = number_sample as u64;
        let k = sample_s as u64;

        let prob = if cumulative {
            hypergeometric_cdf(n_pop, k_pop, n_sample, k)
        } else {
            Hypergeometric::new(n_pop, k_pop, n_sample)
                .map_err(|_| num_error(cell))?
                .pmf(k)
        };
        if !prob.is_finite() {
            return Err(num_error(cell));
        }
        Ok(prob)
    }
}

#[cfg(test)]
mod tests {
    use super::{cdf_within, hypergeometric_cdf, nonzero_terms, MAX_TERMS, ZERO_EXPONENT};
    use statrs::distribution::{DiscreteCDF, Hypergeometric};
    use statrs::function::factorial::ln_binomial;

    /// A small xorshift, so the sweep is the same on every run.
    struct Sweep(u64);
    impl Sweep {
        fn below(&mut self, bound: u64) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0 % bound.max(1)
        }
    }

    fn statrs_cdf(population: u64, successes: u64, draws: u64, x: u64) -> f64 {
        match Hypergeometric::new(population, successes, draws) {
            Ok(dist) => dist.cdf(x),
            Err(_) => f64::NAN,
        }
    }

    #[test]
    fn exp_is_zero_at_the_threshold() {
        assert_eq!(ZERO_EXPONENT.exp().to_bits(), 0.0f64.to_bits());
        assert_eq!((-745.2f64).exp().to_bits(), 0.0f64.to_bits());
        assert!((-745.0f64).exp() > 0.0);
    }

    // Where statrs' sum has at most `MAX_TERMS` terms from max(0, n + K - N), the port sums the
    // same terms and gives its double, bit for bit.
    #[test]
    fn the_port_is_statrs_within_the_cap() {
        let mut sweep = Sweep(0x1873);
        for _ in 0..3_000 {
            let population = 1 + sweep.below(5_000);
            let successes = 1 + sweep.below(population);
            let draws = 1 + sweep.below(population);
            let x = sweep.below(successes.min(draws) + 1);
            assert_eq!(
                hypergeometric_cdf(population, successes, draws, x).to_bits(),
                statrs_cdf(population, successes, draws, x).to_bits(),
                "({population},{successes},{draws},{x})"
            );
        }
    }

    // Past the cap the port skips the terms far from the mode that are exactly 0, and the sum is
    // still statrs' double, bit for bit: 300 sums with a cap of 2^12, populations up to 1E6 with
    // 5,000 to 25,000 successes (a spread of about 35 to 80, so the terms not 0 number about 2,700
    // to 6,100), at x on both sides of the mode. Where more than the cap are left it is NaN.
    #[test]
    fn skipping_the_zero_terms_keeps_statrs_double() {
        let max_terms = 1 << 12;
        let mut sweep = Sweep(0x18_73_02);
        let (mut skipped, mut past_cap) = (0, 0);
        for _ in 0..300 {
            let population = 100_000 + sweep.below(900_000);
            let successes = 5_000 + sweep.below(20_000);
            let draws = population / 4 + sweep.below(population / 2);
            let x = max_terms + sweep.below(successes - max_terms);
            let ours = cdf_within(population, successes, draws, x, max_terms);
            let theirs = statrs_cdf(population, successes, draws, x);
            if ours.is_nan() {
                past_cap += 1;
                continue;
            }
            skipped += 1;
            assert_eq!(
                ours.to_bits(),
                theirs.to_bits(),
                "({population},{successes},{draws},{x})"
            );
        }
        assert!(skipped > 150, "{skipped} sums skipped zero terms");
        assert!(past_cap > 50, "{past_cap} sums past the cap");
    }

    // `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` took about 4.8 s on main (statrs
    // sums 25,000,001 terms, 192.9 ns each in release); 95,942 of them are not 0, and the port
    // sums only those, to the same double (statrs' value, measured in release).
    #[test]
    fn the_slowest_call_on_main_keeps_its_double() {
        let ours = hypergeometric_cdf(100_000_000, 50_000_000, 50_000_000, 25_000_000);
        assert_eq!(ours.to_bits(), 0x3fe0_00a7_2983_49b1, "{ours}");
    }

    // The widest distribution at a population of 1.8E8, 258,239 terms that are not known to
    // be 0, just under the cap (the lift bench's case): statrs' double (measured in release).
    #[test]
    fn a_sum_near_the_cap_keeps_statrs_double() {
        let ours = hypergeometric_cdf(180_000_000, 90_000_000, 90_000_000, 45_500_000);
        assert_eq!(ours.to_bits(), 0x3fef_fffe_147e_ac9c, "{ours}");
    }

    // Both sides of the cap: a sum of exactly `max_terms` terms is statrs' double and one more is
    // NaN, counted from max(0, n + K - N) where nothing is skipped (a population above 2^52) and
    // after the skip.
    #[test]
    fn the_cap_counts_the_terms_summed() {
        let (population, successes, draws) = (1u64 << 60, 1u64 << 59, 1u64 << 20);
        let at_cap = cdf_within(population, successes, draws, 63, 64);
        assert_eq!(
            at_cap.to_bits(),
            statrs_cdf(population, successes, draws, 63).to_bits()
        );
        assert!(cdf_within(population, successes, draws, 64, 64).is_nan());

        let (population, successes, draws, x) = (400_000, 20_000, 200_000, 13_500);
        let ln_denom = ln_binomial(population, draws);
        let exponent = |i: u64| {
            ln_binomial(successes, i) + ln_binomial(population - successes, draws - i) - ln_denom
        };
        let (first, end) = nonzero_terms(population, successes, draws, 0, x, exponent).unwrap();
        let summed = end - first;
        assert!(first > 0 && end <= x && summed < x, "[{first}, {end})");
        let at_cap = cdf_within(population, successes, draws, x, summed);
        assert_eq!(
            at_cap.to_bits(),
            statrs_cdf(population, successes, draws, x).to_bits()
        );
        assert!(cdf_within(population, successes, draws, x, summed - 1).is_nan());
    }

    #[test]
    fn a_call_past_its_cap_is_nan() {
        // The widest distribution at a population of 2^32: about 72,000 sigma of 16,384 each
        // side, so far more than 2^18 terms are not 0.
        let n = 1u64 << 32;
        assert!(hypergeometric_cdf(n, n / 2, n / 2, n / 4 + 1_000_000).is_nan());
        // A population above 2^52 skips nothing.
        let n = 1u64 << 60;
        assert!(hypergeometric_cdf(n, n / 2, 1 << 20, 1 << 19).is_nan());
        // Within the cap counted from max(0, n + K - N), and at the ends, the cdf is statrs'.
        assert_eq!(hypergeometric_cdf(n, n / 2, 1 << 20, 0).to_bits(), {
            statrs_cdf(n, n / 2, 1 << 20, 0).to_bits()
        });
        assert_eq!(hypergeometric_cdf(n, n / 2, 1 << 20, 1 << 20), 1.0);
        // The 2^19 + 1 terms above are past the cap.
        const { assert!(MAX_TERMS < 1 << 19) };
    }
}
