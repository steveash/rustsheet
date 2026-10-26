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
const LOG2_E_64: u128 = 0xB8AA_3B29_5C17_F0BC;

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
    exp_with(a, two_to_the_minus_one)
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
/// tests can run the whole model on the reference series.
fn exp_with(a: Register, two_to_the_minus_one: fn(u128, u32) -> (u128, i32)) -> f64 {
    let (negative, a_significand, a_exponent) = a.parts();
    // A zero (no top bit) gives 1. Otherwise 2^top <= |a| < 2^(top + 1); a significand has at
    // most 64 bits, so its bit index fits an i32.
    let Some(top_bit) = a_significand.checked_ilog2() else {
        return 1.0;
    };
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
    // Step 1: t = T * 2^t_exponent, T a 64-bit integer: the product (at least 2^63, as the
    // constant is) shifted up by its leading zeros has its top bit set, and its top 64 bits take
    // one rounding on the low 64. u64 halves, not `round_to_bits`' general u128 width (T-2102).
    let product = a_significand * LOG2_E_64;
    let zeros = product.leading_zeros();
    let (t, t_exponent) = round_top_64(
        product << zeros,
        a_exponent + 1 - i32::try_from(zeros).unwrap_or(0),
    );
    // Step 2: |t| <= T_LIMIT < 2^11 and |t| >= 2^-66, so `drop` is in 53..=130. The guards above
    // make every `try_from` fallback below unreachable.
    let Ok(drop) = u32::try_from(-t_exponent) else {
        return f64::NAN;
    };
    // n = nint(t), ties to even, and the fraction's magnitude (below 2^64) and sign.
    let (n, magnitude, fraction_negative) = if drop >= 65 {
        // |t| < 2^(64 - drop) <= 1/2: n = 0 and f = t.
        (0_i32, t, false)
    } else {
        // drop in 53..=64; a shift by 64 (t < 1) leaves no integer part.
        let integer = t.checked_shr(drop).unwrap_or(0);
        let mask = u64::MAX >> (64 - drop);
        let rest = t & mask;
        let half = 1_u64 << (drop - 1);
        // Integer bit operations, not `||`: a data-dependent branch here mispredicts half the time.
        let up = (rest > half) | ((rest == half) & (integer & 1 == 1));
        // Rounding up leaves f = rest - 2^drop < 0, of magnitude 2^drop - rest (rest > 0).
        let magnitude = if up { rest.wrapping_neg() & mask } else { rest };
        // integer < 2^11, so the conversion cannot fail.
        (
            i32::try_from(integer + u64::from(up)).unwrap_or(0),
            magnitude,
            up,
        )
    };
    // f * sign(a) is the true fraction: `fraction_negative` is the magnitude-side sign of |t|.
    let negative_f = fraction_negative != negative;
    let n = if negative { -n } else { n };
    // Step 3: w = W * 2^w_exponent, W a 64-bit integer with its top bit set.
    let (mut w, mut w_exponent) = (1_u64 << 63, -63);
    if magnitude != 0 {
        let (u, u_exponent) = two_to_the_minus_one(u128::from(magnitude), drop);
        let u = u64::try_from(u).unwrap_or(0);
        // u < 0.415 with a 64-bit significand, so u_exponent <= -65: 1 + u is 2^63 + u >> s on
        // the grid 2^(u_exponent + s), s = -u_exponent - 63 >= 2, rounded on u's s low bits (no
        // carry: u >> s < 2^62). s >= 65 (u below 2^-64) leaves w = 1.
        if let Ok(s) = u32::try_from(-u_exponent - 63) {
            if (2..=64).contains(&s) {
                let kept = u.checked_shr(s).unwrap_or(0);
                let dropped = u & (u64::MAX >> (64 - s));
                let half = 1_u64 << (s - 1);
                let up = (dropped > half) | ((dropped == half) & (kept & 1 == 1));
                w = (1 << 63) + kept + u64::from(up);
                w_exponent = u_exponent + i32::try_from(s).unwrap_or(0);
            }
        }
        if negative_f {
            let (q, q_shift) = reciprocal(w);
            (w, w_exponent) = (q, -127 - w_exponent + q_shift);
        }
    }
    // Step 4: one rounding of the 64-bit w * 2^n to a double, or to the subnormal grid.
    let exponent = w_exponent + n;
    round_64_to_double(w, exponent).unwrap_or_else(|| {
        if exponent > 0 {
            f64::INFINITY
        } else {
            round_extended_to_subnormal(u128::from(w), exponent, false).unwrap_or(0.0)
        }
    })
}

