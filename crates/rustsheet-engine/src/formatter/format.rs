use super::elapsed::{elapsed_text, ElapsedUnit};
use super::general::format_general;
use crate::{
    locale::Locale,
    number_format::{parse_excel_number, to_precision},
    types::DateSystem,
};

use super::{
    date_text::{parse_date, CurrentYear},
    dates::{excel_serial_to_ymd, negative_serial_allowed},
    parser::{GeneralPart, ParsePart, Parser, TextToken},
};

pub struct Formatted {
    pub color: Option<i32>,
    pub text: String,
    pub error: Option<String>,
}

/// The workbook inputs of number formatting: the locale and the date system (SPEC-0005).
#[derive(Clone, Copy)]
pub struct FormatContext<'a> {
    pub locale: &'a Locale,
    pub date_system: DateSystem,
}

impl<'a> FormatContext<'a> {
    /// A context in the default (1900) date system, for formatting outside any workbook.
    pub fn new(locale: &'a Locale) -> Self {
        FormatContext {
            locale,
            date_system: DateSystem::default(),
        }
    }
}

/// Returns the vector of chars of the fractional part of a *positive* number:
/// 3.1415926 ==> ['1', '4', '1', '5', '9', '2', '6']
fn get_fract_part(value: f64, precision: i32, int_len: usize) -> Vec<char> {
    let b = format!("{:.1$}", value.fract(), precision as usize)
        .chars()
        .collect::<Vec<char>>();
    let l = b.len() - 1;
    let mut last_non_zero = b.len() - 1;
    for i in 0..l {
        if b[l - i] != '0' {
            last_non_zero = l - i + 1;
            break;
        }
    }
    if last_non_zero < 2 {
        return vec![];
    }
    let max_len = if int_len > 15 {
        2_usize
    } else {
        15_usize - int_len + 1
    };
    let last_non_zero = usize::min(last_non_zero, max_len + 1);
    b[2..last_non_zero].to_vec()
}

/// Return true if we need to add a separator in position `digit_index`
/// It normally happens if if `digit_index` -1 is 3, 6, 9,... `digit_index` ≡ 1 mod 3
fn use_group_separator(use_thousands: bool, digit_index: i32, group_sizes: &str) -> bool {
    if use_thousands {
        if group_sizes == "#,##0.###" {
            if digit_index > 1 && (digit_index - 1) % 3 == 0 {
                return true;
            }
        } else if group_sizes == "#,##,##0.###"
            && (digit_index == 3 || (digit_index > 3 && digit_index % 2 == 0))
        {
            return true;
        }
    }
    false
}

/// Index into the Sunday-first day-name tables for a serial. Excel numbers weekdays as if serial 1
/// were a Sunday, so serial 0 is a Saturday and 60 (the fictitious 1900-02-29) a Wednesday
/// (probes/dates-early-1900-edges A26-A34); the same formula is the real weekday from 61 on.
fn excel_weekday_index(serial: i64) -> usize {
    (serial - 1).rem_euclid(7) as usize
}

fn invalid_date(value: f64) -> Formatted {
    Formatted {
        text: "#VALUE!".to_owned(),
        color: None,
        error: Some(format!("Invalid date value: '{value}'")),
    }
}

/// Formats a TEXT value, `None` being a blank. A blank with an empty format gives "" and any other
/// blank formats as 0 (see "An empty format string returns empty text" in
/// docs/excel-semantics/number-formats.md).
pub fn format_text(value: Option<f64>, format: &str, ctx: &FormatContext) -> Formatted {
    if value.is_none() && format.is_empty() {
        return Formatted {
            color: None,
            text: String::new(),
            error: None,
        };
    }
    format_number(value.unwrap_or(0.0), format, ctx)
}

