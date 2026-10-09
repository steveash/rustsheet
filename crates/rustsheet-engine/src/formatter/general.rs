//! The General number format: fixed or E notation (T-0938).

/// Decimals General keeps in fixed notation: "0." plus 9 decimals fills the 11 characters Excel
/// allows below 1 (poi/GeneralFormatTests A27 `1E-09` → "0.000000001", A61 `-1E-09` →
/// "-0.000000001": the sign does not count). From 1 on the engine keeps 9 decimals as well;
/// whether the integer digits take their share of the 11 characters is unprobed (T-1225).
const GENERAL_DECIMALS: usize = 9;

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
        let fixed = format!("{value:.GENERAL_DECIMALS$}");
        let fixed = fixed.trim_end_matches('0').trim_end_matches('.');
        let mantissa = scientific.split('E').next().unwrap_or_default();
        if significant_digits(fixed) >= significant_digits(mantissa) {
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
