//! Excel's own `exp`, an x87 `2^(a*log2 e)` evaluation that is not correctly rounded (T-1778 model
//! V6; docs/excel-semantics/numbers.md "EXP is an x87 2^(a*log2 e) evaluation, not correctly
//! rounded"; reference code `docs/research/t1778/round8/exp_v6.py`).
//!
//! The steps, with RN64 a round to nearest (ties to even) to a 64-bit significand:
//! 1. t = RN64(a * RN64(log2 e)) (FLDL2E, then FMUL in extended precision);
//! 2. n = nint(t), ties to even (FRNDINT), and f = t - n, exact, with |f| <= 1/2;
//! 3. u = RN64(2^|f| - 1) (F2XM1), w = RN64(1 + u), and w = RN64(1/w) when f < 0;
//! 4. EXP = w * 2^n rounded once to a double, with gradual underflow (FSCALE, then the store).
//!
//! It matches 2,139,591 of 2,139,684 EXP cells of the T-1778 `kmyst-erfc-*` probes; the 93 misses
//! are points where 2^|f| - 1 lies within 0.09 ulp of a 64-bit rounding midpoint, where Excel's
//! F2XM1 is faithful but not correctly rounded (T-1940). The worksheet EXP (T-1931) and the ERFC
//! kernel use it.

use crate::arithmetic::{
    round_extended_to_double, round_extended_to_subnormal, round_to_bits, significand_and_exponent,
};

/// RN64(log2 e), the 64-bit significand FLDL2E loads, with the value `LOG2_E_64 * 2^-63`
/// (`round8/exp_v6.py` `_L2E64`; log2 e = 1.442695040888963407359924681001892137..., whose
/// significand `0xB8AA3B295C17F0BB_BE87`... rounds up).
const LOG2_E_64: u128 = 0xB8AA_3B29_5C17_F0BC;

/// ln 2 as a double-double (hi + lo, |lo| <= ulp(hi)/2), from 300-bit mpmath.
pub(crate) const LN_2_HI: f64 = f64::from_bits(0x3FE6_2E42_FEFA_39EF); // 0x1.62e42fefa39efp-1
pub(crate) const LN_2_LO: f64 = f64::from_bits(0x3C7A_BC9E_3B39_803F); // 0x1.abc9e3b39803fp-56

/// |a| below this gives exactly 1: |u| < 2^-64 then, under half a 64-bit ulp of 1, so w = 1.
const TINY_ARGUMENT: f64 = 1.0 / 36_893_488_147_419_103_232.0; // 2^-65

/// |t| beyond this is far outside the double range either way (|t| > 1075 is 0 or infinite), and
/// keeps every shift below in range.
const T_LIMIT: f64 = 2000.0;

/// Excel's EXP(a) under model V6 (module docs). `a` is finite; a NaN gives NaN, and an argument
/// past the double range gives 0 or infinity.
pub(crate) fn excel_exp(a: f64) -> f64 {
    if a.is_nan() {
        return f64::NAN;
    }
    if a.abs() < TINY_ARGUMENT {
        return 1.0;
    }
    if a.abs() * std::f64::consts::LOG2_E > T_LIMIT {
        return if a > 0.0 { f64::INFINITY } else { 0.0 };
    }
    // Step 1: t = T * 2^t_exponent, T a 64-bit integer (a is normal here, |a| >= 2^-65).
    let (a_significand, a_exponent) = significand_and_exponent(a);
    let (t, shift) = round_to_bits(u128::from(a_significand) * LOG2_E_64, 64, false);
    let t_exponent = a_exponent - 63 + shift;
    // Step 2: |t| <= T_LIMIT < 2^11 and |t| >= 2^-66, so `drop` is in 53..=130. The guards above
    // make every `try_from` fallback below unreachable: t < 2^64, n <= 2^11 and |f| < 2^64.
    let Ok(drop) = u32::try_from(-t_exponent) else {
        return f64::NAN;
    };
    let (n, fraction) = if drop >= 65 {
        // |t| < 2^(64 - drop) <= 1/2: n = 0 and f = t.
        (0_i32, i128::try_from(t).unwrap_or(0))
    } else {
        let integer = t >> drop;
        let rest = t & ((1 << drop) - 1);
        let half = 1 << (drop - 1);
        let n = integer + u128::from(rest > half || (rest == half && integer & 1 == 1));
        let f = i128::try_from(t).unwrap_or(0) - i128::try_from(n << drop).unwrap_or(0);
        (i32::try_from(n).unwrap_or(0), f)
    };
    // f * sign(a) is the true fraction: `fraction` is the magnitude-side remainder of |t|.
    let negative_f = (fraction < 0) != (a < 0.0);
    let n = if a < 0.0 { -n } else { n };
    // Step 3: w = W * 2^w_exponent, W a 64-bit integer.
    let (mut w, mut w_exponent) = (1_u128 << 63, -63);
    if fraction != 0 {
        let (u, u_exponent) = two_to_the_minus_one(fraction.unsigned_abs(), drop);
        // u < 0.415, so u_exponent <= -65 and 1 = 2^gap units of u.
        let gap = u32::try_from(-u_exponent).unwrap_or(128);
        if gap < 128 {
            let (sum, sum_shift) = round_to_bits((1 << gap) + u, 64, false);
            (w, w_exponent) = (sum, u_exponent + sum_shift);
        }
        if negative_f {
            // w is in [1, 2), so 2^127 / W is in (2^63, 2^64]: the integer quotient already has
            // the 64 bits, and the remainder rounds it (a tie needs W = 2^63, an exact quotient).
            let numerator = 1_u128 << 127;
            let (quotient, remainder) = (numerator / w, numerator % w);
            let rounded = quotient + u128::from(2 * remainder > w);
            let (q, q_shift) = round_to_bits(rounded, 64, false);
            (w, w_exponent) = (q, -127 - w_exponent + q_shift);
        }
    }
    // Step 4: one rounding of the 64-bit w * 2^n to a double, or to the subnormal grid.
    let exponent = w_exponent + n;
    round_extended_to_double(w, exponent, false).unwrap_or_else(|| {
        if exponent > 0 {
            f64::INFINITY
        } else {
            round_extended_to_subnormal(w, exponent, false).unwrap_or(0.0)
        }
    })
}

