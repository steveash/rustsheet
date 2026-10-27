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
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;

/// RN64(log2 e), the 64-bit significand FLDL2E loads, with the value `LOG2_E_64 * 2^-63`
/// (`round8/exp_v6.py` `_L2E64`; log2 e = 1.442695040888963407359924681001892137..., whose
/// significand `0xB8AA3B295C17F0BB_BE87`... rounds up).
const LOG2_E_64: u64 = 0xB8AA_3B29_5C17_F0BC;

/// ln 2 as a double-double (hi + lo, |lo| <= ulp(hi)/2), from 300-bit mpmath.
pub(crate) const LN_2_HI: f64 = f64::from_bits(0x3FE6_2E42_FEFA_39EF); // 0x1.62e42fefa39efp-1
pub(crate) const LN_2_LO: f64 = f64::from_bits(0x3C7A_BC9E_3B39_803F); // 0x1.abc9e3b39803fp-56

/// |t| beyond this is far outside the double range either way (|t| > 1075 is 0 or infinite), and
/// keeps every shift below in range.
const T_LIMIT: f64 = 2000.0;

/// Excel's EXP(a) under model V6 (module docs). A NaN gives NaN, and an argument past the
/// double range (an infinity included) gives 0 or infinity. It is [`excel_exp_extended`] of the
/// double loaded into a register, so the two agree on every double.
pub(crate) fn excel_exp(a: f64) -> f64 {
    if a.is_nan() {
        return f64::NAN;
    }
    if a.is_infinite() {
        return if a > 0.0 { f64::INFINITY } else { 0.0 };
    }
    excel_exp_extended(Register::load(a))
}

/// Excel's EXP of an argument held in an x87 register (a 64-bit significand, never stored to a
/// double), under model V6 (module docs): step 1 multiplies the register's own significand by
/// RN64(log2 e). Excel applies EXP to an unstored exponent in its Loader pmf and its beta density
/// (docs/excel-semantics/statistics.md "BINOM.DIST pmf: closed forms at k = 0 and k = n, and one
/// kernel with BETA.DIST", T-2513; the mpmath reference is `docs/research/t2464/w1-density/lib.py`
/// `exp_v6`, which this matches on the 5,200 random extended arguments of
/// `test/fixtures/exp_extended.fixture`, T-2645). An argument past the double range gives 0 or
/// infinity.
pub(crate) fn excel_exp_extended(a: Register) -> f64 {
    exp_with::<true>(a, two_to_the_minus_one)
}

/// Excel's expm1, under SINH, TANH and PPMT: Kahan's `(u - 1) * x / ln(u)` with `u` Excel's own
/// EXP and the log Excel's LN, each step a double (T-2240, docs/excel-semantics/numbers.md
/// "Excel's expm1 is Kahan's formula"). It fits all 1015 SINH and TANH cells of probes-numeric
/// t2178-hyperbolic-unit-sweep, where libm's expm1 fits 773 and 781. `u == 1` (|x| tiny) gives `x`.
pub(crate) fn excel_expm1(x: f64) -> f64 {
    let u = excel_exp(x);
    if u == 1.0 {
        return x;
    }
    let u_minus_1 = u - 1.0;
    if u_minus_1 == -1.0 {
        return -1.0;
    }
    u_minus_1 * x / excel_ln(u)
}

