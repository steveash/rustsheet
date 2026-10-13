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

/// Days from 1900-01-00 (1900-system serial 0) to 1904-01-01 (1904-system serial 0), counting
/// Excel's fictitious 1900-02-29: 1904 serial 0 is 1900 serial 1462 (probes/date-system-1904-negatives
/// A8-A12, docs/excel-semantics/dates.md "The 1904 date system").
const EPOCH_1904_IN_1900_SERIALS: i64 = 1462;

/// The 1900 serial the financial functions read for 1904 serial 0: one more than
/// `EPOCH_1904_IN_1900_SERIALS` (see `DateSystem::financial_1900_serial`).
const FINANCIAL_EPOCH_1904_IN_1900_SERIALS: i64 = EPOCH_1904_IN_1900_SERIALS + 1;

impl DateSystem {
    /// The 1900-system serial of this system's day `serial`, the form the calendar helpers
    /// (`excel_serial_to_ymd`) read. `None` for a 1904 serial before the epoch, which is no date
    /// (DATE gives no serial below 0 there). Checked, since serials come from untrusted input.
    pub(crate) fn to_1900_serial(self, serial: i64) -> Option<i64> {
        match self {
            Self::Excel1900 => Some(serial),
            Self::Excel1904 if serial < 0 => None,
            Self::Excel1904 => serial.checked_add(EPOCH_1904_IN_1900_SERIALS),
        }
    }

    /// This system's serial for the 1900-system serial `serial` that the calendar helpers produce
    /// (`DATE`, date text); `None` for a day before the 1904 epoch, which has no 1904 serial.
    pub(crate) fn serial_from_1900_serial(self, serial: i64) -> Option<i64> {
        match self {
            Self::Excel1900 => Some(serial),
            Self::Excel1904 => serial
                .checked_sub(EPOCH_1904_IN_1900_SERIALS)
                .filter(|s| *s >= 0),
        }
    }

    /// Like `serial_from_1900_serial`, but a day before the 1904 epoch keeps its negative serial:
    /// EDATE and EOMONTH return such days (`EDATE(0,-1)` is -31 and `EOMONTH(0,-1)` is -1 in a 1904
    /// workbook; probes/date-system-1904-readers A8, A9). WORKDAY gives #NUM! there instead (A43).
    pub(crate) fn signed_serial_from_1900_serial(self, serial: i64) -> i64 {
        match self {
            Self::Excel1900 => serial,
            Self::Excel1904 => serial - EPOCH_1904_IN_1900_SERIALS,
        }
    }

    /// The 1900-system serial that YEARFRAC and the financial functions (COUP*, PRICE, ACCRINT,
    /// AMORLINC, ...) read for this system's date argument `serial`, before truncation. In a 1904
    /// workbook they read every date one day late, as if it were serial + 1463, not + 1462:
    /// `COUPPCD(DATE(2030,1,15),DATE(2030,8,30),4)` is 2029-11-29 and `YEARFRAC(0,59,0)` is 59/360
    /// (probes/date-system-1904-financial-shift A1, A26; dates.md "The 1904 date system"). A
    /// negative 1904 serial stays negative, so it is no date there, as in the 1900 system.
    pub(crate) fn financial_1900_serial(self, serial: f64) -> f64 {
        match self {
            Self::Excel1904 if serial >= 0.0 => {
                serial + FINANCIAL_EPOCH_1904_IN_1900_SERIALS as f64
            }
            Self::Excel1900 | Self::Excel1904 => serial,
        }
    }

    /// The 1900 serial ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD read for this system's date
    /// argument `serial`; `None` where they are #NUM!. In a 1904 workbook they move the date 1462
    /// days forward and then four calendar years back. Across four years with one 29 February that
    /// is the one-day-late reading of `financial_1900_serial`, but a date within four years before
    /// 1 March of a non-leap century year (2100, 2200, ..., 9900) moves one day less, and a date
    /// that moves past 9999-12-31 is #NUM! (probes/date-system-1904-odd-years, T-1457).
    /// A forward move onto a 29 February whose year four back has none lands on 28 February
    /// (`shift_months`; UNVERIFIED). A negative 1904 serial stays negative, as in
    /// `financial_1900_serial`. `serial` is untrusted, so the arithmetic is checked.
    pub(crate) fn odd_period_1900_serial(self, serial: f64) -> Option<i64> {
        let day = serial.floor() as i64;
        match self {
            Self::Excel1904 if day >= 0 => {
                let moved = day
                    .checked_add(EPOCH_1904_IN_1900_SERIALS)?
                    .checked_add(EPOCH_1904_IN_1900_SERIALS)?;
                let date = shift_months(from_excel_date(moved).ok()?, -48)?;
                real_date_to_serial(date).map(i64::from)
            }
            Self::Excel1900 | Self::Excel1904 => Some(day),
        }
    }