/// RN64 of `value` * 2^(`exponent` - 64) for a `value` with its top bit set: the 64-bit
/// significand and its exponent (a carry out of all ones is one more bit).
fn round_top_64(value: u128, exponent: i32) -> (u64, i32) {
    // The top and low halves of a u128 always fit a u64.
    let high = u64::try_from(value >> 64).unwrap_or(0);
    let low = u64::try_from(value & u128::from(u64::MAX)).unwrap_or(0);
    let half = 1_u64 << 63;
    let up = (low > half) | ((low == half) & (high & 1 == 1));
    let (rounded, carry) = high.overflowing_add(u64::from(up));
    if carry {
        (half, exponent + 1)
    } else {
        (rounded, exponent)
    }
}

/// RN64(2^127 / `w`) for 2^63 <= `w` < 2^64: the 64-bit significand and the exponent to add to
/// -127 - w's. The integer quotient, in (2^63, 2^64], already has the 64 bits, and the remainder
/// rounds it (a tie needs w = 2^63, an exact quotient).
fn reciprocal(w: u64) -> (u64, i32) {
    let numerator = 1_u128 << 127;
    let w = u128::from(w);
    let quotient = numerator / w;
    let remainder = numerator - quotient * w;
    let q = quotient + u128::from(2 * remainder > w);
    // q = 2^64 only for w = 2^63.
    let carry = u32::from(q >> 64 != 0);
    (
        u64::try_from(q >> carry).unwrap_or(0),
        i32::try_from(carry).unwrap_or(0),
    )
}