/// [`excel_exp_extended`] with step 3's RN64(2^f - 1) taken from `two_to_the_minus_one`, so the
/// tests can run the whole model on the reference series. Generic, not a function pointer, so the
/// fast path's estimate is a direct (inlinable) call. With `SHORTCUT`, steps 3 and 4 first try
/// [`two_to_the_fraction`], which skips them when their roundings cannot change the double.
///
/// The widths are fixed by the model, so the integer work is mostly 64-bit (T-2102): the argument's
/// significand and RN64(log2 e) are 64-bit, so step 1 is one 64 x 64 multiply; t, the fraction
/// and w all fit 64 bits. The reciprocal for f < 0 and step 4 use the floating-point unit where it
/// is exact (an estimate corrected by an exact remainder, and a correctly rounded u64 to f64
/// conversion), instead of a u128 division and a general rounding. `tests::exp_with_reference`
/// keeps the earlier u128 core, and `integer_core_matches_reference` checks they agree.
fn exp_with<const SHORTCUT: bool>(
    a: Register,
    two_to_the_minus_one: impl Fn(u128, u32) -> (u128, i32),
) -> f64 {
    let (negative, a_significand, a_exponent) = a.parts();
    // A register's significand has at most 64 bits; a zero (no top bit) gives 1.
    let Ok(a_significand) = u64::try_from(a_significand) else {
        return f64::NAN;
    };
    let Some(top_bit) = a_significand.checked_ilog2() else {
        return 1.0;
    };
    // 2^top <= |a| < 2^(top + 1); a bit index of a u64 fits an i32.
    let top = a_exponent + i32::try_from(top_bit).unwrap_or(0);
    // |a| < 2^-65 gives exactly 1: |u| < 2^-64 then, under half a 64-bit ulp of 1, so w = 1.
    if top < -65 {
        return 1.0;
    }
    // |a| * log2 e beyond T_LIMIT: certain from the exponent outside [1024, 2048), and for a
    // double the same f64 test as before on the stored value (exact for a loaded double).
    if top >= 11 || (top == 10 && a.store().abs() * std::f64::consts::LOG2_E > T_LIMIT) {
        return if negative { 0.0 } else { f64::INFINITY };
    }
    // Step 1: t = T * 2^t_exponent, T a 64-bit integer. The product of two 64-bit integers, the
    // constant at least 2^63, has 64 to 128 bits.
    let (t, shift) = round_to_64_bits(u128::from(a_significand) * u128::from(LOG2_E_64));
    let t_exponent = a_exponent - 63 + shift;
    // Step 2: |t| <= T_LIMIT < 2^11 and |t| >= 2^-66, so `drop` is in 53..=130. f is held as its
    // magnitude and whether it is below n, so it stays a u64: |f| <= 2^(drop - 1) <= 2^63.
    let Ok(drop) = u32::try_from(-t_exponent) else {
        return f64::NAN;
    };
    let (n, fraction, below_n) = if drop >= 65 {
        // |t| < 2^(64 - drop) <= 1/2: n = 0 and f = t.
        (0_i32, t, false)
    } else {
        // drop <= 64: shift in u128 so drop = 64 needs no special case.
        let t = u128::from(t);
        let integer = t >> drop;
        let rest = t & ((1 << drop) - 1);
        let half = 1 << (drop - 1);
        // Integer bit operations, not `||`: a data-dependent branch here mispredicts half the time.
        let up = u128::from(rest > half) | (u128::from(rest == half) & (integer & 1));
        // Rounding up leaves f = rest - 2^drop, negative, of magnitude 2^drop - rest.
        let magnitude = if up == 1 { (1 << drop) - rest } else { rest };
        let n = i32::try_from(integer + up).unwrap_or(0);
        (n, u64::try_from(magnitude).unwrap_or(0), up == 1)
    };
    // f * sign(a) is the true fraction: `fraction` is the magnitude-side remainder of |t|.
    let negative_f = below_n != negative;
    let n = if negative { -n } else { n };
    // 2^f * 2^n with 2^f in [0.7, 1.42] is a normal double for -1021 <= n <= 1023.
    if SHORTCUT && fraction != 0 && (-1021..=1023).contains(&n) {
        if let Some(power) = two_to_the_fraction(fraction, drop, negative_f) {
            return power * power_of_two(n);
        }
    }
    // Step 3: w = W * 2^w_exponent, W a 64-bit integer.
    let (mut w, mut w_exponent) = (1_u64 << 63, -63);
    if fraction != 0 {
        let (u, u_exponent) = two_to_the_minus_one(u128::from(fraction), drop);
        // u < 0.415 is 64-bit with u_exponent <= -65; 1 + u in [1, 2) is 2^63 + u / 2^(gap - 63)
        // units of 2^-63, rounded (a carry to 2 cannot happen). gap >= 128 leaves w = 1.
        let gap = u32::try_from(-u_exponent).unwrap_or(128);
        if gap < 128 {
            w = (1 << 63) + round_shifted(u, gap - 63);
        }
        if negative_f {
            (w, w_exponent) = reciprocal(w);
        }
    }
    // Step 4: one rounding of the 64-bit w * 2^n to a double, or to the subnormal grid.
    let exponent = w_exponent + n;
    if (-1085..=960).contains(&exponent) {
        // w * 2^exponent is at least 2^-1022, so the double is normal (or infinite): the u64 to
        // f64 conversion is the one rounding (Rust rounds it to nearest, ties to even), and the
        // scalings by powers of two are exact. A rounding up to 2^64 at the top overflows to
        // infinity, as the rounding to a double does.
        return w as f64 * power_of_two(-63) * power_of_two(exponent + 63);
    }
    let w = u128::from(w);
    round_extended_to_double(w, exponent, false).unwrap_or_else(|| {
        if exponent > 0 {
            f64::INFINITY
        } else {
            round_extended_to_subnormal(w, exponent, false).unwrap_or(0.0)
        }
    })
}