/// Formats a text value (a boolean's "TRUE"/"FALSE" included) the way `TEXT` does: only the
/// 4th section applies, `@` stands for the text and the section's other tokens print as literals
/// (poi `TextFormatTests` A3:A10, A12:A29; docs/excel-semantics/number-formats.md "Text and
/// booleans use the format's text section"). A single section with `@` is a text section too
/// (`@" ""q"""` on "x" is "x q", probes/format-doubled-quotes C11). Any other format with fewer
/// than four sections returns the text unchanged; that case is unprobed. A code that does not
/// parse is `#VALUE!` (probes/text-format-length-limit-n, -o). A result over 255
/// characters is `#VALUE!` ([`MAX_TEXT_FORMAT_CHARS`]).
pub fn format_text_value(text: &str, format: &str) -> Formatted {
    let mut parser = Parser::new(format);
    parser.parse();
    // An unparseable code is `#VALUE!` for text too, even without a text section: `@_""` (the `_`
    // takes the first quote, leaving an unclosed one) and `@"` are `#VALUE!` on "x"
    // (probes/text-format-length-limit-n A5, -o A41; docs/excel-semantics/number-formats.md).
    if parser.parts.iter().any(ParsePart::is_error) {
        return format_error();
    }
    let section = match parser.parts.as_slice() {
        [only]
            if section_tokens(only)
                .is_some_and(|t| t.iter().any(|t| matches!(t, TextToken::Raw))) =>
        {
            0
        }
        parts if parts.len() < 4 => return capped_text(text.to_string()),
        _ => 3,
    };
    let tokens = match parser.parts.get(section) {
        Some(part) => match section_tokens(part) {
            Some(tokens) => tokens,
            None => return capped_text(text.to_string()),
        },
        None => return capped_text(text.to_string()),
    };
    let mut out = String::new();
    let mut chars = 0;
    for token in tokens {
        let start = out.len();
        match token {
            TextToken::Literal(c) => out.push(*c),
            TextToken::Text(t) => out.push_str(t),
            TextToken::Ghost(_) | TextToken::Spacer(_) => out.push(' '),
            TextToken::Raw => out.push_str(text),
            _ => {}
        }
        // Checked as it grows, so a format of many `@` cannot build a huge string.
        chars += out[start..].chars().count();
        if chars > MAX_TEXT_FORMAT_CHARS {
            return text_too_long();
        }
    }
    capped_text(out)
}

/// Excel: TEXT of a text value is `#VALUE!` once the result is over 255 characters, also when the
/// format leaves the text unchanged (probes/text-format-text-limit A10, A11, A19-A21, A24, A25).
const MAX_TEXT_FORMAT_CHARS: usize = 255;

/// The `#VALUE!` of a format code that does not parse.
fn format_error() -> Formatted {
    Formatted {
        text: "#VALUE!".to_owned(),
        color: None,
        error: Some("Problem parsing format string".to_owned()),
    }
}

fn text_too_long() -> Formatted {
    Formatted {
        text: "#VALUE!".to_owned(),
        color: None,
        error: Some("TEXT result over 255 characters".to_owned()),
    }
}

/// `text` as a TEXT result, or `#VALUE!` when it is over [`MAX_TEXT_FORMAT_CHARS`].
fn capped_text(text: String) -> Formatted {
    if text.chars().count() > MAX_TEXT_FORMAT_CHARS {
        return text_too_long();
    }
    Formatted {
        color: None,
        text,
        error: None,
    }
}

/// The tokens of a number or date section; `None` for General or an unparsable section.
fn section_tokens(part: &ParsePart) -> Option<&Vec<TextToken>> {
    match part {
        ParsePart::Number(p) => Some(&p.tokens),
        ParsePart::Date(p) => Some(&p.tokens),
        ParsePart::General(_) | ParsePart::Error(_) => None,
    }
}

