//! Decimal-correct rounding for ROUND, ROUNDUP and ROUNDDOWN.
//!
//! Excel rounds in two decimal stages, never by scaling with a binary power of ten
//! (docs/excel-semantics/numbers.md, "ROUND family rounds in two decimal stages"; evidence
//! probes/round-family and probes/precision-and-rounding A6–A8, oracle excel-graph):
//! 1. the exact binary value is cut to 15 significant digits. An exact tie at the 16th digit goes
//!    toward zero for ROUND (`ROUND(999999999999957.5,0)` = 999999999999957) but away from zero
//!    for ROUNDUP and ROUNDDOWN (`ROUNDDOWN(999999999999957.5,0)` = 999999999999958);
//! 2. that 15-digit decimal is rounded at `num_digits` (half away from zero, up or down), and the
//!    decimal result is converted to the nearest double. ROUNDUP alone does this in binary: it
//!    converts the rounded-down decimal and then adds the unit `10^-num_digits` as doubles, so
//!    `ROUNDUP(123.456,2)` is 123.45 + 0.01 = 123.46000000000001 (docs/excel-semantics/numbers.md
//!    "ROUNDUP adds the unit in binary"; probes/roundup-wrapped-scaling, oracle excel-graph).
//!
//! So `ROUND(1.005,2)` is 1.01 (1.00499999999999989… becomes 1.00500000000000 first) and
//! `ROUNDDOWN(123456789.123,-5)` is exactly 123400000.

use crate::arithmetic::{flush_subnormal, TINY_SUBNORMAL_BELOW};
use crate::constants::EXCEL_PRECISION;
use crate::number_format::{significant_15, to_precision};

/// How the final digit is chosen in stage 2.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RoundMode {
    /// ROUND: half away from zero.
    HalfAwayFromZero,
    /// ROUNDUP: away from zero.
    Up,
    /// ROUNDDOWN: toward zero.
    Down,
}

/// Significant digits Excel keeps before rounding.
const SIGNIFICANT_DIGITS: i64 = 15;
/// `num_digits` beyond this magnitude rounds every finite double the same way, so clamping it
/// keeps the exponent arithmetic small without changing any result.
const MAX_NUM_DIGITS: i64 = 10_000;

/// The integer `num_digits` Excel actually rounds at, for a value whose 15-digit decimal form
/// has decimal exponent `exponent` (docs/excel-semantics/numbers.md, "ROUND family: huge
/// `num_digits` wrap"; probes/round-huge-digits, -wrap and -negative, oracle excel-graph):
/// - ROUNDUP and ROUNDDOWN keep the sign of the truncated `num_digits` and take its magnitude,
///   capped at 2^32 - 1, modulo 65,536: `ROUNDDOWN(123.456,65537)` = 123.4,
///   `ROUNDUP(123.456,-65537)` = 130, `ROUNDDOWN(0.5,2^31)` = 0, but 2^32 and beyond act as 65,535.
/// - ROUND saturates `num_digits` to a 32-bit integer `k`, then counts the digits to keep,
///   `k + exponent + 1`, in 32 bits with wraparound: past the top it wraps to a huge negative
///   count (`ROUND(42,2^31-1)` = 0, `ROUND(1E+15,1E+15)` = 0, while `ROUND(0.5,2^31-1)` = 0.5),
///   and past the bottom to a huge positive one (`ROUND(5E-10,-2147483640)` = 5E-10).
fn effective_digits(num_digits: f64, exponent: i64, mode: RoundMode) -> i64 {
    // NaN cannot come from a cell; the saturating casts would read it as 0.
    let truncated = num_digits.trunc();
    if mode == RoundMode::HalfAwayFromZero {
        let k = i64::from(truncated as i32);
        let kept = k + exponent + 1;
        i64::from(kept as i32) - exponent - 1
    } else {
        let magnitude = (truncated.abs().min(f64::from(u32::MAX)) as i64) % 65_536;
        if truncated < 0.0 {
            -magnitude
        } else {
            magnitude
        }
    }
}

/// The double Excel returns for the 15-digit cut 2.22507385850720E-308: 2^-1022 - 6·2^-1074, 3 ulps
/// below the nearest double (probes/t2756-roundup-smallest-normal-threshold rows 52-58). It is the
/// only subnormal cut a normal argument can have, since every normal value cuts to at least it.
const SMALLEST_NORMAL_CUT: f64 = f64::from_bits(0x000F_FFFF_FFFF_FFFA);

