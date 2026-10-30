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

use super::gammaln::excel_gammaln;
use super::incomplete_beta::{rlog1, stirling_correction};
use super::special::refine_increasing;
use crate::functions::util::poly;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;

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
        return LOWER_OF_UNDERFLOW;
    }
    if !scaled.is_finite() {
        return f64::NAN;
    }
    incomplete_gamma_pair(dist.shape(), scaled, terms).0
}

/// P(a, x) where `x / beta` underflows to 0 for an x above 0 (statrs panics there, a domain
/// error): 0 at every shape, as Excel's `GAMMA.DIST(1E-300,a,1E100,TRUE)` is for a = 2, 1, 0.5,
/// 0.01, 1E-17 and 1E-300 (probes/incomplete-gamma-cap E7, probes/t2982-gamma-inv-huge-shape
/// A117-A122), although the true P(1E-17, 1E-400) is 1 - 9E-15 (T-2982).
const LOWER_OF_UNDERFLOW: f64 = 0.0;

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

/// The largest a·LN(x) at which [`excel_small_shape_lower`] takes Excel's form: from |a ln x| = 16
/// on the double exponent's ulp is 2^-48 or more, so its rounding moves P by up to 1.8E-15, the 15th
/// digit (probes/t3012-gamma-dist-at-quantile A3-A5, A16-A18, A21, A24: up to 102 ulps between
/// the forms). Nearer 0 Excel still takes the form (its value is the exact cdf with the double
/// exponent to within 0.1 ulp of that exponent at A20, A25, A30 and A31), but the forms are a few
/// ulps apart and the engine's EXP, LN and GAMMALN leave a 1-ulp residue either way: A31
/// `GAMMA.DIST(1E-5,0.25,1,TRUE)` and C25 are bit-exact on the pair and 1 ulp off with the form,
/// so the pair is kept there (T-3060). No probe cell pins -16 itself: the in-region cells
/// nearest the cut are at about -23.05 (rows 1, 19, 24) and the excluded ones at -11.54 to
/// -11.61 (rows 2, 6, 10, 20, 25), so any cut in between scores the same; -16 is the ulp
/// argument above (T-3242 probes the gap).
const EXCEL_EXPONENT_MAX: f64 = -16.0;

/// GAMMA.DIST's cdf P(a, x) at a shape below 1 and a normal x below 1 where a·LN(x) is at most
/// [`EXCEL_EXPONENT_MAX`] (so the pair takes P from the series, not as 1 - Q): Excel's
/// leading factor `EXP(a*LN(x))/EXP(GAMMALN(a+1))`, with a·LN(x) rounded to a double, times e^-x
/// and the series (T-3060). That rounding is Excel's error, up to 1.2E-14 of P where |a ln x| is
/// near 230: `GAMMA.DIST(4.147567013992206E-134,0.75,1,TRUE)` (its literal read as
/// 4.14756701399220E-134) is Excel's 1.000000000000026E-100 against the true
/// 1.0000000000000131E-100 (probes/t3012-gamma-dist-at-quantile A16; docs/excel-semantics/
/// statistics.md, T-3060 bullet). `None` elsewhere, and where the series runs out of terms.
pub(crate) fn excel_small_shape_lower(a: f64, x: f64) -> Option<f64> {
    if !(a > 0.0 && a < 1.0 && (f64::MIN_POSITIVE..1.0).contains(&x)) {
        return None;
    }
    let exponent = a * excel_ln(x);
    if exponent > EXCEL_EXPONENT_MAX {
        return None;
    }
    // P is then about e^-16 / Gamma(a + 1), under 2E-7, so the pair takes it from the series (it
    // takes 1 - Q only where Q is at most 1/2).
    excel_form_lower(a, x, exponent)
}

