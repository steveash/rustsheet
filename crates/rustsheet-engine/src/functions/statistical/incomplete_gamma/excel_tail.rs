//! Excel's far upper tail of Q(a, x) at a shape from 3/2 to [`FAR_TAIL_MAX_SHAPE`] (T-3269):
//! POISSON's cdf and CHISQ.DIST.RT (CHIDIST, CHISQ.TEST) past x = [`FAR_TAIL_MIN_X`].
//!
//! There Excel is 10 to 400 ulp from the true Q (probes-numeric/t3284-gamma-deep-upper-tail,
//! docs/excel-semantics/statistics.md "Excel's far upper tail (T-3269)"), and its error is the one
//! T-2638 found above shape 20 away from the mode: the prefactor is Stirling's ratio form
//! sqrt(a / 2 pi) exp(a ln(x/a) + (a - x) - correction(a)) ([`excel_ratio_exponent`],
//! [`stirling_prefactor`]), whose exponent, rounded to a double, carries an ulp of a number near
//! ln Q into the result. Over the 32 adjacent doubles of `POISSON(2,100+j*2^-46,TRUE)` Excel jumps
//! by 64 ulp exactly where that exponent's rounding does. The continued fraction it multiplies is
//! taken accurately ([`accurate_fraction`]); the correction is the exact Stirling error
//! ([`stirlerr_exact`]), since TOMS's series is only good from 8.
//!
//! The form is applied only where it was measured (decision 0188 R4.3): x in
//! ([`FAR_TAIL_MIN_X`], [`FAR_TAIL_MAX_X`]] at the multiples of 1/2 up to [`FAR_TAIL_MAX_SHAPE`].
//! At and below x = 30 Excel is the accurate value at integer shapes (where this form is up to 22
//! ulp off), and above shape 11 next to the cut Excel is not this form, so the callers keep their
//! accurate kernels there.

use super::super::half_integer_beta::is_half_integer_shape;
use super::super::loader::stirlerr_exact;
use super::fraction::accurate_fraction;
use super::{excel_ratio_exponent, stirling_prefactor, Terms};

/// The x above which Excel takes the ratio form. Bracketed by the T-3269 boundary probe
/// (probes-numeric/t3269-gamma-tail-switch, x from 26 to 40 by 0.5 at shapes 2 to 4): at shapes
/// 2, 3 and 4 Excel is the correctly rounded Q at every x up to 30 and the ratio form from 30.5 on
/// (`POISSON(2,30,TRUE)` exact, `POISSON(2,30.5,TRUE)` 32 ulp from exact and 3 from the form).
/// FITTED(T-3269, 145 cells, x 26..40, shape 2..4)
const FAR_TAIL_MIN_X: f64 = 30.0;

/// The largest shape the form is applied at: the cut at [`FAR_TAIL_MIN_X`] is Excel's up to here.
/// On the R4 hold-out's train cells next to the cut (30 < x <= 45, docs/research/t3269/holdout.py),
/// counting only gaps above the oracle's 4-ulp floor (decision 0188 R4.4), the form is closer to
/// Excel than the accurate kernel at 80 cells and farther at 12 for shapes 3/2 to 11 (test: 8 and
/// 1); above 11 it is closer at 1 and farther at 3 (`CHISQ.DIST.RT(63.1168,23)` 19 ulp against 10,
/// `CHISQ.DIST.RT(68.6972159,27)` 10 against 2, `POISSON(17,31.4903,TRUE)` 36 against 28), so
/// Excel's switch is elsewhere there (T-3469) and the accurate kernel stays.
/// FITTED(T-3269, 153 cells, shape 1.5..19.5, x 30..45)
const FAR_TAIL_MAX_SHAPE: f64 = 11.0;

/// The largest x the form was measured at (`POISSON(19,600,TRUE)`, t3284-gamma-deep-upper-tail
/// A111); beyond it the callers keep the accurate kernel (decision 0188 R4.3). (UNVERIFIED,
/// T-3469): no cell past it says where Excel leaves the form.
const FAR_TAIL_MAX_X: f64 = 600.0;

/// Excel's Q(a, x) for a shape that is a multiple of 1/2 from 3/2 to [`FAR_TAIL_MAX_SHAPE`] and
/// [`FAR_TAIL_MIN_X`] < x <= [`FAR_TAIL_MAX_X`]; `None` elsewhere (a = 1 is exp(-x), which
/// Excel has correctly rounded), where the result is not a normal double, or past `terms`.
pub(super) fn excel_far_upper(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    let measured = is_half_integer_shape(a) && a > 1.0 && a <= FAR_TAIL_MAX_SHAPE;
    if !(measured && x > FAR_TAIL_MIN_X && x <= FAR_TAIL_MAX_X) {
        return None;
    }
    let fraction = accurate_fraction(a, x, terms)?;
    let prefactor = stirling_prefactor(a, excel_ratio_exponent(a, x), stirlerr_exact(a).0);
    let upper = prefactor * (fraction.0 + fraction.1);
    upper.is_normal().then_some(upper)
}
