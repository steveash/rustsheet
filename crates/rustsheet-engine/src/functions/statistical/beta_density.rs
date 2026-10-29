//! BETA.DIST's density x^(a-1) (1-x)^(b-1) / B(a, b) for min(a, b) <= 1: T-2513's model M2
//! (docs/excel-semantics/statistics.md "Beta density for min(a,b) <= 1: lnB and a double-double
//! exponent split"; reference code `docs/research/t2464/r3-m1-residual/final_score.py` `lnb_m2`
//! and `m1lib.py` `combine(.., "m2")`). Every step that the model rounds through an x87 register
//! is an `extended_*` op here:
//! ```text
//! tx = fl(fl(a-1)·LN(x));  ty = fl(fl(b-1)·ly)     ly = Excel's ln(1 - x)
//! lnB: a = 1 or b = 1     -> -LN(the other shape)
//!      min < 1, max <= 8  -> fl(GL(min) + fl(GL(max) - GL(a+b)))     GL = Excel's GAMMALN
//!      min < 1, max > 8   -> fl(GL(min) + algdiv(min, max))          TOMS 708 algdiv in doubles
//! hi = (tx + ty) - lnB in doubles;  lo = the exact remainder of those two additions
//! density = EXP(hi)·fl(1 + lo)
//! ```
//! The lo term carries the exponent's rounding error into the product, so the result is close to
//! correctly rounded (T-2513: 12,799 of w1's 12,800 probe cells within 15 digits, against 10,375
//! bit-exact for the extended-exponent model M1 it replaces). The engine's GAMMALN differs from
//! Excel's at some shapes below 1 (T-2122), which moves last bits, not the 15 digits.

use super::gammaln::excel_gammaln;
use crate::arithmetic::{extended_product, extended_sum};
use crate::x87_exp::{excel_exp, two_sum};
use crate::x87_log10::excel_ln;
use crate::x87_log1p::excel_ln_one_minus;

/// The larger shape up to which lnB is the GAMMALN form, 8 included (TOMS 708 `betaln` switches to
/// `algdiv` at 8 exclusive; Excel at 8 inclusive, statistics.md).
const GAMMALN_FORM_MAX: f64 = 8.0;

/// TOMS 708 `algdiv`'s (and `bcorr`'s) Stirling coefficients c0..c5.
pub(super) const ALGDIV_C: [f64; 6] = [
    0.083_333_333_333_333_3,
    -0.002_777_777_777_609_91,
    7.936_506_668_253_9e-4,
    -5.952_029_313_518_7e-4,
    8.373_080_340_312_15e-4,
    -0.001_653_229_627_807_13,
];

/// TOMS 708 `alnrel`'s (3,3) rational: numerator p1..p3 and denominator q1..q3.
const ALNREL_P: [f64; 3] = [
    -1.294_189_230_219_93,
    0.405_303_492_862_024,
    -0.017_887_454_601_221_4,
];
const ALNREL_Q: [f64; 3] = [
    -1.627_522_563_553_23,
    0.747_811_014_037_616,
    -0.084_510_421_794_556_5,
];

/// The density for 0 < x < 1 and min(a, b) <= 1 by model M2 (module docs); `None` for other
/// shapes and where the result is not finite (the caller keeps its own path there).
pub(crate) fn beta_density_small_shape(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(x > 0.0 && x < 1.0 && a > 0.0 && b > 0.0 && a.min(b) <= 1.0) {
        return None;
    }
    let tx = extended_product(extended_sum(a, -1.0), excel_ln(x));
    let ty = extended_product(extended_sum(b, -1.0), excel_ln_one_minus(x));
    let ln_beta = ln_beta_small_shape(a, b);
    let (sum, sum_error) = two_sum(tx, ty);
    let (hi, hi_error) = two_sum(sum, -ln_beta);
    let density = excel_exp(hi) * (1.0 + (sum_error + hi_error));
    density.is_finite().then_some(density)
}

