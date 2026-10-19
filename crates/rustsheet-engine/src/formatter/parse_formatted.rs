//! Reading typed text as a number together with the format it implies ("$1,234.50" ->
//! (1234.5, "$#,##0.00")), split from format.rs, which renders numbers.

use crate::{
    functions::date_and_time::time_text::parse_date_clock_serial_in, locale::Locale,
    number_format::parse_excel_number, types::DateSystem,
};

use super::{
    date_text::{parse_date, CurrentYear},
    lexer::is_likely_date_number_format,
};

/// Format Excel gives typed date+time text (probes/date-text-glued-month-2 N8, excel-graph
/// 2026-10-18: `July1 10:00` -> 46204.41666..., `m/d/yyyy h:mm`). Non-glued dates and clocks with
/// seconds or AM/PM get the same format (probes/date-text-non-glued-timevalue, T-1952;
/// docs/research/non-glued-typed-input-2026-10-19.md).
const TYPED_DATE_CLOCK_FORMAT: &str = "m/d/yyyy h:mm";

/// [`parse_formatted_number`] for text typed into a cell, which also reads date+time text
/// (coercion reads that through `parse_time_serial` instead).
pub(crate) fn parse_typed_number(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Result<(f64, Option<String>), String> {
    parse_formatted_number(original, currencies, locale, current_year, date_system).or_else(|err| {
        let Some(shape) = typed_clock_shape(original) else {
            return Err(err);
        };
        parse_date_clock_serial_in(original, locale, current_year, date_system)
            .map(|serial| (serial, shape.format()))
            .ok_or(err)
    })
}

/// Cell format of typed text that stays text because its clock is rejected: a date followed by a
/// number-like clock and a spaced `AM`/`PM` (`1/2/2020 13:00 PM`, `13 PM`, `1300 PM`, `99:00 PM`)
/// carries the format of its date part (`m/d/yyyy`, `d-mmm-yy` for `July 1, 2026 13:00 PM`), while
/// a glued `10:00AM`, `10:00 XM`, `10:00 PM PM`, a clock with no date and `1/2/2020 abc` stay
/// General (probes/date-text-typed-clock-shapes N14-N15, date-text-rejected-clock-formats N1-N16;
/// formats in docs/research/rejected-clock-text-formats-2026-10-19.md).
pub(crate) fn rejected_clock_text_format(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Option<String> {
    let (rest, word) = original.rsplit_once(' ')?;
    if !(word.eq_ignore_ascii_case("am") || word.eq_ignore_ascii_case("pm")) {
        return None;
    }
    let rest = rest.trim_end_matches(' ');
    let (date, clock) = rest.rsplit_once(' ')?;
    if !clock.bytes().all(|b| b.is_ascii_digit() || b == b':')
        || !clock.bytes().any(|b| b.is_ascii_digit())
    {
        return None;
    }
    parse_formatted_number(
        date.trim_end_matches(' '),
        currencies,
        locale,
        current_year,
        date_system,
    )
    .ok()
    .and_then(|(_, format)| format)
    // Only a date part counts: a percent, currency or grouped number before the clock is unprobed.
    .filter(|format| is_likely_date_number_format(format))
    // An ISO date gets `m/d/yyyy` here (N10), not the `yyyy-mm-dd` the engine gives a bare typed
    // ISO date (bare typed ISO dates are unprobed).
    .map(|format| {
        if format == "yyyy-mm-dd" {
            "m/d/yyyy".to_string()
        } else {
            format
        }
    })
}

/// Number format a typed clock shape gets (`None` is General).
enum TypedClock {
    /// `H:MM[:SS]` with an in-range value, with or without `AM`/`PM`, or hour-only `H AM`.
    Time,
    /// A fractional second (`H:MM:SS.f`, `M:SS.f`).
    Fraction,
    /// A component out of its usual range (hour from 24, minute or second from 60).
    OutOfRange,
}

impl TypedClock {
    fn format(&self) -> Option<String> {
        match self {
            Self::Time => Some(TYPED_DATE_CLOCK_FORMAT.to_string()),
            Self::Fraction => Some("mm:ss.0".to_string()),
            Self::OutOfRange => None,
        }
    }
}

/// Classifies the clock at the end of typed date+clock text: `H:MM`, `H:MM:SS`, `H:MM:SS.f` or
/// `M:SS.f`, optionally followed by spaces and `AM`/`PM` in any case, or an hour-only `H AM`.
/// Probed (probes/date-text-non-glued-timevalue rows 11-17, date-text-typed-clock-shapes,
/// excel-graph; formats in docs/research/typed-clock-shapes-2026-10-19.md): all of these become
/// numbers, with `m/d/yyyy h:mm` except a fraction (`mm:ss.0`) and an out-of-range component
/// (General); a glued `10:00AM` and `13:00 PM` stay text.
fn typed_clock_shape(text: &str) -> Option<TypedClock> {
    let mut text = text.trim_end_matches(' ');
    let mut suffixed = false;
    if let Some((rest, word)) = text.rsplit_once(' ') {
        if word.eq_ignore_ascii_case("am") || word.eq_ignore_ascii_case("pm") {
            suffixed = true;
            text = rest.trim_end_matches(' ');
        }
    }
    let clock = text.rsplit(' ').next().unwrap_or("");
    let digits =
        |s: &str, max: usize| (1..=max).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit());
    if suffixed && digits(clock, 2) && clock != text {
        return Some(TypedClock::Time);
    }
    let (clock, fraction) = match clock.split_once('.') {
        Some((head, f)) if digits(f, 9) => (head, true),
        Some(_) => return None,
        None => (clock, false),
    };
    let mut parts = clock.split(':');
    let (Some(first), Some(second), third, None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    let two = |s: &str| s.len() == 2 && digits(s, 2);
    if !digits(first, 2) || !two(second) || third.is_some_and(|s| !two(s)) {
        return None;
    }
    let value = |s: &str| s.parse::<u32>().unwrap_or(u32::MAX);
    let (hour, minute, sec) = match third {
        Some(s) => (value(first), value(second), value(s)),
        None if fraction => (0, value(first), value(second)),
        None => (value(first), value(second), 0),
    };
    if fraction {
        Some(TypedClock::Fraction)
    } else if hour >= 24 || minute >= 60 || sec >= 60 {
        Some(TypedClock::OutOfRange)
    } else {
        Some(TypedClock::Time)
    }
}

/// Parses a formatted number, returning the numeric value together with the format
/// Uses heuristics to guess the format string
/// "$ 123,345.678" => (123345.678, "$#,##0.00")
/// "30.34%" => (0.3034, "0.00%")
/// 100€ => (100, "100€")
pub(crate) fn parse_formatted_number(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Result<(f64, Option<String>), String> {
    let value = original.trim();

    let symbols = &locale.numbers.symbols;
    let decimal_separator = symbols.decimal.chars().next().unwrap_or('.');
    let group_separator = symbols.group.chars().next().unwrap_or(',');

    if let Some(parsed) =
        parse_percent_or_currency(value, currencies, decimal_separator, group_separator)?
    {
        return Ok(parsed);
    }

    // Date text, untrimmed: number coercion rejects date text with surrounding spaces (see
    // `parse_date`); VALUE trims before calling.
    // The parser counts from the 1900 epoch; a 1904 workbook counts from 1904-01-01
    // (`="1904-01-02"+0` is 1, probes/date-system-1904-readers A20).
    if let Ok((serial_number, format)) = parse_date(original, locale, current_year) {
        if let Some(serial) = date_system.serial_from_1900_serial(i64::from(serial_number)) {
            return Ok((serial as f64, Some(format)));
        }
    }

    // Excel reads accounting text in parentheses as negative: `0+"(1)"` = -1, `0+"(1,000)"` =
    // -1000, `0+"( 1 )"` = -1, `0+"($1)"` = -1, `0+"(1%)"` = -0.01, but `0+"-(1)"`, `0+"(-1)"` and
    // `0+"((1))"` are #VALUE! (docs/excel-semantics/coercion.md "Parenthesised number text is
    // negative", probes/paren-negative-variants).
    if let Some(inner) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
        let inner = inner.trim();
        // Inside parentheses a currency symbol is only valid directly before unsigned digits:
        // `($-1)`, `($ -1)` and `(1$)` are #VALUE! (probes/paren-number-text-forms), as are `($+1)`,
        // `(1 $)` and `(€-1)`; only a trailing euro sign is valid: `(1€)` = -1
        // (probes/paren-number-text-currency-variants).
        let bad_currency = currencies.iter().any(|c| {
            (!c.is_empty() && *c != "€" && inner.strip_suffix(c).is_some())
                || inner
                    .strip_prefix(c)
                    .is_some_and(|rest| rest.trim_start().starts_with(['-', '+']))
        });
        if inner.starts_with(['-', '+', '(']) || bad_currency {
            return Err("Cannot parse number".to_string());
        }
        let (f, format) =
            match parse_percent_or_currency(inner, currencies, decimal_separator, group_separator)?
            {
                Some(parsed) => parsed,
                None => parse_plain_number(inner, decimal_separator, group_separator)?,
            };
        return Ok((-f, format));
    }
    parse_plain_number(value, decimal_separator, group_separator)
}

/// Format given to scientific-notation text such as `1E3`.
const SCIENTIFIC_FORMAT: &str = "0.00E+00";

/// A number written with a percent sign or a currency symbol, with the format that implies.
/// `Ok(None)` means the text has neither.
fn parse_percent_or_currency(
    value: &str,
    currencies: &[&str],
    decimal_separator: char,
    group_separator: char,
) -> Result<Option<(f64, Option<String>)>, String> {
    // Check if it is a percentage. Excel types `1%` as `0%` and `1.5%` as `0.00%` (the decimals
    // are not kept beyond two), scientific text as `0.00E+00`; the format ignores group
    // separators (`1,000%` is `0%`). Probes: typed-currency-percent-input, typed-paren-number-input;
    // docs/excel-semantics/coercion.md "Typed currency and percent formats".
    if let Some(p) = value.strip_suffix('%') {
        let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
        if options.is_scientific {
            return Ok(Some((f / 100.0, Some(SCIENTIFIC_FORMAT.to_string()))));
        }
        let format = if options.decimal_digits > 0 {
            "0.00%"
        } else {
            "0%"
        };
        return Ok(Some((f / 100.0, Some(format.to_string()))));
    }

    // check if it is a currency in currencies
    for currency in currencies {
        let prefixed = value
            .strip_prefix(&format!("-{currency}"))
            .map(|p| (p, -1.0))
            .or_else(|| value.strip_prefix(currency).map(|p| (p, 1.0)));
        if let Some((p, sign)) = prefixed {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            // Excel: `-$1e3` is -1000 like `$-1e3` (typed-currency-percent-input A29).
            let f = sign * f;
            if options.is_scientific {
                return Ok(Some((f, Some(SCIENTIFIC_FORMAT.to_string()))));
            }
            return Ok(Some((
                f,
                Some(prefix_currency_format(currency, options.decimal_digits > 0)),
            )));
        } else if let Some(p) = value.strip_suffix(currency) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok(Some((f, Some(SCIENTIFIC_FORMAT.to_string()))));
            }
            if options.decimal_digits > 0 {
                let currency_format = &format!("#,##0.00{currency}");
                return Ok(Some((f, Some(currency_format.to_string()))));
            }
            let currency_format = &format!("#,##0{currency}");
            return Ok(Some((f, Some(currency_format.to_string()))));
        }
    }
    Ok(None)
}