pub fn format_number(value_original: f64, format: &str, ctx: &FormatContext) -> Formatted {
    let locale = ctx.locale;
    let mut parser = Parser::new(format);
    parser.parse();
    let parts = parser.parts;
    // There are four parts:
    // 1) Positive numbers
    // 2) Negative numbers
    // 3) Zero
    // 4) Text
    // If you specify only one section of format code, the code in that section is used for all numbers.
    // If you specify two sections of format code, the first section of code is used
    // for positive numbers and zeros, and the second section of code is used for negative numbers.
    // When you skip code sections in your number format,
    // you must include a semicolon for each of the missing sections of code.
    // You can use the ampersand (&) text operator to join, or concatenate, two values.
    let mut value = value_original;
    let general = ParsePart::General(GeneralPart {});
    let part;
    match parts.len() {
        // A single text section (`@` in it) does not apply to numbers, which print in General:
        // `TEXT(1,"@""aaa""")` is "1" (probes/text-format-length-limit-i A19:A26, T-1434).
        1 if section_tokens(&parts[0])
            .is_some_and(|t| t.iter().any(|t| matches!(t, TextToken::Raw))) =>
        {
            part = &general;
        }
        1 => {
            part = &parts[0];
        }
        2 => {
            if value >= 0.0 {
                part = &parts[0];
            } else {
                value = -value;
                part = &parts[1];
            }
        }
        3 => {
            if value > 0.0 {
                part = &parts[0];
            } else if value < 0.0 {
                value = -value;
                part = &parts[1];
            } else {
                value = 0.0;
                part = &parts[2];
            }
        }
        4 => {
            if value > 0.0 {
                part = &parts[0];
            } else if value < 0.0 {
                value = -value;
                part = &parts[1];
            } else {
                value = 0.0;
                part = &parts[2];
            }
        }
        _ => {
            return Formatted {
                text: "#VALUE!".to_owned(),
                color: None,
                error: Some("Too many parts".to_owned()),
            };
        }
    }
    match part {
        ParsePart::Error(..) => format_error(),
        ParsePart::General(..) => {
            let mut text = format_general(value);
            if locale.numbers.symbols.decimal != "." {
                text = text.replace('.', &locale.numbers.symbols.decimal.to_string());
            }
            Formatted {
                text,
                color: None,
                error: None,
            }
        }
        ParsePart::Date(p) => {
            // Excel: whether a negative serial formats or is #VALUE! depends on the value's own
            // sign, also when a negative section already dropped it (`TEXT(-1,"d;d")` is #VALUE!
            // in the 1900 system), and on the date system (`negative_serial_allowed`;
            // docs/excel-semantics/dates.md "The 1904 date system" and "Negative serials under
            // an explicit negative section").
            if value_original < 0.0 && !negative_serial_allowed(&p.tokens, ctx.date_system) {
                return invalid_date(value_original);
            }
            // A negative that gets here (a one-section format) is formatted as its magnitude after
            // one leading minus, in front of any literal (`TEXT(-1.5,"""x""[h]")` is "-x36",
            // probes/negative-date-sections-1900 A18; poi ElapsedFormatTests A40).
            let mut text = if value < 0.0 { "-" } else { "" }.to_string();
            let value = value.abs();
            let tokens = &p.tokens;
            let time_fract = value.fract();
            let hours = (time_fract * 24.0).floor();
            let minutes = ((time_fract * 24.0 - hours) * 60.0).floor();
            let seconds = ((((time_fract * 24.0 - hours) * 60.0) - minutes) * 60.0).round();
            // Day/month/year and weekday tokens show Excel's exact components (including the
            // fictitious 1900-02-29) and "1900-01-00" (serial 0). In a 1904 workbook they read the
            // 1904 epoch (`TEXT(0,"yyyy-mm-dd")` is "1904-01-01"; dates.md "The 1904 date system");
            // -1 stands for no date. The fallback is never read: `ymd` is `None` exactly when
            // `date_valid` is false.
            let serial_1900 = ctx.date_system.to_1900_serial(value as i64).unwrap_or(-1);
            let ymd = excel_serial_to_ymd(serial_1900);
            let date_valid = ymd.is_some();
            let (ymd_year, ymd_month, ymd_day) = ymd.unwrap_or((0, 1, 0));
            for token in tokens {
                match token {
                    TextToken::Literal(c) => {
                        text = format!("{text}{c}");
                    }
                    TextToken::Text(t) => {
                        text = format!("{text}{t}");
                    }
                    TextToken::Ghost(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Spacer(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Raw => {
                        text = format!("{text}{value}");
                    }
                    TextToken::Digit(_) => {}
                    TextToken::Period => {}
                    TextToken::Day => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        text = format!("{text}{ymd_day}");
                    }
                    TextToken::DayPadded => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let day = ymd_day as usize;
                        text = format!("{text}{day:02}");
                    }
                    TextToken::DayNameShort => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let day = excel_weekday_index(serial_1900);
                        text = format!("{}{}", text, locale.dates.day_names_short[day]);
                    }
                    TextToken::DayName => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let day = excel_weekday_index(serial_1900);
                        text = format!("{}{}", text, locale.dates.day_names[day]);
                    }
                    TextToken::Month => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        text = format!("{text}{month}");
                    }
                    TextToken::MonthPadded => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        text = format!("{text}{month:02}");
                    }
                    TextToken::MonthNameShort => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        text = format!("{}{}", text, locale.dates.months_short[month - 1]);
                    }
                    TextToken::MonthName => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        text = format!("{}{}", text, locale.dates.months[month - 1]);
                    }
                    TextToken::MonthLetter => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        let months_letter = &locale.dates.months_letter[month - 1];
                        text = format!("{text}{months_letter}");
                    }
                    TextToken::YearShort => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        text = format!("{text}{:02}", ymd_year % 100);
                    }
                    TextToken::Year => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        text = format!("{text}{ymd_year}");
                    }
                    TextToken::Hour => {
                        let mut hour = hours as i32;
                        if p.use_ampm {
                            if hour == 0 {
                                hour = 12;
                            } else if hour > 12 {
                                hour -= 12;
                            }
                        }
                        text = format!("{text}{hour}");
                    }
                    TextToken::HourPadded => {
                        let mut hour = hours as i32;
                        if p.use_ampm {
                            if hour == 0 {
                                hour = 12;
                            } else if hour > 12 {
                                hour -= 12;
                            }
                        }
                        text = format!("{text}{hour:02}");
                    }
                    TextToken::Second => {
                        let second = seconds as i32;
                        text = format!("{text}{second}");
                    }
                    TextToken::SecondPadded => {
                        let second = seconds as i32;
                        text = format!("{text}{second:02}");
                    }
                    TextToken::AMPM => {
                        let ampm = if hours < 12.0 { "AM" } else { "PM" };
                        text = format!("{text}{ampm}");
                    }
                    TextToken::Minute => {
                        let minute = minutes as i32;
                        text = format!("{text}{minute}");
                    }
                    TextToken::MinutePadded => {
                        let minute = minutes as i32;
                        text = format!("{text}{minute:02}");
                    }
                    TextToken::ElapsedHour => {
                        let hour = elapsed_text(value, false, ElapsedUnit::Hour);
                        text = format!("{text}{hour}");
                    }
                    TextToken::ElapsedHourPadded => {
                        let hour = elapsed_text(value, true, ElapsedUnit::Hour);
                        text = format!("{text}{hour}");
                    }
                    TextToken::ElapsedMinute => {
                        let minute = elapsed_text(value, false, ElapsedUnit::Minute);
                        text = format!("{text}{minute}");
                    }
                    TextToken::ElapsedMinutePadded => {
                        let minute = elapsed_text(value, true, ElapsedUnit::Minute);
                        text = format!("{text}{minute}");
                    }
                    TextToken::ElapsedSecond => {
                        let second = elapsed_text(value, false, ElapsedUnit::Second);
                        text = format!("{text}{second}");
                    }
                    TextToken::ElapsedSecondPadded => {
                        let second = elapsed_text(value, true, ElapsedUnit::Second);
                        text = format!("{text}{second}");
                    }
                }
            }
            // Excel: an elapsed-token result over 255 characters is #VALUE! (probes/elapsed-huge-g-1900);
            // applying it to every date/time section is an assumption, unprobed for other tokens.
            if text.chars().count() > MAX_TEXT_FORMAT_CHARS {
                return text_too_long();
            }
            Formatted {
                text,
                color: p.color,
                error: None,
            }
        }
        ParsePart::Number(p) => {
            let mut text = "".to_string();
            if let Some(c) = p.currency {
                text = format!("{c}");
            }
            let tokens = &p.tokens;
            value = value * 100.0_f64.powi(p.percent) / (1000.0_f64.powi(p.comma));
            // p.precision is the number of significant digits _after_ the decimal point
            value = to_precision(
                value,
                (p.precision as usize) + format!("{}", value.abs().floor()).len(),
            );
            let mut value_abs = value.abs();
            let mut exponent_part: Vec<char> = vec![];
            let mut exponent_is_negative = value_abs < 10.0;
            if p.is_scientific {
                if value_abs == 0.0 {
                    exponent_part = vec!['0'];
                    exponent_is_negative = false;
                } else {
                    // TODO: Implement engineering formatting.
                    let exponent = value_abs.log10().floor();
                    exponent_part = format!("{}", exponent.abs()).chars().collect();
                    value /= 10.0_f64.powf(exponent);
                    value = to_precision(value, 15);
                    value_abs = value.abs();
                }
            }
            let l_exp = exponent_part.len() as i32;
            let int_number = if p.precision == 0 {
                value_abs.round()
            } else {
                value_abs.floor()
            };
            let mut int_part: Vec<char> = format!("{int_number}").chars().collect();
            if int_number as i64 == 0 {
                int_part = vec![];
            }
            let fract_part = get_fract_part(value_abs, p.precision, int_part.len());
            // ln is the number of digits of the integer part of the value
            let ln = int_part.len() as i32;
            // digit count is the number of digit tokens ('0', '?' and '#') to the left of the decimal point
            let digit_count = p.digit_count;
            // digit_index points to the digit index in value that we have already formatted
            let mut digit_index = 0;

            let symbols = &locale.numbers.symbols;
            let group_sizes = locale.numbers.decimal_formats.standard.to_owned();
            let group_separator = symbols.group.to_owned();
            let decimal_separator = symbols.decimal.to_owned();
            // There probably are better ways to check if a number at a given precision is negative :/
            // `<=`: a value exactly one unit at the last shown place (-1 for `0`, -1 after a `#,,`
            // scaling) keeps its sign (poi/NumberFormatTests.xlsx Tests!A95).
            let is_negative = value <= -(10.0_f64.powf(-(p.precision as f64)));
            let mut needs_period = false;

            for token in tokens {
                match token {
                    TextToken::Literal(c) => {
                        text = format!("{text}{c}");
                    }
                    TextToken::Text(t) => {
                        text = format!("{text}{t}");
                    }
                    TextToken::Ghost(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Spacer(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Raw => {
                        text = format!("{text}{value}");
                    }
                    TextToken::Period => {
                        // if !fract_part.is_empty() &&  {
                        //     text = format!("{text}{decimal_separator}");
                        // }
                        needs_period = true;
                    }
                    TextToken::Digit(digit) => {
                        if digit.number.is_integer() {
                            // 1. Integer part
                            let index = digit.index;
                            let number_index = ln - digit_count + index;
                            if index == 0 && is_negative {
                                text = format!("-{text}");
                            }
                            if ln <= digit_count {
                                // The number of digits is less or equal than the number of digit tokens
                                // i.e. the value is 123 and the format_code is ##### (ln = 3 and digit_count = 5)
                                if !(number_index < 0 && digit.kind == '#') {
                                    let c = if number_index < 0 {
                                        if digit.kind == '0' {
                                            '0'
                                        } else {
                                            // digit.kind = '?'
                                            ' '
                                        }
                                    } else {
                                        int_part[number_index as usize]
                                    };
                                    // The group positions count from the format's last digit
                                    // placeholder, so padding is grouped too (a padded `?`
                                    // separator is a space; docs/excel-semantics/number-formats.md,
                                    // "A comma between digit placeholders turns on thousands grouping").
                                    let sep = if use_group_separator(
                                        p.use_thousands,
                                        digit_count - digit_index,
                                        &group_sizes,
                                    ) {
                                        if number_index < 0 && digit.kind == '?' {
                                            " "
                                        } else {
                                            &group_separator
                                        }
                                    } else {
                                        ""
                                    };
                                    text = format!("{text}{c}{sep}");
                                }
                                digit_index += 1;
                            } else {
                                // The number is larger than the formatting code 12345 and 0##
                                // We just hit the first formatting digit (0 in the example above) so we write as many digits as we can (123 in the example)
                                for i in digit_index..number_index + 1 {
                                    let sep = if use_group_separator(
                                        p.use_thousands,
                                        ln - i,
                                        &group_sizes,
                                    ) {
                                        &group_separator
                                    } else {
                                        ""
                                    };
                                    text = format!("{}{}{}", text, int_part[i as usize], sep);
                                }
                                digit_index = number_index + 1;
                            }
                        } else if digit.number.is_decimal() {
                            // 2. After the decimal point
                            let index = digit.index as usize;
                            if index < fract_part.len() {
                                if needs_period {
                                    text = format!(
                                        "{}{}{}",
                                        text, decimal_separator, fract_part[index]
                                    );
                                } else {
                                    text = format!("{}{}", text, fract_part[index]);
                                }
                            } else if digit.kind == '0' {
                                if needs_period {
                                    text = format!("{text}{decimal_separator}0");
                                } else {
                                    text = format!("{text}0");
                                }
                            } else if digit.kind == '?' {
                                text = format!("{text} ");
                            } else if digit.kind == '#' && needs_period {
                                // FIXME: This is what Excel does, but it transforms "3" into "3."
                                text = format!("{text}{decimal_separator}");
                            }
                            needs_period = false;
                        } else if digit.number.is_exponent() {
                            // 3. Exponent part
                            let index = digit.index;
                            if index == 0 {
                                if exponent_is_negative {
                                    text = format!("{text}E-");
                                } else if p.scientific_minus {
                                    text = format!("{text}E");
                                } else {
                                    text = format!("{text}E+");
                                }
                            }
                            let number_index = l_exp - (p.exponent_digit_count - index);
                            if l_exp <= p.exponent_digit_count {
                                if !(number_index < 0 && digit.kind == '#') {
                                    let c = if number_index < 0 {
                                        if digit.kind == '?' {
                                            ' '
                                        } else {
                                            '0'
                                        }
                                    } else {
                                        exponent_part[number_index as usize]
                                    };

                                    text = format!("{text}{c}");
                                }
                            } else {
                                for i in 0..number_index + 1 {
                                    text = format!("{}{}", text, exponent_part[i as usize]);
                                }
                                digit_index += number_index + 1;
                            }
                        }
                    }
                    // Date tokens should not be present
                    TextToken::Day => {}
                    TextToken::DayPadded => {}
                    TextToken::DayNameShort => {}
                    TextToken::DayName => {}
                    TextToken::Month => {}
                    TextToken::MonthPadded => {}
                    TextToken::MonthNameShort => {}
                    TextToken::MonthName => {}
                    TextToken::MonthLetter => {}
                    TextToken::YearShort => {}
                    TextToken::Year => {}
                    TextToken::Hour => {}
                    TextToken::HourPadded => {}
                    TextToken::Minute => {}
                    TextToken::MinutePadded => {}
                    TextToken::Second => {}
                    TextToken::SecondPadded => {}
                    TextToken::AMPM => {}
                    TextToken::ElapsedHour => {}
                    TextToken::ElapsedHourPadded => {}
                    TextToken::ElapsedMinute => {}
                    TextToken::ElapsedMinutePadded => {}
                    TextToken::ElapsedSecond => {}
                    TextToken::ElapsedSecondPadded => {}
                }
            }
            Formatted {
                text,
                color: p.color,
                error: None,
            }
        }
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
