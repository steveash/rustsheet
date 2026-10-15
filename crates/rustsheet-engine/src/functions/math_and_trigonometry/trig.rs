//! Excel's sine, cosine and tangent: argument reduction by the x87 64-bit π.
//!
//! Excel reduces an angle by multiples of `π₆₄/2`, where `π₆₄` is π rounded to a 64-bit
//! significand (the x87 `FLDPI` constant, `0xC90FDAA22168C235·2^-62`), and then returns the
//! correctly rounded sine, cosine or tangent of the remainder. So its SIN(x) is the true
//! sin(x + n·δ/2), with n = round(2x/π) and δ = π − π₆₄ ≈ -5.0166e-20: SIN(PI()) is
//! 1.22514845490862E-16 (not 1.2246467991473532E-16), COS(PI()/2) is 6.1257422745431E-17 and
//! SIN(2^27-1) is -0.9559933287726706. This reproduces all 672 numeric cells of
//! probes-numeric/m9-trig-reduction and probes/trig-huge-argument bit for bit
//! (docs/excel-semantics/numbers.md "Trig of a huge angle"; T-1563).
//!
//! The remainder `x − n·π₆₄/2` is computed exactly as a double-double and its sine and cosine
//! are summed in double-double arithmetic, so the final rounding is the only one.

use std::f64::consts::FRAC_PI_2;

/// `π₆₄/2 − FRAC_PI_2`: the 11 low bits of the 64-bit significand that a double drops
/// (`0x235·2^-63`). It is COS(PI()/2) in Excel.
const X87_HALF_PI_TAIL: f64 = 565.0 / 9_223_372_036_854_775_808.0;

/// Horner steps of the Taylor series: |r| <= π/4, so the 15th term is below 2^-110 of the sum.
const SERIES_TERMS: u32 = 15;

/// An unevaluated sum `hi + lo` with |lo| <= ulp(hi)/2.
#[derive(Clone, Copy)]
struct DoubleDouble {
    hi: f64,
    lo: f64,
}

impl DoubleDouble {
    /// `a + b` exactly, for any `a` and `b` (Knuth's two-sum).
    fn two_sum(a: f64, b: f64) -> Self {
        let hi = a + b;
        let b_virtual = hi - a;
        let lo = (a - (hi - b_virtual)) + (b - b_virtual);
        Self { hi, lo }
    }

    /// `hi + lo` renormalized, for |hi| >= |lo| (or `hi == 0`).
    fn fast_two_sum(hi: f64, lo: f64) -> Self {
        let sum = hi + lo;
        Self {
            hi: sum,
            lo: lo - (sum - hi),
        }
    }

    fn mul(self, other: Self) -> Self {
        let hi = self.hi * other.hi;
        let lo = self.hi.mul_add(other.hi, -hi) + (self.hi * other.lo + self.lo * other.hi);
        Self::fast_two_sum(hi, lo)
    }

    fn div(self, other: Self) -> Self {
        let q1 = self.hi / other.hi;
        // self − q1·other, with q1·other.hi taken exactly.
        let product = q1 * other.hi;
        let product_lo = q1.mul_add(other.hi, -product);
        let remainder = ((self.hi - product) - product_lo) + self.lo - q1 * other.lo;
        Self::fast_two_sum(q1, remainder / other.hi)
    }

    fn div_f64(self, d: f64) -> Self {
        let q1 = self.hi / d;
        let remainder = (-q1).mul_add(d, self.hi) + self.lo;
        Self::fast_two_sum(q1, remainder / d)
    }

    /// `1 − self`.
    fn one_minus(self) -> Self {
        let sum = Self::two_sum(1.0, -self.hi);
        Self::fast_two_sum(sum.hi, sum.lo - self.lo)
    }

    fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }
}

/// Sine and cosine of the remainder `r`, |r| <= π/4, by Horner's rule on the Taylor series:
/// sin r = r(1 − r²/(2·3)(1 − r²/(4·5)(…))) and cos r = 1 − r²/(1·2)(1 − r²/(3·4)(…)).
fn sin_cos_of_remainder(r: DoubleDouble) -> (DoubleDouble, DoubleDouble) {
    let square = r.mul(r);
    let mut sin = DoubleDouble { hi: 1.0, lo: 0.0 };
    let mut cos = DoubleDouble { hi: 1.0, lo: 0.0 };
    for k in (1..=SERIES_TERMS).rev() {
        let k = f64::from(k);
        sin = square
            .mul(sin)
            .div_f64(2.0 * k * (2.0 * k + 1.0))
            .one_minus();
        cos = square
            .mul(cos)
            .div_f64((2.0 * k - 1.0) * 2.0 * k)
            .one_minus();
    }
    (r.mul(sin), cos)
}

/// Sine and cosine of `x` as Excel computes them, before the final rounding.
fn excel_sin_cos_exact(x: f64) -> (DoubleDouble, DoubleDouble) {
    let n = (x / FRAC_PI_2).round();
    // x − n·π₆₄/2 exactly: n·FRAC_PI_2 is split by an FMA, n·X87_HALF_PI_TAIL is exact (n has at
    // most 27 bits, the tail 10), and x − n·FRAC_PI_2 is exact by Sterbenz's lemma.
    let product = n * FRAC_PI_2;
    let product_lo = n.mul_add(FRAC_PI_2, -product);
    let head = DoubleDouble::two_sum(x - product, -product_lo);
    let tail = DoubleDouble::two_sum(head.hi, -(n * X87_HALF_PI_TAIL));
    let remainder = DoubleDouble::fast_two_sum(tail.hi, head.lo + tail.lo);
    let (sin, cos) = sin_cos_of_remainder(remainder);
    // |n| < 2^27 (the `trig_arg` cut), so the cast is exact.
    match (n as i64).rem_euclid(4) {
        0 => (sin, cos),
        1 => (cos, sin.neg()),
        2 => (sin.neg(), cos.neg()),
        _ => (cos.neg(), sin),
    }
}

/// Excel's SIN(x) and COS(x), for |x| < 2^27 (the caller's `trig_arg` cut).
pub(super) fn excel_sin_cos(x: f64) -> (f64, f64) {
    let (sin, cos) = excel_sin_cos_exact(x);
    (sin.hi, cos.hi)
}

/// Excel's TAN(x): the correctly rounded quotient of the unrounded sine and cosine
/// (TAN(PI()/2) is 1.6324552277619072E+16, probes-numeric/m9-trig-reduction C82).
pub(super) fn excel_tan(x: f64) -> f64 {
    let (sin, cos) = excel_sin_cos_exact(x);
    sin.div(cos).hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_excel_bit_for_bit() {
        let pi = std::f64::consts::PI;
        // probes-numeric/m9-trig-reduction A1, B82, C82; probes/trig-huge-argument A1.
        assert_eq!(excel_sin_cos(pi).0, 1.225_148_454_908_62e-16);
        assert_eq!(excel_sin_cos(pi / 2.0).1, 6.125_742_274_543_1e-17);
        assert_eq!(excel_tan(pi / 2.0), 1.632_455_227_761_907_2e16);
        assert_eq!(excel_sin_cos(134_217_727.0).0, -0.955_993_328_772_670_6);
        // Small angles keep the correctly rounded value.
        assert_eq!(excel_sin_cos(0.5).0, 0.5_f64.sin());
        assert_eq!(excel_sin_cos(0.0), (0.0, 1.0));
        assert_eq!(excel_sin_cos(-pi).0, -1.225_148_454_908_62e-16);
    }
}