/// The format Excel gives text typed with a leading currency symbol. `$` gets the red-negative
/// accounting-style format (`$1` -> `$#,##0_);[Red]($#,##0)`, `$1.5` -> `$#,##0.00_);[Red]($#,##0.00)`;
/// typed-currency-percent-input). Other symbols are unprobed and keep the plain format.
fn prefix_currency_format(currency: &str, decimals: bool) -> String {
    let digits = if decimals { "#,##0.00" } else { "#,##0" };
    if currency == "$" {
        format!("${digits}_);[Red](${digits})")
    } else {
        format!("{currency}{digits}")
    }
}

/// A plain number (optionally with group separators or an exponent) and its inferred format.
fn parse_plain_number(
    value: &str,
    decimal_separator: char,
    group_separator: char,
) -> Result<(f64, Option<String>), String> {
    let (f, options) = parse_number(value, decimal_separator, group_separator)?;
    if options.is_scientific {
        return Ok((f, Some(SCIENTIFIC_FORMAT.to_string())));
    }
    if options.has_commas {
        if options.decimal_digits > 0 {
            // group separator and two decimal points
            return Ok((f, Some("#,##0.00".to_string())));
        }
        // Group separator and no decimal points
        return Ok((f, Some("#,##0".to_string())));
    }
    Ok((f, None))
}

