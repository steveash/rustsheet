use chrono::Datelike;
use chrono::Duration;
use chrono::Months;
use chrono::NaiveDate;

use crate::constants::EXCEL_DATE_BASE;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;

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

/// Maps an Excel date serial (date part) to the `(year, month, day)` Excel shows for it,
/// including the fictitious 1900-02-29 (serial 60). `None` outside `1..=MAXIMUM_DATE_SERIAL_NUMBER`.
pub fn excel_serial_to_ymd(serial: i64) -> Option<(i32, u32, u32)> {
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

fn is_date_within_range(date: NaiveDate) -> bool {
    convert_to_serial_number(date) >= MINIMUM_DATE_SERIAL_NUMBER
        && convert_to_serial_number(date) <= MAXIMUM_DATE_SERIAL_NUMBER
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
    use super::*;

    #[test]
    fn test_excel_serial_to_ymd_early_1900() {
        assert_eq!(excel_serial_to_ymd(0), None);
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
