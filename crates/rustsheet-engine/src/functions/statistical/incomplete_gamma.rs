//! The regularized incomplete gamma functions behind GAMMA.DIST, GAMMA.INV, CHISQ.DIST,
//! CHISQ.DIST.RT, CHISQ.INV, CHISQ.INV.RT and POISSON.DIST's cdf (and GAMMADIST, GAMMAINV,
//! CHIDIST, CHIINV, POISSON, CHISQ.TEST), with a per-call cap of their own (SPEC-0011 security
//! rule, T-1872).
//!
//! The incomplete gamma itself is an accurate pair P(a, x), Q(a, x) (T-0971; Excel is close to
//! correctly rounded for this family, docs/excel-semantics/statistics.md "Incomplete gamma:
//! accurate kernel"): the prefactor of TOMS 654, the power series for P below x = a + 1
//! and a continued fraction for Q above it, Q computed directly for a small shape, and a = 1 as
//! `-expm1(-x)`. `inverse_cdf_within` is statrs 0.18.0's `Gamma::inverse_cdf` (halve and double
//! a bracket, then 8 bisections and 4 Newton steps) over that cdf; the quantile the functions
//! call is [`gamma_quantile`], which refines it to the crossing in the tails (T-1634). Neither loop is open: the
//! series needs about 6.8 * sqrt(a) terms where x is near a (6.4 million at a = 1E12), and never
//! converges once `a + 1 == a` (a of 2^53 or more: `GAMMA.DIST(1E16,1E16,1,TRUE)`), so every
//! call is NaN, which its callers turn into `#NUM!` (docs/excel-semantics/statistics.md
//! "Incomplete gamma: per-call cap and lifting", a DEVIATION), past [`MAX_TERMS`] series or
//! continued-fraction terms in one cdf or in one whole inversion, where `x * rate` overflows
//! (`GAMMA.DIST(1E300,1,1E-10,TRUE)`, where statrs panicked) and where the search's statrs
//! would have panicked; where `x * rate` underflows to 0 (`GAMMA.DIST(1E-300,2,1E100,TRUE)`) the
//! cdf is 0, as in Excel.
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

use std::cell::RefCell;

use statrs::distribution::{Continuous, ContinuousCDF, Gamma};
use statrs::prec;

use super::incomplete_beta::{rlog1, stirling_correction};
use super::special::refine_increasing;

/// The most series or continued-fraction terms one cdf, or one whole inversion, may take. The
/// series needs about 6.8 * sqrt(a) terms at its slowest, x = a (it is used for x <= a, and needs
/// fewer as x falls): 481,117 at a = 5E9, the largest shape CHISQ.* takes (df 1E10), so 2^19 keeps
/// every chi-squared cdf statrs gives, and every gamma or Poisson cdf with a shape up to about
/// 5.9E9 (675,286 terms at 1E10 are past it). The continued fraction (x above a) also grows with
/// sqrt(a) at x just above a: 8,839 terms at a = 1E9, 90,016 at 1E12, 402,191 at 1E14, and past
/// the cap at about 1E15 (`GAMMA.DIST(1000000000000001,1E15,1,TRUE)`). An inversion's cdfs together need about 26 * sqrt(a) terms for a
/// large shape: in 3 million samples (shapes 1E-3 to 2.4E10, rates 1E-12 to 1E12, probabilities
/// 1E-307 to 1-1E-16, T-1872) the smallest shape whose inversion statrs ends with a number and
/// that needed more than 2^19 terms was 3.5E8, so the inversions with a smaller shape are kept.
/// Those counts are statrs' series (T-1872); the accurate one (T-0971) runs to 2^-54, not 1E-15,
/// about 8% more terms near x = a (a = 5E9 stays under the cap), so an inversion's budget runs
/// out at a smaller shape: 7.9E8 at p = 1E-5 is past it, 3E8 within (T-2367 measures the bound).
pub(crate) const MAX_TERMS: u32 = 1 << 19;

/// The most continued-fraction terms one cdf of shape `a` may take: 256 + 32 * cbrt(a), past
/// which the cdf is NaN, as past [`MAX_TERMS`]. The fraction is used for x >= a + 1 and needs
/// most terms at x just above a + 1, about 10 * cbrt(a) for a above 10 and at most 110 below
/// (T-2795 sweep, run once: shapes 1E-3 to 1.6E15, 4,200 x each; 1,064,339 terms at 1.6E15), so
/// the budget is at least twice what any cdf the cap keeps needs and changes no value. The unit
/// test `the_continued_fraction_stays_within_half_its_budget` keeps that for shapes up to 1E10.
/// It gives the cdf a bound from its shape alone ([`cdf_terms_bound`]), which the lifting table
/// charges by value.
fn continued_fraction_budget(a: f64) -> u32 {
    let budget = 256.0 + 32.0 * a.cbrt();
    if budget < f64::from(MAX_TERMS) {
        // A whole number below 2^19 after `ceil`, so the cast is exact.
        budget.ceil() as u32
    } else {
        MAX_TERMS
    }
}

/// The terms the small-shape Q series ([`small_shape_upper`]) takes at most beside the power
/// series: its terms `x^n / (n! (a + n))` (x < 1, a < 1/2) fall under 2^-54 of its sum by
/// n = 19, as the sum's partial sums stay above 0.41 x (alternating terms that fall).
const SMALL_SHAPE_TERMS: u32 = 20;

/// The terms the power series ([`lower_series`]) takes past `a` at most: it runs for x < a + 1,
/// so every ratio `x / (a + n)` is below 1, and from n = 2x - a on (at most a + 3) each is at most
/// 1/2, so within 55 more terms the last is under 2^-54 of the sum.
const SERIES_TERMS_PAST_SHAPE: u32 = 58;

/// The most terms one cdf of shape at most `a` takes, whatever its x: the power series' bound
/// (with the small-shape Q series) or the continued fraction's budget, at most [`MAX_TERMS`].
/// It does not fall as `a` grows, so a bound of the shape is enough.
pub(crate) fn cdf_terms_bound(a: f64) -> u32 {
    let series = a + f64::from(SERIES_TERMS_PAST_SHAPE + SMALL_SHAPE_TERMS);
    let series = if series < f64::from(MAX_TERMS) {
        // Below 2^19 after `ceil`, so the cast is exact.
        series.ceil() as u32
    } else {
        MAX_TERMS
    };
    series.max(continued_fraction_budget(a)).min(MAX_TERMS)
}

