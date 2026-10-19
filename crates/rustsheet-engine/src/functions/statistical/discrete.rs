//! Binomial and Poisson probabilities as sums of their pmf terms in double-double arithmetic (T-0927).
//!
//! BINOM.DIST, BINOM.DIST.RANGE, NEGBINOM.DIST and POISSON.DIST took their cdfs from statrs'
//! regularized incomplete beta and gamma, about 1E-14 off where Excel is within an ulp or two of the
//! exact value (`BINOM.DIST(7,8,0.7,TRUE)` is Excel's 0.94235199, statrs' 0.9423519899999973). Here a
//! cdf is the sum of its pmf terms, each from the last by the exact ratio of consecutive terms, every
//! product and sum carried in double-double: for the binomial the only rounding that reaches the
//! result is the last one, the exact value rounded. POISSON.DIST is Excel's `fl(fl(S)*exp(-mean))`,
//! S the partial exponential series so summed (docs/excel-semantics/statistics.md "Discrete
//! distributions: within an ulp of the exact sum").
//!
//! A walk takes one step a term, so each one has a budget, [`MAX_STEPS`]; past it the caller falls
//! back to statrs. The terms of a walk start in a tail, where they can be far below the smallest
//! double (`0.5^3000`), and grow towards the mode, so a walk carries a power-of-two exponent beside
//! its double-doubles.

use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_mul, excel_exp, two_sum, LN_2_HI, LN_2_LO};

/// A double-double: the head and the error of `head + tail`.
type Dd = (f64, f64);

/// The most terms one walk takes before its caller falls back to statrs. A step is three
/// double-double products or quotients and a sum, about 32 ns in release (T-0927, 2^20 steps of
/// `binomial_walk` in 34 ms, of `poisson_walk` in 25 to 27 ms), so a walk is bounded at about
/// 33 us and a call, which walks at most twice before statrs, at about 66 us; the lifting table
/// charges that bound (lift/kinds/statistical.rs `DISCRETE_SUM_WEIGHT`; POISSON.DIST's weight,
/// `INCOMPLETE_GAMMA_WEIGHT`, is the 2.6 ms bound of its statrs fallback). 2^10 reaches every
/// suite cell but POISSON.DIST at counts past 1,024 (ironcalc-calc POISSON.xlsx D31 and D55).
pub(super) const MAX_STEPS: u64 = 1 << 10;

/// The scale past which a walk renormalizes its terms and sum by `2^-RESCALE_BITS`. One step grows
/// a term by at most `2^53 * 2^53` (a count below 2^53 times a ratio p/q below 2^53), so a sum
/// below `2^RESCALE_BITS` cannot overflow before its next check.
const RESCALE_BITS: i32 = 600;

/// A term or sum below `2^-NEGLIGIBLE_BITS` of the running sum no longer moves its double-double.
const NEGLIGIBLE_BITS: i32 = 110;

/// `x * 2^k` for any `k`, exact unless the result leaves the double range.
fn scale_by_power_of_two(mut x: f64, k: i64) -> f64 {
    let mut k = k.clamp(-3000, 3000);
    // Steps of at most 2^1000 keep every factor a normal double.
    while k != 0 {
        let step = k.clamp(-1000, 1000);
        x *= f64::from_bits(u64::try_from(step + 1023).unwrap_or(1023) << 52);
        k -= step;
    }
    x
}

/// The binary exponent `e` with `2^e <= |x| < 2^(e+1)` of a finite nonzero double, and `x / 2^e`;
/// 0, an infinity and NaN are their own mantissa at exponent 0.
fn normalize(x: f64) -> (f64, i64) {
    if x == 0.0 || !x.is_finite() {
        return (x, 0);
    }
    let biased = (x.to_bits() >> 52) & 0x7ff;
    if biased == 0 {
        // Subnormal: scale into the normal range first.
        let (mantissa, exponent) = normalize(x * 2f64.powi(64));
        return (mantissa, exponent - 64);
    }
    let exponent = i64::try_from(biased).unwrap_or(1023) - 1023;
    (scale_by_power_of_two(x, -exponent), exponent)
}

/// A positive double-double times `2^exponent`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Scaled {
    value: Dd,
    exponent: i64,
}

impl Scaled {
    const ZERO: Self = Scaled {
        value: (0.0, 0.0),
        exponent: 0,
    };