/// ROUND, ROUNDUP, ROUNDDOWN and TRUNC of `value` at `num_digits`: [`round_decimal`], except for
/// tiny values, whatever `num_digits` (docs/excel-semantics/numbers.md "ROUNDUP and ROUNDDOWN of a
/// number whose 15-digit cut is subnormal"; probes t2756-roundup-smallest-normal-threshold and
/// t2771-roundup-subnormal-cuts, -sign, -threshold and -sign-controls, oracle excel-graph):
/// - ROUNDUP, ROUNDDOWN and TRUNC return a subnormal `value` as it is, with no 15-digit cut, and a
///   normal one whose cut is subnormal (`ROUNDUP(2^-1022,0)`) as ±[`SMALLEST_NORMAL_CUT`], while
///   `ROUNDUP(2^-1022*(1+8*2^-52),0)`, whose cut 2.22507385850721E-308 is normal, is 1;
/// - ROUND returns a positive subnormal below [`TINY_SUBNORMAL_BELOW`] (2^-1026) as it is
///   (`ROUND(x,0)` is `x`); a negative one, or one from there up, is 0.
///
/// INT does not share this path.
pub(crate) fn round_function(value: f64, num_digits: f64, mode: RoundMode) -> Option<f64> {
    if mode == RoundMode::HalfAwayFromZero {
        if value > 0.0 && value < TINY_SUBNORMAL_BELOW {
            return Some(value);
        }
    } else {
        if value.is_subnormal() {
            return Some(value);
        }
        if cut_to_15_digits(value, true).is_subnormal() {
            return Some(SMALLEST_NORMAL_CUT.copysign(value));
        }
    }
    round_decimal(value, num_digits, mode)
}

/// Rounds `value` at `num_digits` decimal places the way Excel's ROUND family does.
///
/// `num_digits` is truncated toward zero and then narrowed to an integer the way Excel does
/// (see [`effective_digits`]). Returns `None` when the result is beyond the double range
/// (`ROUNDUP(1.5,-400)` is `#NUM!` in Excel). Each decimal it converts to a double (the rounded
/// digits, and ROUNDUP's unit) is flushed to 0 when subnormal: `ROUND(2^-1022,309)` (2.2E-308)
/// and `ROUNDDOWN(2^-1022*(1+8*2^-52),308)` (2E-308) are 0, and so is
/// `ROUNDUP(2^-1022*(1+8*2^-52),308)`, its rounded-down 2E-308 and unit 1E-308 both being
/// flushed (probes/t2756-roundup-smallest-normal-threshold rows 30-50).
pub(crate) fn round_decimal(value: f64, num_digits: f64, mode: RoundMode) -> Option<f64> {
    if value == 0.0 || !value.is_finite() {
        return Some(value);
    }
    let tie_away = mode != RoundMode::HalfAwayFromZero;
    let (mantissa, exponent) = significant_15(value.abs(), tie_away);
    // The clamp keeps the exponent arithmetic below small without changing any result.
    let num_digits =
        effective_digits(num_digits, exponent, mode).clamp(-MAX_NUM_DIGITS, MAX_NUM_DIGITS);
    // `mantissa` has 15 digits; its last digit has place value 10^(exponent - 14).
    let last_place = exponent - (SIGNIFICANT_DIGITS - 1);
    let dropped = -num_digits - last_place;
    let (digits, place, add_unit) = if dropped <= 0 {
        (mantissa, last_place, false)
    } else {
        let (kept, round_up) = round_off(mantissa, dropped, mode);
        if mode == RoundMode::Up {
            (kept, -num_digits, round_up)
        } else {
            (kept + u64::from(round_up), -num_digits, false)
        }
    };
    let mut magnitude = flush_subnormal(decimal_to_f64(digits, place)?);
    if add_unit {
        magnitude += flush_subnormal(decimal_to_f64(1, place)?);
    }
    if magnitude.is_infinite() {
        return None;
    }
    if magnitude == 0.0 {
        // Excel has no negative zero (`ROUND(-0.0001,2)` is 0).
        return Some(0.0);
    }
    Some(magnitude.copysign(value))
}

