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

/// A numeric day: one or two digits ("1", "01").
fn parse_day(day_str: &str) -> Result<(u32, &'static str), String> {
    if !is_digits(day_str, 1..=2) {
        return Err("Not a valid day".to_string());
    }
    let day = day_str.parse::<u32>().map_err(|e| e.to_string())?;
    Ok((day, if day_str.len() == 2 { "dd" } else { "d" }))
}

/// A numeric month: one or two digits ("7", "07").
fn parse_month_number(month_str: &str) -> Result<(u32, &'static str), String> {
    if !is_digits(month_str, 1..=2) {
        return Err("Not a valid month".to_string());
    }
    let month = month_str.parse::<u32>().map_err(|e| e.to_string())?;
    Ok((month, if month_str.len() == 2 { "mm" } else { "m" }))
}

/// A month name, ignoring case: the locale's short name, or any prefix of at least three letters
/// of its full name ("Sept" and "Septem" are September; probes/date-text-grammar A23-A25).
fn parse_month_name(name: &str, locale: &Locale) -> Option<(u32, &'static str)> {
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
    if !is_digits(year_str, 2..=2) && !is_digits(year_str, 4..=4) {
        return Err("Not a valid year".to_string());
    }
    let year = year_str.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(match (year_str.len(), year) {
        (2, y) if y < 30 => (2000 + y, "yy"),
        (2, y) => (1900 + y, "yy"),
        (_, y) => (y, "yyyy"),
    })
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

/// "Month year" text is the first of that month; the year must have four digits because Excel
/// reads "Jul-26" as 26 July of the current year (clock-dependent, not implemented).
fn parse_month_year(name: &str, year_str: &str, locale: &Locale) -> Result<(i32, String), String> {
    let (month, _) = parse_month_name(name, locale).ok_or("Not a valid month")?;
    if !is_digits(year_str, 4..=4) {
        return Err("Not a valid year".to_string());
    }
    let (year, _) = parse_year(year_str)?;
    Ok((
        serial_of(1, month, year)?,
        MONTH_NAME_YEAR_FORMAT.to_string(),
    ))
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
    let parts: Vec<&str> = word.split(separator).collect();
    let (day_str, month_str, year_str) = match parts.as_slice() {
        [name, year] => return parse_month_year(name, year, locale),
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
        [day, name, year] if parse_month_name(name, locale).is_some() => {
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
    let words: Vec<&str> = value.split_whitespace().collect();
    match words.as_slice() {
        [word] => parse_separated(word, locale),
        [name, year] => parse_month_year(name, year, locale),
        [name, day, year] if day.ends_with(',') => {
            let (month, _) = parse_month_name(name, locale).ok_or("Not a valid month")?;
            let (day, _) = parse_day(&day[..day.len() - 1])?;
            let (year, _) = parse_year(year)?;
            Ok((
                serial_of(day, month, year)?,
                DAY_MONTH_NAME_YEAR_FORMAT.to_string(),
            ))
        }
        [day, name, year] => {
            let (day, _) = parse_day(day)?;
            let (month, _) = parse_month_name(name, locale).ok_or("Not a valid month")?;
            let (year, _) = parse_year(year)?;
            Ok((
                serial_of(day, month, year)?,
                DAY_MONTH_NAME_YEAR_FORMAT.to_string(),
            ))
        }
        _ => Err("Not a valid date".to_string()),
    }
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
            "Jul-26",
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
