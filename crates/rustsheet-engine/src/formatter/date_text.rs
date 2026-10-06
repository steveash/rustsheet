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
/// Display format Excel gives typed "month-name year" text (`Jul-26`).
const MONTH_NAME_YEAR_FORMAT: &str = "mmm-yy";

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

/// "Month year" text is the first of that month. A one- or two-digit number that can be a day
/// of the month is a day of the current year (#VALUE! until T-0167), any other is a two-digit
/// year: "Jul-45" is 1 July 1945 (probes/date-text-yearless). `numeric_month` admits a month
/// number ("2/30"), only for separated words in month-first locales.
fn parse_month_year(
    name: &str,
    year_str: &str,
    locale: &Locale,
    numeric_month: bool,
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
    // A numeric month takes only the one- or two-digit number probes/date-text-yearless covers;
    // "7/2026" is not probed.
    let year = if is_digits(year_str, 4..=4) && format == MONTH_NAME_YEAR_FORMAT {
        parse_year(year_str)?.0
    } else if is_digits(year_str, 1..=2) {
        let number = year_str.parse::<i32>().map_err(|e| e.to_string())?;
        if u32::try_from(number).is_ok_and(|day| (1..=max_days_in_month(month)).contains(&day)) {
            return Err("Year-less date text needs the current year".to_string());
        }
        two_digit_year(number)
    } else {
        return Err("Not a valid year".to_string());
    };
    Ok((serial_of(1, month, year)?, format.to_string()))
}

/// One word with a date separator: "7/1/2026", "2026-07-01", "1-Jul-2026", "1/Jul/26",
/// "Jul-2026". Separators are `/` and `-`, plus `.` in locales whose short date uses it (en-US
/// rejects "1.7.2026"; probes/date-text-grammar A39/A40).
fn parse_separated(word: &str, locale: &Locale) -> Result<(i32, String), String> {
    let short_format = &locale.dates.date_formats.short;
    let separator = ['/', '-', '.']
        .into_iter()
        .filter(|&s| s != '.' || short_format.contains('.'))
        .find(|&s| word.contains(s))
        .ok_or("Not a valid date")?;
    // At most 4 parts: enough to reject longer text without splitting all of it.
    let parts: Vec<&str> = word.splitn(4, separator).collect();
    let (day_str, month_str, year_str) = match parts.as_slice() {
        [name, year] => return parse_month_year(name, year, locale, true),
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

/// Parses date text, returning its serial and the number format Excel gives it when typed.
///
/// Surrounding spaces make the text invalid: VALUE, DATEVALUE and criteria trim it first, but
/// number coercion does not (`MROUND(" 2024-01-10 ",7)` is #VALUE!; ironcalc-calc
/// MROUND_TRUNC_INT.xlsx MROUND!C32, agreed by the oracle in docs/research/oracle-calibration.md).
///
/// The grammar (repeated inner spaces count as one; probes/date-text-grammar A54):
/// - one word with a separator, see [`parse_separated`];
/// - "Month year": "Jul 2026", "July 2026" (the first of the month);
/// - "Month day, year": "July 1, 2026", "Jul 01, 26" (the comma ends the day; "July 1 2026" and
///   "July 1,2026" are #VALUE!);
/// - "day Month year": "1 July 2026" (no comma).
///
/// Month names follow [`parse_month_name`]; a month name never comes first in a separated word
/// ("Jul-1-2026" is #VALUE!). Text with a clock time is `time_text`'s ("date clock" text).
pub(crate) fn parse_date(value: &str, locale: &Locale) -> Result<(i32, String), String> {
    if value.trim() != value {
        return Err("Not a valid date".to_string());
    }
    // At most 4 words: enough to reject longer text without splitting all of it.
    let words: Vec<&str> = value.split_whitespace().take(4).collect();
    let (day, name, year) = match words.as_slice() {
        [word] => return parse_separated(word, locale),
        [name, year] => return parse_month_year(name, year, locale, false),
        [name, day, year] if day.ends_with(',') => (&day[..day.len() - 1], *name, *year),
        [day, name, year] => (*day, *name, *year),
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
    use crate::locale::get_locale;

    fn en(text: &str) -> Option<i32> {
        let locale = get_locale("en").ok()?;
        parse_date(text, locale).ok().map(|(serial, _)| serial)
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
        // A possible day of the month is the current year's (not implemented yet, T-0167)
        for text in [
            "Jul-26", "7/1", "Jul 31", "Feb-29", "2/29", "13/1", "13/45", "0/45", "Jul-123",
            // A numeric month needs a separator (a space between numbers is not probed), and a
            // four-digit year with it is not probed either
            "7 45", "1 0", "7/2026", "12-1999",
        ] {
            assert_eq!(en(text), None, "{text}");
        }
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
    fn day_first_locale_keeps_its_order() {
        let en_gb = get_locale("en-GB").ok();
        let parsed = en_gb.and_then(|l| parse_date("13/1/2026", l).ok());
        assert_eq!(parsed.map(|(serial, _)| serial), Some(46035));
    }
}
