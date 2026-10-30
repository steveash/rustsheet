//! bpser's ln B for both shapes above 1 and one above 8, where its GAMMALN sum stops
//! (`bpser_neg_ln_beta`; SPEC-0025's `ln_beta.rs`). Excel's own form there is unknown
//! (statistics.md "F.TEST / FTEST", T-2924: no form tried reproduces it). Of the two the research
//! model scores, T-3007 takes the accurate ln B at half-integer shapes ([`neg_ln_beta_half_integer`],
//! also F.TEST's) and TOMS 708 `betaln` in doubles with Excel's GAMMALN and LN elsewhere
//! ([`toms_betaln`]; `docs/research/t2464/r2-bratio/toms_special.py` `betaln`, `bcorr`): TOMS at
//! every shape gains more bit-exact cells but moves a gated BINOM.DIST cell at B(2, 9) off
//! (statistics.md "Incomplete beta bpser", T-3007).

use super::super::beta_density::{algdiv, alnrel, stirling_del, ALGDIV_C};
use super::super::fisher::half_integer_beta;
use super::super::gammaln::excel_gammaln;
use crate::x87_exp::{dd_add, dd_to_extended};
use crate::x87_log10::{excel_ln, ln_double_double};
use crate::x87_register::Register;

/// TOMS's ln(sqrt(2 pi)) as `betaln` writes it.
const LN_SQRT_2PI: f64 = 0.918_938_533_204_673;

/// The smaller shape from which `betaln` takes `bcorr`'s Stirling form.
const BCORR_MIN_SHAPE: f64 = 8.0;

/// The larger shape above which `betaln`'s reduction of the smaller one divides by b.
const REDUCTION_LARGE_B: f64 = 1000.0;

/// ln B(a, b) by TOMS 708 `betaln` for min(a, b) > 1 and max(a, b) > 8; `None` for other shapes.
/// The smaller shape is reduced by one per step below 8, so the loop runs at most 6 times.
pub(super) fn toms_betaln(a: f64, b: f64) -> Option<f64> {
    let (mut a, b) = if a < b { (a, b) } else { (b, a) };
    if !(a > 1.0 && b > BCORR_MIN_SHAPE && b.is_finite()) {
        return None;
    }
    if a >= BCORR_MIN_SHAPE {
        let w = bcorr(a, b);
        let h = a / b;
        let u = -(a - 0.5) * excel_ln(h / (h + 1.0));
        let v = b * alnrel(h);
        let e = (excel_ln(b) * -0.5 + LN_SQRT_2PI) + w;
        return Some(if u > v { (e - v) - u } else { (e - u) - v });
    }
    if a < 2.0 {
        return Some(excel_gammaln(a) + algdiv(a, b));
    }
    // 2 <= a < 8: Gamma(a) = (a - 1)(a - 2)...Gamma(a - n), n = int(a - 1).
    let n = (a - 1.0).trunc();
    let mut w = 1.0;
    let mut i = 0.0;
    while i < n {
        a -= 1.0;
        if b > REDUCTION_LARGE_B {
            w *= a / (a / b + 1.0);
        } else {
            let h = a / b;
            w *= h / (h + 1.0);
        }
        i += 1.0;
    }
    Some(if b > REDUCTION_LARGE_B {
        (excel_ln(w) - n * excel_ln(b)) + (excel_gammaln(a) + algdiv(a, b))
    } else {
        (excel_ln(w) + excel_gammaln(a)) + algdiv(a, b)
    })
}

/// TOMS 708 `bcorr(a, b)` = del(a) + del(b) - del(a + b) for 8 <= a <= b, del the Stirling
/// remainder of ln Gamma.
fn bcorr(a: f64, b: f64) -> f64 {
    let [c0, c1, c2, c3, c4, c5] = ALGDIV_C;
    let h = a / b;
    let w = stirling_del(h / (h + 1.0), 1.0 / (h + 1.0), b);
    let t = 1.0 / a;
    let t = t * t;
    (((((c5 * t + c4) * t + c3) * t + c2) * t + c1) * t + c0) / a + w
}

/// The largest a + b for which [`neg_ln_beta_half_integer`] builds B(a, b) by its recurrence (one
/// step per unit of a + b; directive 5).
pub(in super::super) const HALF_INTEGER_SHAPE_SUM_MAX: f64 = 400.0;

/// -ln B(a, b) for half-integer shapes a, b >= 1/2, accurate and rounded to a register: the
/// double-double B of `fisher::half_integer_beta` and its double-double ln. `None` off those
/// shapes and above [`HALF_INTEGER_SHAPE_SUM_MAX`].
pub(in super::super) fn neg_ln_beta_half_integer(a: f64, b: f64) -> Option<Register> {
    let half_integer = |v: f64| v >= 0.5 && (2.0 * v).fract() == 0.0;
    if !(half_integer(a) && half_integer(b) && a + b <= HALF_INTEGER_SHAPE_SUM_MAX) {
        return None;
    }
    let (hi, lo) = half_integer_beta(a, b);
    let ln_beta = dd_add(ln_double_double(hi), (lo / hi, 0.0));
    // `dd_to_extended` takes a positive head: B < 1, so -ln B > 0, wherever one shape is above 1
    // and neither is below 1/2 (B(1/2, 1/2) = pi and B(1/2, 1) = 2 are the exceptions).
    if ln_beta.0 >= 0.0 {
        return None;
    }
    let (significand, exponent) = dd_to_extended((-ln_beta.0, -ln_beta.1));
    Some(Register::from_parts(false, significand, exponent))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every branch of `betaln` against lgamma(a) + lgamma(b) - lgamma(a + b) (Python's math):
    // bcorr (both >= 8), gamln + algdiv (a < 2), the reduction of a (b <= 1000 and above).
    #[test]
    fn toms_betaln_is_ln_beta() {
        for (a, b, reference) in [
            (20.0, 30.0, -33.968_820_791_977_41),
            (12.25, 8.5, -13.917_079_018_943_006),
            (1.5, 9.7, -3.566_351_991_371_837),
            (9.7, 3.3, -6.869_031_206_977_969),
            (2.5, 1500.0, -17.999_617_542_043_78),
        ] {
            let ln_beta = toms_betaln(a, b).unwrap_or(f64::NAN);
            assert!(
                ((ln_beta - reference) / reference).abs() < 1e-13,
                "ln B({a}, {b}) = {ln_beta}, not {reference}"
            );
        }
        // Outside its domain: a shape of 1 or less, or both shapes up to 8.
        assert_eq!(toms_betaln(1.0, 20.0), None);
        assert_eq!(toms_betaln(0.5, 20.0), None);
        assert_eq!(toms_betaln(3.0, 8.0), None);
    }

    // B(2, 9) = 1/90 exactly, so -ln B is LN(90) correctly rounded; off half-integer shapes and
    // above the shape-sum cap there is no accurate value.
    #[test]
    fn half_integer_ln_beta_is_accurate() {
        let neg = |a, b| neg_ln_beta_half_integer(a, b).map(Register::store);
        assert_eq!(neg(2.0, 9.0), Some(4.499_809_670_330_265));
        assert_eq!(neg(9.0, 2.0), Some(4.499_809_670_330_265));
        assert_eq!(neg(2.2, 9.0), None);
        assert_eq!(neg(2.0, HALF_INTEGER_SHAPE_SUM_MAX), None);
    }
}
