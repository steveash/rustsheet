//! Binomial probabilities as sums of their pmf terms in double-double arithmetic (T-0927).
//!
//! BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST took their cdfs from statrs' regularized
//! incomplete beta, about 1E-14 off where Excel is within an ulp or two of the
//! exact value (`BINOM.DIST(7,8,0.7,TRUE)` is Excel's 0.94235199, statrs' 0.9423519899999973). Here a
//! cdf is the sum of its pmf terms, each from the last by the exact ratio of consecutive terms, every
//! product and sum carried in double-double: the only rounding that reaches the result is the last
//! one, so the result is the exact value rounded (docs/excel-semantics/statistics.md "Discrete
//! distributions: within an ulp of the exact sum").
//!
//! A walk takes one step a term, so each one has a budget, [`MAX_STEPS`]; past it the caller falls
//! back to statrs. The terms of a walk start in a tail, where they can be far below the smallest
//! double (`0.5^3000`), and grow towards the mode, so a walk carries a power-of-two exponent beside
//! its double-doubles.

use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_mul, two_sum};

/// A double-double: the head and the error of `head + tail`.
type Dd = (f64, f64);

/// The most terms one walk takes before its caller falls back to statrs. A step is three
/// double-double products or quotients and a sum, about 32 ns in release (T-0927: 2^20 steps in
/// 34 ms), so a walk is bounded at about 33 us and a call, which walks at most twice before statrs,
/// at about 66 us; the lifting table charges that bound (lift/kinds/statistical.rs
/// `DISCRETE_SUM_WEIGHT`). 2^10 reaches every binomial suite cell.
pub(super) const MAX_STEPS: u64 = 1 << 10;

/// The scale past which a walk renormalizes its terms and sum by `2^-RESCALE_BITS`. One step grows
/// a term by at most `2^53 * 2^53` (a count below 2^53 times a ratio p/q below 2^53), so a sum
/// below `2^RESCALE_BITS` cannot overflow before its next check. A ratio past 2^53 (q below 2^-53,
/// `NEGBINOMDIST(7,123456789.123,1E-200)`) renormalizes once a term passes 1 instead.
const RESCALE_BITS: i32 = 600;

/// 2^53, the largest ratio p/q for which one step stays within [`RESCALE_BITS`]' headroom.
const RESCALE_RATIO_BOUND: f64 = 9_007_199_254_740_992.0;

/// A term or sum below `2^-NEGLIGIBLE_BITS` of the running sum no longer moves its double-double.
const NEGLIGIBLE_BITS: i32 = 110;