/// The double nearest to `digits * 10^place`, where `digits <= 10^15`.
pub(crate) fn decimal_to_f64(digits: u64, place: i64) -> Option<f64> {
    // Fast path: `digits` (< 2^53) and 10^|place| (|place| <= 22) are exact doubles, so one
    // multiplication or division is correctly rounded, like the parse below.
    if let Ok(exact_power) = i32::try_from(place.unsigned_abs()) {
        if exact_power <= 22 {
            let digits = digits as f64; // digits <= 10^15 < 2^53: exact
            let scale = 10_f64.powi(exact_power);
            return Some(if place >= 0 {
                digits * scale
            } else {
                digits / scale
            });
        }
    }
    format!("{digits}e{place}").parse().ok()
}

/// `value` cut to 15 significant digits, as the double nearest that decimal: an exact tie at the
/// 16th digit goes away from zero when `tie_away`, otherwise toward zero. A zero or non-finite
/// `value` is itself. FLOOR's ratio snap and PERCENTRANK's truncation use it, so that a double
/// just below a digit boundary counts as on it.
pub(crate) fn cut_to_15_digits(value: f64, tie_away: bool) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    let (mantissa, exponent) = significant_15(value.abs(), tie_away);
    // `mantissa` has 15 digits, so its last digit has place value 10^(exponent - 14).
    decimal_to_f64(mantissa, exponent - (SIGNIFICANT_DIGITS - 1))
        .map_or(value, |cut| cut.copysign(value))
}

/// Drops the last `dropped` (> 0) digits of the 15-digit `mantissa`: the digits kept, and whether
/// `mode` rounds them up by one unit.
fn round_off(mantissa: u64, dropped: i64, mode: RoundMode) -> (u64, bool) {
    let Ok(dropped) = u32::try_from(dropped) else {
        return (0, mode == RoundMode::Up);
    };
    let Some(unit) = 10_u64.checked_pow(dropped) else {
        // Everything is dropped and the value is below half a unit.
        return (0, mode == RoundMode::Up);
    };
    let (kept, rest) = (mantissa / unit, mantissa % unit);
    let round_up = match mode {
        RoundMode::HalfAwayFromZero => rest >= unit - rest,
        RoundMode::Up => rest > 0,
        RoundMode::Down => false,
    };
    (kept, round_up)
}

/// The value a number format shows at `precision` decimals: the 15-digit decimal rounded half away
/// from zero, as General and ROUND do (`TEXT(2.5,"0")` "3", `TEXT(12.345,"0%")` "1235%"; T-2206).
/// Scientific formats, and a result beyond the double range, keep the significant-digit cut (T-1730).
pub(crate) fn round_shown_place(value: f64, precision: i32, is_scientific: bool) -> f64 {
    let rounded = round_decimal(value, f64::from(precision), RoundMode::HalfAwayFromZero);
    rounded.filter(|_| !is_scientific).unwrap_or_else(|| {
        let integer_digits = format!("{}", value.abs().floor()).len();
        to_precision(
            value,
            usize::min(
                usize::try_from(precision).unwrap_or(0) + integer_digits,
                EXCEL_PRECISION,
            ),
        )
    })
}

/// `value` (nonzero, finite) rounded to the digits a scientific mantissa with one integer digit and
/// `precision` decimals shows, and its exponent: the 15-digit decimal rounded half away from zero,
/// once, at `precision + 1` digits, as the fixed codes round (`0.00E+00` on 1.005, the double
/// 1.00499999…, is "1.01E+00"; `0E+0` on 2.5 "3E+0"; on 12.345 "1.23E+01", where rounding to four
/// digits first gave "1.24E+01"; probes/t2705-scientific-mantissa-rounding, gen-m10; T-2705).
pub(crate) fn round_scientific(value: f64, precision: i32) -> (f64, f64) {
    let shown = (i64::from(precision) + 1).clamp(1, SIGNIFICANT_DIGITS);
    let (mantissa, mut exponent) = significant_15(value.abs(), false);
    let dropped = SIGNIFICANT_DIGITS - shown;
    let (mut digits, round_up) = if dropped == 0 {
        (mantissa, false)
    } else {
        round_off(mantissa, dropped, RoundMode::HalfAwayFromZero)
    };
    digits += u64::from(round_up);
    if Some(digits) == u32::try_from(shown).ok().map(|shown| 10_u64.pow(shown)) {
        digits /= 10;
        exponent += 1;
    }
    let rounded = decimal_to_f64(digits, exponent - shown + 1).unwrap_or(f64::INFINITY);
    (rounded.copysign(value), exponent as f64)
}

