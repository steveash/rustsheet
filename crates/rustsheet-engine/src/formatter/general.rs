//! The General number format: fixed or E notation (T-0938).

use crate::decimal_round::{round_decimal, RoundMode};

/// Characters General's fixed notation may use, the sign excluded and the decimal point counted:
/// "0." plus 9 decimals below 1 (poi/GeneralFormatTests A27 `1E-09` → "0.000000001", A61 `-1E-09`
/// → "-0.000000001"); from 1 on the integer digits take their share, so `123456.123456789` is
/// "123456.1235" (probes/text-general-ties-wide-fixed, docs/excel-semantics/number-formats.md).
const GENERAL_WIDTH: usize = 11;

/// Decimals the fixed form of `magnitude` (non-zero, below 1E+11) may keep.
fn fixed_decimals(magnitude: f64) -> usize {
    if magnitude < 1.0 {
        return GENERAL_WIDTH - 2;
    }
    // The integer part has `floor(log10) + 1` digits; the decimal point takes one more character.
    let integer_digits = magnitude.log10().floor() as usize + 1;
    GENERAL_WIDTH.saturating_sub(integer_digits + 1)
}

/// `value` under the General format, with "." as the decimal point.
///
/// General prints fixed notation when `1E-09 <= |value| < 1E+11` and the fixed form shows at least
/// as many significant digits as E notation would; otherwise it prints E notation. So `1E-09` is
/// "0.000000001" but `1E-10` is "1E-10", and `1.23456789012345E-05` is "1.23457E-05" because the
/// fixed "0.000012346" keeps 5 digits against 6 (probes/number-text-exponent-digits D8,
/// poi/GeneralFormatTests A13-A33; `docs/excel-semantics/number-formats.md`).
pub(crate) fn format_general(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let scientific = format_general_scientific(value);
    // Below 1E-09 General is E notation even when the fixed form would show the same digits:
    // `9.9999996E-10` is "1E-09", not "0.000000001" (probes/text-general-exponent-boundary A10).
    // From 1E+11 the integer part alone needs 12 characters (A18 `1E11` → "1E+11").
    if (1e-9..1e11).contains(&value.abs()) {
        let decimals = fixed_decimals(value.abs());
        // Excel rounds the 15-digit decimal half away from zero (probes/text-general-ties-wide-fixed
        // A18-A24: `12345678902.5` → "12345678903", `123456.12345` → "123456.1235"), as ROUND does.
        let rounded =
            round_decimal(value, decimals as f64, RoundMode::HalfAwayFromZero).unwrap_or(value);
        let fixed = format!("{rounded:.decimals$}");
        // Integer-only text ("10000000000") has no decimal point, so its zeros are not trailing.
        let fixed = if fixed.contains('.') {
            fixed.trim_end_matches('0').trim_end_matches('.')
        } else {
            &fixed
        };
        let mantissa = scientific.split('E').next().unwrap_or_default();
        // A value that rounds up to 1E+11 needs 12 characters (`99999999999.6` → "1E+11").
        if rounded.abs() < 1e11 && significant_digits(fixed) >= significant_digits(mantissa) {
            return fixed.to_string();
        }
    }
    scientific
}

/// `value` (non-zero) in General's E notation: "1.23457E-05", "1.2346E+100".
fn format_general_scientific(mut value: f64) -> String {
    let mut exponent = value.abs().log10().floor();
    value /= 10.0_f64.powf(exponent);
    // One digit fewer from a decimal exponent magnitude of 99 on: "1.2346E+100" against
    // "1.23457E-05" (probes/number-text-exponent-digits D1:D15, numbers.md "Exponent width").
    let decimals_for = |exponent: f64| if exponent.abs() >= 99.0 { 4 } else { 5 };
    let mut decimals = decimals_for(exponent);
    let mut s = format!("{value:.decimals$}");
    // The mantissa is in [1, 10), so only rounding can print "10.". Such a mantissa carries into
    // the exponent: 9.9999996E+98 prints "1E+99", not "10E+98"
    // (probes/text-general-exponent-boundary A5-A8, A10).
    if s.trim_start_matches('-').starts_with("10.") {
        exponent += 1.0;
        value = value.signum();
        decimals = decimals_for(exponent);
        s = format!("{value:.decimals$}");
    }
    let sign = if exponent < 0.0 { '-' } else { '+' };
    format!(
        "{}E{}{:02}",
        s.trim_end_matches('0').trim_end_matches('.'),
        sign,
        exponent.abs()
    )
}

/// The number of significant digits in a plain decimal string: "-0.000012346" → 5, "1200" → 4.
fn significant_digits(text: &str) -> usize {
    text.chars()
        .filter(char::is_ascii_digit)
        .skip_while(|c| *c == '0')
        .count()
}
