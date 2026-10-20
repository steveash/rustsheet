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
//!
//! A cheaper sum runs first: fewer double-double steps, with an error bound proved below
//! ([`Series::fast`]). When every value within that bound rounds to the same double (Ziv's
//! rounding test, [`rounds_safely`]), that double is the rounding of the full sum too, so the full
//! sum runs only for a remainder close to a rounding boundary (about 1 call in 500; T-1637).

use std::f64::consts::FRAC_PI_2;
use std::ops::Neg;

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

/// Horner steps of each series: every other divisor up to [`SERIES_DIVISORS`].
const SERIES_STEPS: usize = SERIES_DIVISORS / 2;

/// Horner steps of each series that run in double-double in the full sum.
const DOUBLE_DOUBLE_STEPS: usize = DOUBLE_DOUBLE_DIVISORS / 2;

/// Horner steps of each series in the fast sum: up to j = 20 for the sine and j = 19 for the
/// cosine, where the next term is below 2^-57 of the double tail it would join.
const FAST_STEPS: usize = 10;

/// The reciprocals of one series' Horner divisors, outermost step first.
struct Series {
    /// The exact double-double reciprocals of the outer steps.
    double_double: [DoubleDouble; DOUBLE_DOUBLE_STEPS],
    /// Every step's reciprocal 1/(j(j+1)) rounded to a double.
    double: [f64; SERIES_STEPS],
    /// The outer steps that the fast sum runs in double-double.
    fast_double_double_steps: usize,
    /// A bound on the fast sum's relative error, with a margin (see [`Series::fast`]).
    fast_error: f64,
}

/// sin r = r(1 − r²/(2·3)(1 − r²/(4·5)(…))): the divisors j(j+1) of even j.
const SINE: Series = Series::new(1, 2, 1.084_202_172_485_504_4e-19);

/// cos r = 1 − r²/(1·2)(1 − r²/(3·4)(…)): the divisors j(j+1) of odd j.
const COSINE: Series = Series::new(0, 3, 1.355_252_715_606_880_5e-20);

/// An unevaluated sum `hi + lo` with |lo| <= ulp(hi)/2.
#[derive(Clone, Copy)]
struct DoubleDouble {
    hi: f64,
    lo: f64,
}

impl DoubleDouble {
    /// `1 − a·b`, for |a·b| <= 1: the product and the difference with an error below 2^-104.
    fn one_minus_product(a: Self, b: Self) -> Self {
        let product = Self::two_prod(a.hi, b.hi);
        let lo = product.lo + (a.hi * b.lo + a.lo * b.hi);
        let difference = Self::fast_two_sum(1.0, -product.hi);
        Self::fast_two_sum(difference.hi, difference.lo - lo)
    }

    /// `a + b` exactly, for any `a` and `b` (Knuth's two-sum).
    fn two_sum(a: f64, b: f64) -> Self {
        let (hi, lo) = crate::x87_exp::two_sum(a, b);
        Self { hi, lo }
    }

    /// `hi + lo` renormalized, for |hi| >= |lo| (or `hi == 0`).
    fn fast_two_sum(hi: f64, lo: f64) -> Self {
        let (hi, lo) = crate::x87_exp::fast_two_sum(hi, lo);
        Self { hi, lo }
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
}

impl Neg for DoubleDouble {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }
}

impl Series {
    /// The series whose divisors are j(j+1) for j = 1 + `skip`, 3 + `skip`, ….
    const fn new(skip: usize, fast_double_double_steps: usize, fast_error: f64) -> Self {
        let mut double_double = [DoubleDouble::new(0.0, 0.0); DOUBLE_DOUBLE_STEPS];
        let mut double = [0.0; SERIES_STEPS];
        let (mut step, mut j) = (0, if skip == 0 { 1.0 } else { 2.0 });
        while step < SERIES_STEPS {
            if step < DOUBLE_DOUBLE_STEPS {
                double_double[step] = DOUBLE_DOUBLE_RECIPROCALS[2 * step + skip];
            }
            double[step] = 1.0 / (j * (j + 1.0));
            step += 1;
            j += 2.0;
        }
        Self {
            double_double,
            double,
            fast_double_double_steps,
            fast_error,
        }
    }