/// `value` rounded to the digits an engineering mantissa of `group` integer placeholders and
/// `precision` decimals shows, and the exponent (a multiple of `group`) it is shown with.
pub(crate) fn round_engineering(value: f64, group: f64, precision: i32) -> (f64, f64) {
    let exponent_of = |v: f64| (v.abs().log10().floor() / group).floor() * group;
    let digits_of = |v: f64, exponent: f64| {
        let integer_digits = v.abs().log10().floor() - exponent + 1.0;
        // At most 15 significant digits (docs/excel-semantics/numbers.md, "Number → text").
        (integer_digits + f64::from(precision)).clamp(1.0, EXCEL_PRECISION as f64) as usize
    };
    let exponent = exponent_of(value);
    let rounded = to_precision(value, digits_of(value, exponent));
    let carried = exponent_of(rounded);
    if carried == exponent {
        (rounded, exponent)
    } else {
        (to_precision(value, digits_of(rounded, carried)), carried)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        round_decimal, round_function, RoundMode, SMALLEST_NORMAL_CUT, TINY_SUBNORMAL_BELOW,
    };

    fn round(value: f64, digits: f64) -> Option<f64> {
        round_decimal(value, digits, RoundMode::HalfAwayFromZero)
    }

    #[test]
    fn binary_inexact_halves_round_up() {
        assert_eq!(round(1.005, 2.0), Some(1.01));
        assert_eq!(round(2.675, 2.0), Some(2.68));
        assert_eq!(round(0.285, 2.0), Some(0.29));
        assert_eq!(round(-2.5, 0.0), Some(-3.0));
    }

    #[test]
    fn sixteenth_digit_tie_depends_on_function() {
        let x = 999_999_999_999_957.5;
        assert_eq!(round(x, 0.0), Some(999_999_999_999_957.0));
        assert_eq!(round(-x, 5.0), Some(-999_999_999_999_957.0));
        assert_eq!(
            round_decimal(x, 0.0, RoundMode::Up),
            Some(999_999_999_999_958.0)
        );
        assert_eq!(
            round_decimal(x, 0.0, RoundMode::Down),
            Some(999_999_999_999_958.0)
        );
        assert_eq!(round(1e15 + 5.0, 0.0), Some(1e15));
        assert_eq!(
            round_decimal(1e15 + 5.0, 0.0, RoundMode::Down),
            Some(1e15 + 10.0)
        );
    }

    #[test]
    fn negative_and_extreme_digits() {
        assert_eq!(round_decimal(42.0, -5.0, RoundMode::Up), Some(100_000.0));
        assert_eq!(round_decimal(1e-15, -5.0, RoundMode::Up), Some(100_000.0));
        assert_eq!(
            round_decimal(123_456_789.123, -5.0, RoundMode::Down),
            Some(123_400_000.0)
        );
        assert_eq!(round(1.5, -400.0), Some(0.0));
        assert_eq!(round(1.5, 400.0), Some(1.5));
        assert_eq!(round_decimal(1.5, -400.0, RoundMode::Up), None);
        // A saturated num_digits makes the 32-bit digit count wrap (probes/round-huge-digits).
        assert_eq!(round(1.5, f64::MAX), Some(0.0));
        assert_eq!(round(0.5, f64::MAX), Some(0.5));
        assert_eq!(round(1.5, f64::MIN), Some(0.0));
        assert_eq!(round(1.234_567_890_123_45e300, -299.0), Some(1.2e300));
        assert_eq!(round(1e-300, 310.0), Some(1e-300));
        assert_eq!(round(-0.0001, 2.0).map(f64::is_sign_negative), Some(false));
    }

    #[test]
    fn rounds_to_15_digits_first() {
        assert_eq!(
            round_decimal(1.0 - 2f64.powi(-53), 0.0, RoundMode::Down),
            Some(1.0)
        );
        assert_eq!(round(1e15 + 86.0, 2.0), Some(1e15 + 90.0));
        assert_eq!(
            round_decimal((15.505 + 1.005) / 2.0, 9.0, RoundMode::Up),
            Some(8.255)
        );
    }

    #[test]
    fn round_digit_count_wraps_in_32_bits() {
        // probes/round-huge-digits-wrap E:O and probes/round-huge-digits-negative.
        let max = f64::from(i32::MAX);
        assert_eq!(round(5.0, max), Some(0.0));
        assert_eq!(round(5.0, max - 1.0), Some(5.0));
        assert_eq!(round(50.0, max - 1.0), Some(0.0));
        assert_eq!(round(5e14, 2_147_483_633.0), Some(0.0));
        assert_eq!(round(5e14, 2_147_483_632.0), Some(5e14));
        assert_eq!(round(1e15, 1e15), Some(0.0));
        assert_eq!(round(-1e15 - 1.0, 1e15), Some(0.0));
        assert_eq!(round(0.05, max), Some(0.05));
        assert_eq!(round(5e-10, -2_147_483_640.0), Some(5e-10));
        assert_eq!(round(5e-10, -2_147_483_639.0), Some(0.0));
        assert_eq!(round(5e-300, -2_147_483_350.0), Some(5e-300));
        assert_eq!(round(5e-300, -2_147_483_349.0), Some(0.0));
        assert_eq!(round(0.5, f64::from(i32::MIN)), Some(0.0));
        assert_eq!(round(123.456, 65_536.0), Some(123.456));
    }

    #[test]
    fn roundup_adds_the_unit_in_binary() {
        // probes/roundup-wrapped-scaling C column; ROUNDDOWN stays decimal-correct (D column).
        let up = |value, digits| round_decimal(value, digits, RoundMode::Up);
        assert_eq!(up(123.456, 2.0), Some(123.460_000_000_000_01));
        assert_eq!(up(0.29, 1.0), Some(0.300_000_000_000_000_04));
        assert_eq!(up(2.675, 2.0), Some(2.679_999_999_999_999_7));
        assert_eq!(up(8.255, 1.0), Some(8.299_999_999_999_999));
        assert_eq!(up(-123.456, 2.0), Some(-123.460_000_000_000_01));
        assert_eq!(up(0.29, 2.0), Some(0.29));
        assert_eq!(up(123.456, 4_294_901_762.0), Some(123.460_000_000_000_01));
        assert_eq!(round_decimal(0.29, 1.0, RoundMode::Down), Some(0.2));
    }

    #[test]
    fn roundup_rounddown_digits_wrap_modulo_65536() {
        // probes/round-huge-digits-wrap A:B.
        let up = |digits| round_decimal(123.456, digits, RoundMode::Up);
        let down = |digits| round_decimal(123.456, digits, RoundMode::Down);
        assert_eq!(up(65_536.0), Some(124.0));
        assert_eq!(down(65_537.0), Some(123.4));
        assert_eq!(up(-65_537.0), Some(130.0));
        assert_eq!(down(-65_536.0), Some(123.0));
        assert_eq!(down(-65_535.0), Some(0.0));
        assert_eq!(up(-65_535.0), None);
        assert_eq!(down(2_147_483_649.0), Some(123.4));
        assert_eq!(up(-2_147_483_650.0), Some(200.0));
        assert_eq!(down(4_294_901_762.0), Some(123.45));
        assert_eq!(up(4_294_967_295.9), Some(123.456));
        assert_eq!(up(4_294_967_298.0), Some(123.456));
        assert_eq!(down(1e300), Some(123.456));
        assert_eq!(up(-1e300), None);
        assert_eq!(
            round_decimal(0.5, 2_147_483_648.0, RoundMode::Down),
            Some(0.0)
        );
        assert_eq!(
            round_decimal(0.5, 2_147_483_648.0, RoundMode::Up),
            Some(1.0)
        );
    }

    /// 2^-1022 * (1 + k * 2^-52), the k-th double above the smallest normal.
    fn above_smallest_normal(k: u32) -> f64 {
        f64::MIN_POSITIVE * (1.0 + f64::from(k) * f64::EPSILON)
    }

    /// probes/t2756-roundup-smallest-normal-threshold: up to k = 7 the 15-digit cut
    /// (2.22507385850720E-308) is subnormal and ROUNDUP/ROUNDDOWN return it; from k = 8 it is
    /// normal and ROUNDUP takes its step.
    #[test]
    fn subnormal_cut_is_returned_by_roundup_and_rounddown() {
        // Rows 52-58 read it back as 2.8206162122887925E-278 through `*2^100`.
        let cut = 2.820_616_212_288_792_5e-278_f64 / 2f64.powi(100);
        assert_eq!(cut, SMALLEST_NORMAL_CUT);
        for k in [0, 7] {
            let x = above_smallest_normal(k);
            // -400, ±1E10, ±2^31 and 2^53 too: no `#NUM!` and no wrap (t2771-roundup-subnormal-cuts rows 22-33).
            for digits in [
                0.0,
                -2.0,
                2.0,
                300.0,
                307.0,
                330.0,
                -400.0,
                1e10,
                -1e10,
                2_147_483_648.0,
                -2_147_483_648.0,
                9_007_199_254_740_992.0,
            ] {
                assert_eq!(round_function(x, digits, RoundMode::Up), Some(cut));
                assert_eq!(round_function(-x, digits, RoundMode::Down), Some(-cut));
            }
            // ROUND rounds the cut and flushes a subnormal result.
            assert_eq!(
                round_function(x, 330.0, RoundMode::HalfAwayFromZero),
                Some(0.0)
            );
            assert_eq!(
                round_function(x, 310.0, RoundMode::HalfAwayFromZero),
                Some(2.23e-308)
            );
        }
        let x = above_smallest_normal(8);
        assert_eq!(round_function(x, 0.0, RoundMode::Up), Some(1.0));
        assert_eq!(round_function(-x, -2.0, RoundMode::Up), Some(-100.0));
        assert_eq!(round_function(x, 300.0, RoundMode::Up), Some(1e-300));
        assert_eq!(round_function(x, 307.0, RoundMode::Up), Some(1e-307));
    }

    /// The rounded-down digits and ROUNDUP's unit are each flushed when subnormal (rows 44-50).
    #[test]
    fn subnormal_parts_are_flushed() {
        let x = above_smallest_normal(8);
        for digits in [308.0, 309.0, 315.0, 321.0] {
            assert_eq!(round_decimal(x, digits, RoundMode::Up), Some(0.0));
            assert_eq!(round_decimal(x, digits, RoundMode::Down), Some(0.0));
        }
        assert_eq!(
            round_decimal(x, 322.0, RoundMode::Up),
            Some(2.225_073_858_507_21e-308)
        );
    }

    /// k·2^-1074: a subnormal.
    fn subnormal(k: u64) -> f64 {
        f64::from_bits(k)
    }

    /// probes/t2771-roundup-subnormal-cuts and -sign: from 2^-1026 up, ROUNDUP, ROUNDDOWN and TRUNC
    /// return a subnormal as it is (no 15-digit cut), and ROUND flushes one from 2^-1026 up.
    #[test]
    fn subnormal_from_two_to_minus_1026_is_kept_by_roundup_and_rounddown() {
        // 2^-1021 - 2^-1022*(1.1), whose 15-digit cut is a different double.
        let x = 2f64.powi(-1021) - f64::MIN_POSITIVE * 1.1;
        assert!(x.is_subnormal() && x.abs() >= TINY_SUBNORMAL_BELOW);
        for digits in [0.0, 2.0, 330.0] {
            assert_eq!(round_function(x, digits, RoundMode::Up), Some(x));
            assert_eq!(round_function(x, digits, RoundMode::Down), Some(x));
            assert_eq!(round_function(-x, digits, RoundMode::Up), Some(-x));
        }
        for digits in [0.0, 320.0, 330.0] {
            assert_eq!(
                round_function(x, digits, RoundMode::HalfAwayFromZero),
                Some(0.0)
            );
        }
    }

    /// probes/t2771-roundup-subnormal-threshold and -sign-controls: ROUNDUP, ROUNDDOWN and TRUNC
    /// keep a subnormal of either sign at any size; ROUND keeps a positive one below 2^-1026 only.
    #[test]
    fn tiny_subnormals_keep_their_sign_and_round_keeps_positive_ones_below_two_to_minus_1026() {
        for k in [1, 12_345, (1 << 48) - 1, 1 << 48, (1 << 50) + 3] {
            let x = subnormal(k);
            for mode in [RoundMode::Up, RoundMode::Down] {
                for digits in [0.0, 2.0, 330.0, -400.0, 2_147_483_648.0, 1e10] {
                    assert_eq!(round_function(x, digits, mode), Some(x));
                    assert_eq!(round_function(-x, digits, mode), Some(-x));
                }
            }
            let kept = if k < 1 << 48 { x } else { 0.0 };
            for digits in [0.0, 320.0, 330.0] {
                assert_eq!(
                    round_function(x, digits, RoundMode::HalfAwayFromZero),
                    Some(kept)
                );
                assert_eq!(
                    round_function(-x, digits, RoundMode::HalfAwayFromZero),
                    Some(0.0)
                );
            }
        }
    }
}