/// ln B(a, b) for min(a, b) <= 1 as model M2 forms it (module docs).
fn ln_beta_small_shape(a: f64, b: f64) -> f64 {
    if a == 1.0 || b == 1.0 {
        // B(1, b) = 1/b: -LN(the other shape), at any size of it (statistics.md: 32 of 32 shapes
        // with min = 1; with the other shape below 1 only (0.37, 1) is measured, where the GAMMALN
        // form is 1 ulp high).
        return -excel_ln(if a == 1.0 { b } else { a });
    }
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    if large <= GAMMALN_FORM_MAX {
        return extended_sum(
            excel_gammaln(small),
            extended_sum(excel_gammaln(large), -excel_gammaln(a + b)),
        );
    }
    extended_sum(excel_gammaln(small), algdiv(small, large))
}

/// TOMS 708 `algdiv(a, b)` = ln(Gamma(b) / Gamma(a + b)) for a < b, b >= 8, in double arithmetic
/// with Excel's LN for ln b: the best of the variants T-2513 tried for Excel's own (T-2512 owns the
/// exact form).
pub(super) fn algdiv(a: f64, b: f64) -> f64 {
    let h = a / b;
    let w = stirling_del(h / (h + 1.0), 1.0 / (h + 1.0), b);
    let d = b + (a - 0.5);
    let u = d * alnrel_positive(a / b);
    let v = a * (excel_ln(b) - 1.0);
    if u > v {
        (w - v) - u
    } else {
        (w - u) - v
    }
}

/// The Stirling correction TOMS 708 `algdiv` and `bcorr` share, in doubles: c/b times the series
/// in t = (1/b)^2 whose coefficients are scaled by s_n = (1 - x^n)/(1 - x), n = 3, 5, ..., 11.
pub(super) fn stirling_del(c: f64, x: f64, b: f64) -> f64 {
    let [c0, c1, c2, c3, c4, c5] = ALGDIV_C;
    // s_n by TOMS's recurrence.
    let x2 = x * x;
    let s3 = (x + x2) + 1.0;
    let s5 = (x + x2 * s3) + 1.0;
    let s7 = (x + x2 * s5) + 1.0;
    let s9 = (x + x2 * s7) + 1.0;
    let s11 = (x + x2 * s9) + 1.0;
    let t = 1.0 / b;
    let t = t * t;
    let mut w = c5 * s11 * t;
    w = (w + c4 * s9) * t;
    w = (w + c3 * s7) * t;
    w = (w + c2 * s5) * t;
    w = (w + c1 * s3) * t;
    (w + c0) * (c / b)
}

/// TOMS 708 `alnrel(q)` = ln(1 + q) for q > -1: the rational ([`alnrel_positive`]'s) for
/// |q| <= 0.375, else Excel's LN of 1 + q.
pub(super) fn alnrel(q: f64) -> f64 {
    if q.abs() > 0.375 {
        excel_ln(1.0 + q)
    } else {
        alnrel_positive(q)
    }
}

