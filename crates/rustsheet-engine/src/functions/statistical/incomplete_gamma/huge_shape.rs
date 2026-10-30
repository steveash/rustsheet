//! Excel's gamma quantile at a huge shape (GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and
//! CHIINV from a shape of 1E9; T-2982, T-3070; docs/excel-semantics/statistics.md "GAMMA.INV and
//! CHISQ.INV at a huge shape").
//!
//! Excel's value there is one of three things, by where its Wilson-Hilferty start lies
//! (probes/t3070-gamma-inv-band, 381 cells, and probes/t2982-gamma-inv-huge-shape, excel-graph):
//! the start itself, a closed form 1.000266 times the true quantile, where the density at the
//! start is not a normal double; the true quantile, inside a band of the start's offset and the
//! normal quantile z; and `#NUM!` outside that band. Excel's search itself is not known: the band's
//! edges are fits, each bracketed by the cells that pin it.

use super::{gamma_density, gamma_quantile, Tail};

/// From this shape on the quantile is [`huge_shape_quantile`]. Below it statrs' search and the
/// refinement give the true quantile, as Excel does at every probed p down to shape 1E8 (T-1634);
/// at 1E9 Excel is already `#NUM!` at p = 1E-300 (probes/t3070-gamma-inv-band A1).
pub(crate) const HUGE_SHAPE_MIN: f64 = 1e9;

/// The offset Excel adds to the Wilson-Hilferty cube root: (1.0002658235505755)^(1/3) - 1 is
/// 0.0000886 to 13 digits (probes/t2982-gamma-inv-huge-shape A7-A23).
const WILSON_HILFERTY_OFFSET: f64 = 0.000_088_6;

/// The factor on Excel's z term, fitted (T-3070) with the three terms below to every closed-form
/// cell from shape 1.5E10 to 1E300 (167 cells of probes/t2982-gamma-inv-huge-shape and
/// probes/t3070-gamma-inv-band, all within 3.6 ulps of the fit; T-2982's 1 - 2.2106E-5 was fitted
/// at 1E13 alone, 100 to 1,500 ulps off at 1E11).
const WILSON_HILFERTY_SPREAD: f64 = 0.999_977_894_167;

/// Excel's terms in 1/a, z^2/a and z^3/a^(3/2) inside the cube, fitted with
/// [`WILSON_HILFERTY_SPREAD`] (UNVERIFIED as a form: the probes pin values, not the formula). They
/// are the absolute -2.3E-3 + 2.0E-4 z^2 that T-2982 found left at 1E11 (A60-A77, 3a times
/// each), and fade as the true quantile's own corrections do.
const WILSON_HILFERTY_INVERSE_SHAPE: f64 = -7.586_532e-4;
const WILSON_HILFERTY_SQUARE: f64 = 6.740_481e-5;
const WILSON_HILFERTY_CUBE: f64 = -3.086_760e-3;

/// Excel's Wilson-Hilferty start for a scale of 1, where `z` is the standard normal quantile of
/// the lower tail's probability: a (1 - 1/(9a) + z/(3 sqrt a))^3 with Excel's offset, spread and
/// higher terms inside the cube (probes/t2982-gamma-inv-huge-shape, probes/t3070-gamma-inv-band).
/// Where its density underflows it is Excel's quantile.
pub(crate) fn wilson_hilferty_quantile(shape: f64, z: f64) -> f64 {
    let root = shape.sqrt();
    let cube_root = 1.0 + WILSON_HILFERTY_OFFSET - 1.0 / (9.0 * shape)
        + z * WILSON_HILFERTY_SPREAD / (3.0 * root)
        + (WILSON_HILFERTY_INVERSE_SHAPE + WILSON_HILFERTY_SQUARE * z * z) / shape
        + WILSON_HILFERTY_CUBE * z * z * z / (shape * root);
    shape * cube_root * cube_root * cube_root
}

/// The weight of the start's offset (in standard deviations, about 2.66E-4 sqrt a) in the upper
/// edge of the band where Excel finds the true quantile: z + 1.35 offset < 27.25. A fit
/// (UNVERIFIED between the cells): with the slope 1.35 the edge lies in (26.93, 27.60), pinned by
/// probes/t3070-gamma-inv-band A77 `GAMMA.INV(0.001,7E9,1)` (a value) and
/// probes/t2982-gamma-inv-huge-shape A45 `GAMMA.INV(1E-16,1E10,1)` (`#NUM!`); slopes 1.30 to 1.41
/// also separate every probed cell, 1.0 and 1.5 do not.
const BAND_UPPER_SLOPE: f64 = 1.35;

/// The upper edge of the band, for [`BAND_UPPER_SLOPE`].
const BAND_UPPER_EDGE: f64 = 27.25;

/// The lower edge of the band: offset - z < 39. A fit in (38.05, 40.07), pinned by
/// probes/t2982-gamma-inv-huge-shape A44 `GAMMA.INV(1E-30,1E10,1)` = 9998853641.00538 (a value)
/// and probes/t3070-gamma-inv-band A44 `GAMMA.INV(1E-100,5E9,1)` (`#NUM!`).
const BAND_LOWER_EDGE: f64 = 39.0;

/// Excel's quantile of the gamma distribution of `shape` (at least [`HUGE_SHAPE_MIN`]) and
/// `scale` where the probability of `tail` is `p` in (0, 1), and `z` the standard normal quantile
/// of the lower tail's probability (for the upper tail, minus that of `p`, so a tiny `p` keeps its
/// digits: `CHISQ.INV.RT(1E-100,1E10)` = 10005667524.015223, probes/t3070-gamma-inv-band A371).
/// NaN where Excel is `#NUM!`.
///
/// - Where the density at the start ([`wilson_hilferty_quantile`]) is below the smallest normal
///   double, the start times the scale: the closed form from shape 2E10 at p = 0.5 and every
///   probed cell with it (a fit: ln density -708.54 at the closed form's
///   `GAMMA.INV(0.999999,1.5E10,1)`, A110, and -701.60 at `GAMMA.INV(0.5,1.95E10,1)`, A210,
///   `#NUM!`; ln of the smallest normal double is -708.40).
/// - Else, inside the band of [`BAND_UPPER_EDGE`] and [`BAND_LOWER_EDGE`], the true quantile
///   ([`gamma_quantile`]): `GAMMA.INV(0.5,5E9,1)` = 4999999999.666667, `CHISQ.INV(1E-100,6E9)` =
///   5997669910.951994 (A358).
/// - Else NaN: `GAMMA.INV(0.5,6E9,1)` (A65), `GAMMA.INV(p,1.2E10,1)` at every probed p (A85-A98).
pub(crate) fn huge_shape_quantile(shape: f64, scale: f64, p: f64, tail: Tail, z: f64) -> f64 {
    let start = wilson_hilferty_quantile(shape, z);
    if gamma_density(shape, start, 1.0).is_none_or(|density| density < f64::MIN_POSITIVE) {
        return start * scale;
    }
    let offset = (wilson_hilferty_quantile(shape, 0.0) - shape) / shape.sqrt();
    if z + BAND_UPPER_SLOPE * offset < BAND_UPPER_EDGE && offset - z < BAND_LOWER_EDGE {
        gamma_quantile(shape, scale, p, tail)
    } else {
        f64::NAN
    }
}