/// The leading-factor series form of P(a, x) for x below 1 at any shape below 1, ungated (T-3060):
/// `EXP(exponent)/EXP(GAMMALN(a+1))` times e^-x and the series, where `exponent` is the double
/// a·LN(x). GAMMA.DIST takes it only where a·LN(x) is at most [`EXCEL_EXPONENT_MAX`]; above that
/// the engine's GAMMA.DIST is the accurate pair, a few ulps away. `None` where the series runs
/// out of terms.
fn excel_form_lower(a: f64, x: f64, exponent: f64) -> Option<f64> {
    let sum = lower_series(a, x, &mut Terms(MAX_TERMS));
    if sum.is_nan() {
        return None;
    }
    let lead = excel_exp(exponent) / excel_exp(excel_gammaln(a + 1.0));
    Some(lead * excel_exp(-x) * sum)
}

/// Below this quantile (at scale 1) GAMMA.INV at a shape below 1 takes the closed form
/// [`excel_small_shape_quantile`] and not the search (T-3013). This is a choice inside the
/// evidence bracket (4.15E-22, 6.75E-21), not a measured Excel constant. A28
/// `GAMMA.INV(1E-16,0.75,1)` = 4.147567013992107E-22 is the closed form, while A18 and B18
/// `GAMMA.INV(1E-5,0.25,..)` (6.75E-21 at scale 1) are neither it nor one Newton step from it on
/// the engine's model of Excel's cdf; so are A34 `(1E-16,0.9,1)` (1.6E-18) and
/// gen-m9/60b0b6e69ad35a45 A210 (shape 1/60, 4.9E-19). The cut is on x, not on p or a·LN(x): A28
/// and A34 share p = 1E-16 and nearly the same a·LN(x) (about -36.9), and only A28 is the closed
/// form. Below it the series' correction (about x / (a + 1)) is far below an ulp, so the closed
/// form is the quantile of the leading factor alone. T-3252 probes the gap.
const CLOSED_FORM_QUANTILE_MAX: f64 = 1e-21;

/// GAMMA.INV takes one Newton step from the closed form where that step is more than this
/// relative part of x, and keeps the closed form otherwise (T-3013). The step is (p - F)/(a F) of
/// x to first order, so a test on the cdf's distance from p alone does not fit. This value is a
/// point inside the evidence bracket (2.78E-14, 4.1E-14), not an Excel constant: 2^-45
/// (2.84E-14) fits too. The step is kept at 2.78E-14 (gen-m9/146b46e74f2dac3b A396
/// `GAMMA.INV(0.05,0.01,EOMONTH(58733,-18))`, where F at the closed form is 2.2E-16 below p) and at
/// 1.33E-14 (A6 `GAMMA.INV(1E-5,0.05,1)`). It is taken at 4.1E-14 (A26 `GAMMA.INV(1E-100,0.75,1)`:
/// F is 3.09E-14 below p at the closed form, and 2.60E-14 above p at the result, Excel's
/// 4.1475670139922064E-134) and at 4.4E-14 (gen-m9/3a25a03eb94fcfbd A254 `GAMMAINV(0.5,0.01,100)`:
/// F is 4.4E-16 below p at the closed form; the result is Excel's 4.4655350189103716E-29). F is
/// [`excel_form_lower`], the leading-factor series form, even where a·LN(x) is above
/// [`EXCEL_EXPONENT_MAX`] and the engine's GAMMA.DIST is the pair. A254 and A396 need that: on
/// GAMMA.DIST's value their steps would be 2.2E-14 (kept) and 4.2E-14 (taken), both against Excel.
/// So either Excel's GAMMA.INV steps on an internal form, or the engine's GAMMA.DIST is off there
/// (T-3252 probes it). One step only: at A26 a second would be 3.46E-14 of x and Excel takes none.
/// A loop under the same test, with the threshold in [3.46E-14, 4.1E-14), fits as well. T-3252
/// probes steps from 2.9E-14 to 4.0E-14.
const CLOSED_FORM_STEP_MIN: f64 = 3.5e-14;

