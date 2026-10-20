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
    exp_with(a, two_to_the_minus_one)
}

/// [`excel_exp`] with step 3's RN64(2^f - 1) taken from `two_to_the_minus_one`, so the tests can
/// run the whole model on the reference series.
fn exp_with(a: f64, two_to_the_minus_one: fn(u128, u32) -> (u128, i32)) -> f64 {
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
        // Integer bit operations, not `||`: a data-dependent branch here mispredicts half the time.
        let n = integer + (u128::from(rest > half) | (u128::from(rest == half) & (integer & 1)));
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
/// exponent (T-1945). A table-driven estimate, good to about 2^-85 relative, gives the rounding
/// unless it lies within [`MIDPOINT_MARGIN`] of a 64-bit rounding midpoint (a Ziv-style test);
/// only then does the full series, [`two_to_the_minus_one_series`], decide. Both return the same
/// bits: the estimate rounds the same way whenever its error cannot reach a midpoint.
fn two_to_the_minus_one(magnitude: u128, drop: u32) -> (u128, i32) {
    let (hi, lo) = two_to_the_minus_one_estimate(magnitude, drop);
    let (significand, exponent) = significand_and_exponent(hi);
    // The 64-bit grid is 2^(exponent - 11) and hi lies on it, so the rounding is hi's significand
    // with 11 more bits plus lo in grid units. A hi that is a power of two can have the rounding
    // in the finer grid below it, so it takes the series.
    if significand != 1 << 52 {
        // lo in half-units of the grid, |z| <= 2^11; midpoints are the odd integers.
        let z = lo * power_of_two(12 - exponent);
        let k = (z - 1.0) * 0.5;
        if (k - nearest_integer(k)).abs() * 2.0 > MIDPOINT_MARGIN {
            // |units| <= 2^10, and 2^63 <= significand * 2^11 + units < 2^64.
            let units = nearest_integer(z * 0.5) as i64;
            let rounded = (u128::from(significand) << 11).wrapping_add_signed(i128::from(units));
            return (rounded, exponent - 11);
        }
    }
    two_to_the_minus_one_series(magnitude, drop)
}

/// The distance from a 64-bit rounding midpoint, in half-units of the 64-bit grid, beyond which
/// the estimate's rounding is trusted: 2^-9 half-units, at least 2^-74 of the value (a grid unit
/// is at least 2^-64 of it). The estimate's error is about 2^-85 (module tests check 2^-80 on a
/// sweep), so the margin is wide; the series runs for about 1 call in 2^9.
const MIDPOINT_MARGIN: f64 = 1.0 / 512.0;

/// 2^(j/64) - 1 for j = 0..=32 as double-doubles (the bits of hi, lo; |lo| <= ulp(hi)/2), from
/// 80-digit `decimal` arithmetic.
const TWO_TO_THE_J_OVER_64_MINUS_ONE: [(u64, u64); 33] = [
    (0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
    (0x3F86_4D1F_3BC0_3077, 0x3C1B_DF2B_293D_E8A7),
    (0x3F96_6C34_C561_5D0F, 0xBC31_83AB_7149_735C),
    (0x3FA0_E8A3_0EB3_7901, 0x3C28_6BE4_BB28_4FF4),
    (0x3FA6_AB0D_9F31_21EC, 0x3C44_C5C9_5B8C_2155),
    (0x3FAC_7D86_5A7A_3440, 0x3C40_3A17_27C5_7B53),
    (0x3FB1_301D_0125_B50A, 0x3C53_AEFC_6BB6_4C63),
    (0x3FB4_29AA_EA92_DDFB, 0x3C4A_080C_A1D9_2C37),
    (0x3FB7_2B83_C7D5_17AE, 0xBC49_041B_9D78_A75B),
    (0x3FBA_35BE_B6FC_B754, 0xBC4A_4B38_4B69_71BE),
    (0x3FBD_4873_168B_9AA8, 0xBC5F_E91F_F5D9_BC3E),
    (0x3FC0_31DC_4314_66B2, 0xBC51_C453_F5AB_DB59),
    (0x3FC1_C3D3_73AB_11C3, 0x3C5B_07EB_6C70_572D),
    (0x3FC3_5A2B_2F13_E6E9, 0x3C55_E99C_CA07_4EC9),
    (0x3FC4_F4EF_A8FE_F709, 0x3C68_4BA2_BEB4_4954),
    (0x3FC6_942D_3720_185A, 0x3BE2_3AA6_DA0E_A709),
    (0x3FC8_37F0_518D_B8A9, 0x3C6B_D1AB_48C6_0B91),
    (0x3FC9_E045_9320_B7FA, 0x3C69_390C_21B2_CD2D),
    (0x3FCB_8D39_B9D5_4E55, 0x3C5C_5154_0BD1_51E6),
    (0x3FCD_3ED9_A72C_FFB7, 0x3C64_3792_533C_143A),
    (0x3FCE_F532_6091_A112, 0xBC64_97DB_B83D_8512),
    (0x3FD0_5828_87DC_B8A8, 0xBC5E_F369_1C30_9278),
    (0x3FD1_3821_8186_24B4, 0x3C48_9B7A_04EF_80D0),
    (0x3FD2_1A8A_D704_F340, 0x3C73_C1A3_B690_62F0),
    (0x3FD2_FF6B_54D8_A89C, 0x3C7D_4397_AFEC_42E2),
    (0x3FD3_E6C9_DA74_B29B, 0xBC72_CC27_4965_5F8C),
    (0x3FD4_D0AD_5A75_3E07, 0x3C7F_0A83_C49D_86A6),
    (0x3FD5_BD1C_DAD4_9F6A, 0xBC79_134F_FB89_B14C),
    (0x3FD6_AC1F_7521_50A5, 0x3C78_C930_1519_1EB3),
    (0x3FD7_9DBC_56B4_8522, 0xBC71_641B_3DFC_668A),
    (0x3FD8_91FA_C0E9_5613, 0xBC6C_1E0B_F205_A4B8),
    (0x3FD9_88E2_0954_8892, 0x3C71_27D9_E29B_8F31),
    (0x3FDA_8279_99FC_EF32, 0x3C70_8B2F_B136_6EA9),
];

/// 1/6 as a double-double.
const ONE_SIXTH: (f64, f64) = (
    f64::from_bits(0x3FC5_5555_5555_5555),
    f64::from_bits(0x3C65_5555_5555_5555),
);

/// 2^f - 1 for f = `magnitude` * 2^-`drop` in (0, 1/2], to about 2^-85 relative, as a positive
/// double-double. f = j/64 + r with |r| <= 1/128, and 2^f - 1 = E + e + E e with E = 2^(j/64) - 1
/// from the table and e = expm1(y), y = r ln 2, |y| <= 0.0055. Mostly plain f64 work: the
/// leading series terms y, y^2/2, y^3/6 and y^4/24 are kept exactly (Dekker products), and the
/// rest, under 2^-37 of e, is summed in f64.
fn two_to_the_minus_one_estimate(magnitude: u128, drop: u32) -> (f64, f64) {
    // drop >= 53 (excel_exp), so both shifts are in range; j = nint(64 f) <= 32, and r is exact
    // with |r| <= 2^(drop - 7) and |r| <= magnitude < 2^64.
    let j = (magnitude + (1 << (drop - 7))) >> (drop - 6);
    let r = i128::try_from(magnitude).unwrap_or(0) - i128::try_from(j << (drop - 6)).unwrap_or(0);
    let (rh, rl) = exact_double_double(r, drop);
    // y = r ln 2: the head product exactly, the cross terms rounded (2^-108).
    let (product, product_error) = two_prod(rh, LN_2_HI);
    let (yh, yl) = fast_two_sum(product, product_error + (rh * LN_2_LO + rl * LN_2_HI));
    // expm1(yh) = yh + p/2 + t + v + tail with p = yh^2, t = yh^3/6, v = yh^4/24, each a
    // double-double; tail = yh^5 (1/120 + yh/720 + ...) to the 1/9! term (the next is 2^-89).
    let (p, p_error) = two_prod(yh, yh);
    let (q, q_error) = two_prod(yh, p);
    let q_error = q_error + yh * p_error;
    let (t, t_error) = two_prod(q, ONE_SIXTH.0);
    let t_error = t_error + q * ONE_SIXTH.1 + q_error * ONE_SIXTH.0;
    // v = t yh/4 (yh/4 is exact).
    let (v, v_error) = two_prod(t, yh * 0.25);
    let v_error = v_error + t_error * yh * 0.25;
    let tail = p
        * q
        * (1.0 / 120.0
            + yh * (1.0 / 720.0
                + yh * (1.0 / 5040.0 + yh * (1.0 / 40_320.0 + yh * (1.0 / 362_880.0)))));
    let (a, a_error) = fast_two_sum(t, v);
    let (a, a_error2) = fast_two_sum(p * 0.5, a);
    let (b, b_error) = fast_two_sum(yh, a);
    // expm1(yh + yl) = expm1(yh) + yl exp(yh), and yl <= 2^-53 yh.
    let lo = b_error
        + a_error2
        + a_error
        + (p_error * 0.5 + t_error + v_error + tail)
        + yl * (1.0 + yh + p * 0.5 + t);
    let (e_hi, e_lo) = TWO_TO_THE_J_OVER_64_MINUS_ONE[usize::try_from(j).unwrap_or(0).min(32)];
    let (e_hi, e_lo) = (f64::from_bits(e_hi), f64::from_bits(e_lo));
    // E e, then E + (e + E e): E >= 0.0109 for j >= 1 and |e| <= 0.0055, so the sum is at
    // least 0.0032 and never cancels by more than a factor 3.4; E = 0 for j = 0.
    let (m, m_error) = two_prod(e_hi, b);
    let m_error = m_error + e_hi * lo + e_lo * b;
    let (s, s_error) = two_sum(b, m);
    let (u, u_error) = two_sum(e_hi, s);
    fast_two_sum(u, u_error + s_error + lo + m_error + e_lo)
}

/// `value` * 2^-`drop` as a double-double, exactly, for |`value`| < 2^64: an arithmetic shift
/// splits it into `high` * 2^11 + `low` with |high| <= 2^53 and 0 <= low < 2^11, both exact
/// doubles (and i64 conversions, which need no runtime call).
fn exact_double_double(value: i128, drop: u32) -> (f64, f64) {
    let scale = power_of_two(-i32::try_from(drop).unwrap_or(0));
    let high = i64::try_from(value >> 11).unwrap_or(0) as f64 * (2048.0 * scale);
    let low = i64::try_from(value & 0x7FF).unwrap_or(0) as f64 * scale;
    two_sum(high, low)
}

/// `x` rounded to the nearest integer, ties to even, for |x| < 2^51: adding and removing
/// 1.5 * 2^52 leaves no fraction bits (and needs no libm `round` call).
fn nearest_integer(x: f64) -> f64 {
    const SHIFTER: f64 = 6_755_399_441_055_744.0; // 1.5 * 2^52
    (x + SHIFTER) - SHIFTER
}

/// RN64(2^f - 1) for f = `magnitude` * 2^-`drop` in (0, 1/2], from the full series: 2^f - 1 =
/// expm1(f ln 2) is summed in double-double arithmetic (about 2^-100 relative), far inside the
/// 2^-64 rounding step; it is never a tie, as 2^f is irrational. The reference the fast path
/// falls back to.
fn two_to_the_minus_one_series(magnitude: u128, drop: u32) -> (u128, i32) {
    dd_to_extended(two_to_the_minus_one_series_sum(magnitude, drop))
}

/// The double-double series sum [`two_to_the_minus_one_series`] rounds.
fn two_to_the_minus_one_series_sum(magnitude: u128, drop: u32) -> (f64, f64) {
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
    dd_mul(y, s)
}

/// 2^k as a double for -1022 <= k <= 1023.
fn power_of_two(k: i32) -> f64 {
    let biased = u64::try_from(k + 1023).unwrap_or(1).clamp(1, 2046);
    f64::from_bits(biased << 52)
}

/// The exact sum `a + b` as a rounded head and its error (Knuth's `TwoSum`).
pub(crate) fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let virtual_b = s - a;
    (s, (a - (s - virtual_b)) + (b - virtual_b))
}

/// `a + b` renormalized, exactly, for |a| >= |b| or a = 0 (Dekker's `FastTwoSum`).
pub(crate) fn fast_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    (s, b - (s - a))
}