/// The cdfs one inversion ([`gamma_quantile`]) runs at most: statrs' search (1,075 halvings,
/// 1,025 doublings, 8 bisections, 4 Newton steps), then one check and at most 129 more of the
/// refinement (T-1634, T-1872). The lifting table charges an inversion by it (T-2795).
pub(crate) const INVERSION_CDFS: u32 = 2_250;

/// The terms a cdf or an inversion has left. Once spent, the cdf in progress is NaN.
struct Terms(u32);

impl Terms {
    /// Takes one term, or `false` when none is left.
    fn take(&mut self) -> bool {
        match self.0.checked_sub(1) {
            Some(left) => {
                self.0 = left;
                true
            }
            None => false,
        }
    }
}

/// The regularized upper incomplete gamma Q(a, x) (POISSON's cdf is Q(k + 1, mean)) for a > 0
/// and a finite x > 0, else NaN, in at most [`MAX_TERMS`] terms.
pub(crate) fn gamma_ur(a: f64, x: f64) -> f64 {
    if !(a > 0.0 && x > 0.0 && x.is_finite()) {
        return f64::NAN;
    }
    incomplete_gamma_pair(a, x, &mut Terms(MAX_TERMS)).1
}

/// statrs' `Gamma::cdf` within `terms`: its closed branches are statrs' own.
fn cdf_within(dist: &Gamma, x: f64, terms: &mut Terms) -> f64 {
    if x <= 0.0 || dist.rate().is_infinite() || x.is_infinite() {
        return dist.cdf(x);
    }
    let scaled = x * dist.rate();
    if scaled == 0.0 {
        return lower_of_underflow(dist.shape());
    }
    if !scaled.is_finite() {
        return f64::NAN;
    }
    incomplete_gamma_pair(dist.shape(), scaled, terms).0
}

/// P(a, x) where `x / beta` underflows to 0 for an x above 0: statrs panics there (a domain
/// error), so this is statrs' value at the smallest x it takes, its `x < 1.1E-15` branch: 0, or 1
/// for a shape that small too. Excel's `GAMMA.DIST(1E-300,2,1E100,TRUE)` is 0
/// (probes/incomplete-gamma-cap E7).
fn lower_of_underflow(a: f64) -> f64 {
    if prec::almost_eq(a, 0.0, prec::DEFAULT_F64_ACC) {
        1.0
    } else {
        0.0
    }
}

/// The shape below which [`gamma_prefactor`] forms x^a e^-x / Gamma(a) directly, and above which
/// it takes TOMS 654's `rcomp`; GAMMA.DIST's tiny-ratio log form keeps to the same split.
pub(super) const DIRECT_SHAPE_MAX: f64 = 20.0;

/// x^a e^-x / Gamma(a), the factor in front of both P(a, x) and Q(a, x). For a large shape the
/// exponent a ln x - x - ln Gamma(a) cancels to a small number and loses its digits, so it is
/// rewritten as sqrt(a / 2 pi) exp(-a rlog1(x/a - 1) - correction(a)) (TOMS 654's
/// `rcomp`); below 20 it is formed directly.
fn gamma_prefactor(a: f64, x: f64) -> f64 {
    if a < DIRECT_SHAPE_MAX {
        // x^a and e^-x each to within an ulp; their logs summed would carry the absolute error
        // of a ln x into the result's relative error (11 ulps at GAMMA.DIST(10,9,2,TRUE)).
        let power = x.powf(a);
        let decay = (-x).exp();
        let direct = power * decay / libm::tgamma(a);
        // Each factor must be normal too: e^-744 is subnormal, with 3 digits left, even where the
        // product is not.
        if power.is_normal() && decay.is_normal() && direct.is_normal() {
            return direct;
        }
        return (a * x.ln() - x - super::special::ln_gamma(a)).exp();
    }
    // rlog(x / a) = x/a - 1 - ln(x / a). Within a factor of 2 of a, x - a is exact (Sterbenz) and
    // (x - a) / a one rounding; further out x / a is the one rounding, where (x - a) / a would lose
    // the digits of 1 + u (2.5E-12 of P(93.4, 0.18)).
    let rlog = if x >= a * 0.5 && x <= a * 2.0 {
        rlog1((x - a) / a)
    } else {
        let ratio = x / a;
        (ratio - 1.0) - ratio.ln()
    };
    let exponent = -a * rlog - stirling_correction(a);
    (a / (2.0 * std::f64::consts::PI)).sqrt() * exponent.exp()
}

/// The gamma density x^(a-1) e^(-x/scale) / (scale^a Gamma(a)) for a > 0, x > 0 and scale > 0,
/// as [`gamma_prefactor`]`(a, x / scale) / x`: the prefactor P and Q share, so the density keeps
/// its digits where statrs' `exp(ln pdf)` loses the absolute error of its log (T-0971). `None` at
/// x = 0 or where x / scale is not normal, which keep statrs' closed forms.
pub(crate) fn gamma_density(a: f64, x: f64, scale: f64) -> Option<f64> {
    let y = x / scale;
    if x <= 0.0 || !y.is_normal() {
        return None;
    }
    // The exponential density e^(-x/scale) / scale, as statrs forms it: one rounding fewer than
    // the prefactor's y e^-y / x (gen-m9/ecc87f80077959e0 A261 `CHISQ.DIST(1.5,2,FALSE)`).
    if a == 1.0 {
        return Some((-y).exp() / scale);
    }
    Some(gamma_prefactor(a, y) / x)
}

/// The shape below which, for x below 1, Q is not 1 - P but [`small_shape_upper`]: there P runs
/// up to 1 as the shape falls (P(0.00113, 8E-4) = 0.992), and 1 - P keeps only the digits P and
/// 1 share. At a shape of 1/2 or more and x below 1, Q is at least Q(1/2, 1) = 0.157.
const SMALL_SHAPE: f64 = 0.5;