/// Excel's GAMMA.INV(p, a, 1) at a shape below 1 other than 1/2, where its quantile is tiny
/// (T-3013; docs/excel-semantics/statistics.md, T-3013 bullet): the closed form
/// `EXP((LN(p)+GAMMALN(a)+LN(a))/a)`, the root of the leading factor p = x^a / Gamma(a + 1),
/// summed in that order (probes/t3012-gamma-inv-small-shape: bit-exact at 14 cells of column A,
/// A9 `GAMMA.INV(1E-30,0.1,1)` = 6.073048362407713E-301 and A16 `(1E-16,0.25,1)` =
/// 6.749697893111757E-65 among them, where `GAMMALN(a+1)` in place of the last two terms is one
/// ulp of the exponent off). Where the Newton step on the leading-factor series form
/// ([`excel_form_lower`]) is more than [`CLOSED_FORM_STEP_MIN`] of x, it takes that one step
/// (A26). The value may be 0 or subnormal (Excel's `#DIV/0!`, the caller's to decide). `None` outside [`CLOSED_FORM_QUANTILE_MAX`], at a
/// p outside (0, 1), and where a cdf or density fails: the search applies there.
pub(crate) fn excel_small_shape_quantile(a: f64, p: f64) -> Option<f64> {
    if !(a > 0.0 && a < 1.0 && a != 0.5 && p > 0.0 && p < 1.0) {
        return None;
    }
    let exponent = (excel_ln(p) + excel_gammaln(a) + excel_ln(a)) / a;
    let mut x = excel_exp(exponent);
    if x.is_nan() || x >= CLOSED_FORM_QUANTILE_MAX {
        return None;
    }
    if x < f64::MIN_POSITIVE {
        return Some(x);
    }
    let step = (excel_form_lower(a, x, a * excel_ln(x))? - p) / gamma_density(a, x, 1.0)?;
    // A subnormal step is no step. The only evidence is one cell (A9, and B9 at scale 1E+15):
    // `GAMMA.INV(1E-30,0.1,1)` = 6.073048362407713E-301 is the closed form though its step, 8.5E-14
    // of x, is about 5E-314, as if Excel flushed it to zero. A NaN step fails both tests, so it
    // is no step either.
    if step.abs() >= f64::MIN_POSITIVE && step.abs() > CLOSED_FORM_STEP_MIN * x {
        x -= step;
    }
    (x > 0.0 && x.is_finite()).then_some(x)
}

/// The shape from which the pair is Temme's uniform asymptotic expansion ([`temme_pair`]) and not a
/// loop: the series needs about 6.8 * sqrt(a) terms near x = a, past [`MAX_TERMS`] from a shape of
/// about 5.9E9, and never ends once `a + 1 == a`, while the expansion takes a fixed number of
/// operations at any shape (T-1891).
const TEMME_SHAPE_MIN: f64 = 1e8;

/// Above this a·phi (`temme_pair`'s exponent) the smaller tail is below e^-800, so 0 in doubles,
/// and the expansion's polynomials are not evaluated (their variable may overflow there).
const TEMME_EXPONENT_MAX: f64 = 800.0;

/// Temme's coefficients `C_k(z)` of the expansion, as polynomials in z, lowest power first (Temme
/// 1979; Didonato and Morris 1986; the 53-bit table of Boost's `igamma_temme_large`). From a shape
/// of [`TEMME_SHAPE_MIN`] each `C_k/a^k` falls by about 1/a, so the `C_3` term is about 1E-28 of the
/// `C_0` term and later ones are dropped; checked against Excel on probes/incomplete-gamma-cap
/// E1-E4, E8-E12.
const TEMME_C: [&[f64]; 4] = [
    &[
        -0.333_333_333_333_333_3,
        0.083_333_333_333_333_33,
        -0.014_814_814_814_814_815,
        0.001_157_407_407_407_407_3,
        0.000_352_733_686_067_019_4,
        -0.000_178_755_144_032_921_8,
        3.919_263_178_522_438e-05,
        -2.185_448_510_679_992_4e-06,
        -1.854_062_210_715_16e-06,
        8.296_711_340_953_087e-07,
        -1.766_595_273_682_607_8e-07,
        6.707_853_543_401_498e-09,
        1.026_180_978_424_030_9e-08,
        -4.382_036_018_453_353e-09,
        9.147_699_582_236_79e-10,
    ],
    &[
        -0.001_851_851_851_851_852,
        -0.003_472_222_222_222_222,
        0.002_645_502_645_502_645_4,
        -0.000_990_226_337_448_559_6,
        0.000_205_761_316_872_427_98,
        -4.018_775_720_164_609e-07,
        -1.809_855_033_448_997_7e-05,
        7.649_160_916_081_11e-06,
        -1.612_090_089_456_344_6e-06,
        4.647_127_802_807_434e-09,
        1.378_633_446_915_721e-07,
        -5.752_545_603_517_705e-08,
        1.195_162_859_977_814_8e-08,
    ],
    &[
        0.004_133_597_883_597_883,
        -0.002_681_327_160_493_827_3,
        0.000_771_604_938_271_604_9,
        2.009_387_860_082_304_7e-06,
        -0.000_107_366_532_263_651_61,
        5.292_344_882_912_012_5e-05,
        -1.276_063_518_861_872_8e-05,
        3.423_578_734_096_138e-08,
        1.372_195_730_906_293_2e-06,
        -6.298_992_138_380_055e-07,
        1.428_061_420_606_424_2e-07,
    ],
    &[
        0.000_649_434_156_378_600_8,
        0.000_229_472_093_621_399_17,
        -0.000_469_189_494_395_255_7,
        0.000_267_720_632_062_838_85,
        -7.561_801_671_883_977e-05,
        -2.396_505_113_867_297e-07,
        1.108_265_411_534_730_2e-05,
        -5.674_952_826_991_596_5e-06,
        1.423_090_073_243_588_3e-06,
    ],
];

