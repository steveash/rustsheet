//! The coupon schedule of the securities functions: walking back from the maturity to the
//! coupon dates around settlement, and the period lengths and coupon counts derived from it.
//!
//! The walk steps through real calendar dates and reads each as an Excel serial, so 1900-01-01
//! is serial 1 and a date before it is 0 or below (`convert_to_serial_number`); the day counts
//! read a serial as the `(year, month, day)` Excel shows for it (`excel_serial_to_ymd`). Excel:
//! see docs/excel-semantics/financial.md#coupon-schedules-at-the-calendars-edges.

use chrono::{Datelike, NaiveDate};

use crate::formatter::dates::{
    convert_to_serial_number, excel_serial_to_ymd, month_arithmetic_date, shift_months,
};

use super::super::dates::{
    actual_days, days_30_360_eu_ymd, days_30_360_us_both_ymd, days_30_360_us_ymd, last_day_of_feb,
};

// ============================================================
// Date helpers
// ============================================================

/// The Excel serial of a coupon date: 0 or below before 1900-01-01.
pub(in crate::functions::financial) fn coupon_serial(d: NaiveDate) -> i64 {
    i64::from(convert_to_serial_number(d))
}

/// The real calendar date of a maturity serial (serial 60, Excel's fictitious 1900-02-29, reads
/// as 1900-02-28).
pub(in crate::functions::financial) fn coupon_date(serial: i64) -> Result<NaiveDate, String> {
    month_arithmetic_date(serial).ok_or_else(|| "Date out of range".to_string())
}

/// The `(year, month, day)` Excel shows for a serial the 30/360 day counts read: serial 0 is
/// 1900-01-00 and serial 60 is 1900-02-29.
pub(in crate::functions::financial) fn serial_ymd(serial: i64) -> Result<(i32, u32, u32), String> {
    excel_serial_to_ymd(serial).ok_or_else(|| "Date out of range".to_string())
}

/// Days from serial `from` to serial `to` on `basis`: the serial difference on the actual bases
/// 1-3, so Excel's fictitious 1900-02-29 counts.
pub(in crate::functions::financial) fn serial_basis_days(
    from: i64,
    to: i64,
    basis: u32,
) -> Result<f64, String> {
    match basis {
        0 => Ok(days_30_360_us_ymd(serial_ymd(from)?, serial_ymd(to)?)),
        1..=3 => Ok((to - from) as f64),
        4 => Ok(days_30_360_eu_ymd(serial_ymd(from)?, serial_ymd(to)?)),
        _ => Err("Invalid basis".to_string()),
    }
}

// ============================================================
// Coupon-schedule helpers
// ============================================================

pub(in crate::functions::financial) fn last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => last_day_of_feb(year),
        _ => 30,
    }
}

pub(in crate::functions::financial) fn is_last_day_of_month(d: NaiveDate) -> bool {
    d.day() == last_day_of_month(d.year(), d.month())
}

// Shifts `date` by `num_months` (clamping the day to the target month.
// When `return_last` is set the result rounds to the last
// day of the target month.
// The end-of-month coupon rule: when maturity
// falls on a month end, every coupon date does too (so a 11/30 maturity yields
// coupons on 5/31, not 5/30).
pub(in crate::functions::financial) fn change_month(
    date: NaiveDate,
    num_months: i32,
    return_last: bool,
) -> Result<NaiveDate, String> {
    let shifted = shift_months(date, num_months).ok_or("Date arithmetic error")?;
    if return_last {
        let last = last_day_of_month(shifted.year(), shifted.month());
        NaiveDate::from_ymd_opt(shifted.year(), shifted.month(), last)
            .ok_or("Date error".to_string())
    } else {
        Ok(shifted)
    }
}