    /// `1 − r²c₁(1 − r²c₂(1 − …))` for the square `r²` of a remainder |r| <= π/4: the inner steps
    /// in doubles, the outer ones in double-double. Its error is below 2^-80.
    fn full(&self, square: DoubleDouble) -> DoubleDouble {
        let mut tail = 1.0;
        for reciprocal in self.double[DOUBLE_DOUBLE_STEPS..].iter().rev() {
            tail = 1.0 - square.hi * tail * reciprocal;
        }
        let mut sum = DoubleDouble::new(tail, 0.0);
        for &reciprocal in self.double_double.iter().rev() {
            sum = square.mul(sum).mul(reciprocal).one_minus();
        }
        sum
    }

    /// The same sum with only the outer `k` steps in double-double, k = `fast_double_double_steps`,
    /// and its relative error below `fast_error`/3.
    ///
    /// With s = r² <= (π/4)² and aᵢ = s·cᵢ, the sum is 1 − a₁(1 − … aₖ(1 − t)) where
    /// t = a₍ₖ₊₁₎·T takes the remaining steps in doubles. T is within 1.1·2^-53 of its exact value
    /// (each step rounds once, inner errors shrink by aᵢ < 0.012 and the dropped term is below
    /// 2^-57), and t gains at most 4 more roundings (dropping `square.lo`, two products, the
    /// reciprocal), so |Δt| <= 5.1·2^-53·a₍ₖ₊₁₎. `1 − t` is kept exact as a double-double, so Δt
    /// reaches the sum scaled by a₁⋯aₖ, and the double-double steps add below 2^-100:
    /// - sine, k = 2: a₁a₂a₃ <= 0.1028·0.03085·0.01469 = 4.66e-5, error <= 2^-65.0 on a sum
    ///   >= 0.897, and the product by r adds 2^-104: under 2^-64.8 = `fast_error`/3.5.
    /// - cosine, k = 3: a₁⋯a₄ <= 0.3085·0.05141·0.02056·0.01102 = 3.59e-6, error <= 2^-68.7 on
    ///   a sum >= 0.707: under 2^-68.2 = `fast_error`/4.6.
    ///
    /// A sweep measures 2^-65.8 and 2^-69.1 (`fast_sums_stay_well_inside_their_error_bounds`).
    // Inlined into TAN, which runs both series, so their steps interleave (148 → 127 ns there).
    #[inline(always)]
    fn fast(&self, square: DoubleDouble) -> DoubleDouble {
        let steps = self.fast_double_double_steps;
        let mut tail = 1.0;
        for reciprocal in self.double[steps + 1..FAST_STEPS].iter().rev() {
            tail = 1.0 - square.hi * tail * reciprocal;
        }
        let mut sum = DoubleDouble::fast_two_sum(1.0, -(square.hi * tail * self.double[steps]));
        for &reciprocal in self.double_double[..steps].iter().rev() {
            // square·cᵢ does not wait for the inner steps, so it overlaps them.
            sum = DoubleDouble::one_minus_product(square.mul(reciprocal), sum);
        }
        sum
    }

    /// This series of the remainder `r` (with the factor `r` for the sine), rounded once.
    fn rounded(&self, r: DoubleDouble, square: DoubleDouble, times_r: bool) -> f64 {
        let scale = |sum: DoubleDouble| if times_r { r.mul(sum) } else { sum };
        let fast = scale(self.fast(square));
        if rounds_safely(fast, self.fast_error) {
            fast.hi
        } else {
            scale(self.full(square)).hi
        }
    }
}

