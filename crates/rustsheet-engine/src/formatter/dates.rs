use chrono::Datelike;
use chrono::Duration;
use chrono::Months;
use chrono::NaiveDate;

use crate::constants::EXCEL_DATE_BASE;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;
use crate::types::DateSystem;

use super::parser::TextToken;

pub const DATE_OUT_OF_RANGE_MESSAGE: &str = "Out of range parameters for date";

/// Excel serial of the first real date after the fictitious 1900-02-29 (serial 60).
const FIRST_SERIAL_AFTER_LEAP_BUG: i32 = 61;

/// Excel's serial for a real calendar date. Excel counts a non-existent 1900-02-29 as serial 60,
/// so real dates before 1900-03-01 sit one lower than the plain day count from `EXCEL_DATE_BASE`.
/// Evidence: probes/dates-and-blanks A1-A3 (docs/excel-semantics/dates.md).
#[inline]
fn convert_to_serial_number(date: NaiveDate) -> i32 {
    let serial = date.num_days_from_ce() - EXCEL_DATE_BASE;
    if serial < FIRST_SERIAL_AFTER_LEAP_BUG {
        serial - 1
    } else {
        serial
    }
}

/// Whether a negative serial is formatted under a date/time section with these tokens rather than
/// being `#VALUE!` (SPEC-0005). In a 1904 workbook it always is (`TEXT(-1,"d")` is "-2"). In the
/// 1900 system only a section whose one date/time token is a single elapsed token (`[h]`, `[mm]`,
/// `[s]`, ...) is, whatever literals surround it (`TEXT(-1.5,"[h]"" h""")` is "-36 h"), while
/// `[h]:mm`, `[h] d` or `[s].00` are `#VALUE!`. Evidence: docs/excel-semantics/dates.md "The 1904
/// date system" and "Negative serials under an explicit negative section".
pub fn negative_serial_allowed(tokens: &[TextToken], date_system: DateSystem) -> bool {
    if date_system == DateSystem::Excel1904 {
        return true;
    }
    let mut elapsed = 0;
    for token in tokens {
        match token {
            TextToken::Literal(_)
            | TextToken::Text(_)
            | TextToken::Ghost(_)
            | TextToken::Spacer(_) => {}
            TextToken::ElapsedHour
            | TextToken::ElapsedHourPadded
            | TextToken::ElapsedMinute
            | TextToken::ElapsedMinutePadded
            | TextToken::ElapsedSecond
            | TextToken::ElapsedSecondPadded => elapsed += 1,
            _ => return false,
        }
    }
    elapsed == 1
}

/// Maps an Excel date serial (date part) to the `(year, month, day)` Excel shows for it,
/// including the fictitious 1900-02-29 (serial 60) and "1900-01-00" (serial 0). `None` outside
/// `0..=MAXIMUM_DATE_SERIAL_NUMBER`.
/// Evidence: probes/dates-early-1900 A20-A22 (`DAY(0)` = 0, `MONTH(0)` = 1, `YEAR(0)` = 1900).
pub fn excel_serial_to_ymd(serial: i64) -> Option<(i32, u32, u32)> {
    if serial == 0 {
        return Some((1900, 1, 0));
    }
    if !(i64::from(MINIMUM_DATE_SERIAL_NUMBER)..=i64::from(MAXIMUM_DATE_SERIAL_NUMBER))
        .contains(&serial)
    {
        return None;
    }
    if serial == 60 {
        return Some((1900, 2, 29));
    }
    let date = if serial < 60 {
        NaiveDate::from_ymd_opt(1900, 1, 1)? + Duration::days(serial - 1)
    } else {
        from_excel_date(serial).ok()?
    };
    Some((date.year(), date.month(), date.day()))
}