/// TOMS 708 `alnrel(q)` = ln(1 + q) for |q| <= 0.375 (algdiv's a/b is at most 1/8): the (3,3)
/// rational in t = q/(q + 2), in doubles.
fn alnrel_positive(q: f64) -> f64 {
    let [p1, p2, p3] = ALNREL_P;
    let [q1, q2, q3] = ALNREL_Q;
    let t = q / (q + 2.0);
    let t2 = t * t;
    let numerator = ((p3 * t2 + p2) * t2 + p1) * t2 + 1.0;
    let denominator = ((q3 * t2 + q2) * t2 + q1) * t2 + 1.0;
    (2.0 * t) * (numerator / denominator)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "test: a malformed fixture fails the test"
    )]

    use super::*;
    use crate::test::util::{ulps, within_fifteen_digits};

    /// (a, b, x, Excel's BETA.DIST(x,a,b,FALSE)) of `test/fixtures/beta_density_small_shape.fixture`.
    fn probe1_rows() -> Vec<(f64, f64, f64, f64)> {
        let bits = |field: &str| f64::from_bits(u64::from_str_radix(field, 16).unwrap());
        let mut shape = (f64::NAN, f64::NAN);
        let mut rows = Vec::new();
        for line in include_str!("../../test/fixtures/beta_density_small_shape.fixture").lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.as_slice() {
                [] => {}
                ["#", ..] => {}
                ["=", a, b] => shape = (bits(a), bits(b)),
                [x, excel] => rows.push((shape.0, shape.1, bits(x), bits(excel))),
                _ => panic!("malformed fixture line {line:?}"),
            }
        }
        rows
    }

    // T-2503 acceptance: w1-density probe1's 12,800 cells with min(a,b) <= 1 (8 shapes x 1,600
    // x, excel-graph). The research model M2 with Excel's GAMMALN is 12,796 exact and 12,799
    // within 15 digits (verify-r3-m1-residual `vscore_all.out`); none of probe1's shapes takes
    // a GAMMALN argument where the engine's differs from Excel's.
    #[test]
    fn probe1_scores_as_model_m2() {
        let rows = probe1_rows();
        assert_eq!(rows.len(), 12_800);
        let (mut exact, mut fifteen) = (0, 0);
        for &(a, b, x, excel) in &rows {
            let density = beta_density_small_shape(x, a, b).unwrap();
            exact += usize::from(density.to_bits() == excel.to_bits());
            fifteen += usize::from(within_fifteen_digits(excel, density));
        }
        assert!(exact >= 12_796, "{exact} bit-exact");
        assert!(fifteen >= 12_799, "{fifteen} within 15 digits");
    }

    // A shape of 1 makes lnB -LN(the other shape) at any size (statistics.md, 32 of 32 shapes);
    // B(1, 9) is where TOMS algdiv would be 1 ulp low.
    #[test]
    fn a_shape_of_one_takes_minus_ln_of_the_other() {
        for (a, b) in [(1.0, 9.0), (2.0, 1.0), (1.0, 1.1e9), (0.37, 1.0)] {
            let other = if a == 1.0 { b } else { a };
            assert_eq!(ln_beta_small_shape(a, b), -excel_ln(other));
        }
        // The density of (1, 1) is 1 everywhere.
        assert_eq!(beta_density_small_shape(0.3, 1.0, 1.0), Some(1.0));
    }

    // TOMS algdiv against ln Gamma(b) - ln Gamma(a + b) from 50-digit mpmath: within a few ulp.
    #[test]
    fn algdiv_is_the_log_gamma_difference() {
        for (a, b, reference) in [
            // mpmath.loggamma(20) - loggamma(20.5)
            (0.5, 20.0, -1.491_616_787_331_304),
            // mpmath.loggamma(1e6) - loggamma(1e6 + 0.25)
            (0.25, 1e6, -3.453_877_545_741_060_7),
        ] {
            let value = algdiv(a, b);
            assert!(ulps(value, reference) <= 4, "{a} {b}: {value}");
        }
    }

    // The 15-digit test reads the exponent of 1000 as 3 (half a unit is 5E-12 there).
    #[test]
    fn fifteen_digits_at_a_power_of_ten() {
        assert!(within_fifteen_digits(1000.0, 1000.0 + 4e-12));
        assert!(!within_fifteen_digits(1000.0, 1000.0 + 6e-12));
    }

    #[test]
    fn other_shapes_and_x_take_the_callers_path() {
        assert_eq!(beta_density_small_shape(0.5, 1.5, 2.0), None);
        assert_eq!(beta_density_small_shape(0.0, 0.5, 2.0), None);
        assert_eq!(beta_density_small_shape(1.0, 0.5, 2.0), None);
    }
}