/// Ziv's rounding test: true when every number within `relative_error`·|v| of `v` rounds to
/// `v.hi`. Rounding is monotonic, so checking both ends of the interval suffices; `lo ± error`
/// rounds by at most 2^-53 of itself, far inside the margin of every `fast_error`.
fn rounds_safely(v: DoubleDouble, relative_error: f64) -> bool {
    let error = relative_error * v.hi.abs();
    v.hi + (v.lo + error) == v.hi && v.hi + (v.lo - error) == v.hi
}

/// sin(r + n·π/2) rounded, from the remainder `r` after `n` quarter turns and its `square`.
fn rounded_sine(n: i64, r: DoubleDouble, square: DoubleDouble) -> f64 {
    match n.rem_euclid(4) {
        0 => SINE.rounded(r, square, true),
        1 => COSINE.rounded(r, square, false),
        2 => -SINE.rounded(r, square, true),
        _ => -COSINE.rounded(r, square, false),
    }
}

/// Sine and cosine of `x` as Excel computes them, before the final rounding.
fn excel_sin_cos_exact(x: f64) -> (DoubleDouble, DoubleDouble) {
    let (n, r) = reduce(x);
    let square = r.mul(r);
    by_quadrant(n, r.mul(SINE.full(square)), COSINE.full(square))
}

/// `n = round(2x/π)` and the remainder `x − n·π₆₄/2` to double-double precision: `n·FRAC_PI_2` is
/// split by an FMA, `n·X87_HALF_PI_TAIL` is exact (n has at most 27 bits, the tail 10), and
/// `x − n·FRAC_PI_2` is exact by Sterbenz's lemma.
fn reduce(x: f64) -> (i64, DoubleDouble) {
    let n = (x / FRAC_PI_2).round();
    let product = DoubleDouble::two_prod(n, FRAC_PI_2);
    let head = DoubleDouble::two_sum(x - product.hi, -product.lo);
    let tail = DoubleDouble::two_sum(head.hi, -(n * X87_HALF_PI_TAIL));
    // |n| < 2^27 (the `trig_arg` cut), so the cast is exact.
    (
        n as i64,
        DoubleDouble::fast_two_sum(tail.hi, head.lo + tail.lo),
    )
}

/// Sine and cosine of `x` from those of its remainder after `n` quarter turns.
fn by_quadrant<T: Neg<Output = T>>(n: i64, sin: T, cos: T) -> (T, T) {
    match n.rem_euclid(4) {
        0 => (sin, cos),
        1 => (cos, -sin),
        2 => (-sin, -cos),
        _ => (-cos, sin),
    }
}

/// Excel's SIN(x), for |x| < [`EXCEL_TRIG_LIMIT`].
pub(super) fn excel_sin(x: f64) -> f64 {
    let (n, r) = reduce(x);
    rounded_sine(n, r, r.mul(r))
}

/// Excel's COS(x), for |x| < [`EXCEL_TRIG_LIMIT`]: cos(r + n·π/2) = sin(r + (n + 1)·π/2).
pub(super) fn excel_cos(x: f64) -> f64 {
    let (n, r) = reduce(x);
    rounded_sine(n + 1, r, r.mul(r))
}

/// Excel's SIN(x) and COS(x), for |x| < [`EXCEL_TRIG_LIMIT`].
pub(crate) fn excel_sin_cos(x: f64) -> (f64, f64) {
    let (n, r) = reduce(x);
    let square = r.mul(r);
    (rounded_sine(n, r, square), rounded_sine(n + 1, r, square))
}