/// The real calendar date EDATE/EOMONTH do month arithmetic on. Serial 60 (the fictitious
/// 1900-02-29) counts as 1900-02-28 and serial 0 (day 0 of January 1900) stands in as 1900-01-01:
/// EOMONTH uses only its month, and EDATE steps back one serial from the shifted result.
/// Evidence: probes/dates-early-1900 A9/A10 (`EDATE(31,1)` = 59), A23/A25 (`EDATE(60,0)` =
/// `EOMONTH(60,0)` = 59); probes/dates-early-1900-edges A36-A44 (`EOMONTH(0,0)` = 31, `EDATE(0,1)` = 31).
pub fn month_arithmetic_date(serial: i64) -> Option<NaiveDate> {
    let (year, month, day) = excel_serial_to_ymd(serial)?;
    let day = day.max(1);
    NaiveDate::from_ymd_opt(year, month, day).or_else(|| NaiveDate::from_ymd_opt(year, month, 28))
}

/// Excel serial of a real calendar date, `None` below serial 1 (e.g. December 1899).
/// Evidence: probes/dates-early-1900 A18/A24 (`EOMONTH(1,-1)`, `EDATE(1,-1)` are #NUM!).
pub fn real_date_to_serial(date: NaiveDate) -> Option<i32> {
    let serial = convert_to_serial_number(date);
    (MINIMUM_DATE_SERIAL_NUMBER..=MAXIMUM_DATE_SERIAL_NUMBER)
        .contains(&serial)
        .then_some(serial)
}

fn is_date_within_range(date: NaiveDate) -> bool {
    real_date_to_serial(date).is_some()
}

pub fn from_excel_date(days: i64) -> Result<NaiveDate, String> {
    if days < MINIMUM_DATE_SERIAL_NUMBER as i64 {
        return Err(format!(
            "Excel date must be greater than {MINIMUM_DATE_SERIAL_NUMBER}"
        ));
    };
    if days > MAXIMUM_DATE_SERIAL_NUMBER as i64 {
        return Err(format!(
            "Excel date must be less than {MAXIMUM_DATE_SERIAL_NUMBER}"
        ));
    };
    #[allow(clippy::expect_used)]
    let dt = NaiveDate::from_ymd_opt(1900, 1, 1).expect("problem with chrono::NaiveDate");
    // NOTE: this is the weekday-faithful stand-in calendar, not Excel's DAY/MONTH/YEAR view: Excel
    // numbers weekdays as if serial 1 were a Sunday, so serials below 60 map one day early here
    // (serial 1 = 1899-12-31). Use `excel_serial_to_ymd` for the exact date components.
    Ok(dt + Duration::days(days - 2))
}

pub fn date_to_serial_number(day: u32, month: u32, year: i32) -> Result<i32, String> {
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(native_date) => Ok(convert_to_serial_number(native_date)),
        None => Err(DATE_OUT_OF_RANGE_MESSAGE.to_string()),
    }
}

pub fn permissive_date_to_serial_number(day: i32, month: i32, year: i32) -> Result<i32, String> {
    // Excel parses `DATE` very permissively. It allows not just for valid date values, but it
    // allows for invalid dates as well. If you for example enter `DATE(1900, 1, 32)` it will
    // return the date `1900-02-01`. Despite giving a day that is out of range it will just
    // wrap the month and year around.
    //
    // This function applies that same logic to dates. And does it in the most compatible way as
    // possible.

    let Some(mut date) = NaiveDate::from_ymd_opt(year, 1, 1) else {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    };

    // One thing to note for example is that even if you started with a year out of range
    // but tried to increment the months so that it wraps around into within range, excel
    // would still return an error.
    //
    // I.E. DATE(0,13,-1) will return an error, despite it being equivalent to DATE(1,1,0) which
    // is within range.
    //
    // As a result, we have to run range checks as we parse the date from the biggest unit to the
    // smallest unit.
    if !is_date_within_range(date) {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    }

    date = {
        let month_diff = month - 1;
        let abs_month = month_diff.unsigned_abs();
        if month_diff <= 0 {
            date = date - Months::new(abs_month);
        } else {
            date = date + Months::new(abs_month);
        }
        if !is_date_within_range(date) {
            return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
        }
        date
    };

    // Days wrap linearly from the first of the month in Excel serials, so the fictitious
    // 1900-02-29 is reachable: DATE(1900,2,29) = 60 and DATE(1900,2,30) = 61.
    let serial = i64::from(convert_to_serial_number(date)) + i64::from(day) - 1;
    // Excel returns 0 for DATE(1900,1,0) (probes/dates-early-1900 A4).
    if serial < 0 || serial > i64::from(MAXIMUM_DATE_SERIAL_NUMBER) {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    }
    Ok(serial as i32)
}