/// (-1)^k (zeta(k) - 1) / k for k = 2, 3, ..., 30 (computed to 40 digits with mpmath), the
/// coefficients of ln Gamma(1 + a) + ln(1 + a) - (1 - Euler's gamma) a, a series in a that
/// converges by a factor of a / 2 a term: at a = 1/2 the 30th term is under 1E-19.
const LN_GAMMA_1P_SERIES: [f64; 29] = [
    0.3224670334241132,
    -0.0673523010531981,
    0.020580808427784546,
    -0.007385551028673986,
    0.0028905103307415234,
    -0.001192753911703261,
    0.0005096695247430425,
    -0.00022315475845357939,
    9.945751278180853e-05,
    -4.492623673813314e-05,
    2.050721277567069e-05,
    -9.439488275268397e-06,
    4.374866789907488e-06,
    -2.039215753801366e-06,
    9.55141213040742e-07,
    -4.492469198764566e-07,
    2.1207184805554665e-07,
    -1.0043224823968099e-07,
    4.7698101693639804e-08,
    -2.2711094608943164e-08,
    1.0838659214896955e-08,
    -5.183475041970047e-09,
    2.4836745438024785e-09,
    -1.1921401405860912e-09,
    5.731367241678862e-10,
    -2.7595228851242334e-10,
    1.330476437424449e-10,
    -6.4229645638381e-11,
    3.1044247747322276e-11,
];

/// ln Gamma(1 + a) for 0 < a < [`SMALL_SHAPE`] to an ulp of its size, which `ln_gamma(1.0 + a)`
/// is not: 1 + a rounds a to the ulp of 1, an absolute error that is a relative 1E-13 of Q at
/// a of 1E-3.
fn ln_gamma_1p_small(a: f64) -> f64 {
    let mut sum = 0.0;
    for &coefficient in LN_GAMMA_1P_SERIES.iter().rev() {
        sum = (sum + coefficient) * a;
    }
    sum *= a;
    sum + (1.0 - EULER_GAMMA) * a - a.ln_1p()
}

/// Euler's constant.
const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

/// Q(a, x) for a below [`SMALL_SHAPE`] and x below 1, formed directly (TOMS 654's
/// small-shape Q): with u = x^a / Gamma(1 + a) and S = sum (-x)^n / (n! (a + n)) over n >= 1,
/// P = u (1 + a S), so Q = (1 - u) - u a S, where 1 - u is `-expm1(a ln x - ln Gamma(1 + a))`.
/// NaN past `terms`.
fn small_shape_upper(a: f64, x: f64, terms: &mut Terms) -> f64 {
    let exponent = a * x.ln() - ln_gamma_1p_small(a);
    let mut power = 1.0;
    let mut series = 0.0;
    let mut n = 1.0;
    // The terms fall by a factor of x / n: under 20 for any x below 1.
    loop {
        if !terms.take() {
            return f64::NAN;
        }
        power *= -x / n;
        let term = power / (a + n);
        series += term;
        if term.abs() <= series.abs() * f64::EPSILON * 0.25 {
            break;
        }
        n += 1.0;
    }
    -exponent.exp_m1() - exponent.exp() * a * series
}

/// The series sum 1 + x / (a + 1) + x^2 / ((a + 1)(a + 2)) + ... for x below a + 1, to full
/// double precision with the geometric sum of the terms it leaves: P(a, x) is
/// `x^a e^-x / Gamma(a + 1)` times it. NaN past `terms`.
fn lower_series(a: f64, x: f64, terms: &mut Terms) -> f64 {
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut denominator = a;
    loop {
        if !terms.take() {
            return f64::NAN;
        }
        denominator += 1.0;
        term *= x / denominator;
        sum += term;
        if term <= sum * f64::EPSILON * 0.25 {
            break;
        }
    }
    // The terms left fall by a ratio just under x / (denominator + 1) each, so near x = a
    // (where that ratio is 1 - 1E-4 at a = 1E6) they add up to thousands of times the last
    // one: 7E-15 of P(1E6, 1E6) were missing. Their geometric sum restores them, to within
    // the ratio's drift over the tail (under 1%).
    let ratio = x / (denominator + 1.0);
    if ratio < 1.0 {
        sum += term * ratio / (1.0 - ratio);
    }
    sum
}