/// Excel's TAN(x): the correctly rounded quotient of the unrounded sine and cosine
/// (TAN(PI()/2) is 1.6324552277619072E+16, probes-numeric/m9-trig-reduction C82). The quotient
/// of the fast sums is off by at most the sum of their errors (the division itself adds below
/// 2^-100, inside their margins).
pub(super) fn excel_tan(x: f64) -> f64 {
    let (n, r) = reduce(x);
    let square = r.mul(r);
    let (sin, cos) = by_quadrant(n, r.mul(SINE.fast(square)), COSINE.fast(square));
    let tan = sin.div(cos);
    if rounds_safely(tan, SINE.fast_error + COSINE.fast_error) {
        tan.hi
    } else {
        let (sin, cos) = excel_sin_cos_exact(x);
        sin.div(cos).hi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole series in double-double, unrounded: the reference the mixed-precision series and
    /// the fast sums must round like.
    fn reference_sin_cos(x: f64) -> (DoubleDouble, DoubleDouble) {
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
        by_quadrant(n, r.mul(sin), cos)
    }

    /// A fixed xorshift sweep of `count` numbers in [0, 1).
    fn units(count: usize) -> impl Iterator<Item = f64> {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..count).map(move |_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        })
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
        // Angles of every magnitude below the 2^27 cut, of both signs. About 1 in 500 results
        // fails the rounding test and takes the full sum, so both paths are checked.
        for (index, unit) in units(400_000).enumerate() {
            let sign = if index % 2 == 0 { 1.0 } else { -1.0 };
            let x = (unit * 27.0).exp2() * sign - 1.0;
            let (sin, cos) = reference_sin_cos(x);
            let expected = (sin.hi, cos.hi);
            assert_eq!(excel_sin_cos(x), expected, "x = {x:e}");
            assert_eq!((excel_sin(x), excel_cos(x)), expected, "x = {x:e}");
            assert_eq!(
                excel_tan(x).to_bits(),
                sin.div(cos).hi.to_bits(),
                "TAN({x:e})"
            );
        }
    }

    #[test]
    fn fast_sums_stay_well_inside_their_error_bounds() {
        // Remainders across the whole reduced range |r| <= π/4 (and slightly past it). The proof
        // in `Series::fast` leaves a margin of 3.5 or more; a change that eats it fails here.
        for unit in units(200_000) {
            let r = DoubleDouble::new((2.0 * unit - 1.0) * 0.786, 0.0);
            let square = r.mul(r);
            for (series, factor) in [(&SINE, r), (&COSINE, DoubleDouble::new(1.0, 0.0))] {
                let (fast, full) = (
                    factor.mul(series.fast(square)),
                    factor.mul(series.full(square)),
                );
                let error = ((fast.hi - full.hi) + (fast.lo - full.lo)) / full.hi;
                assert!(error.abs() <= series.fast_error / 4.0, "r = {:e}", r.hi);
            }
        }
    }

    #[test]
    fn error_bounds_are_powers_of_two() {
        assert_eq!(SINE.fast_error, 2f64.powi(-63));
        assert_eq!(COSINE.fast_error, 2f64.powi(-66));
    }

    #[test]
    fn rounding_test_rejects_values_near_a_rounding_boundary() {
        let half_ulp = f64::EPSILON / 4.0; // of 0.75
        assert!(rounds_safely(DoubleDouble::new(0.75, 0.0), 2f64.powi(-63)));
        assert!(rounds_safely(
            DoubleDouble::new(0.75, half_ulp * 0.99),
            2f64.powi(-63)
        ));
        assert!(!rounds_safely(
            DoubleDouble::new(0.75, half_ulp * (1.0 - 2f64.powi(-12))),
            2f64.powi(-63)
        ));
        assert!(!rounds_safely(
            DoubleDouble::new(-0.75, -half_ulp * (1.0 - 2f64.powi(-12))),
            2f64.powi(-63)
        ));
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
        // Tiny and subnormal angles: r² underflows and the sums are exactly 1.
        for x in [1e-200, -5e-324, f64::MIN_POSITIVE] {
            assert_eq!((excel_sin(x), excel_cos(x), excel_tan(x)), (x, 1.0, x));
        }
    }
}
