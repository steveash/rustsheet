//! Excel's closed-form gamma quantiles in the far lower tail (GAMMA.INV, GAMMAINV and CHISQ.INV;
//! T-3013 below shape 1, T-2561 above it; docs/excel-semantics/statistics.md, T-3013 bullet and
//! "Deep lower tail at a shape above 1"): the root of the cdf's leading factor, with or without
//! one Newton step. Moved out of `mod.rs` unchanged (T-3313).

use super::{
    excel_form_lower, gamma_density, pair_with, Exponent, Terms, DIRECT_SHAPE_MAX, MAX_TERMS,
};
use crate::functions::statistical::gammaln::excel_gammaln;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;

/// Below this quantile (at scale 1) GAMMA.INV at a shape below 1 takes the closed form
/// [`excel_small_shape_quantile`] and not the search (T-3013). This is a choice inside the
/// evidence bracket (4.15E-22, 6.75E-21), not a measured Excel constant. A28
/// `GAMMA.INV(1E-16,0.75,1)` = 4.147567013992107E-22 is the closed form, while A18 and B18
/// `GAMMA.INV(1E-5,0.25,..)` (6.75E-21 at scale 1) are neither it nor one Newton step from it on
/// the engine's model of Excel's cdf; so are A34 `(1E-16,0.9,1)` (1.6E-18) and
/// gen-m9/60b0b6e69ad35a45 A210 (shape 1/60, 4.9E-19). The cut is on x, not on p or a·LN(x): A28
/// and A34 share p = 1E-16 and nearly the same a·LN(x) (about -36.9), and only A28 is the closed
/// form. Below it the series' correction (about x / (a + 1)) is far below an ulp, so the closed
/// form is the quantile of the leading factor alone. T-3265 probes the gap.
/// FITTED(T-3013, 5 cells, x 4.15E-22..1.6E-18, shape 1/60..0.9)
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
/// [`EXCEL_EXPONENT_MAX`](super::EXCEL_EXPONENT_MAX) and the engine's GAMMA.DIST is the pair. A254 and A396 need that: on
/// GAMMA.DIST's value their steps would be 2.2E-14 (kept) and 4.2E-14 (taken), both against Excel.
/// So either Excel's GAMMA.INV steps on an internal form, or the engine's GAMMA.DIST is off there
/// (T-3265 probes it). One step only: at A26 a second would be 3.46E-14 of x and Excel takes none.
/// A loop under the same test, with the threshold in [3.46E-14, 4.1E-14), fits as well. T-3265
/// probes steps from 2.9E-14 to 4.0E-14.
/// FITTED(T-3013, 4 cells, step 1.33E-14..4.4E-14 of x, shape 0.01..0.75)
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
    let mut x = leading_factor_root(a, p);
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

/// The root of the cdf's leading factor, p = x^a / Gamma(a + 1), as Excel forms it:
/// `EXP((LN(p)+GAMMALN(a)+LN(a))/a)`, summed in that order (T-3013 at a shape below 1, T-2561
/// above it). NaN where a logarithm is.
fn leading_factor_root(a: f64, p: f64) -> f64 {
    excel_exp((excel_ln(p) + excel_gammaln(a) + excel_ln(a)) / a)
}

/// How far, relative to p, GAMMA.DIST at [`leading_factor_root`] may be from p for
/// [`excel_tiny_quantile`] to return that root. A fit, not an Excel constant, and a proxy for
/// Excel's own test, which is unknown. Two cells set the bracket [4.09E-14, 5.88E-14): gen-m9/
/// 0a9226016e8f5f33 A240 `CHISQ.INV(1E-200,7)`, the root bit for bit with our cdf 4.09E-14 from
/// p, and d8266bc15533702a A212 `CHISQ.INV(1E-200,10)`, not the root (Excel 111 ulp above it)
/// with our cdf 5.88E-14 from p. Over the cells it was fitted on, 7 distinct (shape, p) points are
/// in and bit-exact and 2 are out. The ones in: (2, 1E-300) at A293 and A123, (2.5, 1E-300) A217,
/// (3.5, 1E-200) A240, (2, 1E-100) A283, (4.5, 1E-200) A252, (3, 1E-100) A184 and A399, and
/// (1.500000000000001, 1E-100) A142. The ones out: (5, 1E-200) A212 and (2.5, 1E-200) at A128, A98
/// and caa1eed48b1883fe A282. The test is not Excel's: at shape 25 Excel's
/// `CHISQ.INV(1E-300,50)` is the root (probes-numeric t1543-inverse-cdf-tiny-p A31) with our cdf
/// 9.8E-12 off (TODO(T-3294)).
/// FITTED(T-2561, 13 cells, shape 1.5..5, p 1E-300..1E-100)
const TINY_QUANTILE_CHECK: f64 = 5e-14;

/// Excel's lower-tail quantile at a shape above 1 (and below [`DIRECT_SHAPE_MAX`]) where it is
/// tiny (T-2561): the root of the leading factor, [`leading_factor_root`], with no Newton step.
/// It applies where GAMMA.DIST (the pair with Excel's prefactor) at that root is within
/// [`TINY_QUANTILE_CHECK`] of p. The check also limits the form to tiny quantiles. The cdf's other
/// factors, e^-x and the series, are 1 - x a / (a + 1) to first order, so above x of about 5E-14
/// they move the cdf past the check. At `CHISQ.INV(1E-16,3)` (x = 2.6E-11, 1.6E-11 off)
/// the search takes over. The check is not Excel's own test. At shape 25, `CHISQ.INV(1E-300,50)`
/// is the root bit for bit in Excel (probes-numeric t1543-inverse-cdf-tiny-p A31), though the cdf
/// there is 9.8E-12 off; the shape bound keeps that cell on the search (TODO(T-3294)). The root
/// differs from the true quantile by the rounding of Excel's exponent: `CHISQ.INV(1E-300,4)` is
/// Excel's 2.828427124746146E-150, 82 ulp below the crossing. `None` elsewhere.
pub(super) fn excel_tiny_quantile(a: f64, p: f64) -> Option<f64> {
    if !(a > 1.0 && a < DIRECT_SHAPE_MAX && p > 0.0 && p < 1.0) {
        return None;
    }
    let x = leading_factor_root(a, p);
    if !(f64::MIN_POSITIVE..1.0).contains(&x) {
        return None;
    }
    // One cdf of a shape below 20 at x below 1: the series, at most `cdf_terms_bound(20)` terms.
    let cdf = pair_with(a, x, &mut Terms(MAX_TERMS), Exponent::Excel).0;
    ((cdf / p - 1.0).abs() <= TINY_QUANTILE_CHECK).then_some(x)
}