/// The accurate regularized incomplete gamma pair (P(a, x), Q(a, x)) for a > 0, x > 0 (T-0971):
/// the series for P below x = a + 1 and the continued fraction (modified Lentz) for Q above it,
/// each taken to full double precision, the other tail as 1 minus it (Q directly for a small
/// shape, [`small_shape_upper`]); a = 1 is exactly
/// (-expm1(-x), exp(-x)). NaN past [`MAX_TERMS`] terms (directive 5).
fn incomplete_gamma_pair(a: f64, x: f64, terms: &mut Terms) -> (f64, f64) {
    if a == 1.0 {
        return (-(-x).exp_m1(), (-x).exp());
    }
    let prefactor = gamma_prefactor(a, x);
    if prefactor == 0.0 {
        // x^a e^-x / Gamma(a) underflows: the tail that has it as a factor is 0, as statrs'
        // `ax < MIN_LN` shortcut has it. Without this the loops can never end: past x = 2^54
        // `b += 2` leaves b unchanged and the continued fraction's delta stays an ulp below 1
        // (`CHISQ.DIST(1E30,10,TRUE)`, Excel 1, gen-m9 984cba67bbe6972b A84).
        return if x < a + 1.0 { (0.0, 1.0) } else { (1.0, 0.0) };
    }
    if x < a + 1.0 {
        // P = prefactor / a * sum x^n / ((a + 1) ... (a + n)).
        let sum = lower_series(a, x, terms);
        if sum.is_nan() {
            return (f64::NAN, f64::NAN);
        }
        let lower = prefactor * sum / a;
        if a < SMALL_SHAPE && x < 1.0 {
            let upper = small_shape_upper(a, x, terms);
            // Where Q is at most 1/2, 1 - Q is P to within an ulp, which the series is not as P
            // nears 1: `GAMMADIST(0.5,1E-300,1,TRUE)` is Excel's 1, the series' 1 - 2^-53
            // (gen-m9/11eb3ab4f4f8104c A252 truncates it in POISSON.DIST).
            return if upper.is_nan() {
                (f64::NAN, f64::NAN)
            } else if upper <= 0.5 {
                (1.0 - upper, upper)
            } else {
                (lower, upper)
            };
        }
        return (lower, 1.0 - lower);
    }
    // Q = prefactor / (x + 1 - a - 1(1-a)/(x + 3 - a - 2(2-a)/(x + 5 - a - ...))).
    let tiny = f64::MIN_POSITIVE / f64::EPSILON;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / tiny;
    let mut d = 1.0 / b;
    let mut h = d;
    let mut i = 1.0;
    let mut budget = Terms(continued_fraction_budget(a));
    loop {
        if !terms.take() || !budget.take() {
            return (f64::NAN, f64::NAN);
        }
        let an = -i * (i - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < tiny {
            d = tiny;
        }
        c = b + an / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() <= f64::EPSILON * 0.25 {
            break;
        }
        i += 1.0;
    }
    let upper = prefactor * h;
    (1.0 - upper, upper)
}

/// [`gamma_cdf`]'s and [`gamma_sf`]'s value at a positive, finite `scaled` = x * rate: the
/// accurate pair's `upper` or lower tail.
fn accurate_tail(a: f64, scaled: f64, upper: bool) -> f64 {
    let (p, q) = incomplete_gamma_pair(a, scaled, &mut Terms(MAX_TERMS));
    if upper {
        q
    } else {
        p
    }
}

/// statrs' `Gamma::cdf` (and `ChiSquared::cdf`, a gamma of shape df/2 and rate 1/2), or NaN
/// where statrs panics or would not finish, in at most [`MAX_TERMS`] terms.
pub(crate) fn gamma_cdf(dist: &Gamma, x: f64) -> f64 {
    cdf_within(dist, x, &mut Terms(MAX_TERMS))
}

/// statrs' `Gamma::sf` (and `ChiSquared::sf`), likewise capped.
pub(crate) fn gamma_sf(dist: &Gamma, x: f64) -> f64 {
    if x <= 0.0 || dist.rate().is_infinite() || x.is_infinite() {
        return dist.sf(x);
    }
    let scaled = x * dist.rate();
    if scaled == 0.0 {
        return 1.0 - lower_of_underflow(dist.shape());
    }
    if !scaled.is_finite() {
        return f64::NAN;
    }
    accurate_tail(dist.shape(), scaled, true)
}

/// statrs' `Gamma::inverse_cdf` (and `ChiSquared::inverse_cdf`) for `p` in [0, 1]: the quantile,
/// or NaN where statrs panics, would not finish, or its cdfs together pass [`MAX_TERMS`] terms.
/// It evaluates at most about 2,120 cdfs (about 1,075 halvings, 1,025 doublings, 8 bisections,
/// 4 Newton steps), all of them sharing the one budget. The functions call [`gamma_quantile`],
/// which adds at most 130 more cdfs of refinement to that same budget; this is its test shim.
#[cfg(test)]
fn gamma_inverse_cdf(dist: &Gamma, p: f64) -> f64 {
    inverse_cdf_within(dist, p, &mut Terms(MAX_TERMS))
}

fn inverse_cdf_within(dist: &Gamma, p: f64, terms: &mut Terms) -> f64 {
    // statrs panics outside [0, 1]; its callers never pass such a p.
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return 0.0;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }
    // A NaN cdf is where statrs panicked or the budget ran out: the search ends there.
    let mut cdf = |x: f64| -> Option<f64> {
        let value = cdf_within(dist, x, terms);
        (!value.is_nan()).then_some(value)
    };

    let mut high = 2.0;
    let mut low = 1.0;
    // Each loop ends within about 1,075 steps: `low` reaches 0, where the cdf is 0, and `high`
    // infinity, where it is 1.
    loop {
        let Some(at_low) = cdf(low) else {
            return f64::NAN;
        };
        if at_low <= p {
            break;
        }
        low /= 2.0;
    }
    loop {
        let Some(at_high) = cdf(high) else {
            return f64::NAN;
        };
        if at_high >= p {
            break;
        }
        high *= 2.0;
    }
    let mut x_0 = (high + low) / 2.0;

    for _ in 0..8 {
        let Some(at_x) = cdf(x_0) else {
            return f64::NAN;
        };
        if at_x >= p {
            high = x_0;
        } else {
            low = x_0;
        }
        if prec::convergence(&mut x_0, (high + low) / 2.0) {
            break;
        }
    }

    for _ in 0..4 {
        let Some(at_x) = cdf(x_0) else {
            return f64::NAN;
        };
        let x_next = x_0 - (at_x - p) / dist.pdf(x_0);
        if prec::convergence(&mut x_0, x_next) {
            break;
        }
    }

    x_0
}

/// Which tail's probability a quantile is given: CHISQ.INV and GAMMA.INV the lower,
/// CHISQ.INV.RT (and CHIINV) the upper.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tail {
    Lower,
    Upper,
}

/// The search's quantile (statrs' search over the accurate cdf, T-0971) is kept where the smaller
/// tail's probability at it is within this relative distance of the target; elsewhere it is
/// refined. Kept quantiles match Excel's at ordinary probabilities (T-1045's m9-chisq-inv and
/// m9-gamma-inv sweeps); the search misses in the tails, where it works on the lower cdf only:
/// statrs' `CHISQ.INV(1-1E-10,10)` was
/// 68.16761840669743, 2E-7 relative off its upper tail, where Excel's 68.16761795190413 is the
/// crossing (T-1634).
const STATRS_QUANTILE_RESIDUAL: f64 = 1e-13;

/// Above this shape the quantile is statrs' search alone, its NaN included: Excel is `#NUM!` for
/// `GAMMAINV(0.999999,10000000000.5,1E+10)` and for a CHISQ.INV.RT of 10000000000.5 degrees of
/// freedom (gen-m9), where statrs' search ends as NaN and a refinement would find the quantile
/// (T-1634).
/// Excel also has values above this shape (`CHISQ.INV(1E-16,10000000000.5)` = 9998837266.383266,
/// gen-m9; `CHISQ.INV(5.6221E-26,4715100000)` = 4714082845.627977, probes/incomplete-gamma-cap),
/// which rest on statrs' search alone, so moving the cut needs both kinds of cell checked.
const REFINE_SHAPE_MAX: f64 = 1e9;

