use super::elapsed::{elapsed_text, DisplayClock, ElapsedUnit};
use super::fraction::{comma_exponent, format_fraction};
use super::general::format_general;
use crate::{
    constants::MAXIMUM_DATE_SERIAL_NUMBER,
    decimal_round::{round_engineering, round_scientific, round_shown_place},
    locale::Locale,
    number_format::{cut_15_digits, to_precision},
    text::Text,
    types::DateSystem,
};
use std::collections::BTreeSet;

pub use super::formatted::Formatted;
use super::formatted::MAX_TEXT_FORMAT_CHARS;
pub(crate) use super::parse_formatted::{
    parse_formatted_number, parse_typed_number, rejected_clock_text_format,
};
use super::{
    calendar_dates::{calendar_ymd, hebrew_month_name},
    date_names::NameTokens,
    dates::negative_serial_allowed,
    locale_tag::{japanese_era_name, japanese_era_year, Calendar},
    parser::{
        conditional_section, is_text_section, numeric_section_count, section_condition,
        section_tokens, ErrorPart, GeneralPart, ParsePart, Parser, TextToken,
    },
};

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

/// Returns the chars of the fractional part of a *positive* number and whether rounding it to
/// `precision` places carried into the integer part, read off its 15
/// significant digits (the value is already rounded to them) and without trailing zeros:
/// 3.1415926 ==> ['1', '4', '1', '5', '9', '2', '6']
/// Excel shows only 15 significant digits and pads the rest of a wide format with zeros, so the
/// cut counts significant digits, not positions after the point (`TEXT(2/3,"0.000000000000000000")`
/// is "0.666666666666667000", probes-numeric/4f413cc257ce15c4 A23).
fn get_fract_part(value: f64, precision: i32) -> (Vec<char>, bool) {
    let (_, fract) = cut_15_digits(value, true);
    let mut fract: Vec<u8> = fract;
    // Round half up on the decimal digits; the flag says the rounding carried into the integer part.
    let shown = usize::try_from(precision).unwrap_or(0);
    let round_up = fract.get(shown).is_some_and(|d| *d >= 5);
    fract.truncate(shown);
    let mut carry = round_up;
    if round_up {
        for d in fract.iter_mut().rev() {
            if *d == 9 {
                *d = 0;
            } else {
                *d += 1;
                carry = false;
                break;
            }
        }
    }
    while fract.last() == Some(&0) {
        fract.pop();
    }
    (
        fract.into_iter().map(|d| char::from(b'0' + d)).collect(),
        carry,
    )
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
    Formatted::unshowable(format!("Invalid date value: '{value}'"))
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
            unshowable: false,
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
///
/// Excel checks the number sections first: a code with an unparsable number section is `#VALUE!`
/// for a text value too, even where the text section would apply (`TEXT("a","ab1")`,
/// `TEXT(TRUE,"n;n;n;@")`; probes/t2202-text-code-check-on-text; number-formats.md "TEXT checks
/// the code before it looks at a text or boolean value"). A bad section a number would not use
/// in a multi-section code rejects the text too; that case is unprobed.
pub fn format_text_value(text: &str, format: &str) -> Formatted {
    let mut parser = Parser::new(format);
    parser.parse();
    let parts = parser.parts.as_slice();
    if parts[..numeric_section_count(parts)]
        .iter()
        .any(ParsePart::is_error)
    {
        return format_error();
    }
    // `TEXT("text",";;;")` is "" (gen-m10/0e3d8eba9432a18d T!A24).
    if parser.has_empty_text_section() {
        return capped_text(String::new());
    }
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
        unshowable: false,
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
        unshowable: false,
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
    // Excel: a bare digit beside a date token in any number section rejects the whole code, also
    // for a number another section formats (`TEXT(3.14159,"0;4b")` is #VALUE!;
    // probes/text-format-digits-date-token A42, B42).
    let sections = numeric_section_count(&parts);
    if parts[..sections]
        .iter()
        .any(|part| matches!(part, ParsePart::Error(ErrorPart { whole_code: true })))
    {
        return format_error();
    }
    let mut value = value_original;
    let general = ParsePart::General(GeneralPart {});
    let part;
    let conditional = conditional_section(&parts, sections, value);
    match sections {
        2..=4 if conditional.is_some() => {
            let (index, drop_sign) = conditional.unwrap_or_default();
            if drop_sign {
                value = value.abs();
            }
            part = &parts[index];
        }
        // Excel: an empty format shows nothing but a negative's minus, also for a value that
        // rounds to zero (`TEXT(-0.0001,"")` = "-", `TEXT(5,"")` = ""; probes/text-format-bare-literals
        // A32-A35).
        0 => {
            return Formatted {
                text: if value < 0.0 { "-" } else { "" }.to_owned(),
                color: None,
                error: None,
                unshowable: false,
            };
        }
        // A single text section (`@` in it) does not apply to numbers, which print in General:
        // `TEXT(1,"@""aaa""")` is "1" (probes/text-format-length-limit-i A19:A26, T-1434).
        1 if is_text_section(&parts[0]) => {
            part = &general;
        }
        // Excel: a single section whose condition the value fails is not used, and the value
        // prints in General (`TEXT(10,"[<10]#"" Wow""")` is "10", poi FormatChoiceTests
        // A3-A26; `TEXT(1,"[>5]0"&REPT(" ",79))` is "1", probes/text-format-length-limit-d S!A31).
        1 if section_condition(&parts[0]).is_some_and(|(cmp, bound)| !cmp.holds(value, bound)) => {
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
                unshowable: false,
            };
        }
    }
    match part {
        ParsePart::Error(..) => format_error(),
        ParsePart::General(..) => Formatted {
            text: general_text(value, locale),
            color: None,
            error: None,
            unshowable: false,
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
            let fraction_digits = tokens.iter().find_map(|t| match t {
                TextToken::SecondFraction(digits) => Some(*digits),
                _ => None,
            });
            // Excel: any token but a lone elapsed one (`[s]`) over a serial past 9999-12-31 is
            // #VALUE!: `TEXT(1234567890,"h:mm")`, `"[h]:mm"`, `"[mm]:ss.0"` (gen-m10
            // 810fee377a42575b T!A16, b4f66526f822ff2c T!A20; gen-m5 6a511beadb56b932 Calc!A126).
            if value >= f64::from(MAXIMUM_DATE_SERIAL_NUMBER) + 1.0
                && !negative_serial_allowed(tokens, DateSystem::Excel1900)
            {
                return invalid_date(value_original);
            }
            let clock = DisplayClock::new(value, fraction_digits);
            let (hours, minutes, seconds) = clock.hms;
            let elapsed_fraction = clock.elapsed.as_ref();
            let day_serial = clock.days;
            // Day/month/year and weekday tokens show Excel's exact components (including the
            // fictitious 1900-02-29) and "1900-01-00" (serial 0). In a 1904 workbook they read the
            // 1904 epoch (`TEXT(0,"yyyy-mm-dd")` is "1904-01-01"; dates.md "The 1904 date system");
            // -1 stands for no date. The fallback is never read: `ymd` is `None` exactly when
            // `date_valid` is false.
            let serial_1900 = ctx.date_system.to_1900_serial(day_serial).unwrap_or(-1);
            let ymd = calendar_ymd(p.calendar, p.hijri, serial_1900);
            let date_valid = ymd.is_some();
            // A Saka or Hebrew section of a date with no such date is `#VALUE!` even with only
            // time tokens (`TEXT(60.5,"[$-80409]h:mm")`; probes/t2779-hebrew-saka-more E5, E6).
            if !date_valid && matches!(p.calendar, Some(Calendar::Saka | Calendar::Hebrew(_))) {
                return invalid_date(value);
            }
            let (ymd_year, ymd_month, ymd_day) = ymd.unwrap_or((0, 1, 0));
            // Under a Japanese locale tag `yy` and `yyyy` after a `g` print the era year: the
            // Japanese one in its own calendar (`[$-411]yy|g|yy` is "23|R|05"), the whole Gregorian
            // year in the others (probes/text-locale-japanese-year A1-A13).
            let mut seen_era = false;
            let mut names = NameTokens::new(p.hijri, p.names.is_some());
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
                    // A `*x` fill adds nothing to the text, as Range.text and TEXT show it:
                    // `TEXT(7,"0*""a")` is "7a", `TEXT(3,"*.0.00")` is "3.00"
                    // (docs/excel-semantics/number-formats.md; T-1735).
                    TextToken::Spacer(_) => {}
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
                        names.number();
                        text = format!("{text}{ymd_day}");
                    }
                    TextToken::DayPadded => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        names.number();
                        let day = ymd_day as usize;
                        text = format!("{text}{day:02}");
                    }
                    TextToken::Month => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        names.number();
                        let month = ymd_month as usize;
                        text = format!("{text}{month}");
                    }
                    TextToken::MonthPadded => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        names.number();
                        let month = ymd_month as usize;
                        text = format!("{text}{month:02}");
                    }
                    TextToken::DayNameShort
                    | TextToken::DayName
                    | TextToken::MonthNameShort
                    | TextToken::MonthName
                    | TextToken::MonthLetter => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        let weekday = excel_weekday_index(serial_1900);
                        if let (Some(Calendar::Hebrew(adars)), Some(names)) = (p.calendar, p.names)
                        {
                            if let Some(name) =
                                hebrew_month_name(token, names, adars, (ymd_year, ymd_month))
                            {
                                text.push_str(name);
                                continue;
                            }
                        }
                        // Lunar months 13 and 14 take the twelfth name (`[$-110409]mmmm` of
                        // 2024-01-11 is "十二月"; probes/text-locale-lunar-edges A26).
                        let month = ymd_month.min(12);
                        names.push(token, &mut text, p.names, locale, month, weekday);
                    }
                    TextToken::YearShort
                    | TextToken::Year
                    | TextToken::BuddhistYearShort
                    | TextToken::BuddhistYear => {
                        if !date_valid {
                            return invalid_date(value);
                        }
                        names.number();
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
                            // After a `g` a Japanese LCID's other calendars print the whole
                            // Gregorian year (`[$-10411]g|yy` is "|2023", `[$-70411]g|yyyy`
                            // "|2023"); other LCIDs are unaffected (`[$-70409]g|yyyy` is "|2566";
                            // probes/text-locale-japanese-year A13, text-locale-era-gregorian).
                            (Some(_), false) if seen_era && p.japanese_lcid => {
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
                        names.number();
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
                        let hour = elapsed_text(value, false, ElapsedUnit::Hour, elapsed_fraction);
                        text = format!("{text}{hour}");
                    }
                    TextToken::ElapsedHourPadded => {
                        let hour = elapsed_text(value, true, ElapsedUnit::Hour, elapsed_fraction);
                        text = format!("{text}{hour}");
                    }
                    TextToken::ElapsedMinute => {
                        let minute =
                            elapsed_text(value, false, ElapsedUnit::Minute, elapsed_fraction);
                        text = format!("{text}{minute}");
                    }
                    TextToken::ElapsedMinutePadded => {
                        let minute =
                            elapsed_text(value, true, ElapsedUnit::Minute, elapsed_fraction);
                        text = format!("{text}{minute}");
                    }
                    TextToken::ElapsedSecond => {
                        let second =
                            elapsed_text(value, false, ElapsedUnit::Second, elapsed_fraction);
                        text = format!("{text}{second}");
                    }
                    TextToken::ElapsedSecondPadded => {
                        let second =
                            elapsed_text(value, true, ElapsedUnit::Second, elapsed_fraction);
                        text = format!("{text}{second}");
                    }
                    TextToken::SecondFraction(digits) => {
                        text.push('.');
                        match elapsed_fraction {
                            Some(fraction) => text.push_str(&fraction.digits_text()),
                            None => text.extend(std::iter::repeat_n('0', *digits)),
                        }
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
                unshowable: false,
            }
        }
        ParsePart::Number(p) => {
            if let Some(fraction) = &p.fraction {
                // Past 255 characters a fraction cannot be shown either: `# ?/4,` on 2 would print
                // 258 (255 `#`, TEXT #VALUE!; probes/fraction-overflow-scaled A49).
                let text = format_fraction(value, p, fraction, &locale.numbers.symbols.group)
                    .filter(|text| text.chars().count() <= MAX_TEXT_FORMAT_CHARS);
                return match text {
                    Some(text) => Formatted {
                        text,
                        color: p.color,
                        error: None,
                        unshowable: false,
                    },
                    None => Formatted::unshowable(format!("Fraction overflows: '{value}'")),
                };
            }
            let mut text = "".to_string();
            if let Some(c) = p.currency {
                text = format!("{c}");
            }
            let tokens = &p.tokens;
            // `General` among literals shows the value itself, with its sign
            let general_value = value;
            // Excel keeps the scaling commas' exponent in a signed byte (`comma_exponent`). Up to
            // 42 commas it does not wrap, and dividing keeps the usual rounding.
            let percent_scaled = value * 100.0_f64.powi(p.percent);
            let comma_exponent = comma_exponent(p.comma, true);
            let scaled = if comma_exponent == p.comma.saturating_mul(-3) {
                percent_scaled / 1000.0_f64.powi(p.comma)
            } else {
                percent_scaled * 10.0_f64.powi(comma_exponent)
            };
            value = scaled;
            // p.precision is the number of significant digits _after_ the decimal point
            // Excel shows at most 15 significant digits (docs/excel-semantics/numbers.md, "Number → text";
            // probes-numeric/4f413cc257ce15c4 A23)
            value = round_shown_place(value, p.precision, p.is_scientific);
            let mut value_abs = value.abs();
            let mut exponent_part: Vec<char> = vec![];
            let mut exponent_is_negative = false;
            if p.is_scientific {
                if value_abs == 0.0 {
                    exponent_part = vec!['0'];
                } else {
                    // Engineering: the exponent is a multiple of the integer-placeholder count,
                    // rounded down (`##0.0E+0` on 123456.789 is "123.5E+3", on 61 "61.0E+0";
                    // `00.0E+0` on 36.5 "36.5E+0"; eval/suites/gen-m10, T-1730). A zero
                    // exponent carries no minus (`0.00E+00` on 1.5 is "1.50E+00").
                    let group = f64::from(p.digit_count.max(1));
                    let exponent;
                    if group > 1.0 {
                        // The mantissa rounds after the shift (`##0.0E+0` on 1/3: "333.3E-3"; T-2226).
                        (value, exponent) = round_engineering(scaled, group, p.precision);
                    } else {
                        (value, exponent) = round_scientific(scaled, p.precision);
                    }
                    exponent_is_negative = exponent < 0.0;
                    exponent_part = format!("{}", exponent.abs()).chars().collect();
                    value /= 10.0_f64.powf(exponent);
                    value = to_precision(value, 15);
                    value_abs = value.abs();
                }
            }
            let l_exp = exponent_part.len() as i32;
            let (fract_part, fract_carry) = get_fract_part(value_abs, p.precision);
            let int_number = if p.precision == 0 {
                value_abs.round()
            } else {
                value_abs.floor() + if fract_carry { 1.0 } else { 0.0 }
            };
            let mut int_part: Vec<char> = format!("{int_number}").chars().collect();
            if int_number as i64 == 0 {
                int_part = vec![];
            }
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
                    // A `*x` fill adds nothing to the text, as Range.text and TEXT show it:
                    // `TEXT(7,"0*""a")` is "7a", `TEXT(3,"*.0.00")` is "3.00"
                    // (docs/excel-semantics/number-formats.md; T-1735).
                    TextToken::Spacer(_) => {}
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
                                // A zero mantissa prints every integer placeholder, even `#`:
                                // `|#.#|E-|#|` on 0 is "|0.|E|0|" (poi/NumberFormatApproxTests.xlsx
                                // A136), where `#.#` alone gives "."; `##0.0E+0` on 0 is "000.0E+0"
                                // (gen-m10 659cd5776d423ce3 F!J14; T-2227).
                                let zero_mantissa = p.is_scientific && ln == 0 && digit.kind == '#';
                                if zero_mantissa {
                                    text = format!("{text}0");
                                } else if !(number_index < 0 && digit.kind == '#') {
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
                                // A padding space, after the point when it is the first decimal:
                                // `?.??` on 1000 is "1000.  " (gen-m10, T-1738).
                                if needs_period {
                                    text = format!("{text}{decimal_separator} ");
                                } else {
                                    text = format!("{text} ");
                                }
                            } else if digit.kind == '#' && needs_period {
                                // FIXME: This is what Excel does, but it transforms "3" into "3."
                                text = format!("{text}{decimal_separator}");
                            }
                            needs_period = false;
                        } else if digit.number.is_exponent() {
                            // 3. Exponent part
                            let index = digit.index;
                            // The `E` itself is a literal at the format's `E+`/`E-` (parser).
                            if index == 0 {
                                if exponent_is_negative {
                                    text.push('-');
                                } else if !p.scientific_minus {
                                    text.push('+');
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
                    TextToken::SecondFraction(_) => {}
                }
            }
            Formatted {
                text,
                color: p.color,
                error: None,
                unshowable: false,
            }
        }
    }
}