/// RN64(2^f - 1) for f = `magnitude` * 2^-`drop` in (0, 1/2]: the 64-bit significand and its
/// exponent. 2^f - 1 = expm1(f ln 2) is summed in double-double arithmetic (about 2^-100
/// relative), far inside the 2^-64 rounding step; it is never a tie, as 2^f is irrational.
fn two_to_the_minus_one(magnitude: u128, drop: u32) -> (u128, i32) {
    // f as a double-double: the integer splits exactly into a rounded head and a small tail.
    let head = magnitude as f64;
    let tail = (i128::try_from(magnitude).unwrap_or(0) - head as i128) as f64;
    let scale = power_of_two(-i32::try_from(drop).unwrap_or(0));
    let f = (head * scale, tail * scale);
    let y = dd_mul(f, (LN_2_HI, LN_2_LO));
    // expm1(y) = y(1 + y/2(1 + y/3(1 + ...))); |y| <= 0.347, so 27 terms reach 2^-120.
    let mut s = (1.0, 0.0);
    for k in (2..=27).rev() {
        s = dd_add_one(dd_div_small(dd_mul(y, s), f64::from(k)));
    }
    dd_to_extended(dd_mul(y, s))
}

/// 2^k as a double for -1022 <= k <= 1023.
pub(crate) fn power_of_two(k: i32) -> f64 {
    let biased = u64::try_from(k + 1023).unwrap_or(1).clamp(1, 2046);
    f64::from_bits(biased << 52)
}

/// The exact sum `a + b` as a rounded head and its error (Knuth's `TwoSum`).
pub(crate) fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let virtual_b = s - a;
    (s, (a - (s - virtual_b)) + (b - virtual_b))
}

/// `a * b` in double-double arithmetic.
pub(crate) fn dd_mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let p = a.0 * b.0;
    let error = a.0.mul_add(b.0, -p) + (a.0 * b.1 + a.1 * b.0);
    two_sum(p, error)
}

/// `1 + a` in double-double arithmetic.
fn dd_add_one(a: (f64, f64)) -> (f64, f64) {
    let (s, e) = two_sum(1.0, a.0);
    two_sum(s, e + a.1)
}

/// `a / k` in double-double arithmetic, for a small integer `k`.
pub(crate) fn dd_div_small(a: (f64, f64), k: f64) -> (f64, f64) {
    let q = a.0 / k;
    // The remainder a - q*k, exactly in its leading part (k*q's error from an FMA).
    let product = q * k;
    let product_error = q.mul_add(k, -product);
    let remainder = ((a.0 - product) - product_error) + a.1;
    two_sum(q, remainder / k)
}

/// RN64 of a positive double-double `hi + lo`: the 64-bit significand and its exponent.
pub(crate) fn dd_to_extended((hi, lo): (f64, f64)) -> (u128, i32) {
    let (significand, exponent) = significand_and_exponent(hi);
    // On the grid 2^(exponent - 24) the head is an integer of 77 bits, and |lo| <= ulp(hi)/2 is at
    // most 2^23 units: its floor is exact, and a nonzero fraction is the sticky bit.
    let scaled = lo * power_of_two(24 - exponent);
    let floor = scaled.floor();
    let units = (i128::from(significand) << 24) + floor as i128;
    let (rounded, shift) = round_to_bits(units.unsigned_abs(), 64, scaled != floor);
    (rounded, exponent - 24 + shift)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_points() {
        assert_eq!(excel_exp(0.0), 1.0);
        assert_eq!(excel_exp(1e-30), 1.0);
        assert_eq!(excel_exp(-1e-30), 1.0);
        assert_eq!(excel_exp(-1000.0), 0.0);
        assert_eq!(excel_exp(1000.0), f64::INFINITY);
        assert!(excel_exp(f64::NAN).is_nan());
    }

    // Excel's own EXP cells (kmyst-erfc-r5expmap1!B1, B21, B61, B25061 and B10288, excel-graph).
    #[test]
    fn excel_cells() {
        assert_eq!(
            excel_exp(-693.144_335_489_521_8),
            9.359_225_999_226_452e-302
        );
        assert_eq!(excel_exp(-17.325_834_443_610_21), 2.988_723_282_343_492e-8);
        assert_eq!(
            excel_exp(693.150_025_630_557_6),
            1.074_561_465_574_963_2e301
        );
        assert_eq!(excel_exp(2.000_755_701_729_514_3), 7.394_642_131_825_609);
        // f < 0 takes the rounded reciprocal: its 64-bit value sits just above a double midpoint.
        assert_eq!(excel_exp(-0.345_024_546_371_140_6), 0.708_202_969_441_512_7);
    }
}