/// The quantile of `dist` where the probability of `tail` is `p` (in [0, 1], else NaN): statrs'
/// search ([`inverse_cdf_within`]) where its result solves the smaller tail to
/// [`STATRS_QUANTILE_RESIDUAL`], else the crossing of that tail (`special::refine_increasing`,
/// at most 129 cdfs, from statrs' result when it has one). The smaller tail is `p` or its exact
/// complement `1 - p`, so a tiny right-tail `p` keeps the digits `1 - p` loses, as in F.INV
/// (T-0974), and the lower tail is the accurate pair's series, which has no shortcut at a tiny x, so
/// `CHISQ.INV(1E-100,1)` is Excel's 1.5707963267948966E-200 (T-1543, T-1634). All cdfs share one
/// budget of [`MAX_TERMS`] terms; NaN once it is spent.
pub(crate) fn gamma_quantile(dist: &Gamma, p: f64, tail: Tail) -> f64 {
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    let lower = match tail {
        Tail::Lower => p,
        Tail::Upper => 1.0 - p,
    };
    if lower == 0.0 && tail == Tail::Lower || p == 1.0 && tail == Tail::Upper {
        return 0.0;
    }
    if p == 1.0 && tail == Tail::Lower || p == 0.0 && tail == Tail::Upper {
        return f64::INFINITY;
    }
    // 1 - p is exact for p of 0.5 or more.
    let (small, target) = match (tail, p <= 0.5) {
        (_, true) => (tail, p),
        (Tail::Lower, false) => (Tail::Upper, 1.0 - p),
        (Tail::Upper, false) => (Tail::Lower, 1.0 - p),
    };
    // One budget for every cdf below. Each `borrow_mut` lasts one cdf call and none nests
    // (`probability` calls no cdf of its own), so no borrow can overlap.
    let terms = RefCell::new(Terms(MAX_TERMS));
    let probability = |x: f64| quantile_tail(dist, x, small, &mut terms.borrow_mut());
    let estimate = inverse_cdf_within(dist, lower, &mut terms.borrow_mut());
    if dist.shape() > REFINE_SHAPE_MAX
        || estimate.is_finite()
            && (probability(estimate) - target).abs() <= STATRS_QUANTILE_RESIDUAL * target
    {
        return estimate;
    }
    let x = match small {
        Tail::Lower => refine_increasing(probability, target, estimate, 0.0, f64::MAX),
        Tail::Upper => refine_increasing(|x| -probability(x), -target, estimate, 0.0, f64::MAX),
    };
    if terms.borrow().0 == 0 {
        return f64::NAN;
    }
    // A quantile below the smallest normal double is 0, as Excel's `CHISQ.INV(1E-200,1)` and
    // `(1E-300,1)` are (the crossings are about 1.6E-400 and 1.6E-600; T-1543 probe A25, A26).
    if x < f64::MIN_POSITIVE {
        return 0.0;
    }
    x
}