struct NumberOptions {
    has_commas: bool,
    is_scientific: bool,
    decimal_digits: usize,
}

// tries to parse 'value' as a number.
// If it is a number it either uses commas as thousands separator or it does not
fn parse_number(
    value: &str,
    decimal_separator: char,
    group_separator: char,
) -> Result<(f64, NumberOptions), String> {
    let mut position = 0;
    let characters: Vec<char> = value.chars().collect();
    let len = characters.len();
    if len == 0 {
        return Err("Cannot parse number".to_string());
    }
    let mut chars = String::from("");
    let mut group_separator_index = Vec::new();
    // get the sign
    let sign = if characters[0] == '-' {
        position += 1;
        -1.0
    } else if characters[0] == '+' {
        position += 1;
        1.0
    } else {
        1.0
    };

    if position >= len {
        return Err("Cannot parse number".to_string());
    }

    if characters[position] == group_separator {
        return Err("Cannot parse number".to_string());
    }

    // numbers before the decimal point
    while position < len {
        let x = characters[position];
        if x.is_ascii_digit() {
            chars.push(x);
        } else if x == group_separator {
            group_separator_index.push(chars.len());
        } else {
            break;
        }
        position += 1;
    }
    // Check the group separator is in multiples of three
    for index in &group_separator_index {
        if (chars.len() - index) % 3 != 0 {
            return Err("Cannot parse number".to_string());
        }
    }
    let mut decimal_digits = 0;
    if position < len && characters[position] == decimal_separator {
        // numbers after the decimal point
        chars.push('.');
        position += 1;
        let start_position = 0;
        while position < len {
            let x = characters[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        decimal_digits = position - start_position;
    }
    let mut is_scientific = false;
    if position + 1 < len && (characters[position] == 'e' || characters[position] == 'E') {
        // exponential side
        is_scientific = true;
        let x = characters[position + 1];
        if x == '-' || x == '+' || x.is_ascii_digit() {
            chars.push('e');
            chars.push(x);
            position += 2;
            while position < len {
                let x = characters[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
    }
    if position != len {
        return Err("Could not parse number".to_string());
    };
    match parse_excel_number(&chars) {
        None => Err("Failed to parse to double".to_string()),
        Some(v) => Ok((
            sign * v,
            NumberOptions {
                has_commas: !group_separator_index.is_empty(),
                is_scientific,
                decimal_digits,
            },
        )),
    }
}
