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
    walk_coupon_schedule(settlement, mat, |_, periods| {
        change_month(mat, -months_per * periods, return_last)
    })
}

// The next coupon date after settlement on the schedule COUPDAYS' basis 1 walks (`coupdays_actual`):
// walked back from the maturity without the end-of-month rule, each date the one after it moved
// back a period, so its day can drift (maturity 2019-03-31 semiannual: 2018-09-30, 2018-03-30).
fn chained_next_coupon(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<NaiveDate, String> {
    let months_per = (12 / frequency) as i32;
    let (_, next) = walk_coupon_schedule(settlement, coupon_date(maturity)?, |later, _| {
        change_month(later, -months_per, false)
    })?;
    Ok(next)
}

// Walks back from `mat` until a date is on or before settlement: `step(later, periods)` gives the
// date `periods` periods before the maturity from the one after it, `later`.
fn walk_coupon_schedule(
    settlement: i64,
    mat: NaiveDate,
    step: impl Fn(NaiveDate, i32) -> Result<NaiveDate, String>,
) -> Result<(NaiveDate, NaiveDate), String> {
    let mut periods = 0;
    let mut front = mat;
    let mut trailing = mat;
    while coupon_serial(front) > settlement {
        trailing = front;
        periods += 1;
        front = step(front, periods)?;
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
        // The period PRICE, YIELD and DURATION use. The COUPDAYS function itself counts another
        // period on month-end schedules (`coupdays_actual`); keep the two apart.
        1 => coupon_period_days(settlement, maturity, frequency),
        3 => Ok(365.0 / frequency as f64),
        _ => Err("Invalid basis".to_string()),
    }
}

// COUPDAYS on basis 1, which is not COUPNCD - COUPPCD. Excel takes the next coupon date B of the
// schedule walked back from the maturity without the end-of-month rule (`chained_next_coupon`) and
// counts from B moved back one period: to its month's last day when B is a month end, else the
// day clamped. So maturity 2019-03-31 semiannual gives 181 for settlement 1950-01-15 (1949-09-30
// to 1950-03-30) and 183 for 1950-06-15 (1950-03-31 to 1950-09-30), and maturity 2025-02-28 gives
// 184 for 2023-11-15 (2023-08-28 to 2024-02-28: 2024-02-28 is not a month end). Excel's 1900 has a
// 29 February, so 1900-02-28 is not a month end. When that start is after settlement (settlement
// 1900-03-30, maturity 2019-03-31: the start is 1900-03-31), Excel moves back once more (B becomes
// the start). A date before 1900 compares one higher than its serial here (1899-12-31 as 1, so
// settlement 0 to maturity 2019-12-31 counts 1899-06-30 to 1899-12-31, 184). A period that starts
// before 1900-01-01 counts real days, except after that extra step into the calendar, which counts
// B's serial (the start reads as serial 0): `COUPDAYS(90,DATE(2019,3,31),2,1)` is 91.
// Excel: see docs/excel-semantics/financial.md#coupdays-basis-1-counts-from-the-chained-next-coupon.
pub(in crate::functions::financial) fn coupdays_actual(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<f64, String> {
    let months_per = (12 / frequency) as i32;
    let calendar_start = coupon_date(1)?;
    // 1900-02-28 is not a month end in Excel's 1900, which has a 29 February.
    let back_one = |later: NaiveDate| {
        let month_end =
            is_last_day_of_month(later) && !(later.year() == 1900 && later.month() == 2);
        change_month(later, -months_per, month_end)
    };
    // A date before 1900 compares as its serial plus one.
    let compare_serial = |d: NaiveDate| coupon_serial(d) + i64::from(d < calendar_start);
    let mut next = chained_next_coupon(settlement, maturity, frequency)?;
    let mut prev = back_one(next)?;
    let stepped_back = compare_serial(prev) > settlement;
    if stepped_back {
        next = prev;
        prev = back_one(next)?;
    }
    if next < calendar_start || (prev < calendar_start && !stepped_back) {
        return Ok(actual_days(prev, next));
    }
    Ok((coupon_serial(next) - coupon_serial(prev).max(0)) as f64)
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