/// The probability of `tail` at `x * rate` for [`gamma_quantile`]: the accurate pair, with the
/// ends of `x * rate` closed. NaN past `terms`.
fn quantile_tail(dist: &Gamma, x: f64, tail: Tail, terms: &mut Terms) -> f64 {
    let scaled = x * dist.rate();
    if scaled.is_nan() {
        return f64::NAN;
    }
    if scaled <= 0.0 || scaled == f64::INFINITY {
        return if (scaled > 0.0) == (tail == Tail::Lower) {
            1.0
        } else {
            0.0
        };
    }
    let a = dist.shape();
    // In the series' range the lower tail is `x^a * e^(-x - ln Gamma(a + 1))` times the series
    // (T-1634): `powf` keeps a tiny quantile's digits (`GAMMA.INV(1E-300,1,1)` is within an ulp
    // of Excel's 1E-300), and against `x^a e^-x / Gamma(a)` of the pair it is Excel's double in
    // the far lower tail at shape 1/2 (`CHISQ.INV(1E-16,1)` = 1.5707963267948967E-32 and
    // `CHISQ.INV(0.001,1)` = 1.57079714926249E-6, gen-m9; the pair's quantiles are 2 ulps
    // off). Both factors must be normal doubles: at a large shape `x^a` overflows or the
    // exponential underflows, and the pair's prefactor is used.
    if tail == Tail::Lower && (scaled <= 1.0 || scaled <= a) {
        let power = scaled.powf(a);
        let rest = (-scaled - super::special::ln_gamma(a + 1.0)).exp();
        if power.is_finite() && power >= f64::MIN_POSITIVE && rest >= f64::MIN_POSITIVE {
            return power * rest * lower_series(a, scaled, terms);
        }
    }
    let (lower, upper) = incomplete_gamma_pair(a, scaled, terms);
    match tail {
        Tail::Lower => lower,
        Tail::Upper => upper,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::{
        cdf_terms_bound, continued_fraction_budget, gamma_cdf, gamma_density, gamma_inverse_cdf,
        gamma_quantile, gamma_sf, gamma_ur, incomplete_gamma_pair, inverse_cdf_within, Tail, Terms,
        MAX_TERMS,
    };
    use statrs::distribution::{ChiSquared, ContinuousCDF, DiscreteCDF, Gamma, Poisson};
    use statrs::function::gamma::{gamma_lr as statrs_lr, gamma_ur as statrs_ur};

    /// A small xorshift, so the sweep is the same on every run.
    struct Sweep(u64);
    impl Sweep {
        fn unit(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn pair(a: f64, x: f64) -> (f64, f64) {
        incomplete_gamma_pair(a, x, &mut Terms(MAX_TERMS))
    }

    /// Within `ulps` units of 2^-52 relative to `expected`.
    fn within_ulps(actual: f64, expected: f64, ulps: f64) -> bool {
        (actual - expected).abs() <= ulps * f64::EPSILON * expected.abs()
    }

    /// Close to statrs' value, or both NaN. statrs forms the prefactor as exp(a ln x - x - ln
    /// Gamma(a)), whose relative error is the absolute error of that exponent, about an ulp of
    /// `scale` = a |ln x| + x + |ln Gamma(a)|, and takes Q as 1 - P below x = a; the accurate
    /// kernel avoids both,
    /// so the two may differ by that much.
    /// [`near_at`] for Q(a, x): statrs takes Q as 1 - P below x = a (and below 1), so there its
    /// Q is only as good as P is in absolute terms.
    fn near_upper(ours: f64, theirs: f64, a: f64, x: f64, scale: f64) -> bool {
        let scale = if x < 1.0 || x <= a {
            (scale + 10.0) / theirs.max(1e-300)
        } else {
            scale
        };
        near_at(ours, theirs, scale)
    }

    fn near_at(ours: f64, theirs: f64, scale: f64) -> bool {
        let tolerance = 1e-14 * (scale + 10.0);
        ours.to_bits() == theirs.to_bits()
            || (ours.is_nan() && theirs.is_nan())
            // statrs flushes an exponent below ln of the smallest normal to 0.
            || (theirs == 0.0 && ours.abs() < f64::MIN_POSITIVE)
            || (ours - theirs).abs() <= tolerance * theirs.abs().max(1e-300)
    }

    // P(a, x) and Q(a, x) against mpmath's regularized `gammainc` at 50 digits, both tails
    // directly (a in {1/2, 9, 500} and small shapes, where Q is not 1 - P).
    #[test]
    fn the_pair_is_within_a_few_ulps_of_the_reference() {
        let reference = [
            (0.5, 0.1, 0.345279153981423, 0.654720846018577),
            (0.5, 1.2, 0.8786647496415179, 0.12133525035848215),
            (0.5, 3.0, 0.9856941215645704, 0.01430587843542964),
            (9.0, 2.0, 0.00023744732826116177, 0.9997625526717389),
            (9.0, 9.5, 0.6081765174550605, 0.39182348254493954),
            (9.0, 30.0, 0.9999979539240957, 2.0460759042705716e-06),
            (500.0, 450.0, 0.010717238091289742, 0.9892827619087102),
            (500.0, 500.0, 0.5059471461707603, 0.49405285382923964),
            (500.0, 560.0, 0.9953068340007052, 0.004693165999294769),
        ];
        // Two arrays, not one: thirteen rows alike to the duplication ratchet's normalizer
        // read as copied code.
        let small_shapes = [
            (0.00113, 0.0008, 0.9926197543481239, 0.0073802456518761664),
            (0.01, 0.5, 0.9943732438060329, 0.0056267561939671844),
            (0.3, 0.9, 0.9022526480296695, 0.09774735197033045),
            (0.001, 1e-05, 0.9891230446957827, 0.01087695530421733),
        ];
        for (a, x, p, q) in reference.into_iter().chain(small_shapes) {
            let (lower, upper) = pair(a, x);
            // At a = 500 the prefactor's exponent a rlog1(x/a - 1) (2.68 at x = 450) carries an ulp
            // of itself into P: 6.5 ulps of 1 there.
            assert!(within_ulps(lower, p, 8.0), "P({a},{x}) = {lower}, not {p}");
            assert!(within_ulps(upper, q, 8.0), "Q({a},{x}) = {upper}, not {q}");
            assert!(
                (lower + upper - 1.0).abs() <= 2.0 * f64::EPSILON,
                "P + Q at ({a},{x})"
            );
        }
    }

    // a = 1 is the exponential distribution: P = -expm1(-x) and Q = exp(-x), exactly.
    #[test]
    fn the_density_is_within_a_few_ulps_of_the_reference() {
        // (a, x, scale, density) from mpmath at 50 digits. The first is Excel's
        // `GAMMA.DIST(10,9,2,FALSE)` (ironcalc-calc GAMMA.xlsx E3), the last `CHISQ.DIST(37.5,6,FALSE)`.
        for (a, x, scale, expected) in [
            (9.0, 10.0, 2.0, 0.032_639_019_674_079_374),
            (2.5, 0.3, 1.0, 0.091_570_869_452_182_3),
            (500.0, 480.0, 1.0, 0.012_319_344_669_927_745),
            (0.5, 1e-3, 2.0, 12.609_356_355_490_782),
            (3.0, 37.5, 2.0, 6.322_968_483_684_419e-7),
        ] {
            let ours = gamma_density(a, x, scale).unwrap();
            assert!(within_ulps(ours, expected, 4.0), "{a} {x} {scale}: {ours}");
        }
        assert_eq!(gamma_density(1.0, 1.5, 2.0), Some((-0.75f64).exp() / 2.0));
        assert_eq!(gamma_density(2.0, 0.0, 1.0), None);
    }

    #[test]
    fn a_tiny_shape_s_lower_tail_is_one_minus_its_upper() {
        // `GAMMADIST(0.5,1E-300,1,TRUE)` is Excel's 1 (gen-m9/11eb3ab4f4f8104c A252), where the
        // series gives 1 - 2^-53.
        let (lower, upper) = pair(1e-300, 0.5);
        assert_eq!(lower, 1.0);
        assert!(upper > 0.0 && upper < 1e-299);
    }

    #[test]
    fn shape_one_is_expm1() {
        for x in [1e-300, 1e-10, 0.5, 1.0, 2.0, 30.0, 700.0] {
            assert_eq!(pair(1.0, x), (-(-x).exp_m1(), (-x).exp()), "x = {x}");
        }
        // GAMMA.DIST(1,1,1,TRUE) is Excel's 0.6321205588285577 (BETA_GAMMA.xlsx L2).
        let dist = Gamma::new(1.0, 1.0).unwrap();
        assert_eq!(gamma_cdf(&dist, 1.0), 0.632_120_558_828_557_7);
    }

    // A huge scale (beta = 1E6, rate 1E-6) is the same pair at x / beta.
    #[test]
    fn a_huge_scale_is_the_pair_at_x_over_beta() {
        for (shape, x) in [(0.5, 3e5), (9.0, 9.5e6), (500.0, 5.6e8)] {
            let dist = Gamma::new(shape, 1e-6).unwrap();
            let (lower, upper) = pair(shape, x * 1e-6);
            assert_eq!(gamma_cdf(&dist, x).to_bits(), lower.to_bits());
            assert_eq!(gamma_sf(&dist, x).to_bits(), upper.to_bits());
        }
    }

    // Across shapes 1E-3 to 1E8 the pair stays within statrs' own error of statrs' value, and
    // so do the cdfs, survival functions and quantiles GAMMA.INV and CHISQ.INV hand it.
    #[test]
    fn the_pair_is_near_statrs() {
        let mut sweep = Sweep(0x1872);
        for _ in 0..20_000 {
            let a = 10f64.powf(sweep.unit() * 11.0 - 3.0);
            let x = a * 10f64.powf(sweep.unit() * 6.0 - 3.0);
            let near = a * (1.0 + (sweep.unit() - 0.5) * 8.0 / a.sqrt());
            // statrs takes P as 0 below x = 1.1E-15 (its `almost_eq(x, 0)`).
            for x in [x, near].into_iter().filter(|&x| x > 1.1e-15) {
                let (lower, upper) = pair(a, x);
                // statrs' exponent also subtracts ln Gamma(a), about a ln a.
                let scale = a * x.ln().abs() + x + a * a.ln().abs();
                assert!(
                    near_at(lower, statrs_lr(a, x), scale),
                    "P({a},{x}) {lower} {upper} {} {}",
                    statrs_lr(a, x),
                    statrs_ur(a, x)
                );
                assert!(
                    near_upper(upper, statrs_ur(a, x), a, x, scale),
                    "Q({a},{x})"
                );
            }
        }
        for _ in 0..3_000 {
            let shape = 10f64.powf(sweep.unit() * 9.0 - 3.0);
            let rate = 10f64.powf(sweep.unit() * 12.0 - 6.0);
            let p = if sweep.unit() < 0.3 {
                10f64.powf(-sweep.unit() * 300.0)
            } else {
                sweep.unit()
            };
            let dist = Gamma::new(shape, rate).unwrap();
            let x = shape / rate * (0.5 + sweep.unit());
            let scale = shape * (x * rate).ln().abs() + x * rate;
            assert!(
                near_at(gamma_cdf(&dist, x), dist.cdf(x), scale),
                "cdf Gamma({shape},{rate}) at {x}"
            );
            assert!(
                near_upper(gamma_sf(&dist, x), dist.sf(x), shape, x * rate, scale),
                "sf Gamma({shape},{rate}) at {x}"
            );
            // statrs' search (halve and double, 8 bisections, 4 Newton steps) can stop far from
            // the quantile in the far tail (at p = 1.26E-291 and shape 14.3 both searches end
            // where the cdf is about 1E-88), so the two are compared only where statrs' quantile
            // reproduces p.
            let theirs = dist.inverse_cdf(p);
            if (dist.cdf(theirs) / p - 1.0).abs() < 1e-6 {
                // A cdf's relative error e moves a quantile by e / shape where the shape is small.
                assert!(
                    near_at(
                        gamma_inverse_cdf(&dist, p),
                        theirs,
                        (shape + 10.0) / shape.min(1.0)
                    ),
                    "Gamma({shape},{rate}) at {p:e}"
                );
            }
        }
        for df in [1.0, 2.0, 5.0, 10.0, 99.0, 1e4, 1e6, 1e10] {
            let chi = ChiSquared::new(df).unwrap();
            let dist = Gamma::new(df / 2.0, 0.5).unwrap();
            for p in [1e-10, 0.05, 0.4, 0.5, 0.95, 0.999_999_999_9] {
                assert!(
                    near_at(gamma_inverse_cdf(&dist, p), chi.inverse_cdf(p), df),
                    "ChiSquared({df}) at {p}"
                );
                let x = df * (0.5 + p);
                let scale = df * x.ln().abs() + x;
                assert!(
                    near_at(gamma_cdf(&dist, x), chi.cdf(x), scale),
                    "ChiSquared({df}) cdf at {x}"
                );
                assert!(
                    near_upper(gamma_sf(&dist, x), chi.sf(x), df / 2.0, x / 2.0, scale),
                    "ChiSquared({df}) sf at {x}"
                );
            }
        }
        for (k, lambda) in [(5u64, 1.0), (0, 0.5), (100, 98.5), (1_000_000, 999_000.0)] {
            let poisson = Poisson::new(lambda).unwrap();
            let scale = k as f64 * lambda.ln().abs() + lambda;
            let a = k as f64 + 1.0;
            assert!(near_upper(
                gamma_ur(a, lambda),
                poisson.cdf(k),
                a,
                lambda,
                scale
            ));
        }
    }

    // The arguments where statrs panicked or never finished end as NaN (`#NUM!` in the
    // functions), or as Excel's value where x / beta underflows.
    #[test]
    fn statrs_panics_and_hangs_end_as_nan() {
        // `a + 1 == a`: the series never ends (GAMMA.DIST(1E16,1E16,1,TRUE)).
        assert!(gamma_cdf(&Gamma::new(1e16, 1.0).unwrap(), 1e16).is_nan());
        assert!(gamma_ur(1e16 + 1.0, 1e16).is_nan());
        // x * rate overflows (statrs: an unwrapped domain error).
        assert!(gamma_cdf(&Gamma::new(1.0, 1e10).unwrap(), 1e300).is_nan());
        // Where it underflows to 0 (statrs panicked too) the value is statrs' at its smallest x.
        assert_eq!(gamma_cdf(&Gamma::new(2.0, 1e-100).unwrap(), 1e-300), 0.0);
        assert_eq!(gamma_sf(&Gamma::new(2.0, 1e-100).unwrap(), 1e-300), 1.0);
        assert_eq!(gamma_cdf(&Gamma::new(1e-16, 1e-100).unwrap(), 1e-300), 1.0);
        // A shape of 1E300 and a mean of 1: the inversion's cdfs never finish.
        assert!(gamma_inverse_cdf(&Gamma::new(1e300, 1e300).unwrap(), 0.5).is_nan());
        // The ends and invalid arguments are statrs' (its panics outside [0, 1] are NaN).
        let dist = Gamma::new(2.0, 1.0).unwrap();
        assert_eq!(gamma_inverse_cdf(&dist, 0.0), 0.0);
        assert_eq!(gamma_inverse_cdf(&dist, 1.0), f64::INFINITY);
        assert!(gamma_inverse_cdf(&dist, 1.5).is_nan());
        assert_eq!(gamma_cdf(&dist, 0.0), 0.0);
        assert_eq!(gamma_sf(&dist, 0.0), 1.0);
        assert!(gamma_ur(0.0, 1.0).is_nan());
        assert!(gamma_ur(1.0, f64::NAN).is_nan());
        assert!(gamma_ur(2.0, f64::INFINITY).is_nan());
    }

    /// The terms `incomplete_gamma_pair(a, x)` takes, checked to be the exact boundary: the
    /// value with that many, NaN with one fewer.
    fn terms_needed(a: f64, x: f64) -> u32 {
        let mut terms = Terms(MAX_TERMS);
        let full = incomplete_gamma_pair(a, x, &mut terms);
        let used = MAX_TERMS - terms.0;
        assert_eq!(incomplete_gamma_pair(a, x, &mut Terms(used)), full);
        assert!(incomplete_gamma_pair(a, x, &mut Terms(used - 1)).0.is_nan());
        used
    }

    // The cap branches, both sides: a cdf or an inversion that needs more terms than its budget
    // is NaN, and the same call with exactly the terms it needs is its value.
    #[test]
    fn a_call_past_its_cap_is_nan() {
        // The series at a = x = 5E9 (CHISQ.DIST(1E10,1E10,TRUE)) stays under the cap.
        assert!(terms_needed(5e9, 5e9) < MAX_TERMS);
        // At a = x = 1E10 it is past it.
        assert!(pair(1e10, 1e10).0.is_nan());
        assert!(!incomplete_gamma_pair(1e10, 1e10, &mut Terms(u32::MAX))
            .0
            .is_nan());
        // The continued fraction (x above a) and the small-shape Q's series.
        terms_needed(1e8, 1.0001e8);
        terms_needed(0.00113, 8e-4);
        // An inversion shares one budget over its cdfs.
        let dist = Gamma::new(3e8, 1e6).unwrap();
        assert!(near_at(
            inverse_cdf_within(&dist, 1e-5, &mut Terms(MAX_TERMS)),
            dist.inverse_cdf(1e-5),
            dist.shape()
        ));
        assert!(inverse_cdf_within(&dist, 1e-5, &mut Terms(1_000)).is_nan());
    }

    // The continued fraction's own budget (T-2795) is at least twice the terms it needs at
    // every shape the sweep reached, so it changes no value; past it the cdf is NaN.
    #[test]
    fn the_continued_fraction_stays_within_half_its_budget() {
        let mut a: f64 = 1e-3;
        while a < 1e10 {
            let step = a.sqrt().max(1.0) * 0.05;
            let near = (0..60).map(|k| a + 1.0 + f64::from(k) * step);
            let far = [10.0, 1e3, 1e5].map(|k| a + 1.0 + k * a.sqrt().max(1.0));
            for x in near.chain(far) {
                let mut terms = Terms(MAX_TERMS);
                let (p, _) = incomplete_gamma_pair(a, x, &mut terms);
                assert!(!p.is_nan(), "a = {a:e}, x = {x:e}");
                let used = MAX_TERMS - terms.0;
                assert!(
                    2 * used <= continued_fraction_budget(a),
                    "a = {a:e}, x = {x:e}: {used} terms"
                );
                assert!(used <= cdf_terms_bound(a));
            }
            a *= 3.1;
        }
        // A slow fraction (`terms_needed` checks its NaN one term short of the shared budget)
        // stays within its own; the budget saturates at the cap for a huge shape.
        let used = terms_needed(1e8, 1.0001e8);
        assert!(used < continued_fraction_budget(1e8));
        assert_eq!(continued_fraction_budget(1e30), MAX_TERMS);
        assert_eq!(cdf_terms_bound(1e9), MAX_TERMS);
        assert_eq!(cdf_terms_bound(2.5), 256 + 44);
    }

    // Excel's quantiles in the tails and at a small shape, where statrs' search is off or NaN
    // (T-1543 probes-numeric/t1543-inverse-cdf-tiny-p; T-2255 gen-m9 triage). Each is within
    // half a unit of Excel's 15th significant digit.
    #[test]
    fn quantiles_in_the_tails_are_excel_s() {
        let chi = |df: f64| Gamma::new(df / 2.0, 0.5).unwrap();
        let cases = [
            (chi(1.0), 1e-100, Tail::Lower, 1.570_796_326_794_896_6e-200),
            (chi(10.0), 1.0 - 1e-10, Tail::Lower, 68.167_617_951_904_13),
            (
                chi(4.0),
                1.0 - 1e-16,
                Tail::Upper,
                2.980_232_253_572_505_4e-8,
            ),
            (chi(1.0), 0.025, Tail::Lower, 0.000_982_069_117_175_256_4),
            (chi(9.0), 1.0 - 1e-16, Tail::Lower, 95.798_170_395_038_67),
            (chi(8.0), 1e-10, Tail::Upper, 63.397_964_411_135_6),
            (Gamma::new(1.0, 1.0).unwrap(), 1e-300, Tail::Lower, 1e-300),
            (chi(1.0), 1e-200, Tail::Lower, 0.0),
        ];
        for (dist, p, tail, excel) in cases {
            let ours = gamma_quantile(&dist, p, tail);
            assert!(
                (ours - excel).abs() <= 5e-15 * excel,
                "{p} at shape {}: {ours:e}, Excel {excel:e}",
                dist.shape()
            );
        }
        // At a large shape `x^a` overflows, and the prefactor is TOMS 654's `rcomp`: the quantile
        // is within 1E-13 of `GAMMAINV(1E-10,99999,2)`, Excel's 196001.02844708483 (gen-m9
        // 165db58d1540f014; before T-1634's overflow guard it was 2.01).
        let ours = gamma_quantile(&Gamma::new(99_999.0, 0.5).unwrap(), 1e-10, Tail::Lower);
        assert!(
            (ours - 196_001.028_447_084_83).abs() < 1e-13 * ours,
            "{ours}"
        );
    }
}