#[cfg(test)]
mod tests {
    #[test]
    fn month_arithmetic_and_real_serials_skip_the_fictitious_dates() {
        use super::{month_arithmetic_date, real_date_to_serial};
        use chrono::NaiveDate;
        assert_eq!(
            month_arithmetic_date(0),
            NaiveDate::from_ymd_opt(1900, 1, 1)
        );
        assert_eq!(
            month_arithmetic_date(60),
            NaiveDate::from_ymd_opt(1900, 2, 28)
        );
        assert_eq!(
            month_arithmetic_date(61),
            NaiveDate::from_ymd_opt(1900, 3, 1)
        );
        let feb_28 = NaiveDate::from_ymd_opt(1900, 2, 28).and_then(real_date_to_serial);
        assert_eq!(feb_28, Some(59));
        let dec_31 = NaiveDate::from_ymd_opt(1899, 12, 31).and_then(real_date_to_serial);
        assert_eq!(dec_31, None);
    }

    use super::*;

    #[test]
    fn test_excel_serial_to_ymd_early_1900() {
        assert_eq!(excel_serial_to_ymd(0), Some((1900, 1, 0)));
        assert_eq!(excel_serial_to_ymd(1), Some((1900, 1, 1)));
        assert_eq!(excel_serial_to_ymd(59), Some((1900, 2, 28)));
        assert_eq!(excel_serial_to_ymd(60), Some((1900, 2, 29)));
        assert_eq!(excel_serial_to_ymd(61), Some((1900, 3, 1)));
        assert_eq!(excel_serial_to_ymd(2_958_465), Some((9999, 12, 31)));
        assert_eq!(excel_serial_to_ymd(2_958_466), None);
    }

    #[test]
    fn test_permissive_date_early_1900_serials() {
        assert_eq!(permissive_date_to_serial_number(1, 1, 1900), Ok(1));
        assert_eq!(permissive_date_to_serial_number(28, 2, 1900), Ok(59));
        assert_eq!(permissive_date_to_serial_number(29, 2, 1900), Ok(60));
        assert_eq!(permissive_date_to_serial_number(1, 3, 1900), Ok(61));
        // Days wrap linearly from the first of the month (oracle-confirmed: probes/dates-early-1900 A2).
        assert_eq!(permissive_date_to_serial_number(30, 2, 1900), Ok(61));
    }

    #[test]
    fn test_permissive_date_to_serial_number() {
        assert_eq!(
            permissive_date_to_serial_number(42, 42, 2002),
            date_to_serial_number(12, 7, 2005)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 42, 2002),
            date_to_serial_number(1, 6, 2005)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 15, 2000),
            date_to_serial_number(1, 3, 2001)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000),
            date_to_serial_number(1, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000),
            date_to_serial_number(1, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 49, 2000),
            date_to_serial_number(31, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 49, 2000),
            date_to_serial_number(12, 9, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 1, 2004),
            date_to_serial_number(12, 9, 2004)
        );
    }

    #[test]
    fn test_max_and_min_dates() {
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 9999),
            Ok(MAXIMUM_DATE_SERIAL_NUMBER),
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 1, 1900),
            Ok(MINIMUM_DATE_SERIAL_NUMBER),
        );
        // DATE adds 1900 to years below 1900, so 1899 only reaches here from direct callers.
        assert!(permissive_date_to_serial_number(31, 12, 1899).is_err());
    }
}