/// The 64-bit `w` (top bit set) times 2^`exponent` rounded once to a normal double, or `None`
/// past the normal range ([`round_extended_to_double`] for this width, without its u128 work).
fn round_64_to_double(w: u64, exponent: i32) -> Option<f64> {
    let kept = w >> 11;
    let dropped = w & 0x7FF;
    let up = (dropped > 0x400) | ((dropped == 0x400) & (kept & 1 == 1));
    // kept + up is in [2^52, 2^53]: added to the biased exponent less one, a carry to 2^53 moves
    // into the exponent field as the next binade.
    let significand = kept + u64::from(up);
    let biased = exponent + 11 + 1075 + i32::from(significand >> 53 != 0);
    if !(1..=2046).contains(&biased) {
        return None;
    }
    let field = u64::try_from(exponent + 11 + 1074).ok()?;
    Some(f64::from_bits((field << 52) + significand))
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
        // lo in units of the grid, |units| <= 2^10, exactly (a power-of-two scaling), and its
        // distance from the nearest integer, exact too: the midpoints are at distance 1/2.
        let units = lo * power_of_two(11 - exponent);
        let nearest = nearest_integer(units);
        if (units - nearest).abs() < 0.5 - 0.5 * MIDPOINT_MARGIN {
            // 2^63 <= significand * 2^11 + nearest < 2^64.
            let rounded =
                (u128::from(significand) << 11).wrapping_add_signed(i128::from(nearest as i64));
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
/// double-double. f = j/64 + r with |r| <= 1/128, and 2^f - 1 = E + e (1 + E) with E = 2^(j/64) - 1
/// from the table and e = expm1(y), y = r ln 2, |y| <= 0.0055. Mostly plain f64 work: the
/// leading series terms y, y^2/2, y^3/6 and y^4/24 are kept exactly (Dekker products), and the
/// rest, under 2^-37 of e, is summed in f64.
fn two_to_the_minus_one_estimate(magnitude: u128, drop: u32) -> (f64, f64) {
    // drop >= 53 (excel_exp), so both shifts are in range; j = nint(64 f) <= 32, and r is exact
    // with |r| <= 2^(drop - 7) and |r| <= magnitude < 2^64.
    let j = (magnitude + (1 << (drop - 7))) >> (drop - 6);
    let r = i128::try_from(magnitude).unwrap_or(0) - i128::try_from(j << (drop - 6)).unwrap_or(0);
    let (yh, yl) = times_ln_2(r, drop);
    // expm1(yh) = yh + p/2 + t + v + tail with p = yh^2, t = yh^3/6, v = yh^4/24, each a
    // double-double; tail = yh^5 (1/120 + yh/720 + ...) to the 1/9! term (the next is 2^-89).
    let (p, p_error) = two_prod(yh, yh);
    // t = p (yh/6), with yh/6 a double-double formed alongside p, not after it (T-2102: one
    // Dekker product less on the chain the call waits on); q = yh^3 only feeds the tail.
    let (sixth, sixth_error) = two_prod(yh, ONE_SIXTH.0);
    let sixth_error = sixth_error + yh * ONE_SIXTH.1;
    let (t, t_error) = two_prod(p, sixth);
    let t_error = t_error + p * sixth_error + p_error * sixth;
    let q = yh * p;
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
    // expm1(yh + yl) = expm1(yh) + yl exp(yh), and |yl| < 2^-51 |yh|.
    let lo = b_error
        + a_error2
        + a_error
        + (p_error * 0.5 + t_error + v_error + tail)
        + yl * (1.0 + yh + p * 0.5 + t);
    let (e_hi, e_lo) = TWO_TO_THE_J_OVER_64_MINUS_ONE[usize::try_from(j).unwrap_or(0).min(32)];
    let (e_hi, e_lo) = (f64::from_bits(e_hi), f64::from_bits(e_lo));
    // E + e F with F = 1 + E = 2^(j/64), exactly a double-double: one product and two
    // renormalisations on the chain (T-2102). E >= 0.0109 for j >= 1 and |e F| <= 0.0078, so the
    // FastTwoSum holds, the sum is at least 0.0032 and never cancels by more than a factor 3.4;
    // E = 0 for j = 0.
    let (f_hi, f_error) = fast_two_sum(1.0, e_hi);
    let f_lo = f_error + e_lo;
    let (m, m_error) = two_prod(b, f_hi);
    let m_error = m_error + b * f_lo + lo * f_hi;
    let (u, u_error) = fast_two_sum(e_hi, m);
    fast_two_sum(u, u_error + m_error + e_lo)
}

/// ln 2 * 2^128, truncated, as its high and low 64 bits.
const LN_2_128: (u64, u64) = (0xB172_17F7_D1CF_79AB, 0xC9E3_B398_03F2_F6AF);

/// y = `r` * 2^-`drop` * ln 2 as a double-double (|lo| < 2^-51 |hi|, to about 2^-104 relative),
/// for |`r`| < 2^64, from one 64 x 128-bit integer product: the normalised |r| times ln 2 * 2^128
/// has its top bit at 126 or 127, and its top 53 and next 53 bits convert exactly. Shorter than
/// a Dekker product of the double-double r, which this hot path waits on (T-2102).
fn times_ln_2(r: i128, drop: u32) -> (f64, f64) {
    let Ok(magnitude) = u64::try_from(r.unsigned_abs()) else {
        return (0.0, 0.0);
    };
    if magnitude == 0 {
        return (0.0, 0.0);
    }
    let zeros = magnitude.leading_zeros();
    let normalised = u128::from(magnitude << zeros);
    // The top 128 bits of the 192-bit product (the dropped part is under 2^-126 of it).
    let top = normalised * u128::from(LN_2_128.0) + ((normalised * u128::from(LN_2_128.1)) >> 64);
    // y = top * 2^(-64 - drop - zeros); both pieces are below 2^53, so they convert exactly.
    let exponent = -64 - i32::try_from(drop + zeros).unwrap_or(0);
    let high = u64::try_from(top >> 75).unwrap_or(0) as f64 * power_of_two(exponent + 75);
    let low = u64::try_from((top >> 22) & ((1 << 53) - 1)).unwrap_or(0) as f64
        * power_of_two(exponent + 22);
    if r < 0 {
        (-high, -low)
    } else {
        (high, low)
    }
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
mod tests {
    use super::*;

    fn excel_exp_series(a: f64) -> f64 {
        exp_with(Register::load(a), two_to_the_minus_one_series)
    }

    #[test]
    fn dd_rounded_twice_ties_after_the_64_bit_rounding() {
        let one_up = 1.0 + f64::EPSILON;
        // 1 + 2^-53 + 2^-80: once rounded it is 1 + 2^-52, but RN64 drops the 2^-80 and leaves
        // the 53-bit midpoint, which RN53 ties to the even 1.0.
        let tail = -power_of_two(-53) + power_of_two(-80);
        assert_eq!(one_up + tail, one_up);
        assert_eq!(dd_rounded_twice((one_up, tail)), 1.0);
        assert_eq!(dd_rounded_twice((-one_up, -tail)), -1.0);
        // Away from a midpoint both roundings agree with the head.
        assert_eq!(dd_rounded_twice((one_up, -power_of_two(-55))), one_up);
        assert_eq!(dd_rounded_twice((0.0, 0.0)), 0.0);
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
            let limit = 1_u128
                .checked_shl(drop - 1)
                .map_or(u128::from(u64::MAX), |l| l.min(u128::from(u64::MAX)));
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
            let limit = 1_u128
                .checked_shl(drop - 1)
                .map_or(u128::from(u64::MAX), |l| l.min(u128::from(u64::MAX)));
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

    /// The T-1945 integer core (steps 1, 2 and 4 and the reciprocal on general u128 values with
    /// `round_to_bits` and a u128 division), kept as the reference T-2102's u64 core must match.
    fn exp_reference(a: Register) -> f64 {
        use crate::arithmetic::round_extended_to_double;
        let (negative, a_significand, a_exponent) = a.parts();
        let Some(top_bit) = a_significand.checked_ilog2() else {
            return 1.0;
        };
        let top = a_exponent + i32::try_from(top_bit).unwrap();
        if top < -65 {
            return 1.0;
        }
        if top >= 11 || (top == 10 && a.store().abs() * std::f64::consts::LOG2_E > T_LIMIT) {
            return if negative { 0.0 } else { f64::INFINITY };
        }
        let (t, shift) = round_to_bits(a_significand * LOG2_E_64, 64, false);
        let drop = u32::try_from(63 - a_exponent - shift).unwrap();
        let (n, fraction) = if drop >= 65 {
            (0_i32, i128::try_from(t).unwrap())
        } else {
            let integer = t >> drop;
            let rest = t & ((1 << drop) - 1);
            let half = 1 << (drop - 1);
            let n = integer + u128::from(rest > half || (rest == half && integer & 1 == 1));
            let f = i128::try_from(t).unwrap() - i128::try_from(n << drop).unwrap();
            (i32::try_from(n).unwrap(), f)
        };
        let negative_f = (fraction < 0) != negative;
        let n = if negative { -n } else { n };
        let (mut w, mut w_exponent) = (1_u128 << 63, -63);
        if fraction != 0 {
            let (u, u_exponent) = two_to_the_minus_one(fraction.unsigned_abs(), drop);
            let gap = u32::try_from(-u_exponent).unwrap();
            if gap < 128 {
                let (sum, sum_shift) = round_to_bits((1 << gap) + u, 64, false);
                (w, w_exponent) = (sum, u_exponent + sum_shift);
            }
            if negative_f {
                let numerator = 1_u128 << 127;
                let (quotient, remainder) = (numerator / w, numerator % w);
                let (q, q_shift) =
                    round_to_bits(quotient + u128::from(2 * remainder > w), 64, false);
                (w, w_exponent) = (q, -127 - w_exponent + q_shift);
            }
        }
        let exponent = w_exponent + n;
        round_extended_to_double(w, exponent, false).unwrap_or_else(|| {
            if exponent > 0 {
                f64::INFINITY
            } else {
                round_extended_to_subnormal(w, exponent, false).unwrap_or(0.0)
            }
        })
    }

    // The u64 core returns the reference core's bits (T-2102): random doubles of every finite
    // bit pattern (so every exponent, the range guards and subnormals), uniform in [-750, 750]
    // and in [-30, 30], random significands at every binade from 2^-70 up, and full-width
    // registers with random 64-bit significands at those binades.
    #[test]
    fn integer_core_matches_the_reference() {
        let mut state = 0x3C6E_F372_FE94_F82B_u64;
        for _ in 0..300_000 {
            let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
            let raw = f64::from_bits(next(&mut state));
            let binade =
                f64::from_bits(((next(&mut state) % 80 + 953) << 52) | (next(&mut state) >> 12));
            for a in [
                raw,
                (unit - 0.5) * 1500.0,
                (unit - 0.5) * 60.0,
                binade,
                -binade,
            ] {
                if a.is_finite() {
                    let register = Register::load(a);
                    assert_eq!(
                        excel_exp(a).to_bits(),
                        exp_reference(register).to_bits(),
                        "a = {a:e}"
                    );
                }
            }
            let exponent = i32::try_from(next(&mut state) % 80).unwrap() - 134;
            let significand = u128::from(next(&mut state) | (1 << 63));
            for negative in [false, true] {
                let register = Register::from_parts(negative, significand, exponent);
                assert_eq!(
                    excel_exp_extended(register).to_bits(),
                    exp_reference(register).to_bits(),
                    "{negative} {significand:#x} {exponent}"
                );
            }
        }
    }

    /// EXP of an extended argument equals w1-density's mpmath model V6 (`lib.py` `exp_v6`) on the
    /// 5,200 random 64-bit arguments of `test/fixtures/exp_extended.fixture` (lines
    /// `±significand exponent model`, the significand and the model's bits in hex).
    #[test]
    fn extended_arguments_match_the_mpmath_model() {
        let mut rows = 0;
        for line in include_str!("test/fixtures/exp_extended.fixture").lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split_whitespace().collect();
            let (negative, significand) = fields[0].split_at(1);
            let significand = u128::from_str_radix(significand, 16).unwrap();
            let exponent: i32 = fields[1].parse().unwrap();
            let model = u64::from_str_radix(fields[2], 16).unwrap();
            let a = Register::from_parts(negative == "-", significand, exponent);
            assert_eq!(
                a.parts(),
                (negative == "-", significand, exponent),
                "{line}"
            );
            assert_eq!(excel_exp_extended(a).to_bits(), model, "{line}");
            rows += 1;
        }
        assert_eq!(rows, 5200);
    }

    /// On a double, EXP of the double as a full-width register (its significand shifted up to 64
    /// bits, as an operation leaves it, not the 53 bits of a load) is EXP of the double, bit for
    /// bit: random doubles in [-750, 750], at every binade from 2^-70 up, and at the range guards.
    fn full_width(a: f64) -> Register {
        let register = Register::load(a).mul(Register::load(1.0));
        assert!(a == 0.0 || register.parts().1 >> 63 == 1, "{a:e}");
        register
    }

    #[test]
    fn extended_argument_of_a_double_is_excel_exp() {
        let mut state = 0x2633_0000_9E37_79B9_u64;
        for _ in 0..100_000 {
            let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
            let wide = (unit - 0.5) * 1500.0;
            let binade =
                f64::from_bits(((next(&mut state) % 80 + 953) << 52) | (next(&mut state) >> 12));
            for a in [wide, binade, -binade] {
                let extended = excel_exp_extended(full_width(a));
                assert_eq!(extended.to_bits(), excel_exp(a).to_bits(), "a = {a:e}");
            }
        }
        // Around the T_LIMIT guard (|a| log2 e = 2000 at |a| about 1386.29, inside the 2^10
        // binade that takes the stored test) and the binade edges 1024 and 2048.
        let guard = 2000.0 / std::f64::consts::LOG2_E;
        let mut edges = vec![0.0, 1386.0, 1387.0, f64::MAX];
        for centre in [guard, 1024.0, 2048.0] {
            let bits = centre.to_bits();
            edges.extend([f64::from_bits(bits - 1), centre, f64::from_bits(bits + 1)]);
        }
        let negated: Vec<f64> = edges.iter().map(|a| -a).collect();
        for a in edges.into_iter().chain(negated) {
            assert_eq!(
                excel_exp_extended(full_width(a)).to_bits(),
                excel_exp(a).to_bits(),
                "a = {a:e}"
            );
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
