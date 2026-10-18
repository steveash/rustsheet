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

/// 2^27: [`excel_sin_cos`] and [`excel_tan`] take |x| below it, where SIN and the others are
/// numbers (SIN(2^27) is `#NUM!`; probes/trig-huge-argument).
pub(crate) const EXCEL_TRIG_LIMIT: f64 = 134_217_728.0;

/// Horner divisors are j(j+1): step k of the sine divides by (2k)(2k+1) (even j), of the cosine
/// by (2k-1)(2k) (odd j). Up to j = 28, |r| <= π/4 leaves the next term below 2^-106 of the sum.
const SERIES_DIVISORS: usize = 28;

/// The outer divisors (j <= 10, five steps of each series), whose reciprocals run in
/// double-double. Past them a term is below 2^-27 of the sum, so the inner steps run in plain
/// doubles: their rounding error stays below 2^-80 of the result, far under the double-double
/// error. Multiplying by reciprocals keeps a chain of dependent divisions out of every call
/// (T-1563 performance review).
const DOUBLE_DOUBLE_DIVISORS: usize = 10;

/// 1/(j(j+1)) for j = 1..=10 as exact double-double pairs (`hi` = the quotient rounded, `lo` =
/// the rest rounded; checked by `reciprocal_pairs_are_exact`).
const DOUBLE_DOUBLE_RECIPROCALS: [DoubleDouble; DOUBLE_DOUBLE_DIVISORS] = [
    DoubleDouble::new(0.5, 0.0),
    DoubleDouble::new(0.166_666_666_666_666_66, 9.251_858_538_542_97e-18),
    DoubleDouble::new(0.083_333_333_333_333_33, 4.625_929_269_271_485e-18),
    DoubleDouble::new(0.05, -2.775_557_561_562_891_5e-18),
    DoubleDouble::new(0.033_333_333_333_333_33, 4.625_929_269_271_486e-19),
    DoubleDouble::new(0.023_809_523_809_523_808, 1.321_694_076_934_71e-18),
    DoubleDouble::new(0.017_857_142_857_142_856, 9.912_705_577_010_326e-19),
    DoubleDouble::new(0.013_888_888_888_888_888, 7.709_882_115_452_476e-19),
    DoubleDouble::new(0.011_111_111_111_111_112, -4.240_435_163_498_861_6e-19),
    DoubleDouble::new(0.009_090_909_090_909_09, 4.415_659_757_031_872e-19),
];

/// 1/(j(j+1)) rounded to a double, for j = 11..=28.
const DOUBLE_RECIPROCALS: [f64; SERIES_DIVISORS - DOUBLE_DOUBLE_DIVISORS] = {
    let mut reciprocals = [0.0; SERIES_DIVISORS - DOUBLE_DOUBLE_DIVISORS];
    let (mut index, mut j) = (0, 11.0);
    while index < reciprocals.len() {
        reciprocals[index] = 1.0 / (j * (j + 1.0));
        index += 1;
        j += 1.0;
    }
    reciprocals
};

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

    const fn new(hi: f64, lo: f64) -> Self {
        Self { hi, lo }
    }

    /// `a · b` exactly (no overflow or underflow here).
    fn two_prod(a: f64, b: f64) -> Self {
        let hi = a * b;
        Self {
            hi,
            lo: a.mul_add(b, -hi),
        }
    }

    fn mul(self, other: Self) -> Self {
        let product = Self::two_prod(self.hi, other.hi);
        let lo = product.lo + (self.hi * other.lo + self.lo * other.hi);
        Self::fast_two_sum(product.hi, lo)
    }

    fn div(self, other: Self) -> Self {
        let q1 = self.hi / other.hi;
        // self − q1·other, with q1·other.hi taken exactly.
        let product = Self::two_prod(q1, other.hi);
        let remainder = ((self.hi - product.hi) - product.lo) + self.lo - q1 * other.lo;
        Self::fast_two_sum(q1, remainder / other.hi)
    }

    #[cfg(test)]
    fn div_f64(self, d: f64) -> Self {
        let q1 = self.hi / d;
        let product = Self::two_prod(q1, d);
        let remainder = ((self.hi - product.hi) - product.lo) + self.lo;
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
    (r.mul(series(square, 1)), series(square, 0))
}