/// Steps 3 and 4's double, RN53(w), for f = ±`magnitude` * 2^-`drop` in [-1/2, 1/2] \ {0}
/// (`negative` gives the sign), when it is certain without w (T-2102). w is 2^f after
/// three RN64 roundings (2^|f| - 1, 1 + u and, for f < 0, the reciprocal), so |w - 2^f| <= 2^-63
/// 2^f; a 2^f more than that from every double rounding midpoint rounds to the same double as w.
/// A double-double estimate of 2^f, good to about 2^-67, decides that unless it lies within
/// [`DOUBLE_MIDPOINT_MARGIN`] of a midpoint, or its head is 1 (a power of two, with the finer
/// midpoints below it); then `None`, and the exact chain runs (about 1 call in 30).
#[inline]
fn two_to_the_fraction(magnitude: u64, drop: u32, negative: bool) -> Option<f64> {
    let (high, low) = two_to_the_fraction_estimate(u128::from(magnitude), drop);
    let (head, tail) = if negative {
        reciprocal_near_one(high, low)
    } else {
        (high, low)
    };
    if head == 1.0 {
        return None;
    }
    // Half an ulp of the head, which is in [0.7, 1.42]; |tail| is at most that.
    let half_ulp = if head > 1.0 {
        power_of_two(-53)
    } else {
        power_of_two(-54)
    };
    (half_ulp - tail.abs() > DOUBLE_MIDPOINT_MARGIN).then_some(head)
}

/// 1 / (`high` + `low`) as a double-double, for a value in [1, 1.42]: a quotient and one
/// correction from its exact residual (1 - q high is exact by Sterbenz, as q high is within an
/// ulp of 1).
#[inline]
fn reciprocal_near_one(high: f64, low: f64) -> (f64, f64) {
    let quotient = 1.0 / high;
    let (product, product_error) = two_prod(quotient, high);
    let residual = ((1.0 - product) - product_error) - quotient * low;
    fast_two_sum(quotient, quotient * residual)
}

/// How far [`two_to_the_fraction`]'s estimate must be from a double rounding midpoint: 2^-60,
/// above the estimate's error (about 2^-67) plus w's (at most 2^-63 * 1.42).
const DOUBLE_MIDPOINT_MARGIN: f64 = 1.0 / 1_152_921_504_606_846_976.0; // 2^-60

/// 2^f for f = `magnitude` * 2^-`drop` in (0, 1/2], to about 2^-67, as a double-double: the
/// [`two_to_the_minus_one_estimate`] scheme (sharing its reduction and table term) with a plain
/// f64 series instead of its Dekker-product terms, as [`two_to_the_fraction`] needs only 2^-60.
/// 2^f = 1 + E + e + E e with E = 2^(j/64) - 1 from the table and e = expm1(y),
/// y = r ln 2, |y| <= 0.0055: e = yh + yh^2 (1/2 + yh/6 + ... + yh^5/5040) + yl (1 + yh), the
/// bracket in plain f64 (its error about 2^-69, the next term 2^-75).
#[inline]
fn two_to_the_fraction_estimate(magnitude: u128, drop: u32) -> (f64, f64) {
    let (j, yh, yl) = reduce_fraction(magnitude, drop);
    let rest = yh
        * yh
        * (0.5
            + yh * (1.0 / 6.0
                + yh * (1.0 / 24.0
                    + yh * (1.0 / 120.0 + yh * (1.0 / 720.0 + yh * (1.0 / 5040.0))))));
    let (b, b_error) = fast_two_sum(yh, rest);
    let (u, u_low) = add_table_term(j, b, b_error + yl * (1.0 + yh));
    // 1 + u, u in (0, 0.415], so 1 leads.
    let (one_plus, one_plus_error) = fast_two_sum(1.0, u);
    fast_two_sum(one_plus, one_plus_error + u_low)
}

/// `value` (at least 2^63, at most 128 bits) rounded to nearest, ties to even, to 64 bits: the
/// significand and the number of low bits dropped ([`round_to_bits`] specialized to a 64-bit
/// result, inlined on the hot path).
#[inline]
fn round_to_64_bits(value: u128) -> (u64, i32) {
    // 0..=64 low bits to drop.
    let shift = 64 - value.leading_zeros();
    let kept = u64::try_from(value >> shift).unwrap_or(u64::MAX);
    let (rounded, carry) = kept.overflowing_add(round_up(value, shift, kept));
    // A carry out of the top bit (all ones rounded up) leaves 2^64 = 2^63 * 2.
    let shift = i32::try_from(shift).unwrap_or(0);
    if carry {
        (1 << 63, shift + 1)
    } else {
        (rounded, shift)
    }
}