    /// The value with its head in `[1, 2)`; zero stays zero.
    fn normalized(self) -> Self {
        if self.value.0 == 0.0 || !self.value.0.is_finite() {
            return self;
        }
        let (_, shift) = normalize(self.value.0);
        Scaled {
            value: (
                scale_by_power_of_two(self.value.0, -shift),
                scale_by_power_of_two(self.value.1, -shift),
            ),
            exponent: self.exponent.saturating_add(shift),
        }
    }

    fn mul(self, other: Self) -> Self {
        Scaled {
            value: dd_mul(self.value, other.value),
            exponent: self.exponent.saturating_add(other.exponent),
        }
        .normalized()
    }

    /// The value as a double-double at exponent 0 (each half rounded once if it leaves the range).
    fn to_dd(self) -> Dd {
        (
            scale_by_power_of_two(self.value.0, self.exponent),
            scale_by_power_of_two(self.value.1, self.exponent),
        )
    }

    /// The value rounded to a double (twice where it is subnormal, which the walks' results are
    /// not in the cases Excel's workbooks probe).
    pub(super) fn to_f64(self) -> f64 {
        let (head, tail) = two_sum(self.value.0, self.value.1);
        scale_by_power_of_two(head, self.exponent) + scale_by_power_of_two(tail, self.exponent)
    }
}

/// `base^power` for a double-double base in (0, 1], by squaring.
fn power(base: Dd, mut power: u64) -> Scaled {
    let mut result = Scaled {
        value: (1.0, 0.0),
        exponent: 0,
    };
    let mut square = Scaled {
        value: base,
        exponent: 0,
    }
    .normalized();
    while power > 0 {
        if power & 1 == 1 {
            result = result.mul(square);
        }
        power >>= 1;
        if power > 0 {
            square = square.mul(square);
        }
    }
    result
}

/// `exp(-mean)` for a mean of at least 0, as a scaled double-double: `2^-j * exp(r)` with
/// `r = j ln 2 - mean` in [-0.35, 0.35] and `exp(r)` its Taylor series in double-double.
fn exp_negative(mean: f64) -> Scaled {
    // Past 2^40 the reduction below would lose bits; such a mean puts every count a walk can
    // reach (at most 2^10) at a probability far below the smallest double.
    if mean > 2f64.powi(40) {
        return Scaled::ZERO;
    }
    let j = (mean / std::f64::consts::LN_2).round();
    // r = j * ln 2 - mean, with j * LN_2_HI exact as a head and its FMA error.
    let product = j * LN_2_HI;
    let product_error = j.mul_add(LN_2_HI, -product);
    let r = dd_add(
        dd_add((product, product_error), (-mean, 0.0)),
        (j * LN_2_LO, j.mul_add(LN_2_LO, -(j * LN_2_LO))),
    );
    // |r| < 0.35: 27 terms leave a remainder below 0.35^27 / 27! < 1E-40.
    let mut term: Dd = (1.0, 0.0);
    let mut sum: Dd = (1.0, 0.0);
    for i in 1..=27u8 {
        term = dd_div_small(dd_mul(term, r), f64::from(i));
        sum = dd_add(sum, term);
    }
    Scaled {
        value: sum,
        exponent: -(j as i64),
    }
    .normalized()
}

/// What a walk returns: the sum of its terms `lo..=hi` and its term at `hi`.
#[derive(Clone, Copy, Debug)]
struct Walk {
    sum: Scaled,
    last: Scaled,
}

/// The binomial terms `C(n, i) p^i q^(n-i)` summed over `lo..=hi`, with `p + q = 1` exactly as
/// double-doubles and `0 < p, q`. The walk starts at `q^n` and takes `hi` steps (`None` past
/// [`MAX_STEPS`] or 2^53 trials), fewer when the terms past the mode fall below the sum's last bit
/// (only when `lo < hi`, so that a pmf, `lo = hi`, always reaches its term).
fn binomial_walk(n: u64, p: Dd, q: Dd, lo: u64, hi: u64) -> Option<Walk> {
    // Past 2^53 trials a count is no longer exact as a double, so neither is a ratio of terms.
    if n > 1 << 53 {
        return None;
    }
    let ratio = dd_div(p, q);
    let start = power(q, n);
    // The walk's terms share the exponent of the start term.
    let mut exponent = start.exponent;
    let mut term = start.value;
    let mut sum: Dd = if lo == 0 { term } else { (0.0, 0.0) };
    for i in 0..hi {
        if i >= MAX_STEPS {
            return None;
        }
        let count = (n - i) as f64;
        let next = i + 1;
        term = dd_div_small(dd_mul(dd_mul(term, ratio), (count, 0.0)), next as f64);
        if next >= lo {
            sum = dd_add(sum, term);
        }
        if sum.0 > 2f64.powi(RESCALE_BITS) || term.0 > 2f64.powi(RESCALE_BITS) {
            let factor = 2f64.powi(-RESCALE_BITS);
            term = (term.0 * factor, term.1 * factor);
            sum = (sum.0 * factor, sum.1 * factor);
            exponent = exponent.saturating_add(i64::from(RESCALE_BITS));
        }
        // Past the mode the terms only shrink: once one is negligible against the sum, so is the
        // rest of the range.
        let past_mode = count - 1.0 < (next as f64 + 1.0) * (q.0 / p.0);
        if next >= lo && lo < hi && past_mode && term.0 < sum.0 * 2f64.powi(-NEGLIGIBLE_BITS) {
            break;
        }
    }
    Some(Walk {
        sum: Scaled {
            value: sum,
            exponent,
        },
        last: Scaled {
            value: term,
            exponent,
        },
    })
}