/// `1 − r²c₁(1 − r²c₂(1 − …))` with the reciprocals `c` of every other divisor j(j+1), from
/// j = 1 + `skip`: the inner steps in doubles, the outer ones in double-double.
fn series(square: DoubleDouble, skip: usize) -> DoubleDouble {
    let mut tail = 1.0;
    for reciprocal in DOUBLE_RECIPROCALS.iter().skip(skip).step_by(2).rev() {
        tail = 1.0 - square.hi * tail * reciprocal;
    }
    let mut sum = DoubleDouble::new(tail, 0.0);
    for &reciprocal in DOUBLE_DOUBLE_RECIPROCALS.iter().skip(skip).step_by(2).rev() {
        sum = square.mul(sum).mul(reciprocal).one_minus();
    }
    sum
}

/// Sine and cosine of `x` as Excel computes them, before the final rounding.
fn excel_sin_cos_exact(x: f64) -> (DoubleDouble, DoubleDouble) {
    let (n, remainder) = reduce(x);
    let (sin, cos) = sin_cos_of_remainder(remainder);
    by_quadrant(n, sin, cos)
}

/// `n = round(2x/π)` and the remainder `x − n·π₆₄/2` to double-double precision: `n·FRAC_PI_2` is
/// split by an FMA, `n·X87_HALF_PI_TAIL` is exact (n has at most 27 bits, the tail 10), and
/// `x − n·FRAC_PI_2` is exact by Sterbenz's lemma.
fn reduce(x: f64) -> (f64, DoubleDouble) {
    let n = (x / FRAC_PI_2).round();
    let product = DoubleDouble::two_prod(n, FRAC_PI_2);
    let head = DoubleDouble::two_sum(x - product.hi, -product.lo);
    let tail = DoubleDouble::two_sum(head.hi, -(n * X87_HALF_PI_TAIL));
    (n, DoubleDouble::fast_two_sum(tail.hi, head.lo + tail.lo))
}

/// Sine and cosine of `x` from those of its remainder after `n` quarter turns.
fn by_quadrant(n: f64, sin: DoubleDouble, cos: DoubleDouble) -> (DoubleDouble, DoubleDouble) {
    // |n| < 2^27 (the `trig_arg` cut), so the cast is exact.
    match (n as i64).rem_euclid(4) {
        0 => (sin, cos),
        1 => (cos, sin.neg()),
        2 => (sin.neg(), cos.neg()),
        _ => (cos.neg(), sin),
    }
}

/// Excel's SIN(x) and COS(x), for |x| < [`EXCEL_TRIG_LIMIT`].
pub(crate) fn excel_sin_cos(x: f64) -> (f64, f64) {
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

    /// The whole series in double-double: the reference the mixed-precision series must equal.
    fn reference_sin_cos(x: f64) -> (f64, f64) {
        let (n, r) = reduce(x);
        let square = r.mul(r);
        let one = DoubleDouble::new(1.0, 0.0);
        let (mut sin, mut cos) = (one, one);
        for k in (1..=20).rev() {
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
        let (sin, cos) = by_quadrant(n, r.mul(sin), cos);
        (sin.hi, cos.hi)
    }

    #[test]
    fn reciprocal_pairs_are_exact() {
        let one = DoubleDouble::new(1.0, 0.0);
        for (j, pair) in (1..).zip(DOUBLE_DOUBLE_RECIPROCALS) {
            let d = f64::from(j * (j + 1));
            // One divided by d: q = 1/d rounded, and the exact residual 1 − q·d over d.
            let exact = one.div_f64(d);
            assert_eq!((pair.hi, pair.lo), (exact.hi, exact.lo), "1/{d}");
        }
    }

    #[test]
    fn mixed_precision_series_equals_the_full_double_double_series() {
        // A fixed xorshift sweep over angles of every magnitude below the 2^27 cut.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..200_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            let x = (unit * 27.0).exp2() * if state & 1 == 0 { 1.0 } else { -1.0 } - 1.0;
            assert_eq!(excel_sin_cos(x), reference_sin_cos(x), "x = {x:e}");
        }
    }

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
