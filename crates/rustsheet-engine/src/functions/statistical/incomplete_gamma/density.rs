//! The gamma density, on the prefactor P and Q share ([`gamma_prefactor`]), and Excel's own form
//! of it above the mode below shape 20 (T-3147).

use super::super::loader::stirlerr_exact;
use super::{
    excel_ratio_exponent, gamma_prefactor, stirling_prefactor, Exponent, DIRECT_SHAPE_MAX,
    NEAR_MODE,
};

/// The smallest shape below [`DIRECT_SHAPE_MAX`] at which the density above the mode takes Excel's
/// ratio form ([`density_with`]). A fit inside its evidence bracket (2.5, 3] (T-3147; ulp from
/// Excel, the ratio form against the direct one): at 3 `CHISQ.DIST(37.5,6,FALSE)` is -1 against -6
/// (gen-m9/984cba67bbe6972b A42), at 5 `CHISQ.DIST(40,10,FALSE)` 0 against -8 (probes-numeric/
/// m9-chisq-dist A50); at 2.5 `CHISQ.DIST(50,5,FALSE)` is 23 against 5 (probes-numeric/
/// t1049-chisqdist A46).
const SMALL_RATIO_SHAPE_MIN: f64 = 3.0;

/// The gamma density x^(a-1) e^(-x/scale) / (scale^a Gamma(a)) for a > 0, x > 0 and scale > 0,
/// as [`gamma_prefactor`]`(a, x / scale) / x`: the prefactor P and Q share, so the density keeps
/// its digits where statrs' `exp(ln pdf)` loses the absolute error of its log (T-0971). `None` at
/// x = 0 or where x / scale is not normal, which keep statrs' closed forms.
pub(crate) fn gamma_density(a: f64, x: f64, scale: f64) -> Option<f64> {
    density_with(a, x, scale, Exponent::Excel)
}

/// [`gamma_density`] with the accurate exponent throughout, for the large-shape beta paths, where
/// the ratio form is unprobed (T-3252).
pub(crate) fn accurate_gamma_density(a: f64, x: f64, scale: f64) -> Option<f64> {
    density_with(a, x, scale, Exponent::Accurate)
}

fn density_with(a: f64, x: f64, scale: f64, form: Exponent) -> Option<f64> {
    let y = x / scale;
    if x <= 0.0 || !y.is_normal() {
        return None;
    }
    // The exponential density e^(-x/scale) / scale, as statrs forms it: one rounding fewer than
    // the prefactor's y e^-y / x (gen-m9/ecc87f80077959e0 A261 `CHISQ.DIST(1.5,2,FALSE)`).
    if a == 1.0 {
        return Some((-y).exp() / scale);
    }
    // Below shape 20, above the mode, Excel's density takes its large-shape ratio form too, with
    // the exact Stirling error (the asymptotic series of `stirling_correction` is 2E-13 off at a
    // shape of 5): `CHISQ.DIST(40,10,FALSE)` is Excel's 6.870512074795199E-6, 7 ulp above the
    // true value, where the direct form is 8 ulp below Excel (T-3147). Its cdf does not follow:
    // the same prefactor moves `CHIDIST(38.5,6)` from Excel's value to 8 ulp off, so P and Q keep
    // the direct form (T-3219). Below the mode the direct form is as close or closer (the 1-ulp
    // `GAMMA.DIST(10,9,2,FALSE)` of ironcalc-calc GAMMA.xlsx E3 at 0.56a).
    if form == Exponent::Excel
        && (SMALL_RATIO_SHAPE_MIN..DIRECT_SHAPE_MAX).contains(&a)
        && y - a > a * NEAR_MODE
    {
        return Some(stirling_prefactor(a, excel_ratio_exponent(a, y), stirlerr_exact(a).0) / x);
    }
    Some(gamma_prefactor(a, y, form) / x)
}