/// `2^RESCALE_BITS`, `2^-RESCALE_BITS` and `2^-NEGLIGIBLE_BITS`, hoisted out of the walks' loops.
const RESCALE_LIMIT: f64 = f64::from_bits(((1023 + RESCALE_BITS) as u64) << 52);
const RESCALE_FACTOR: f64 = f64::from_bits(((1023 - RESCALE_BITS) as u64) << 52);
const NEGLIGIBLE: f64 = f64::from_bits(((1023 - NEGLIGIBLE_BITS) as u64) << 52);

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
    // A range starting past the budget cannot be reached; give up before walking.
    if n > 1 << 53 || lo > MAX_STEPS {
        return None;
    }
    let ratio = dd_div(p, q);
    // Past 2^53 a step can grow a term by up to 2^53 times the ratio, so the walk renormalizes as
    // soon as a term passes 1 (the factor is a power of two, so the terms' bits do not change).
    let rescale_limit = if ratio.0 < RESCALE_RATIO_BOUND {
        RESCALE_LIMIT
    } else {
        1.0
    };
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
        // A ratio near f64::MAX can still overflow one step: give up, the caller falls back.
        if !term.0.is_finite() {
            return None;
        }
        if sum.0 > rescale_limit || term.0 > rescale_limit {
            let factor = RESCALE_FACTOR;
            term = (term.0 * factor, term.1 * factor);
            sum = (sum.0 * factor, sum.1 * factor);
            exponent = exponent.saturating_add(i64::from(RESCALE_BITS));
        }
        // Past the mode the terms only shrink: once one is negligible against the sum, so is the
        // rest of the range.
        let past_mode = count - 1.0 < (next as f64 + 1.0) * (q.0 / p.0);
        if next >= lo && lo < hi && past_mode && term.0 < sum.0 * NEGLIGIBLE {
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

/// The most steps one hypergeometric walk takes (its start term's factors and its ratio steps)
/// before its caller falls back to statrs: 2^12 steps of at most two double-double products and two
/// quotients, 78 us in release for a walk of 4,095 steps (T-0977; lift/kinds/statistical.rs
/// `HYPGEOMDIST_WEIGHT`), where [`MAX_STEPS`]' 2^10 would not reach the population of 1E5 that
/// probes-numeric/t1049-hypgeomdist checks (1,100 steps).
const HYPERGEOMETRIC_MAX_STEPS: u64 = 1 << 12;

/// The hypergeometric terms `C(K, i) C(N - K, n - i) / C(N, n)` (population N, K successes, n
/// draws) summed over `lo..=hi`, with `max(0, n + K - N) <= lo <= hi <= min(n, K)`. The walk starts
/// at the support's lowest term m, a product of `min(n, K)` ratios when m = 0 and of `N - max(n, K)`
/// ratios otherwise (the pmf is symmetric in n and K), then steps by the exact ratio of consecutive
/// terms `(K - i)(n - i) / ((i + 1)(N - K - n + i + 1))`. `None` past [`HYPERGEOMETRIC_MAX_STEPS`]
/// or above a population of 2^52.
fn hypergeometric_walk(
    population: u64,
    successes: u64,
    draws: u64,
    lo: u64,
    hi: u64,
) -> Option<Walk> {
    // Above 2^52 the caller keeps statrs, which is #NUM! near 2^53 as Excel is
    // (`HYPGEOMDIST(1,25,3,ROUND(2^53+1,8))`, gen-m9/35e8168aae2b00f8 A128) where a walk would give
    // 8.3E-15. Excel's own limit is far lower, between 2E9 and 1E10 (statistics.md, T-2429).
    if population > 1 << 52 || draws > population || successes > population {
        return None;
    }
    let (small, big) = (draws.min(successes), draws.max(successes));
    let first = (draws + successes).saturating_sub(population);
    if lo < first || hi < lo || hi > small {
        return None;
    }
    let factors = if first == 0 { small } else { population - big };
    if factors.saturating_add(hi - first) > HYPERGEOMETRIC_MAX_STEPS {
        return None;
    }
    let n = population as f64;
    // The start term: prod (N - big - i) / (N - i) for i < small when m = 0, else
    // prod (small - i) / (N - i) for i < N - big, every factor below 1.
    let mut term: Dd = (1.0, 0.0);
    let mut exponent: i64 = 0;
    for i in 0..factors {
        let numerator = if first == 0 {
            (population - big - i) as f64
        } else {
            (small - i) as f64
        };
        term = dd_div_small(dd_mul(term, (numerator, 0.0)), n - i as f64);
        if term.0 < RESCALE_FACTOR {
            term = (term.0 * RESCALE_LIMIT, term.1 * RESCALE_LIMIT);
            exponent = exponent.saturating_sub(i64::from(RESCALE_BITS));
        }
    }
    let mode = (u128::from(draws) + 1) * (u128::from(successes) + 1) / (u128::from(population) + 2);
    let mut sum: Dd = if lo == first { term } else { (0.0, 0.0) };
    for i in first..hi {
        let next = i + 1;
        term = dd_mul(
            dd_mul(term, ((successes - i) as f64, 0.0)),
            ((draws - i) as f64, 0.0),
        );
        term = dd_div_small(
            dd_div_small(term, next as f64),
            (population + next - successes - draws) as f64,
        );
        if next >= lo {
            sum = dd_add(sum, term);
        }
        if sum.0 > RESCALE_LIMIT || term.0 > RESCALE_LIMIT {
            term = (term.0 * RESCALE_FACTOR, term.1 * RESCALE_FACTOR);
            sum = (sum.0 * RESCALE_FACTOR, sum.1 * RESCALE_FACTOR);
            exponent = exponent.saturating_add(i64::from(RESCALE_BITS));
        } else if sum.0 < RESCALE_FACTOR && term.0 < RESCALE_FACTOR {
            // Past the mode a pmf's terms shrink: lift them (and a sum as small) away from the
            // subnormals.
            term = (term.0 * RESCALE_LIMIT, term.1 * RESCALE_LIMIT);
            sum = (sum.0 * RESCALE_LIMIT, sum.1 * RESCALE_LIMIT);
            exponent = exponent.saturating_sub(i64::from(RESCALE_BITS));
        }
        // Past the mode the terms only shrink: once one is negligible against the sum, so is the
        // rest of the range.
        if next >= lo && lo < hi && u128::from(next) > mode && term.0 < sum.0 * NEGLIGIBLE {
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

/// `P(X = k)` for a hypergeometric X (population N, K successes, n draws), k in its support:
/// the exact value rounded once (T-0977); `None` past the walk's budget.
pub(super) fn hypergeometric_pmf(
    population: u64,
    successes: u64,
    draws: u64,
    k: u64,
) -> Option<f64> {
    hypergeometric_walk(population, successes, draws, k, k).map(|walk| walk.last.to_f64())
}

/// `P(X <= k)` for a hypergeometric X, k in its support: the sum of its terms from the lowest, the
/// exact value rounded once (T-0977); `None` past the walk's budget.
pub(super) fn hypergeometric_cdf_sum(
    population: u64,
    successes: u64,
    draws: u64,
    k: u64,
) -> Option<f64> {
    let first = draws.saturating_add(successes).saturating_sub(population);
    hypergeometric_walk(population, successes, draws, first, k).map(|walk| walk.sum.to_f64())
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
        // A range starting past the budget gives up at once.
        assert!(binomial_walk(4096, (0.5, 0.0), (0.5, 0.0), 2048, 2048).is_none());
        // Past the mode a cdf stops once its terms are negligible, whatever k.
        assert_eq!(binomial_cdf(1 << 40, odds(1e-12), 1 << 39), Some(1.0));
    }

    /// C(a, b) exactly, for the small populations of the hypergeometric tests.
    fn choose(a: u64, b: u64) -> u64 {
        (0..b).fold(1, |acc, i| acc * (a - i) / (i + 1))
    }

    #[test]
    fn hypergeometric_walks_are_the_exact_value_rounded() {
        // Up to a population of 30 every numerator and C(N, n) are below 2^53, so one division
        // of the two doubles is the exact value rounded once.
        for population in 1..=30u64 {
            for successes in 1..=population {
                for draws in 1..=population {
                    let first = (draws + successes).saturating_sub(population);
                    let total = choose(population, draws) as f64;
                    let mut numerator = 0;
                    for k in first..=draws.min(successes) {
                        let term = choose(successes, k) * choose(population - successes, draws - k);
                        numerator += term;
                        let case = format!("{k} of {draws}, {successes} of {population}");
                        let pmf = hypergeometric_pmf(population, successes, draws, k);
                        assert_eq!(pmf, Some(term as f64 / total), "pmf {case}");
                        let cdf = hypergeometric_cdf_sum(population, successes, draws, k);
                        assert_eq!(cdf, Some(numerator as f64 / total), "cdf {case}");
                    }
                }
            }
        }
    }

    #[test]
    fn hypergeometric_walks_at_their_limits() {
        // Excel's HYPGEOM.DIST(1,4,8,2E9,FALSE) is 1.5999999831999996E-08, 3 ulp from the exact
        // value (probes-numeric/t1049-hypgeomdist A51): four start factors and one step.
        let pmf = hypergeometric_pmf(2_000_000_000, 8, 4, 1).unwrap_or(f64::NAN);
        assert!(
            pmf.to_bits().abs_diff(1.5999999831999996e-08f64.to_bits()) <= 4,
            "{pmf}"
        );
        assert!(hypergeometric_pmf(1 << 30, 1 << 20, 1 << 20, 1 << 10).is_none());
        assert!(hypergeometric_pmf((1 << 52) + 1, 3, 9, 1).is_none());
        // Outside the support the caller decides.
        assert!(hypergeometric_pmf(20, 8, 4, 5).is_none());
        assert!(hypergeometric_cdf_sum(10, 9, 9, 7).is_none());
        // A tail far below the smallest double stays finite: 0 rounded.
        assert_eq!(hypergeometric_pmf(4000, 2000, 2000, 0), Some(0.0));
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
    }
}
