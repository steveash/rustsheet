//! Date text ("7/1/2026", "2026-07-01", "July 1, 2026", "1-Jul-26", "Jul 2026"): the one date
//! grammar shared by VALUE, DATEVALUE, date arguments (`cast_number`), criteria and typed input.
//!
//! Excel: see docs/excel-semantics/dates.md ("Date text is parsed month-first"); evidence
//! probes/date-text-grammar and probes/date-text-locale (excel-graph, en-US).

use crate::{
    constants::{MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    locale::Locale,
};

use super::dates::date_to_serial_number;

/// Display format Excel gives typed text in the "day month-name year" family (`1-Jul-26`).
const DAY_MONTH_NAME_YEAR_FORMAT: &str = "d-mmm-yy";
/// Display format Excel gives typed "month-name year" text (`Jul-2026`).
const MONTH_NAME_YEAR_FORMAT: &str = "mmm-yy";
/// Display format given to typed year-less text (`Jul-26`, `7/1`, `1-Jul`).
const DAY_MONTH_NAME_FORMAT: &str = "d-mmm";

/// The current year, asked for only when year-less text ("7/1", "Jul-26") needs it: it reads
/// the clock, which other date text must not pay for. `None` (no clock) makes such text an error.
pub(crate) type CurrentYear<'a> = &'a dyn Fn() -> Option<i32>;
const NO_CLOCK: &str = "Year-less date text needs the current year";

fn is_digits(s: &str, lengths: std::ops::RangeInclusive<usize>) -> bool {
    lengths.contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
}

/// A numeric day or month: one or two digits, with its format token (`d`/`dd`, `m`/`mm`).
fn parse_day_or_month(
    part: &str,
    [short, padded]: [&'static str; 2],
) -> Result<(u32, &'static str), String> {
    if !is_digits(part, 1..=2) {
        return Err(format!("Not a valid {short} part"));
    }
    let value = part.parse::<u32>().map_err(|e| e.to_string())?;
    Ok((value, if part.len() == 2 { padded } else { short }))
}

fn parse_day(day_str: &str) -> Result<(u32, &'static str), String> {
    parse_day_or_month(day_str, ["d", "dd"])
}

fn parse_month_number(month_str: &str) -> Result<(u32, &'static str), String> {
    parse_day_or_month(month_str, ["m", "mm"])
}

/// Longest text that can be a month name; longer words are rejected before any lowercasing,
/// since this runs on every text-to-number coercion.
const MAX_MONTH_NAME_BYTES: usize = 32;

/// A month name, ignoring case: the locale's short name, or any prefix of at least three letters
/// of its full name ("Sept" and "Septem" are September; probes/date-text-grammar A23-A25).
fn parse_month_name(name: &str, locale: &Locale) -> Option<(u32, &'static str)> {
    if name.len() > MAX_MONTH_NAME_BYTES || name.bytes().any(|b| b.is_ascii_digit()) {
        return None;
    }
    let lower = name.to_lowercase();
    let month = |m: usize| u32::try_from(m + 1).ok();
    if let Some(m) = locale
        .dates
        .months_short
        .iter()
        .position(|short| short.to_lowercase() == lower)
    {
        return Some((month(m)?, "mmm"));
    }
    if lower.chars().count() < 3 {
        return None;
    }
    let m = locale
        .dates
        .months
        .iter()
        .position(|long| long.to_lowercase().starts_with(&lower))?;
    Some((month(m)?, "mmmm"))
}

/// A two- or four-digit year; two-digit years below 30 are 20xx, the rest 19xx
/// (probes/date-text-grammar A21/A22: "Jul 1, 26" is 2026, "Jul 1, 30" is 1930).
fn parse_year(year_str: &str) -> Result<(i32, &'static str), String> {
    if !matches!(year_str.len(), 2 | 4) || !is_digits(year_str, 2..=4) {
        return Err("Not a valid year".to_string());
    }
    let year = year_str.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(if year_str.len() == 2 {
        (two_digit_year(year), "yy")
    } else {
        (year, "yyyy")
    })
}

/// The year of a two-digit year: below 30 is 20xx, the rest 19xx.
fn two_digit_year(year: i32) -> i32 {
    if year < 30 {
        2000 + year
    } else {
        1900 + year
    }
}

/// The serial of a calendar date, including the fictitious 1900-02-29 (serial 60), or an error
/// outside Excel's date range ("Jan 1, 1899" and "Jan 1, 10000" are #VALUE!).
fn serial_of(day: u32, month: u32, year: i32) -> Result<i32, String> {
    let serial = if (year, month, day) == (1900, 2, 29) {
        60
    } else {
        date_to_serial_number(day, month, year)?
    };
    if (MINIMUM_DATE_SERIAL_NUMBER..=MAXIMUM_DATE_SERIAL_NUMBER).contains(&serial) {
        Ok(serial)
    } else {
        Err("Not a valid date".to_string())
    }
}

/// Most days the month can have in any year; February's 29 depends on the year.
fn max_days_in_month(month: u32) -> u32 {
    match month {
        2 => 29,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// "Month year" text is the first of that month. A one- or two-digit number that is a day of
/// the month in the current year makes it that day of the current year ("Jul-26", "7/1"), any
/// other is a two-digit year: "Jul-45" is 1 July 1945, and in 2026 "Feb-29" is 1 February 2029
/// (probes/date-text-yearless). `numeric_month` admits a month number ("2/30"), only for
/// separated words in month-first locales.
fn parse_month_year(
    name: &str,
    year_str: &str,
    locale: &Locale,
    numeric_month: bool,
    current_year: CurrentYear,
) -> Result<(i32, String), String> {
    let (month, format) = match parse_month_name(name, locale) {
        Some((month, _)) => (month, MONTH_NAME_YEAR_FORMAT),
        None if numeric_month && !locale.dates.date_formats.short.starts_with('d') => {
            let (month, _) = parse_month_number(name)?;
            (month, "m/yy")
        }
        None => return Err("Not a valid month".to_string()),
    };
    if !(1..=12).contains(&month) {
        return Err("Not a valid month".to_string());
    }
    // A four-digit year gives the first of that month for a numeric month too: "7/2026",
    // "7-2026" and "12/1999" (probes/date-text-yearless-2 rows 7-9).
    let year = if is_digits(year_str, 4..=4) {
        parse_year(year_str)?.0
    } else if is_digits(year_str, 1..=2) {
        let number = year_str.parse::<u32>().map_err(|e| e.to_string())?;
        // The cheap bound first: only a possible day of the month reads the clock.
        if (1..=max_days_in_month(month)).contains(&number) {
            let this_year = current_year().ok_or(NO_CLOCK)?;
            if let Ok(serial) = serial_of(number, month, this_year) {
                return Ok((serial, DAY_MONTH_NAME_FORMAT.to_string()));
            }
        }
        two_digit_year(i32::try_from(number).map_err(|e| e.to_string())?)
    } else {
        return Err("Not a valid year".to_string());
    };
    Ok((serial_of(1, month, year)?, format.to_string()))
}

/// The serial of a day of the current year, or an error when it is not a date this year
/// ("Feb 29" outside a leap year) or there is no clock.
fn serial_of_this_year(day: u32, month: u32, current_year: CurrentYear) -> Result<i32, String> {
    let year = current_year().ok_or(NO_CLOCK)?;
    serial_of(day, month, year)
}

/// "Day month" text ("1-Jul", "1/Jul", "1 July"): that day of the current year. Unlike
/// month-first text, a number that is not a day of the month this year is #VALUE!, never a
/// year ("45-Jul", "0-Jul", "32 Jul"; probes/date-text-yearless-2, date-text-yearless-3).
fn parse_day_month(
    day_str: &str,
    month: u32,
    current_year: CurrentYear,
) -> Result<(i32, String), String> {
    let (day, _) = parse_day(day_str)?;
    if !(1..=max_days_in_month(month)).contains(&day) {
        return Err("Not a valid day".to_string());
    }
    Ok((
        serial_of_this_year(day, month, current_year)?,
        DAY_MONTH_NAME_FORMAT.to_string(),
    ))
}

/// One word with a date separator: "7/1/2026", "2026-07-01", "1-Jul-2026", "1/Jul/26",
/// "Jul-2026". Separators are `/` and `-`, plus `.` in locales whose short date uses it (en-US
/// rejects "1.7.2026"; probes/date-text-grammar A39/A40).
fn parse_separated(
    word: &str,
    locale: &Locale,
    current_year: CurrentYear,
) -> Result<(i32, String), String> {
    let short_format = &locale.dates.date_formats.short;
    let separator = ['/', '-', '.']
        .into_iter()
        .filter(|&s| s != '.' || short_format.contains('.'))
        .find(|&s| word.contains(s))
        .ok_or("Not a valid date")?;
    // At most 4 parts: enough to reject longer text without splitting all of it.
    let parts: Vec<&str> = word.splitn(4, separator).collect();
    let (day_str, month_str, year_str) = match parts.as_slice() {
        [first, second] => {
            return match parse_month_name(second, locale) {
                Some((month, _)) => parse_day_month(first, month, current_year),
                None => parse_month_year(first, second, locale, true, current_year),
            };
        }
        // Year first is numeric only: "2023/Jan/15" is #VALUE! (probes/date-text-grammar A20).
        [year, month, day] if is_digits(year, 4..=4) => {
            let (day, day_format) = parse_day(day)?;
            let (month, month_format) = parse_month_number(month)?;
            let (year, _) = parse_year(year)?;
            return Ok((
                serial_of(day, month, year)?,
                format!("yyyy{separator}{month_format}{separator}{day_format}"),
            ));
        }
        [day, name, year] if !is_digits(name, 1..=2) => {
            let (day, day_format) = parse_day(day)?;
            let (month, month_format) =
                parse_month_name(name, locale).ok_or("Not a valid month")?;
            let (year, year_format) = parse_year(year)?;
            return Ok((
                serial_of(day, month, year)?,
                format!("{day_format}{separator}{month_format}{separator}{year_format}"),
            ));
        }
        // Numeric order follows the locale: month first in en-US, where "13/1/2026" and
        // "26-01-01" are #VALUE! (probes/date-text-grammar A32, probes/date-text-locale B1).
        [first, second, year] if short_format.starts_with('d') => (*first, *second, *year),
        [first, second, year] => (*second, *first, *year),
        _ => return Err("Not a valid date".to_string()),
    };
    let (day, day_format) = parse_day(day_str)?;
    let (month, month_format) = parse_month_number(month_str)?;
    let (year, year_format) = parse_year(year_str)?;
    let format = if short_format.starts_with('d') {
        format!("{day_format}{separator}{month_format}{separator}{year_format}")
    } else {
        format!("{month_format}{separator}{day_format}{separator}{year_format}")
    };
    Ok((serial_of(day, month, year)?, format))
}

/// `value` without the spaces next to a `-` or `/`: a run of spaces on either side goes.
fn remove_separator_spaces(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_spaces = 0_usize;
    let mut after_separator = false;
    for c in value.chars() {
        if c == ' ' {
            if !after_separator {
                pending_spaces += 1;
            }
            continue;
        }
        let separator = c == '-' || c == '/';
        if !separator {
            out.extend(std::iter::repeat_n(' ', pending_spaces));
        }
        pending_spaces = 0;
        out.push(c);
        after_separator = separator;
    }
    out.extend(std::iter::repeat_n(' ', pending_spaces));
    out
}

/// A word whose month name is glued to digits, split where letters meet digits:
/// `[day?, month name, number?]` ("1JUN2021" is "1", "JUN", "2021"; "July1" is "July", "1"), with
/// whether a number follows the name. `None` when the word is not that shape (punctuation, no
/// digits, or letters that are no month name: "1e5", "10AM", "July1,2026" are left whole).
fn split_glued_month<'a>(word: &'a str, locale: &Locale) -> Option<([&'a str; 3], bool)> {
    let digits = |s: &str| s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let (day, rest) = word.split_at(digits(word));
    let (name, number) = rest.split_at(
        rest.find(|c: char| c.is_ascii_digit())
            .unwrap_or(rest.len()),
    );
    let glued = !name.is_empty() && (!day.is_empty() || !number.is_empty());
    if !glued || digits(number) != number.len() || !name.chars().all(char::is_alphabetic) {
        return None;
    }
    parse_month_name(name, locale)?;
    Some(([day, name, number], !number.is_empty()))
}

/// Parses date text, returning its serial and the number format Excel gives it when typed.
///
/// Surrounding spaces make the text invalid: VALUE, DATEVALUE and criteria trim it first, but
/// number coercion does not (`MROUND(" 2024-01-10 ",7)` is #VALUE!; ironcalc-calc
/// `MROUND_TRUNC_INT.xlsx` MROUND!C32, agreed by the oracle in docs/research/oracle-calibration.md).
///
/// The grammar (repeated inner spaces count as one; probes/date-text-grammar A54):
/// - one word with a separator, see [`parse_separated`];
/// - "Month year": "Jul 2026", "July 2026" (the first of the month);
/// - "Month day, year": "July 1, 2026", "Jul 01, 26" (the comma ends the day; "July 1 2026" and
///   "July 1,2026" are #VALUE!);
/// - "day Month year": "1 July 2026" (no comma);
/// - the comma of "Month day, year" may also stand as a word of its own after the day
///   ("July 1 , 2026"), after the month ("July , 1, 2026") or both ("July , 1 , 2026"); it still
///   needs a space after it ("July 1 ,2026" is #VALUE!). Day-first text never takes a comma
///   ("1 July , 2026" is #VALUE!);
/// - a month name glued to digits reads as with a space ("July1", "1JUN2021"), see
///   [`split_glued_month`].
///
/// Month names follow [`parse_month_name`]; a month name never comes first in a separated word
/// ("Jul-1-2026" is #VALUE!). Text with a clock time is `time_text`'s ("date clock" text).
pub(crate) fn parse_date(
    value: &str,
    locale: &Locale,
    current_year: CurrentYear,
) -> Result<(i32, String), String> {
    if value.trim() != value {
        return Err("Not a valid date".to_string());
    }
    // Spaces around a date separator are ignored ("2026 - 01 - 01", "4 - Feb - 2026"; probes/
    // datevalue-separator-spaces rows 1-4, 7-10, 13-14, 16-18 and datevalue-separator-spaces-paths, ironcalc-calc DATEVALUE_issue_1380.xlsx).
    if value.contains(" -") || value.contains("- ") || value.contains(" /") || value.contains("/ ")
    {
        let compact = remove_separator_spaces(value);
        // The compact text has no such space, so the recursion ends after one level.
        if compact != value {
            return parse_date(&compact, locale, current_year);
        }
    }
    // At most 5 words: one more than the longest form (4), enough to reject longer text without
    // splitting all of it.
    let words: Vec<&str> = value.split_whitespace().take(5).collect();
    // A month name glued to digits reads as with spaces between them ("July1" is "July 1",
    // "1JUN2021" is "1 JUN 2021"), except that a month glued to a number must end the text: a
    // month glued to a day that a year follows is #VALUE! ("July1,2026", "July12026",
    // "Jun1 2021"; probes/date-text-glued-month rows 1-24, excel-graph 2026-10-17).
    let mut parts: Vec<&str> = Vec::with_capacity(words.len());
    for (i, word) in words.iter().enumerate() {
        match split_glued_month(word, locale) {
            Some((_, true)) if i + 1 < words.len() => return Err("Not a valid date".to_string()),
            Some((glued, _)) => parts.extend(glued.into_iter().filter(|p| !p.is_empty())),
            None => parts.push(word),
        }
    }
    let (day, name, year) = match parts.as_slice() {
        [word] => return parse_separated(word, locale, current_year),
        [first, second] => {
            return match parse_month_name(second, locale) {
                Some((month, _)) => parse_day_month(first, month, current_year),
                None => parse_month_year(first, second, locale, false, current_year),
            };
        }
        [name, day, year] if day.ends_with(',') => (&day[..day.len() - 1], *name, *year),
        [day, name, year] => (*day, *name, *year),
        // A comma word of its own: "July 1 , 2026", "July , 1, 2026" (probes/datevalue-comma-spaces).
        [name, day, ",", year] => (*day, *name, *year),
        [name, ",", day, year] if day.ends_with(',') => (&day[..day.len() - 1], *name, *year),
        [name, ",", day, ",", year] => (*day, *name, *year),
        _ => return Err("Not a valid date".to_string()),
    };
    let (day, _) = parse_day(day)?;
    let (month, _) = parse_month_name(name, locale).ok_or("Not a valid month")?;
    let (year, _) = parse_year(year)?;
    Ok((
        serial_of(day, month, year)?,
        DAY_MONTH_NAME_YEAR_FORMAT.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse_date;
    use crate::functions::date_and_time::time_text::parse_date_clock_serial;
    use crate::locale::get_locale;

    /// The serial of en-US date text read with the clock in `year` (`None`: no clock).
    fn en_in(text: &str, year: Option<i32>) -> Option<i32> {
        let locale = get_locale("en").ok()?;
        parse_date(text, locale, &|| year)
            .ok()
            .map(|(serial, _)| serial)
    }

    fn en(text: &str) -> Option<i32> {
        en_in(text, Some(2026))
    }

    #[test]
    fn month_first_numeric_forms() {
        assert_eq!(en("1/13/2026"), Some(46035));
        assert_eq!(en("2/3/2023"), Some(44960));
        assert_eq!(en("10-12-1999"), Some(36445));
        assert_eq!(en("12/31/99"), Some(36525));
        assert_eq!(en("2026/1/13"), Some(46035));
        assert_eq!(en("2026-01-01"), Some(46023));
    }

    #[test]
    fn spaces_around_date_separators_are_ignored() {
        // probes/datevalue-separator-spaces rows 1-4, 7-10, 13-14, 16-18
        for (text, serial) in [
            ("2026 - 01 - 01", 46023),
            ("1 / 2 / 2026", 46024),
            ("4 - Feb - 2026", 46057),
            ("2026- 01-01", 46023),
            ("2026 -01-01", 46023),
            ("1 /2/2026", 46024),
            ("Jul - 2026", 46204),
            ("1 - Jul", 46204),
            ("7 - 1", 46204),
            ("2026  -  01  -  01", 46023),
            ("2026 / 1 / 13", 46035),
            ("1 / Jul / 2026", 46204),
        ] {
            assert_eq!(en(text), Some(serial), "{text}");
        }
        // A long run of spaces is removed in one pass.
        let padded = format!("2026-{}01-01", " ".repeat(32_000));
        assert_eq!(en(&padded), Some(46023));
        // Spaces away from a separator still make a different word count.
        assert_eq!(en("2026 01 01"), None);
    }

    #[test]
    fn comma_may_stand_as_its_own_word() {
        // probes/datevalue-comma-spaces
        for text in [
            "July 1 , 2026",
            "July 1  ,  2026",
            "July 1 ,  2026",
            "July , 1, 2026",
            "July , 1 , 2026",
        ] {
            assert_eq!(en(text), Some(46204), "{text}");
        }
        assert_eq!(en("July 1 , 26"), Some(46204));
        for text in [
            "July 1 ,2026",
            "Jul 1 ,2026",
            "1 July , 2026",
            "1 July, 2026",
        ] {
            assert_eq!(en(text), None, "{text}");
        }
    }

    #[test]
    fn day_first_only_forms_are_rejected() {
        for text in [
            "13/1/2026",
            "26-01-01",
            "31/12/99",
            "13-1-26",
            "15-01-2023",
            "29/02/2020",
        ] {
            assert_eq!(en(text), None, "{text}");
        }
    }

    #[test]
    fn month_name_forms() {
        for text in [
            "July 1, 2026",
            "Jul 1, 2026",
            "JULY 1, 2026",
            "July  1, 2026",
            "July 01, 2026",
            "Jul 1, 26",
            "1 July 2026",
            "1 Jul 2026",
            "1-Jul-2026",
            "1-Jul-26",
            "1-July-2026",
            "1/Jul/2026",
            "Jul-2026",
            "July 2026",
            "Jul 2026",
        ] {
            assert_eq!(en(text), Some(46204), "{text}");
        }
        assert_eq!(en("2-Feb-2026"), Some(46055));
        assert_eq!(en("Sept 1, 2026"), Some(46266));
        assert_eq!(en("Septem 1, 2026"), Some(46266));
        assert_eq!(en("Jul 1, 30"), Some(11140));
        assert_eq!(en("Feb 29, 2024"), Some(45351));
        assert_eq!(en("Feb 29, 1900"), Some(60));
    }

    #[test]
    fn month_and_two_digit_year_when_the_number_cannot_be_a_day() {
        // probes/date-text-yearless B13:B17
        assert_eq!(en("Jul-45"), Some(16619));
        assert_eq!(en("Jul-32"), Some(11871));
        assert_eq!(en("Jul-0"), Some(36708));
        assert_eq!(en("Jul-99"), Some(36342));
        assert_eq!(en("2/30"), Some(10990));
        // A numeric month with a four-digit year is the first of that month
        // (probes/date-text-yearless-2 rows 7-9).
        assert_eq!(en("7/2026"), Some(46204));
        assert_eq!(en("7-2026"), Some(46204));
        assert_eq!(en("12-1999"), Some(36495));
        // Not a day of February in 2026: a two-digit year, 2029-02-01.
        assert_eq!(en("Feb-29"), Some(47150));
        assert_eq!(en("2/29"), Some(47150));
        for text in [
            "13/1", "13/45", "0/45", "Jul-123",
            // A numeric month needs a separator: a space between numbers is #VALUE!
            // (probes/date-text-yearless-2 row 6)
            "7 45", "1 0",
            // Day first, a number that is not a day is never a year
            // (probes/date-text-yearless-2, date-text-yearless-3 rows 9-11)
            "45-Jul", "45 Jul", "45/Jul", "0-Jul", "32 Jul",
        ] {
            assert_eq!(en(text), None, "{text}");
        }
    }

    #[test]
    fn year_less_text_is_a_day_of_the_current_year() {
        // probes/date-text-yearless rows 1-10, date-text-yearless-3 rows 1-5, 12-13
        let days: [(i32, &[&str]); 5] = [
            (46229, &["Jul-26", "Jul 26", "Jul/26"]),
            (
                46204,
                &["7/1", "7-1", "1-Jul", "1 Jul", "1/Jul", "01-Jul", "Jul-01"],
            ),
            (46204, &["July 1", "July-1", "1 July"]),
            (46234, &["Jul-31"]),
            (46387, &["12/31"]),
        ];
        for (serial, texts) in days {
            for text in texts {
                assert_eq!(en(text), Some(serial), "{text}");
            }
        }
        assert_eq!(en("1-Sept"), Some(46266));
        // The year comes from the clock: 26 July 2024.
        assert_eq!(en_in("Jul-26", Some(2024)), Some(45499));
    }

    #[test]
    fn february_29_follows_the_current_year() {
        // Hand-derived from the rule (UNVERIFIED: the oracle has only run in 2026).
        for text in ["2/29", "Feb-29", "29-Feb", "Feb 29"] {
            assert_eq!(en_in(text, Some(2024)), Some(45351), "{text}");
        }
        assert_eq!(en_in("29-Feb", Some(2026)), None);
    }

    #[test]
    fn year_less_text_needs_a_clock() {
        for text in ["Jul-26", "7/1", "1-Jul"] {
            assert_eq!(en_in(text, None), None, "{text}");
        }
        // A two-digit year needs no clock.
        assert_eq!(en_in("Jul-45", None), Some(16619));
    }

    #[test]
    fn year_less_text_with_a_clock_time() {
        // probes/date-text-yearless D11, date-text-yearless-3 rows 6-8: VALUE - DATEVALUE
        let locale = get_locale("en").ok();
        let serial = |text| locale.and_then(|l| parse_date_clock_serial(text, l, &|| Some(2026)));
        for text in ["7/1 10:00", "Jul 1 10:00", "1-Jul 10:00"] {
            assert_eq!(
                serial(text).map(|s| s - 46204.0),
                Some(0.416_666_666_664_241_35),
                "{text}"
            );
        }
        assert_eq!(serial("Jul-26 6 PM"), Some(46229.75));
    }

    #[test]
    fn malformed_month_name_forms_are_rejected() {
        for text in [
            "July 1 2026",
            "July 1,2026",
            "Jul-1-2026",
            "July-1-2026",
            "Jul. 1, 2026",
            "July 1st, 2026",
            "Wednesday, July 1, 2026",
            "1 July, 2026",
            "July, 1 2026",
            "2026 July 1",
            "2023/Jan/15",
            "2023-Jan-15",
            "Ju 1, 2026",
            "July 32, 2026",
            "Feb 29, 2025",
            "Jan 0, 2026",
            "Jan 1, 10000",
            "Jan 1, 1899",
            "1.7.2026",
            "2026.01.01",
            " 2024-01-10 ",
            "July 1, 2026 ",
        ] {
            assert_eq!(en(text), None, "{text}");
        }
    }

    #[test]
    fn month_name_glued_to_digits() {
        // probes/date-text-glued-month column I (excel-graph, 2026-10-17), clock in 2026. Month
        // then one number is the year-less "Month d" rule (rows 1-9, 20, 22-23); day then month,
        // with or without a year, parses glued on either side (rows 12-16, 18-19, 24).
        let glued: [(i32, &[&str]); 10] = [
            (
                46204,
                &["July1", "Jul1", "Jul2026", "July2026", "1July", "1Jul2026"],
            ),
            (46213, &["July10"]),
            (46234, &["July31"]),
            (11871, &["July32"]),
            (16619, &["July45"]),
            (36708, &["July0"]),
            (46266, &["Sept1"]),
            (36526, &["January0"]),
            (36861, &["December0"]),
            (
                44348,
                &[
                    "01JUN21",
                    "1June2021",
                    "1jun2021",
                    "1JUN 2021",
                    "1 JUN2021",
                    "1JUN2021",
                ],
            ),
        ];
        for (serial, texts) in glued {
            for text in texts {
                assert_eq!(en(text), Some(serial), "{text}");
            }
        }
        // A month glued to a day that a year follows, and a name that is no month (rows 10, 11,
        // 17, 21); "Jul1 , 2026" follows rows 10 and 17 ("whatever joins them").
        for text in ["July1,2026", "July12026", "Jun1 2021", "Ju1", "Jul1 , 2026"] {
            assert_eq!(en(text), None, "{text}");
        }
        // The current year comes from the clock, as for spaced year-less text.
        assert_eq!(en_in("July1", None), None);
        assert_eq!(en_in("July45", None), Some(16619));
        // Letters that are no month name are never split off.
        for text in ["1e5", "Q12026", "A1", "10AM"] {
            assert_eq!(en(text), None, "{text}");
        }
    }

    #[test]
    fn day_first_locale_keeps_its_order() {
        let en_gb = get_locale("en-GB").ok();
        let parsed = en_gb.and_then(|l| parse_date("13/1/2026", l, &|| Some(2026)).ok());
        assert_eq!(parsed.map(|(serial, _)| serial), Some(46035));
    }
}
