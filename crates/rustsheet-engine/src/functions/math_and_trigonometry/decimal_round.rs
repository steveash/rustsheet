//! Decimal-correct rounding for ROUND, ROUNDUP and ROUNDDOWN.
//!
//! Excel rounds in two decimal stages, never by scaling with a binary power of ten
//! (docs/excel-semantics/numbers.md, "ROUND family rounds in two decimal stages"; evidence
//! probes/round-family and probes/precision-and-rounding A6–A8, oracle excel-graph):
//! 1. the exact binary value is cut to 15 significant digits. An exact tie at the 16th digit goes
//!    toward zero for ROUND (`ROUND(999999999999957.5,0)` = 999999999999957) but away from zero
//!    for ROUNDUP and ROUNDDOWN (`ROUNDDOWN(999999999999957.5,0)` = 999999999999958);
//! 2. that 15-digit decimal is rounded at `num_digits` (half away from zero, up or down), and the
//!    decimal result is converted to the nearest double.
//!
//! So `ROUND(1.005,2)` is 1.01 (1.00499999999999989… becomes 1.00500000000000 first) and
//! `ROUNDDOWN(123456789.123,-5)` is exactly 123400000.

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
/// More significant digits than any double's exact decimal expansion (at most 767).
const EXACT_DIGITS: usize = 800;
/// `num_digits` beyond this magnitude rounds every finite double the same way, so clamping it
/// keeps the exponent arithmetic small without changing any result.
const MAX_NUM_DIGITS: f64 = 10_000.0;

/// Rounds `value` at `num_digits` decimal places the way Excel's ROUND family does.
///
/// `num_digits` is truncated toward zero. Returns `None` when the result is beyond the double
/// range (`ROUNDUP(1.5,-400)` is `#NUM!` in Excel).
pub(crate) fn round_decimal(value: f64, num_digits: f64, mode: RoundMode) -> Option<f64> {
    if value == 0.0 || !value.is_finite() {
        return Some(value);
    }
    // Truncation toward zero, as Excel does for a fractional num_digits; the clamp keeps the
    // cast exact (NaN cannot reach here from a cell, but would become 0).
    let num_digits = num_digits.clamp(-MAX_NUM_DIGITS, MAX_NUM_DIGITS).trunc() as i64;
    let tie_away = mode != RoundMode::HalfAwayFromZero;
    let (mantissa, exponent) = significant_15(value.abs(), tie_away);
    // `mantissa` has 15 digits; its last digit has place value 10^(exponent - 14).
    let last_place = exponent - (SIGNIFICANT_DIGITS - 1);
    let dropped = -num_digits - last_place;
    let (digits, place) = if dropped <= 0 {
        (mantissa, last_place)
    } else {
        (round_off(mantissa, dropped, mode), -num_digits)
    };
    let magnitude: f64 = format!("{digits}e{place}").parse().ok()?;
    if magnitude.is_infinite() {
        return None;
    }
    Some(magnitude.copysign(value))
}

/// Drops the last `dropped` (> 0) digits of the 15-digit `mantissa`, rounding per `mode`.
fn round_off(mantissa: u64, dropped: i64, mode: RoundMode) -> u64 {
    let Ok(dropped) = u32::try_from(dropped) else {
        return u64::from(mode == RoundMode::Up);
    };
    let Some(unit) = 10_u64.checked_pow(dropped) else {
        // Everything is dropped and the value is below half a unit.
        return u64::from(mode == RoundMode::Up);
    };
    let (kept, rest) = (mantissa / unit, mantissa % unit);
    let round_up = match mode {
        RoundMode::HalfAwayFromZero => rest >= unit - rest,
        RoundMode::Up => rest > 0,
        RoundMode::Down => false,
    };
    kept + u64::from(round_up)
}

/// The exact decimal value of positive finite `value` cut to 15 significant digits:
/// `(mantissa, exponent)` with `value ≈ mantissa / 10^14 * 10^exponent` and
/// `10^14 <= mantissa < 10^15`. Digits past the 15th round to nearest; an exact tie goes away
/// from zero when `tie_away`, otherwise toward zero.
fn significant_15(value: f64, tie_away: bool) -> (u64, i64) {
    // 25 correctly rounded digits decide every case except a possible exact tie, which needs
    // the exact expansion (rare: only values with a 5 as 16th digit and zeros after it).
    let (mut digits, mut exponent) = scientific_digits(value, 25);
    if is_possible_tie(&digits) {
        (digits, exponent) = scientific_digits(value, EXACT_DIGITS);
    }
    let lead = digits.get(..15).unwrap_or_default();
    let mut mantissa = lead
        .iter()
        .fold(0_u64, |acc, d| acc * 10 + u64::from(d - b'0'));
    let rest = digits.get(15..).unwrap_or_default();
    let round_up = match rest.split_first() {
        Some((&first, _)) if first > b'5' => true,
        Some((&b'5', tail)) => tie_away || tail.iter().any(|&d| d != b'0'),
        _ => false,
    };
    if round_up {
        mantissa += 1;
        if mantissa == 1_000_000_000_000_000 {
            mantissa /= 10;
            exponent += 1;
        }
    }
    (mantissa, exponent)
}

/// True when the 16th significant digit is 5 and every later printed digit is 0.
fn is_possible_tie(digits: &[u8]) -> bool {
    digits.get(15) == Some(&b'5') && digits.iter().skip(16).all(|&d| d == b'0')
}

/// The first `count` significant digits of positive finite `value` (correctly rounded) and its
/// decimal exponent.
fn scientific_digits(value: f64, count: usize) -> (Vec<u8>, i64) {
    let text = format!("{:.*e}", count - 1, value);
    let (mantissa, exponent) = text.split_once('e').unwrap_or((&text, "0"));
    let digits = mantissa.bytes().filter(u8::is_ascii_digit).collect();
    (digits, exponent.parse().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{round_decimal, RoundMode};

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
        assert_eq!(round(1.5, f64::MAX), Some(1.5));
        assert_eq!(round(1.5, f64::MIN), Some(0.0));
        assert_eq!(round(1.234_567_890_123_45e300, -299.0), Some(1.2e300));
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
}