/// `value` (below 2^64) / 2^`shift` rounded to nearest, ties to even, for 2 <= `shift` < 65: at
/// most 62 bits.
#[inline]
fn round_shifted(value: u128, shift: u32) -> u64 {
    let kept = u64::try_from(value >> shift).unwrap_or(0);
    kept + round_up(value, shift, kept)
}

/// 1 if `value` / 2^`shift`, truncated to `kept`, rounds up to nearest, ties to even, else 0
/// (`shift` <= 64; 0 drops nothing). Integer bit operations, not `||`: the round-up is
/// data-dependent, and a branch on it mispredicts about half the time.
#[inline]
fn round_up(value: u128, shift: u32, kept: u64) -> u64 {
    if shift == 0 {
        return 0;
    }
    let dropped = value & ((1 << shift) - 1);
    let half = 1 << (shift - 1);
    u64::from(dropped > half) | (u64::from(dropped == half) & kept & 1)
}

/// RN64(1 / w) for w = `significand` * 2^-63 in [1, 2) with a 64-bit significand: the 64-bit
/// significand and its exponent. 1 / w = 2^127 / W * 2^-64 with the integer quotient in
/// (2^63, 2^64), so it already has the 64 bits and the remainder rounds it (a tie needs W = 2^63,
/// an exact quotient). The quotient comes from an f64 estimate, good to about 2^12 units,
/// refined by one more f64 step from its exact remainder and then made exact by at most one unit
/// either way; this replaces a u128 division, a runtime call.
#[inline]
fn reciprocal(significand: u64) -> (u64, i32) {
    const TWO_TO_THE_127: u128 = 1 << 127;
    if significand == 1 << 63 {
        return (1 << 63, -63);
    }
    let divisor = significand as f64;
    // In (2^63, 2^64): the saturating conversion keeps it a u64.
    let estimate = (TWO_TO_THE_127 as f64 / divisor) as u64;
    // |remainder| < 2^78, exact in i128 (the product is below 2^128).
    let wide = i128::from(significand);
    let remainder = TWO_TO_THE_127
        .wrapping_sub(u128::from(estimate) * u128::from(significand))
        .cast_signed();
    // The remainder as a double from its two halves (no runtime i128 conversion); the correction
    // is under 2^13 units and its floor is off by at most one.
    let high = i64::try_from(remainder >> 64).unwrap_or(0) as f64;
    let low = (remainder as u64) as f64;
    let correction = ((high * power_of_two(64) + low) / divisor).floor() as i64;
    let mut quotient = i128::from(estimate) + i128::from(correction);
    let mut remainder = remainder - i128::from(correction) * wide;
    if remainder < 0 {
        quotient -= 1;
        remainder += wide;
    } else if remainder >= wide {
        quotient += 1;
        remainder -= wide;
    }
    // A significand above 2^63 has a quotient at most 2^64 - 2, so the rounded one is a u64.
    let quotient = u64::try_from(quotient).unwrap_or(u64::MAX);
    (quotient + u64::from(2 * remainder > wide), -64)
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
/// is at least 2^-64 of it). The estimate's error budget: the f64 tail about 2^-89, the neglected
/// yl yh^4/24 about 2^-87, the rounded cross terms and the table 2^-104 or less, times at most 3.4
/// for the E + e sum: under 2^-84 (module tests check 2^-80 on a sweep). The series runs for
/// about 1 call in 2^9.
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
    let (j, yh, yl) = reduce_fraction(magnitude, drop);
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
    add_table_term(j, b, lo)
}

/// f = `magnitude` * 2^-`drop` in (0, 1/2] split as j/64 + r: j = nint(64 f) <= 32 and
/// y = r ln 2 as a double-double (yh, yl), |y| <= 0.0055. drop >= 53 ([`excel_exp`]), so both
/// shifts are in range, and r is exact with |r| <= 2^(drop - 7) and |r| <= magnitude < 2^64.
/// Shared by [`two_to_the_minus_one_estimate`] and [`two_to_the_fraction_estimate`].
fn reduce_fraction(magnitude: u128, drop: u32) -> (usize, f64, f64) {
    let j = (magnitude + (1 << (drop - 7))) >> (drop - 6);
    let r = i128::try_from(magnitude).unwrap_or(0) - i128::try_from(j << (drop - 6)).unwrap_or(0);
    let (rh, rl) = exact_double_double(r, drop);
    // y = r ln 2: the head product exactly, the cross terms rounded (2^-108).
    let (product, product_error) = two_prod(rh, LN_2_HI);
    let (yh, yl) = fast_two_sum(product, product_error + (rh * LN_2_LO + rl * LN_2_HI));
    (usize::try_from(j).unwrap_or(0).min(32), yh, yl)
}