/// The partial exponential series `mean^i / i!` summed over `0..=hi`, and its term at `hi`: the
/// Poisson pmf terms before their factor `exp(-mean)`, whose binary exponent is `scale` (for the
/// pmf's stop). As [`binomial_walk`], a cdf stops early once the terms past the mode are
/// negligible; a pmf (`pmf_only`) stops early only once its term times `exp(-mean)` has fallen
/// below every double.
fn poisson_walk(mean: f64, hi: u64, pmf_only: bool, scale: i64) -> Option<Walk> {
    let mut exponent = 0i64;
    let mut term: Dd = (1.0, 0.0);
    let mut sum = term;
    for i in 1..=hi {
        if i > MAX_STEPS {
            return None;
        }
        let index = i as f64;
        term = dd_div_small(dd_mul(term, (mean, 0.0)), index);
        sum = dd_add(sum, term);
        if sum.0 > 2f64.powi(RESCALE_BITS) {
            let factor = 2f64.powi(-RESCALE_BITS);
            term = (term.0 * factor, term.1 * factor);
            sum = (sum.0 * factor, sum.1 * factor);
            exponent = exponent.saturating_add(i64::from(RESCALE_BITS));
        }
        let past_mode = index > mean;
        if past_mode {
            if !pmf_only && term.0 < sum.0 * 2f64.powi(-NEGLIGIBLE_BITS) {
                break;
            }
            // A pmf whose term has underflowed past every double stays 0 from here on.
            let magnitude = exponent
                .saturating_add(scale)
                .saturating_add(normalize_exponent(term.0));
            if term.0 == 0.0 || magnitude < -1100 {
                term = (0.0, 0.0);
                break;
            }
        }
    }
    Some(Walk {
        sum: Scaled {
            value: sum,
            exponent,
        },
        last: Scaled {
            value: term,
            exponent,
        },
    })
}

/// `exp(-mean)` as a double times a power of two: Excel's own EXP where that is a normal double,
/// past it the double-double series rounded once.
fn exp_negative_rounded(mean: f64) -> Scaled {
    if mean <= 700.0 {
        return Scaled {
            value: (excel_exp(-mean), 0.0),
            exponent: 0,
        }
        .normalized();
    }
    let exact = exp_negative(mean);
    Scaled {
        value: (two_sum(exact.value.0, exact.value.1).0, 0.0),
        exponent: exact.exponent,
    }
}

/// Excel's Poisson value from a walk's sum or term: that series value rounded once to a double,
/// times `exp(-mean)`, rounded once more (fitted to ironcalc-calc STATISTICAL/POISSON.xlsx: 8 of
/// 11 distinct small cdfs and all 5 pmfs bit for bit, the rest within an ulp; the exact sum gives
/// only 6 of the 11 cdfs).
fn times_exp_negative(series: Scaled, factor: Scaled) -> f64 {
    let rounded = Scaled {
        value: (two_sum(series.value.0, series.value.1).0, 0.0),
        exponent: series.exponent,
    };
    rounded.mul(factor).to_f64()
}

/// The binary exponent of a positive double.
fn normalize_exponent(x: f64) -> i64 {
    normalize(x).1
}

/// A success probability and its complement as exact double-doubles, `0 < p < 1`.
pub(super) type Odds = (Dd, Dd);

/// `p` and `1 - p` as exact double-doubles.
pub(super) fn odds(p: f64) -> Odds {
    ((p, 0.0), two_sum(1.0, -p))
}