// Walks the coupon schedule backwards from maturity to find the previous coupon
// date (<= settlement) and the next coupon date (> settlement). Port of
// `findPcdNcd`/`findCouponDates` from ExcelFinancialFunctions, including the
// end-of-month rule anchored on the maturity date. The previous coupon date is 0 when the
// walk passes the calendar's start (`COUPPCD(1,1000,2)` is 0, not 1899-09-26).
pub(in crate::functions::financial) fn find_pcd_ncd(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<(i64, i64), String> {
    let (pcd, ncd) = coupon_period(settlement, maturity, frequency)?;
    Ok((coupon_serial(pcd).max(0), coupon_serial(ncd)))
}

// The real calendar dates of the coupon period around settlement, with the end-of-month rule
// of the maturity. The previous one may fall before 1900-01-01. Each date is the maturity moved
// back a whole number of periods, its day clamped to the month, so the day does not drift after
// a short month: maturity 2027-11-29 quarterly has 2003-08-29 and 2003-11-29, not the 28th
// (`COUPPCD(37913,46720,4)` is 37862, `COUPDAYBS(37913,46720,4,4)` 50).
// Excel: see docs/excel-semantics/financial.md#coupon-dates-keep-the-maturitys-day.
fn coupon_period(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<(NaiveDate, NaiveDate), String> {
    let mat = coupon_date(maturity)?;
    let return_last = is_last_day_of_month(mat);
    let months_per = (12 / frequency) as i32;
    let mut periods = 0;
    let mut front = mat;
    let mut trailing = mat;
    while coupon_serial(front) > settlement {
        trailing = front;
        periods += 1;
        front = change_month(mat, -months_per * periods, return_last)?;
    }
    Ok((front, trailing))
}

// The coupon period around settlement on the schedule COUPDAYS' basis 1 counts around a
// 29 February (`coupdays_actual`): walked back from the maturity without the end-of-month rule,
// each date the one after it moved back a period, so its day can drift to the 28th.
fn chained_coupon_period(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<(NaiveDate, NaiveDate), String> {
    let months_per = (12 / frequency) as i32;
    let mat = coupon_date(maturity)?;

    let mut front = mat;
    let mut trailing = mat;
    while coupon_serial(front) > settlement {
        trailing = front;
        front = change_month(front, -months_per, false)?;
    }
    Ok((front, trailing))
}

// Latest coupon date that is <= settlement, anchored by maturity & frequency.
pub(in crate::functions::financial) fn prev_coupon_date(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<i64, String> {
    Ok(find_pcd_ncd(settlement, maturity, frequency)?.0)
}

// Earliest coupon date strictly after settlement.
pub(in crate::functions::financial) fn next_coupon_date(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<i64, String> {
    Ok(find_pcd_ncd(settlement, maturity, frequency)?.1)
}

// Number of coupon payments strictly after settlement up to and including maturity. #NUM!
// when the previous coupon date falls before 1900-01-01 (`COUPNUM(1,1000,2)`); PRICE, YIELD,
// DURATION and MDURATION count with it, so they are #NUM! there too.
pub(in crate::functions::financial) fn coupon_count(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<u32, String> {
    let months_per = 12 / frequency as i32;
    let (pcd, next) = coupon_period(settlement, maturity, frequency)?;
    if coupon_serial(pcd) < 1 {
        return Err("Previous coupon before the calendar".to_string());
    }
    let mat = coupon_date(maturity)?;

    let next_months = next.year() * 12 + next.month() as i32;
    let mat_months = mat.year() * 12 + mat.month() as i32;
    let diff = mat_months - next_months;
    if diff < 0 {
        return Err("Maturity before next coupon".to_string());
    }
    Ok((diff / months_per) as u32 + 1)
}

// Actual days of the coupon period that contains settlement: the serial difference.
pub(in crate::functions::financial) fn coupon_period_days(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<f64, String> {
    let (prev, next) = find_pcd_ncd(settlement, maturity, frequency)?;
    Ok((next - prev) as f64)
}

// Total days in the coupon period that contains settlement (E).
pub(in crate::functions::financial) fn coupon_days_e(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    match basis {
        0 | 2 | 4 => Ok(360.0 / frequency as f64),
        // The period PRICE, YIELD and DURATION use. The COUPDAYS function itself differs around
        // 29 February (`coupdays_actual`); keep the two apart.
        1 => coupon_period_days(settlement, maturity, frequency),
        3 => Ok(365.0 / frequency as f64),
        _ => Err("Invalid basis".to_string()),
    }
}

// COUPDAYS on basis 1. When the coupon period around settlement starts or ends on a 29 February
// that is not the maturity, Excel counts the period of the schedule walked back from the maturity
// without the end-of-month rule (every date clamped from the one after it): with maturity
// 2025-02-28 and settlement 2023-11-15 it is 2023-08-28 to 2024-02-28, 184 days, although
// COUPPCD/COUPNCD are 2023-08-31/2024-02-29. PRICE and the other readers keep the 182-day period.
// 1900-02-28 counts as such a 29 February, as if Excel's fictitious 1900-02-29 were the month's
// last day. A period that starts before 1900-01-01 is counted in the real calendar
// (`COUPDAYS(1,43830,4,1)` is 90, 1899-12-31 to 1900-03-31), one inside it by serials.
// Excel: see docs/excel-semantics/financial.md#coupdays-basis-1-around-29-february.
pub(in crate::functions::financial) fn coupdays_actual(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<f64, String> {
    let mat = coupon_date(maturity)?;
    let (prev, next) = coupon_period(settlement, maturity, frequency)?;
    let on_leap_day = |d: NaiveDate| {
        d != mat && d.month() == 2 && (d.day() == 29 || (d.year() == 1900 && d.day() == 28))
    };
    let (prev, next) = if on_leap_day(prev) || on_leap_day(next) {
        chained_coupon_period(settlement, maturity, frequency)?
    } else {
        (prev, next)
    };
    let prev_serial = coupon_serial(prev);
    if prev_serial < 1 {
        return Ok(actual_days(prev, next));
    }
    Ok((coupon_serial(next) - prev_serial) as f64)
}

// Accrued days from the previous coupon to settlement (A).
pub(in crate::functions::financial) fn coupon_days_a(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let prev = prev_coupon_date(settlement, maturity, frequency)?;
    serial_basis_days(prev, settlement, basis)
}

// COUPDAYSNC: days from settlement to the next coupon date.
pub(in crate::functions::financial) fn coupon_days_nc(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let (prev, next) = find_pcd_ncd(settlement, maturity, frequency)?;
    if basis == 0 {
        // ExcelFinancialFunctions enforces coupDaysNC = coupDays - coupDaysBS
        // for the US 30/360 basis: the total 30/360 days in the coupon period
        // (ModifyBothDates) minus the days from its start to settlement
        // (ModifyStartDate). This is *not* simply 30/360(settlement, next).
        let total = days_30_360_us_both_ymd(serial_ymd(prev)?, serial_ymd(next)?);
        return Ok(total - serial_basis_days(prev, settlement, basis)?);
    }
    serial_basis_days(settlement, next, basis)
}