/// (P(a, x), Q(a, x)) by Temme's uniform asymptotic expansion, for a shape of at least
/// [`TEMME_SHAPE_MIN`] (T-1891). With phi = (x - a)/a - ln(x/a) and z = ±sqrt(2 phi) (the sign of
/// x - a), the smaller tail is erfc(sqrt(a phi))/2 ± e^(-a phi)/sqrt(2 pi a) · sum `C_k(z)/a^k`
/// (- for P below a, + for Q from a on), and the other tail 1 minus it. Excel is close to
/// correctly rounded here: `GAMMA.DIST(1E10,1E10,1,TRUE)` = 0.5000013298076014 and
/// `GAMMA.DIST(1E16,1E16,1,TRUE)` = 0.5000000013298076 (probes/incomplete-gamma-cap E8, E1).
fn temme_pair(a: f64, x: f64) -> (f64, f64) {
    // Within a factor of 2 of a, x - a is exact (Sterbenz), as in `gamma_prefactor`.
    let phi = if x >= a * 0.5 && x <= a * 2.0 {
        rlog1((x - a) / a)
    } else {
        let ratio = x / a;
        (ratio - 1.0) - ratio.ln()
    };
    let exponent = a * phi;
    let below = x < a;
    let small = if exponent > TEMME_EXPONENT_MAX {
        0.0
    } else {
        let z = if below {
            -(2.0 * phi).sqrt()
        } else {
            (2.0 * phi).sqrt()
        };
        let inverse = 1.0 / a;
        let series = TEMME_C
            .iter()
            .rev()
            .fold(0.0, |acc, c| acc * inverse + poly(c, z));
        let correction = series * (-exponent).exp() / (2.0 * std::f64::consts::PI * a).sqrt();
        // libm's erfc, within an ulp, not Excel's ERFC.PRECISE (`erf::erfc`), whose
        // EXP(-fl(x*x)) loses about 2x^2 ulps in the tail: this family is close to correctly
        // rounded in Excel (decision 0114, accuracy stage).
        let base = libm::erfc(exponent.sqrt()) / 2.0;
        if below {
            base - correction
        } else {
            base + correction
        }
    };
    if below {
        (small, 1.0 - small)
    } else {
        (1.0 - small, small)
    }
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
    if a >= TEMME_SHAPE_MIN {
        // A fixed amount of work, charged as one term.
        if !terms.take() {
            return (f64::NAN, f64::NAN);
        }
        return temme_pair(a, x);
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
        return 1.0 - LOWER_OF_UNDERFLOW;
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

/// Above this shape the quantile is statrs' search alone, its NaN included: Excel is `#NUM!` for
/// `GAMMAINV(0.999999,10000000000.5,1E+10)` and for a CHISQ.INV.RT of 10000000000.5 degrees of
/// freedom (gen-m9), where statrs' search ends as NaN and a refinement would find the quantile
/// (T-1634).
/// Excel also has values above this shape (`CHISQ.INV(1E-16,10000000000.5)` = 9998837266.383266,
/// gen-m9; `CHISQ.INV(5.6221E-26,4715100000)` = 4714082845.627977, probes/incomplete-gamma-cap),
/// which rest on statrs' search alone, so moving the cut needs both kinds of cell checked.
const REFINE_SHAPE_MAX: f64 = 1e9;

/// From this shape on, Excel's GAMMA.INV and GAMMAINV are not a search but
/// [`wilson_hilferty_quantile`]: `GAMMA.INV(0.5,a,1)` / a tends to c = 1.0002658235505755, and is
/// c (a - 1/3) / a from 1E13 to 1E300 (probes/t2982-gamma-inv-huge-shape A7-A23; the one cell at 2E10, A7, is at
/// p = 0.5), while at 5E9 the quantile is the true one and
/// at 7E9 and 1E10 Excel is mostly `#NUM!` (A5, A6, A42-A59; that band and the shapes from 1E9 to
/// it stay on the search, TODO(T-3070)).
pub(crate) const WILSON_HILFERTY_SHAPE_MIN: f64 = 2e10;

/// The offset Excel adds to the Wilson-Hilferty cube root: (1.0002658235505755)^(1/3) - 1 is
/// 0.0000886 to 13 digits (A7-A23).
const WILSON_HILFERTY_OFFSET: f64 = 0.000_088_6;

/// The factor on Excel's z term, fitted to the p sweep at shape 1E13 (A25-A41, 1 - 2.2106E-5;
/// the cells are within 7 ulps of it, 3 at most from p = 1E-16 to 1 - 1E-6).
const WILSON_HILFERTY_SPREAD: f64 = 0.999_977_893_94;

/// Excel's gamma quantile at a shape from [`WILSON_HILFERTY_SHAPE_MIN`], for a scale of 1, where
/// `z` is the standard normal quantile of the lower tail's probability: the Wilson-Hilferty
/// approximation a (1 - 1/(9a) + z/(3 sqrt a))^3 with Excel's offset and spread inside the cube
/// (probes/t2982-gamma-inv-huge-shape, docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV
/// at a huge shape"). It is within 7 ulps of Excel from a shape of 1E13 and about 100 to 1,500 ulps
/// off at 1E11 (a term in z^2 that the fit does not have; T-3070).
pub(crate) fn wilson_hilferty_quantile(shape: f64, z: f64) -> f64 {
    let cube_root = 1.0 + WILSON_HILFERTY_OFFSET - 1.0 / (9.0 * shape)
        + z * WILSON_HILFERTY_SPREAD / (3.0 * shape.sqrt());
    shape * cube_root * cube_root * cube_root
}

/// The quantile of the gamma distribution of `shape` and `scale` where the probability of `tail`
/// is `p` (in [0, 1], else NaN). NaN where the rate 1 / scale is not a positive finite double, at
/// every shape (a subnormal scale stays `#NUM!` in GAMMA.INV, as statrs' `Gamma::new` made it).
///
/// At shape 1 it is [`exponential_quantile`] times the scale, Excel's double (T-3016): the scale
/// multiplies the unit quantile, where dividing by the rate is an ulp off
/// (`GAMMA.INV(0.95,1,1000000)` = 2995732.27355399, probes-numeric m9-gamma-inv A26).
///
/// Other shapes take the crossing of the smaller tail of the cdf of rate 1 / scale
/// (`special::refine_increasing`, at most 129 cdfs, from statrs' search, [`inverse_cdf_within`],
/// when it has a result). Excel is close to the true quantile (against mpmath, 509 of 545
/// literal-argument quantile cells are within half a unit of the 15th digit), so statrs' result
/// is never kept as it is: it can be 7E-15 off (`GAMMA.INV(0.5,9,1)`). The smaller tail is `p` or
/// its exact complement `1 - p`, so a tiny right-tail `p` keeps the digits `1 - p` loses, as in
/// F.INV (T-0974), and the lower tail is the accurate pair's series, which has no shortcut at a
/// tiny x, so `CHISQ.INV(1E-100,1)` is Excel's 1.5707963267948966E-200 (T-1543, T-1634). All cdfs
/// share one budget of [`MAX_TERMS`] terms; NaN once it is spent.
pub(crate) fn gamma_quantile(shape: f64, scale: f64, p: f64, tail: Tail) -> f64 {
    let rate = 1.0 / scale;
    if !((0.0..=1.0).contains(&p) && rate > 0.0 && rate.is_finite()) {
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
    let x = if shape == 1.0 {
        exponential_quantile(small, target) * scale
    } else {
        let Ok(dist) = Gamma::new(shape, rate) else {
            return f64::NAN;
        };
        // One budget for every cdf below. Each `borrow_mut` lasts one cdf call and none nests
        // (`probability` calls no cdf of its own), so no borrow can overlap.
        let terms = RefCell::new(Terms(MAX_TERMS));
        let probability = |x: f64| quantile_tail(&dist, x, small, &mut terms.borrow_mut());
        let estimate = inverse_cdf_within(&dist, lower, &mut terms.borrow_mut());
        if dist.shape() > REFINE_SHAPE_MAX {
            return estimate;
        }
        let crossing = match small {
            Tail::Lower => refine_increasing(probability, target, estimate, 0.0, f64::MAX),
            Tail::Upper => refine_increasing(|x| -probability(x), -target, estimate, 0.0, f64::MAX),
        };
        if terms.borrow().0 == 0 {
            return f64::NAN;
        }
        crossing
    };
    // A quantile below the smallest normal double is 0, as Excel's `CHISQ.INV(1E-200,1)` and
    // `(1E-300,1)` are (the crossings are about 1.6E-400 and 1.6E-600; T-1543 probe A25, A26).
    if x < f64::MIN_POSITIVE {
        return 0.0;
    }
    x
}

/// The quantile of the unit exponential (shape 1, rate 1) where the probability of `tail` is `t`
/// in (0, 1/2]: the closed form `-ln(1 - t)` of the lower tail, by `ln_1p`, or `-ln(t)` of the
/// upper. Excel's quantile at shape 1 is this double (106 of the 108 literal-argument cells in
/// eval/ of CHISQ.INV, CHISQ.INV.RT and CHIINV at df 2 and GAMMA.INV at shape 1: `CHISQ.INV(0.5,2)`
/// = 1.3862943611198906 = 2 ln 2, `GAMMA.INV(0.025,1,1E+15)` = 25317807984289.875,
/// `CHISQ.INV(1E-16,2)` = 2E-16; T-3016). The crossing of the cdf lands 1 to 2 ulps off it on 69
/// of them. The two others are 1 ulp off (`CHISQ.INV.RT(0.9995,2)` and `(0.99999,2)`,
/// probes-numeric t3148-gamma-quantile-flat-band A97, A113).
fn exponential_quantile(tail: Tail, t: f64) -> f64 {
    match tail {
        Tail::Lower => -(-t).ln_1p(),
        Tail::Upper => -t.ln(),
    }
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
    // At x * rate up to 1 the lower tail is `x^a * e^(-x - ln Gamma(a + 1))` times the series
    // (T-1634). Only there: at a larger x the exponent is large and its absolute error becomes
    // the tail's relative error (1E-14 at `GAMMA.INV(0.5,50,1)`, whose true quantile is Excel's
    // 49.66706461799423, 13 ulps off with this form; T-0972). `powf` keeps a tiny quantile's
    // digits (`GAMMA.INV(1E-300,1,1)` is within an ulp of Excel's 1E-300), and against
    // `x^a e^-x / Gamma(a)` of the pair it is Excel's double in the far lower tail at shape 1/2
    // (`CHISQ.INV(1E-16,1)` = 1.5707963267948967E-32 and `CHISQ.INV(0.001,1)` =
    // 1.57079714926249E-6, gen-m9; the pair's quantiles are 2 ulps off). Both factors must be
    // normal doubles: at a large shape `x^a` overflows or the exponential underflows, and the
    // pair's prefactor is used.
    if tail == Tail::Lower && scaled <= 1.0 {
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
mod tests;