/// `P(X = k)` for X ~ Binomial(n, p), walking from the nearer end; `None` past the budget.
pub(super) fn binomial_pmf(n: u64, (p, q): Odds, k: u64) -> Option<Scaled> {
    if k > n {
        return Some(Scaled::ZERO);
    }
    let walk = if k <= n - k {
        binomial_walk(n, p, q, k, k)
    } else {
        binomial_walk(n, q, p, n - k, n - k)
    };
    walk.map(|walk| walk.last)
}

/// `P(lo <= X <= hi)` for X ~ Binomial(n, p): a walk from the end whose walk is shorter, then from
/// the other; `None` when both pass the budget.
pub(super) fn binomial_range(n: u64, (p, q): Odds, lo: u64, hi: u64) -> Option<f64> {
    if lo > hi || hi > n {
        return None;
    }
    let lower = || binomial_walk(n, p, q, lo, hi);
    let upper = || binomial_walk(n, q, p, n - hi, n - lo);
    let walk = if hi <= n - lo {
        lower().or_else(upper)
    } else {
        upper().or_else(lower)
    };
    walk.map(|walk| walk.sum.to_f64())
}

/// `P(X <= k)` for X ~ Binomial(n, p): the sum from 0, or past the budget 1 minus the sum from n
/// when that is not small (below 2^-20 its subtraction would cost bits); `None` otherwise.
pub(super) fn binomial_cdf(n: u64, (p, q): Odds, k: u64) -> Option<f64> {
    if k >= n {
        return Some(1.0);
    }
    if let Some(walk) = binomial_walk(n, p, q, 0, k) {
        return Some(walk.sum.to_f64());
    }
    let upper = binomial_walk(n, q, p, 0, n - k - 1)?;
    let value = one_minus(upper.sum.to_dd());
    (value >= 2f64.powi(-20)).then_some(value)
}

/// `P(X = k)` for X ~ Poisson(mean), `mean > 0`; `None` past the budget.
pub(super) fn poisson_pmf(mean: f64, k: u64) -> Option<f64> {
    let factor = exp_negative_rounded(mean);
    if factor.value.0 == 0.0 {
        // Past a mean of 2^40 (below, a term grows by at most 2^40 a step, which the rescaling
        // allows for) every count a walk reaches has a probability below the smallest double.
        return (k <= MAX_STEPS).then_some(0.0);
    }
    let walk = poisson_walk(mean, k, true, factor.exponent)?;
    Some(times_exp_negative(walk.last, factor))
}

/// `P(X <= k)` for X ~ Poisson(mean), `mean > 0`; `None` past the budget.
pub(super) fn poisson_cdf(mean: f64, k: u64) -> Option<f64> {
    let factor = exp_negative_rounded(mean);
    if factor.value.0 == 0.0 {
        // Past a mean of 2^40 (below, a term grows by at most 2^40 a step, which the rescaling
        // allows for) every count a walk reaches has a probability below the smallest double.
        return (k <= MAX_STEPS).then_some(0.0);
    }
    let walk = poisson_walk(mean, k, false, factor.exponent)?;
    Some(times_exp_negative(walk.sum, factor))
}

/// `x * a / b` for a scaled double-double and two counts below 2^53, rounded to a double.
pub(super) fn times_ratio(x: Scaled, a: u64, b: u64) -> f64 {
    let product = dd_div(dd_mul(x.value, (a as f64, 0.0)), (b as f64, 0.0));
    Scaled {
        value: product,
        exponent: x.exponent,
    }
    .to_f64()
}