/// The exact product `a * b` as a rounded head and its error (Dekker's `TwoProduct` with
/// Veltkamp splitting), for operands well inside the double range. Plain arithmetic, unlike
/// `mul_add`, which without the `fma` target feature is a runtime-dispatched call.
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (a_hi, a_lo) = veltkamp_split(a);
    let (b_hi, b_lo) = veltkamp_split(b);
    (
        p,
        ((a_hi * b_hi - p) + a_hi * b_lo + a_lo * b_hi) + a_lo * b_lo,
    )
}

/// `a` as two halves of at most 26 significant bits each, so their products are exact.
fn veltkamp_split(a: f64) -> (f64, f64) {
    let c = 134_217_729.0 * a; // 2^27 + 1
    let hi = c - (c - a);
    (hi, a - hi)
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

/// `a + b` in double-double arithmetic.
pub(crate) fn dd_add(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let (s, e) = two_sum(a.0, b.0);
    two_sum(s, e + a.1 + b.1)
}

/// `a / b` in double-double arithmetic: a quotient, then one correction from the remainder.
pub(crate) fn dd_div(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let q = a.0 / b.0;
    let product = dd_mul((q, 0.0), b);
    let remainder = dd_add(a, (-product.0, -product.1));
    two_sum(q, remainder.0 / b.0)
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

    fn excel_exp_series(a: f64) -> f64 {
        exp_with(a, two_to_the_minus_one_series)
    }

    #[test]
    fn exact_points() {
        assert_eq!(excel_exp(0.0), 1.0);
        assert_eq!(excel_exp(1e-30), 1.0);
        assert_eq!(excel_exp(-1e-30), 1.0);
        assert_eq!(excel_exp(-1000.0), 0.0);
        assert_eq!(excel_exp(1000.0), f64::INFINITY);
        assert!(excel_exp(f64::NAN).is_nan());
    }

    /// A deterministic xorshift64* stream, so the sweep is reproducible.
    fn next(state: &mut u64) -> u64 {
        *state ^= *state >> 12;
        *state ^= *state << 25;
        *state ^= *state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    // The fast path returns the series' bits for every f = m * 2^-drop excel_exp can produce:
    // drop in 53..=130, m up to 64 bits and at most 2^(drop - 1), plus each table node j/64 and
    // its neighbours, where r is 0 or one unit.
    #[test]
    fn fast_path_matches_series() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut checked = 0;
        for drop in 53..=130_u32 {
            let limit = (1_u128 << (drop - 1)).min(u128::from(u64::MAX));
            let mut magnitudes: Vec<u128> = (0..4_000)
                .map(|i| {
                    // Random bit lengths, so tiny f (j = 0, r tiny) is as common as large f.
                    let bits = 1 + u32::try_from(next(&mut state) % 64).unwrap_or(0);
                    let m = u128::from(next(&mut state)) >> (64 - bits.min(64));
                    if i % 2 == 0 {
                        m.min(limit)
                    } else {
                        u128::from(next(&mut state)) % limit + 1
                    }
                })
                .collect();
            for j in 0..=32_u128 {
                let node = j << (drop - 6);
                magnitudes.extend(
                    [node.saturating_sub(1), node, node + 1]
                        .into_iter()
                        .filter(|&m| m <= limit),
                );
            }
            for m in magnitudes.into_iter().filter(|&m| m > 0) {
                assert_eq!(
                    two_to_the_minus_one(m, drop),
                    two_to_the_minus_one_series(m, drop),
                    "m = {m:#x}, drop = {drop}"
                );
                checked += 1;
            }
        }
        assert!(checked > 300_000);
    }

    // The estimate is within 2^-80 of the series sum (itself good to about 2^-100), far inside
    // the 2^-74 that MIDPOINT_MARGIN allows for.
    #[test]
    fn estimate_error_bound() {
        let mut state = 0x6A09_E667_F3BC_C908_u64;
        for drop in 53..=130_u32 {
            let limit = (1_u128 << (drop - 1)).min(u128::from(u64::MAX));
            for _ in 0..2_000 {
                let bits = 1 + u32::try_from(next(&mut state) % 64).unwrap_or(0);
                let m = (u128::from(next(&mut state)) >> (64 - bits)).clamp(1, limit);
                let (hi, lo) = two_to_the_minus_one_estimate(m, drop);
                let (series_hi, series_lo) = two_to_the_minus_one_series_sum(m, drop);
                let error = ((hi - series_hi) + (lo - series_lo)).abs();
                assert!(
                    error <= series_hi * 2.0_f64.powi(-80),
                    "m = {m:#x}, drop = {drop}"
                );
            }
        }
    }

    // The same through excel_exp over a wide range of a: random doubles in [-750, 750] and
    // random significands at every binade from 2^-65 up.
    #[test]
    fn excel_exp_fast_path_matches_series() {
        let mut state = 0xD1B5_4A32_D192_ED03_u64;
        for _ in 0..200_000 {
            let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
            let wide = (unit - 0.5) * 1500.0;
            let binade =
                f64::from_bits(((next(&mut state) % 75 + 958) << 52) | (next(&mut state) >> 12));
            for a in [wide, binade, -binade] {
                assert_eq!(
                    excel_exp(a).to_bits(),
                    excel_exp_series(a).to_bits(),
                    "a = {a:e}"
                );
            }
        }
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

    /// The per-call microbenchmark of T-1945's Outcome (not a check; run it with
    /// `cargo test --release -p rustsheet_engine --lib x87_exp::tests::per_call_cost -- --ignored
    /// --nocapture`): 200,000 arguments in [-30, 30], the model against the series path and libm.
    #[test]
    #[ignore = "a timing, not a check"]
    fn per_call_cost() {
        let arguments: Vec<f64> = (0..200_000)
            .map(|i| (f64::from(i) * 0.000_731_7).sin() * 30.0)
            .collect();
        let time = |exp: &dyn Fn(f64) -> f64| {
            let start = std::time::Instant::now();
            let sum: f64 = arguments
                .iter()
                .map(|&a| exp(std::hint::black_box(a)))
                .sum();
            (start.elapsed() / 200_000, sum)
        };
        let (fast, fast_sum) = time(&excel_exp);
        let (series, series_sum) = time(&excel_exp_series);
        let (libm, _) = time(&f64::exp);
        assert_eq!(fast_sum.to_bits(), series_sum.to_bits());
        println!("excel_exp {fast:?}, series path {series:?}, libm {libm:?} per call");
    }
}
