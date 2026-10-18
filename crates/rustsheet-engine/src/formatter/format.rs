use super::elapsed::{elapsed_text, ElapsedUnit};
use super::general::format_general;
use crate::{locale::Locale, number_format::to_precision, text::Text, types::DateSystem};
use std::collections::BTreeSet;

pub(crate) use super::parse_formatted::parse_formatted_number;
use super::{
    dates::{excel_serial_to_ymd, negative_serial_allowed},
    locale_tag::{japanese_era_name, japanese_era_year, Calendar},
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
/// booleans use the format's text section"). A last section with `@` is the text section too,
/// whatever the section count (`@" ""q"""` on "x" is "x q", probes/format-doubled-quotes C11;
/// `TEXT("abc","0;x@")` is "xabc", probes/text-at-token-sections A10, A11, T-1656). Any other
/// format with fewer than four sections returns the text unchanged; that case is unprobed. A
/// result over 255 characters is `#VALUE!` ([`MAX_TEXT_FORMAT_CHARS`]).
pub fn format_text_value(text: &str, format: &str) -> Formatted {
    let mut parser = Parser::new(format);
    parser.parse();
    let parts = parser.parts.as_slice();
    let section = match parts.last() {
        Some(last) if parts.len() < 4 && is_text_section(last) => parts.len() - 1,
        _ if parts.len() < 4 => return capped_text(text.to_string()),
        _ => 3,
    };
    let tokens = match parser.parts.get(section) {
        Some(ParsePart::Error(_)) => return format_error(),
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
            // Excel: `_x` leaves a space but a `*x` fill prints nothing, and `AM/PM` prints "AM"
            // (`TEXT("abc","x@_x")` is "xabc ", `"x@*-"` "xabc", `"x@AM/PM"` "xabcAM";
            // probes/text-at-with-tokens A23, A37, A38, B37, B38, T-1656).
            TextToken::Ghost(_) => out.push(' '),
            TextToken::AMPM => out.push_str("AM"),
            // `A/P` prints its morning letter (probes/text-ap-locale-bracket A21-A23).
            TextToken::AP(am, _) => out.push(*am),
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

/// The private-use characters [`format_text_value_exact`] picks its stand-in from.
const STAND_IN_RANGE: std::ops::RangeInclusive<char> = '\u{E000}'..='\u{F8FF}';

/// [`format_text_value`] for exact text: a lone surrogate in `text` is kept where `@` prints it
/// (probes/text-lone-surrogates, TEXT(H,"@") has length 1). The format runs on a one-character
/// stand-in for the text, a private-use character the format does not hold, so the 255-character
/// limit and every other rule are the well-formed path's; the stand-ins in the result are then
/// replaced by the exact text. `Err` is the formatter's error message.
pub(crate) fn format_text_value_exact(text: &Text, format: &str) -> Result<Text, String> {
    if !text.has_lone_surrogate() {
        let f = format_text_value(text, format);
        return f.error.map_or(Ok(Text::from(f.text)), Err);
    }
    let in_format: BTreeSet<char> = format
        .chars()
        .filter(|c| STAND_IN_RANGE.contains(c))
        .collect();
    // The format holds at most one private-use character per character, so one of the first
    // `in_format.len() + 1` is free; the range holds 6,400.
    let stand_in = STAND_IN_RANGE
        .clone()
        .find(|c| !in_format.contains(c))
        .ok_or_else(|| "No free stand-in character".to_owned())?;
    let f = format_text_value(stand_in.encode_utf8(&mut [0; 4]), format);
    if let Some(error) = f.error {
        return Err(error);
    }
    // The stand-in counted as one character, so the limit is checked again on the real length
    // (this also bounds the result).
    let copies = f.text.matches(stand_in).count();
    let chars = f.text.chars().count() - copies + copies.saturating_mul(text.chars().count());
    if chars > MAX_TEXT_FORMAT_CHARS {
        return Err(TEXT_TOO_LONG.to_owned());
    }
    let mut out = Text::from("");
    let mut parts = f.text.split(stand_in);
    if let Some(first) = parts.next() {
        out.push(&Text::from(first));
    }
    for part in parts {
        out.push(text);
        out.push(&Text::from(part));
    }
    Ok(out)
}

/// Excel: TEXT of a text value is `#VALUE!` once the result is over 255 characters, also when the
/// format leaves the text unchanged (probes/text-format-text-limit A10, A11, A19-A21, A24, A25).
const MAX_TEXT_FORMAT_CHARS: usize = 255;

/// The Buddhist era counts from 543 BC: serial 100 (1900-04-09) is "43" under `b` and "2443" under
/// `bbbb` (probes/text-format-bare-literals A41-A46; gen-m7/6db767d720c5b55a A30).
const BUDDHIST_ERA_OFFSET: i32 = 543;

/// The Korean calendar (locale tag calendar digit 5) counts from 2333 BC: `[$-50409]yyyy` of 45000
/// is "4356" (probes/text-locale-calendars A44-A46).
const KOREAN_ERA_OFFSET: i32 = 2333;

const TEXT_TOO_LONG: &str = "TEXT result over 255 characters";

/// A `#VALUE!` result, with `message` saying why.
fn value_error(message: &str) -> Formatted {
    Formatted {
        text: "#VALUE!".to_owned(),
        color: None,
        error: Some(message.to_owned()),
    }
}

/// The `#VALUE!` of a format code that does not parse.
fn format_error() -> Formatted {
    value_error("Problem parsing format string")
}

fn text_too_long() -> Formatted {
    value_error(TEXT_TOO_LONG)
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

/// `value` in General, with the locale's decimal separator.
fn general_text(value: f64, locale: &Locale) -> String {
    let text = format_general(value);
    if locale.numbers.symbols.decimal == "." {
        text
    } else {
        text.replace('.', &locale.numbers.symbols.decimal)
    }
}

/// Whether `part` holds `@`, which makes it a text section.
fn is_text_section(part: &ParsePart) -> bool {
    section_tokens(part).is_some_and(|t| t.iter().any(|t| matches!(t, TextToken::Raw)))
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
    // A last section with `@` in a format of two or three sections is the text section: numbers
    // use the sections before it (`TEXT(-1,"0;x@")` is "-1", `TEXT(-1,"0;0;x@")` "1";
    // probes/text-at-token-sections C10, C11, T-1656).
    let numeric_sections = match parts.last() {
        Some(last) if (2..4).contains(&parts.len()) && is_text_section(last) => parts.len() - 1,
        _ => parts.len(),
    };
    match numeric_sections {
        // Excel: an empty format shows nothing but a negative's minus, also for a value that
        // rounds to zero (`TEXT(-0.0001,"")` = "-", `TEXT(5,"")` = ""; probes/text-format-bare-literals
        // A32-A35).
        0 => {
            return Formatted {
                text: if value < 0.0 { "-" } else { "" }.to_owned(),
                color: None,
                error: None,
            };
        }
        // A single text section (`@` in it) does not apply to numbers, which print in General:
        // `TEXT(1,"@""aaa""")` is "1" (probes/text-format-length-limit-i A19:A26, T-1434).
        1 if is_text_section(&parts[0]) => {
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
        ParsePart::General(..) => Formatted {
            text: general_text(value, locale),
            color: None,
            error: None,
        },
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
            // Under a locale tag `yy` and `yyyy` after a `g` print the era year: the Japanese one
            // under a Japanese locale (`[$-411]yy|g|yy` is "23|R|05"), the whole Gregorian year
            // otherwise (probes/text-locale-japanese-year A1-A13).
            let mut seen_era = false;
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
                    TextToken::General => {}
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
                        text.push_str(
                            p.names
                                .map_or(&locale.dates.day_names_short[day], |n| n.days_short[day]),
                        );
                    }
                    TextToken::DayName => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let day = excel_weekday_index(serial_1900);
                        text.push_str(
                            p.names
                                .map_or(&locale.dates.day_names[day], |n| n.days[day]),
                        );
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
                        text.push_str(p.names.map_or(&locale.dates.months_short[month - 1], |n| {
                            n.months_short[month - 1]
                        }));
                    }
                    TextToken::MonthName => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        text.push_str(
                            p.names
                                .map_or(&locale.dates.months[month - 1], |n| n.months[month - 1]),
                        );
                    }
                    TextToken::MonthLetter => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let month = ymd_month as usize;
                        // Under a locale tag `mmmmm` is the first character of the month name
                        // (`[$-804]mmmmm` of December is "十"; probes/text-locale-am-neutral A28).
                        match p.names.and_then(|n| n.months[month - 1].chars().next()) {
                            Some(letter) => text.push(letter),
                            None => text.push_str(&locale.dates.months_letter[month - 1]),
                        }
                    }
                    TextToken::YearShort
                    | TextToken::Year
                    | TextToken::BuddhistYearShort
                    | TextToken::BuddhistYear => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let buddhist = matches!(
                            token,
                            TextToken::BuddhistYearShort | TextToken::BuddhistYear
                        );
                        let short =
                            matches!(token, TextToken::YearShort | TextToken::BuddhistYearShort);
                        // Excel: `b` is the Buddhist year with no locale tag and the Gregorian one
                        // under a tag but in the Thai calendar (`[$-40C]bbbb` is "2023",
                        // `[$-7041E]bbbb` "2566"); `yy` and `yyyy` count in the tag's calendar
                        // (probes/text-locale-calendars, text-locale-calendars-detail A2, A14-A23).
                        let year = match (p.calendar, buddhist) {
                            (Some(Calendar::Japanese { explicit }), false)
                                if explicit || seen_era =>
                            {
                                let era_year = japanese_era_year(serial_1900, ymd_year);
                                text = format!("{text}{era_year:02}");
                                continue;
                            }
                            // After a `g` any other tagged calendar prints the whole Gregorian
                            // year (`[$-10411]g|yy` is "|2023", `[$-70411]g|yyyy` "|2023";
                            // probes/text-locale-japanese-year A13, text-locale-calendars A25).
                            (Some(_), false) if seen_era => {
                                text = format!("{text}{ymd_year}");
                                continue;
                            }
                            (None, true) | (Some(Calendar::Thai), _) => {
                                ymd_year + BUDDHIST_ERA_OFFSET
                            }
                            (Some(Calendar::Korean), false) => ymd_year + KOREAN_ERA_OFFSET,
                            _ => ymd_year,
                        };
                        // The Taiwan calendar's `yy` is four digits (probes/text-locale-japanese-year
                        // A15).
                        let two_digits =
                            short && (buddhist || p.calendar != Some(Calendar::Taiwan));
                        if two_digits {
                            text = format!("{text}{:02}", year % 100);
                        } else {
                            text = format!("{text}{year}");
                        }
                    }
                    TextToken::EraYear(padded) => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        if let Some(Calendar::Japanese { .. }) = p.calendar {
                            let era_year = japanese_era_year(serial_1900, ymd_year);
                            if *padded {
                                text = format!("{text}{era_year:02}");
                            } else {
                                text = format!("{text}{era_year}");
                            }
                        } else {
                            text = format!("{text}{ymd_year}");
                        }
                    }
                    TextToken::Era(length) => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        seen_era = true;
                        if let Some(Calendar::Japanese { .. }) = p.calendar {
                            text.push_str(japanese_era_name(serial_1900, *length));
                        }
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
                    // Under a locale tag both print the locale's strings, so `A/P` becomes
                    // `AM/PM` (`[$-409]h A/P` is "6 PM", `[$-411]h a/p` "6 午後";
                    // probes/text-locale-name-tables A547-A554, probes/text-locale-am-neutral).
                    TextToken::AMPM | TextToken::AP(..) => {
                        let morning = hours < 12.0;
                        match (p.day_half, token) {
                            (Some(n), _) => text.push_str(if morning { n.am } else { n.pm }),
                            (None, TextToken::AP(am, pm)) => {
                                text.push(if morning { *am } else { *pm });
                            }
                            (None, _) => text.push_str(if morning { "AM" } else { "PM" }),
                        }
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
            // `General` among literals shows the value itself, with its sign
            let general_value = value;
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
            // With no integer placeholder no digit carries the minus, so it leads the literals
            // (`TEXT(-PI(),"x")` = "-x", `TEXT(-5,"2")` = "-2"; probes/text-format-bare-literals A3, A7).
            let has_general = tokens.iter().any(|t| matches!(t, TextToken::General));
            if digit_count == 0 && is_negative && !has_general {
                text = format!("-{text}");
            }

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
                    TextToken::General => {
                        text.push_str(&general_text(general_value, locale));
                    }
                    TextToken::Period => {
                        if p.precision == 0 {
                            // No decimal placeholder prints it, so it stands on its own: `0.1`
                            // on 12.3 is "12.1" (probes/text-format-bare-literals A21, A11).
                            text = format!("{text}{decimal_separator}");
                        } else {
                            needs_period = true;
                        }
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
                    TextToken::EraYear(_) => {}
                    TextToken::Era(_) => {}
                    TextToken::BuddhistYearShort => {}
                    TextToken::BuddhistYear => {}
                    TextToken::Hour => {}
                    TextToken::HourPadded => {}
                    TextToken::Minute => {}
                    TextToken::MinutePadded => {}
                    TextToken::Second => {}
                    TextToken::SecondPadded => {}
                    TextToken::AMPM | TextToken::AP(..) => {}
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