/// `1 - x` for a double-double, rounded to a double.
fn one_minus(x: Dd) -> f64 {
    let (head, tail) = dd_add((1.0, 0.0), (-x.0, -x.1));
    head + tail
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_negative_matches_libm_near_one() {
        for mean in [0.0, 0.001, 1.0, 5.0, 100.0, 700.0] {
            let value = exp_negative(mean).to_f64();
            let libm = (-mean).exp();
            assert!(
                (value - libm).abs() <= libm * 2e-16,
                "{mean}: {value} vs {libm}"
            );
        }
    }

    #[test]
    fn exp_negative_scales_past_the_double_range() {
        // exp(-1000) = 5.075958897549457E-435: its scaled head and exponent.
        let scaled = exp_negative(1000.0);
        let log2 = (scaled.value.0).log2() + scaled.exponent as f64;
        assert!((log2 - (-1000.0 / std::f64::consts::LN_2)).abs() < 1e-9);
    }

    #[test]
    fn power_underflows_into_the_exponent() {
        let half = power((0.5, 0.0), 3000);
        assert_eq!(half.value, (1.0, 0.0));
        assert_eq!(half.exponent, -3000);
        assert_eq!(half.to_f64(), 0.0);
    }

    /// C(n, i) / 2^n summed over i <= k: exact in a double for n <= 52, so a walk at p = 1/2 must
    /// give it bit for bit.
    fn exact_half_cdf(n: u64, k: u64) -> f64 {
        let mut coefficient: u128 = 1;
        let mut sum: u128 = 0;
        for i in 0..=k {
            sum += coefficient;
            coefficient = coefficient * u128::from(n - i) / u128::from(i + 1);
        }
        sum as f64 / 2f64.powi(i32::try_from(n).unwrap_or(0))
    }

    #[test]
    fn a_fair_coin_is_exact() {
        for n in [1u64, 2, 7, 10, 31, 52] {
            for k in 0..=n {
                assert_eq!(
                    binomial_cdf(n, odds(0.5), k),
                    Some(exact_half_cdf(n, k)),
                    "{k} of {n}"
                );
                let below = if k == 0 {
                    0.0
                } else {
                    exact_half_cdf(n, k - 1)
                };
                let pmf = exact_half_cdf(n, k) - below;
                let walked = binomial_pmf(n, odds(0.5), k).map(Scaled::to_f64);
                assert_eq!(walked, Some(pmf), "{k} of {n}");
                assert_eq!(binomial_range(n, odds(0.5), k, k), Some(pmf), "{k} of {n}");
            }
        }
    }

    #[test]
    fn excel_cells_within_an_ulp() {
        // ironcalc-calc STATISTICAL/BINOM.xlsx and POISSON.xlsx.
        let ulps = |a: f64, b: f64| a.to_bits().abs_diff(b.to_bits());
        let cases = [
            (binomial_cdf(8, odds(0.7), 7), 0.94235199),
            (binomial_cdf(10_000, odds(0.1), 1000), 0.508421039265265),
            (poisson_cdf(5.0, 2), 0.12465201948308113),
            (poisson_cdf(5.0, 10), 0.9863047314016171),
            (poisson_cdf(100.0, 100), 0.5265621985299984),
            (poisson_pmf(5.0, 10), 0.018132788707821874),
        ];
        for (ours, excel) in cases {
            let ours = ours.unwrap_or(f64::NAN);
            assert!(ulps(ours, excel) <= 1, "{ours} vs {excel}");
        }
    }

    #[test]
    fn walks_past_the_budget_are_none() {
        assert!(binomial_pmf(1 << 40, odds(0.5), 1 << 39).is_none());
        assert!(binomial_cdf(1 << 40, odds(0.5), 1 << 39).is_none());
        assert!(binomial_range(1 << 40, odds(0.5), 1 << 38, 1 << 39).is_none());
        assert!(poisson_cdf(2000.0, 2000).is_none());
        // Past the mode a cdf stops once its terms are negligible, whatever k.
        assert_eq!(binomial_cdf(1 << 40, odds(1e-12), 1 << 39), Some(1.0));
        assert_eq!(poisson_cdf(5.0, 1 << 40), Some(1.0));
        // A pmf far past the mode stops once its term is below every double.
        assert_eq!(poisson_pmf(5.0, 1 << 40), Some(0.0));
    }

    #[test]
    fn extreme_odds_stay_finite() {
        // Terms far below the smallest double, a ratio near 2^53 and a start term of 0.5^3000.
        assert_eq!(
            binomial_pmf(3000, odds(0.5), 0).map(Scaled::to_f64),
            Some(0.0)
        );
        assert_eq!(binomial_cdf(3000, odds(0.5), 3000), Some(1.0));
        let near_one = 1.0 - f64::EPSILON / 2.0;
        assert_eq!(
            binomial_cdf(100, odds(near_one), 99).map(f64::is_finite),
            Some(true)
        );
        let all = binomial_pmf(100, odds(near_one), 100).map(Scaled::to_f64);
        assert_eq!(all, Some(near_one.powi(100)));
        assert_eq!(poisson_pmf(1e12, 3), Some(0.0));
        assert_eq!(poisson_cdf(1e300, 3), Some(0.0));
        assert_eq!(poisson_cdf(1000.0, 900).map(f64::is_finite), Some(true));
    }
}