    /// Whether Excel's coupon schedule breaks for a maturity the financial functions read as
    /// `financial_maturity` (a `financial_1900_serial`). In a 1904 workbook it does for a maturity
    /// serial m with m + 2 * 1462 past the last date, 2958465: 9995-12-30 still has a schedule,
    /// 9995-12-31 does not (probes/date-system-1904-coupon-bound G1-G2, T-1440). Excel then takes
    /// the previous coupon as 1900 serial 1 and the next as the maturity read. A maturity past the
    /// last date stays #NUM! (`COUPPCD(DATE(2024,1,15),2957004,2)`, date-system-1904-financial A44).
    /// The maturity is untrusted (a saturated `i64::MAX` for `1E+300`), so the bounds are compared
    /// without arithmetic on it.
    pub(crate) fn coupon_schedule_breaks(self, financial_maturity: i64) -> bool {
        let last = i64::from(MAXIMUM_DATE_SERIAL_NUMBER);
        match self {
            Self::Excel1900 => false,
            Self::Excel1904 => {
                // The 1904 maturity is `financial_maturity - FINANCIAL_EPOCH_1904_IN_1900_SERIALS`.
                let first_broken = last - 2 * EPOCH_1904_IN_1900_SERIALS
                    + 1
                    + FINANCIAL_EPOCH_1904_IN_1900_SERIALS;
                let last_date =
                    last - EPOCH_1904_IN_1900_SERIALS + FINANCIAL_EPOCH_1904_IN_1900_SERIALS;
                (first_broken..=last_date).contains(&financial_maturity)
            }
        }
    }

    /// This system's serial for a date the financial functions computed as a 1900 serial in the
    /// space of `financial_1900_serial` (COUPNCD, COUPPCD): one day before that date in a 1904
    /// workbook, so it can be before the epoch (`COUPPCD(10,DATE(1905,6,30),2)` is -1).
    pub(crate) fn serial_from_financial_1900_serial(self, serial: i64) -> i64 {
        match self {
            Self::Excel1900 => serial,
            Self::Excel1904 => serial - FINANCIAL_EPOCH_1904_IN_1900_SERIALS,
        }
    }

    /// The weekday-faithful calendar date (`from_excel_date`) of this system's day `serial`, the
    /// date WEEKDAY, WORKDAY, NETWORKDAYS, YEARFRAC and DATEDIF read; `None` where they are #NUM!.
    /// A 1904 serial below 0 is no date: `WEEKDAY(-1)` is #NUM! (probes/date-system-1904-readers A41).
    pub(crate) fn date_of_serial(self, serial: i64) -> Option<NaiveDate> {
        from_excel_date(self.to_1900_serial(serial)?).ok()
    }
}

/// Whether a negative serial is formatted under a date/time section with these tokens rather than
/// being `#VALUE!` (SPEC-0005). In a 1904 workbook it always is (`TEXT(-1,"d")` is "-2"). In the
/// 1900 system only a section whose one date/time token is a single elapsed token (`[h]`, `[mm]`,
/// `[s]`, ...) is, whatever literals surround it (`TEXT(-1.5,"[h]"" h""")` is "-36 h"), while
/// `[h]:mm`, `[h] d` or `[s].00` are `#VALUE!`. Evidence: docs/excel-semantics/dates.md "The 1904
/// date system" and "Negative serials under an explicit negative section". `_x` and `*x` count as
/// literals by extrapolation (unprobed, T-1366).
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

/// `date` moved by `months` calendar months, `None` past chrono's range so a huge offset
/// (`EDATE(40000,2000000000)`) is #NUM! instead of a panic.
pub fn shift_months(date: NaiveDate, months: i32) -> Option<NaiveDate> {
    let shift = Months::new(months.unsigned_abs());
    if months > 0 {
        date.checked_add_months(shift)
    } else {
        date.checked_sub_months(shift)
    }
}

/// DATE's month and day arguments, floored and bounded as Excel does: a month outside
/// -32767..=32766 is `None` (#NUM!) even when the date it names is in the calendar
/// (`DATE(2000,32767,1)`, `DATE(9999,-32768,1)`), and a day outside the 16-bit range
/// -32768..=32767 counts as 32767 (`DATE(2000,1,-1E+15)` = `DATE(2000,1,32767)`).
/// Evidence: probes/date-huge-month-day, docs/excel-semantics/dates.md "DATE month and day bounds".
pub fn date_month_and_day(month: f64, day: f64) -> Option<(i32, i32)> {
    let month = month.floor();
    if !(-32767.0..=32766.0).contains(&month) {
        return None;
    }
    let day = day.floor();
    let day = if (-32768.0..=32767.0).contains(&day) {
        day as i32
    } else {
        32767
    };
    Some((month as i32, day))
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

    // A month near `i32::MAX`/`MIN` would overflow `month - 1` and chrono's calendar (a panic in
    // `NaiveDate + Months`, fuzz T-1419), so both steps are checked and an overflow is out of range.
    date = match month
        .checked_sub(1)
        .and_then(|offset| shift_months(date, offset))
    {
        Some(date) if is_date_within_range(date) => date,
        _ => return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string()),
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
    fn coupon_schedule_breaks_at_its_bounds_without_overflow() {
        use crate::types::DateSystem;
        // 1904 serials 2955541 (9995-12-30) and 2955542 (9995-12-31) read one day late (+1463);
        // 2957003 is the last date, 2957004 is past it (probes/date-system-1904-coupon-bound).
        let read = |serial: i64| serial + 1463;
        assert!(!DateSystem::Excel1904.coupon_schedule_breaks(read(2_955_541)));
        assert!(DateSystem::Excel1904.coupon_schedule_breaks(read(2_955_542)));
        assert!(DateSystem::Excel1904.coupon_schedule_breaks(read(2_957_003)));
        assert!(!DateSystem::Excel1904.coupon_schedule_breaks(read(2_957_004)));
        // A saturated `1E+300` or `-1E+300` maturity (M6-M8).
        assert!(!DateSystem::Excel1904.coupon_schedule_breaks(i64::MAX));
        assert!(!DateSystem::Excel1904.coupon_schedule_breaks(i64::MIN));
        assert!(!DateSystem::Excel1900.coupon_schedule_breaks(2_958_465));
    }

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
