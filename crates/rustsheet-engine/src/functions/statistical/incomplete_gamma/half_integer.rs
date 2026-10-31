//! The upper incomplete gamma Q(a, x) at a half-integer shape a = n + 1/2 below
//! [`DIRECT_SHAPE_MAX`], for x >= a, in double-double and rounded once (T-3285): CHISQ.DIST.RT,
//! CHIDIST and CHISQ.TEST at an odd df from 3 on, where x lies in [`half_integer_upper`]'s range.
//!
//! There Excel is correctly rounded at the probed points (60-digit mpmath, ulps of Excel's value):
//! `CHISQ.DIST.RT(10,5)` = Q(2.5, 5) is 0.07523524614651218 (+0.3; probes-numeric/
//! m9-chisq-dist-rt A19, m9-legacy-chidist A45 and A46, t1049-chisqdistrt A19) and
//! `CHISQ.DIST.RT(8,3.34)` = Q(1.5, 4) 0.04601170568923137 (-0.0; ironcalc-calc STATISTICAL/
//! CHISQ.xlsx G3), where the double kernel was 5.7 and 11 ulp off. Its loss was the forward
//! (Lentz) continued fraction: every step's rounding is carried into the product, 5.1 ulp at
//! Q(2.5, 5) and 16 at Q(1.5, 3), while the prefactor x^a e^-x / Gamma(a) was within 1.3
//! (docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)").
//!
//! So both factors are taken in double-double: the prefactor as
//! sqrt(x) e^-x / sqrt(pi) * prod_(j=1..n) 2x / (2j - 1), which is exact in its shape
//! (Gamma(n + 1/2) = sqrt(pi) (2n - 1)!! / 2^n), and the fraction evaluated backward, from a
//! depth past the forward one's stopping point, where each step's rounding is damped rather than
//! multiplied.

use super::super::half_integer_beta::{is_half_integer_shape, FRAC_1_SQRT_PI};
use super::fraction::{accurate_fraction, Dd};
use super::{Terms, DIRECT_SHAPE_MAX};
use crate::x87_exp::{dd_div_small, dd_exp, dd_mul, dd_sqrt};

/// Past this x the prefactor's e^-x is past [`dd_exp`]'s double-double range (its tail word turns
/// subnormal); such points, Q below about 1E-282, keep the double kernel.
const MAX_X: f64 = 650.0;

/// The largest x / a at which CHISQ.DIST.RT takes [`accurate_half_integer_upper`]. Measured on
/// the gated cells the accurate Q changed when applied from a to 30a (T-3285 attempt 1, x/a from
/// 1.05 to 13.3): up to 3a it fixed or moved closer every one but two that went from 2 to 3 ulp
/// (`CHISQ.DIST.RT(8,3.34)` and `(12.3,7.2)`, ironcalc-calc CHISQ.xlsx G3 and G4, and
/// `CHISQ.DIST.RT(10,5)` four times in probes-numeric), while past it Excel is not correctly
/// rounded either and the accurate Q lost `CHIDIST(42,7)` = Q(3.5, 21) (gen-m9/fec39efaba9284c3
/// A249), where Excel is 1 ulp above it.
/// FITTED(T-3285, 17 cells, x/a 1.05..13.3)
const ACCURATE_MAX_RATIO: f64 = 3.0;

/// CHISQ.DIST.RT's Q(a, x) at a half-integer shape from 3/2 up to [`DIRECT_SHAPE_MAX`] for
/// `a + 1 <= x <= 3a` ([`ACCURATE_MAX_RATIO`]): the accurate Q there, `None` elsewhere. Below a + 1
/// Excel is 1 - P (`CHIDIST(5,5)` = Q(2.5, 2.5) and `CHIDIST(7,7)` are its bits, 1 ulp from the
/// accurate Q: gen-m9/4250f880ab640c49 A331, 899f9555e67da2d6 A72), so the caller keeps its pair.
pub(super) fn half_integer_upper(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    if a < 1.5 || !(x >= a + 1.0 && x <= ACCURATE_MAX_RATIO * a) {
        return None;
    }
    accurate_half_integer_upper(a, x, terms)
}

/// Q(a, x) for a half-integer `a` from 1/2 up to [`DIRECT_SHAPE_MAX`] and `a <= x <= MAX_X`, to
/// within about half an ulp; `None` elsewhere, where Q is not a normal double, or past `terms`
/// (the forward fraction that sets the depth draws on them), so the caller keeps its kernel.
pub(super) fn accurate_half_integer_upper(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    if !(is_half_integer_shape(a) && a.fract() != 0.0 && a < DIRECT_SHAPE_MAX) {
        return None;
    }
    if !(x >= a && x <= MAX_X) {
        return None;
    }
    let upper = dd_mul(prefactor(a, x), accurate_fraction(a, x, terms)?);
    upper.0.is_normal().then_some(upper.0)
}

/// x^a e^-x / Gamma(a) at a = n + 1/2: sqrt(x) e^-x / sqrt(pi) times 2x / (2j - 1) for j = 1..n.
fn prefactor(a: f64, x: f64) -> Dd {
    let mut factor = dd_mul(dd_mul(dd_sqrt((x, 0.0)), dd_exp((-x, 0.0))), FRAC_1_SQRT_PI);
    // 2x is exact (x <= MAX_X), and 2j - 1 a small odd integer; a < 20 bounds the loop.
    let mut odd = 1.0;
    while odd < 2.0 * a - 0.5 {
        factor = dd_div_small(dd_mul(factor, (2.0 * x, 0.0)), odd);
        odd += 2.0;
    }
    factor
}