/// 2^(j/64) - 1 + e = E + e + E e as a double-double, for e = `b` + `lo` = expm1(y) from
/// [`reduce_fraction`]'s y and E = 2^(j/64) - 1 from the table. E >= 0.0109 for j >= 1 and
/// |e| <= 0.0055, so the sum is at least 0.0032 and never cancels by more than a factor 3.4;
/// E = 0 for j = 0.
fn add_table_term(j: usize, b: f64, lo: f64) -> (f64, f64) {
    let (e_hi, e_lo) = TWO_TO_THE_J_OVER_64_MINUS_ONE[j];
    let (e_hi, e_lo) = (f64::from_bits(e_hi), f64::from_bits(e_lo));
    // E e, then E + (e + E e).
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
/// `mul_add` (trig.rs's `DoubleDouble::two_prod`), which without the `fma` target feature is a
/// runtime-dispatched call: on this hot path that call cost more than the 17 flops.
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

/// [`dd_mul`] by [`two_prod`] instead of `mul_add`, for operands well inside the double range
/// (|a|, |b| < 2^995, product not subnormal): the same exact product error, so the same result,
/// without the runtime-dispatched FMA call that costs most of a hot kernel built on it
/// (`x87_log10`, T-2016).
pub(crate) fn dd_mul_in_range(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let (p, p_error) = two_prod(a.0, b.0);
    two_sum(p, p_error + (a.0 * b.1 + a.1 * b.0))
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

/// sqrt of a positive double-double: the double root and one Newton correction.
pub(crate) fn dd_sqrt(a: (f64, f64)) -> (f64, f64) {
    let root = a.0.sqrt();
    let square = dd_mul((root, 0.0), (root, 0.0));
    let residual = dd_add(a, (-square.0, -square.1));
    two_sum(root, residual.0 / (2.0 * root))
}

/// Taylor terms of [`dd_exp`] on its reduced argument, |r| <= ln 2 / 2^9: the first omitted
/// term is below 2^-120 relative.
const DD_EXP_TERMS: u32 = 11;

/// Squarings that undo [`dd_exp`]'s argument halving (2^8 = 256).
const DD_EXP_SQUARINGS: u32 = 8;

/// e^a in double-double (about 2^-98 relative) for |a| <= 650: a = k ln 2 + r, e^(r/256) by its
/// Taylor series, squared 8 times, times 2^k. Below -650 the tail word turns subnormal and loses
/// bits, and from about 709 the head overflows; callers check first.
pub(crate) fn dd_exp(a: (f64, f64)) -> (f64, f64) {
    let k = (a.0 / std::f64::consts::LN_2).round();
    let reduced = dd_add(a, dd_mul((-k, 0.0), (LN_2_HI, LN_2_LO)));
    let small = (reduced.0 / 256.0, reduced.1 / 256.0);
    let mut term = (1.0, 0.0);
    let mut sum = (1.0, 0.0);
    for i in 1..=DD_EXP_TERMS {
        term = dd_div_small(dd_mul(term, small), f64::from(i));
        sum = dd_add(sum, term);
    }
    for _ in 0..DD_EXP_SQUARINGS {
        sum = dd_mul(sum, sum);
    }
    // |k| <= 938 for |a| <= 650; 2^k as a double is exact.
    let scale = power_of_two(k as i32);
    (sum.0 * scale, sum.1 * scale)
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

/// The double-double `(hi, lo)` rounded to a 64-bit significand and then to a double,
/// RN53(RN64(v)), as an x87 result is stored. Zero, and a result past the normal range, keep the
/// single rounding `hi`.
pub(crate) fn dd_rounded_twice((hi, lo): (f64, f64)) -> f64 {
    if hi == 0.0 {
        return hi;
    }
    // `dd_to_extended` adds the tail to the head's magnitude, so it takes |v|.
    let negative = hi < 0.0;
    let magnitude = if negative { (-hi, -lo) } else { (hi, lo) };
    let (extended, exponent) = dd_to_extended(magnitude);
    round_extended_to_double(extended, exponent, negative).unwrap_or(hi)
}

#[cfg(test)]
mod tests;
