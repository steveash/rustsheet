//! Mixed-fraction number text (`3 1/2`), shared by VALUE, coercion, typed input and criteria.

/// A mixed fraction `w n/d` (ASCII digits, one space) as `w + n/d`, with an optional sign (spaces
/// may follow it), a leading symbol from `currencies` and a trailing `%`.
/// Excel: `VALUE("3 1/2")` is 3.5, `"-3 1/2"` and `"- 3 1/2"` -3.5, `"$3 1/2"` 3.5, `"3 1/2%"`
/// 0.034999999999999996 (the whole and the fraction divided by 100 apart), `"1 3/2"` 2.5 and
/// `"3 01/02"` 3.5; `"3 1/0"`, `"3  1/2"`, `"3 1 /2"` and `"3 1/ 2"` are #VALUE!
/// (docs/excel-semantics/text.md "VALUE reads a mixed fraction", probes/value-sign-space-mixed-
/// fraction-dash-date A7-A12, A24-A35, probes/eng-inumber-date-boundary B22).
/// A currency symbol and a `%` together are `#VALUE!` (`"$3 1/2%"`, probes/typed-input-sign-space-
/// mixed-fraction F8); the signed `"-$3 1/2%"` is text too (probes/typed-mixed-fraction-number-format A15).
pub(crate) fn mixed_fraction(text: &str, currencies: &[&str]) -> Option<f64> {
    let (sign, rest) = match text.as_bytes().first()? {
        b'-' => (-1.0, text[1..].trim_start_matches(' ')),
        b'+' => (1.0, text[1..].trim_start_matches(' ')),
        _ => (1.0, text),
    };
    let unprefixed = currencies.iter().find_map(|c| rest.strip_prefix(c));
    let rest = unprefixed.unwrap_or(rest);
    let (rest, percent) = match rest.strip_suffix('%') {
        Some(rest) => (rest, true),
        None => (rest, false),
    };
    if percent && unprefixed.is_some() {
        return None;
    }
    let (whole, fraction) = rest.split_once(' ')?;
    let (numerator, denominator) = fraction.split_once('/')?;
    let digits = |part: &str| -> Option<f64> {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        part.parse::<f64>().ok()
    };
    let (whole, numerator, denominator) =
        (digits(whole)?, digits(numerator)?, digits(denominator)?);
    if denominator == 0.0 {
        return None;
    }
    let fraction = numerator / denominator;
    let value = if percent {
        whole / 100.0 + fraction / 100.0
    } else {
        whole + fraction
    };
    value.is_finite().then_some(sign * value)
}
