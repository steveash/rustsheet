//! The coupon schedule of the securities functions: walking back from the maturity to the
//! coupon dates around settlement, and the period lengths and coupon counts derived from it.

use chrono::{Datelike, NaiveDate};

use crate::constants::EXCEL_DATE_BASE;
use crate::formatter::dates::{from_excel_date, shift_months};

use super::super::dates::{actual_days, basis_days, last_day_of_feb};

// ============================================================
// Date helpers
// ============================================================

fn naivedate_to_serial(d: NaiveDate) -> i64 {
    (d.num_days_from_ce() - EXCEL_DATE_BASE) as i64
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
// end-of-month rule anchored on the maturity date.
pub(in crate::functions::financial) fn find_pcd_ncd(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<(i64, i64), String> {
    let return_last = is_last_day_of_month(from_excel_date(maturity)?);
    walk_coupon_schedule(settlement, maturity, frequency, return_last)
}

// The schedule walk of `find_pcd_ncd`, with the end-of-month rule given by the caller.
fn walk_coupon_schedule(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    return_last: bool,
) -> Result<(i64, i64), String> {
    let months_per = (12 / frequency) as i32;
    let mat = from_excel_date(maturity)?;

    let mut front = mat;
    let mut trailing = mat;
    while naivedate_to_serial(front) > settlement {
        trailing = front;
        front = change_month(front, -months_per, return_last)?;
    }
    Ok((naivedate_to_serial(front), naivedate_to_serial(trailing)))
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

// Number of coupon payments strictly after settlement up to and including maturity.
pub(in crate::functions::financial) fn coupon_count(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<u32, String> {
    let months_per = 12 / frequency as i32;
    let next = from_excel_date(next_coupon_date(settlement, maturity, frequency)?)?;
    let mat = from_excel_date(maturity)?;

    let next_months = next.year() * 12 + next.month() as i32;
    let mat_months = mat.year() * 12 + mat.month() as i32;
    let diff = mat_months - next_months;
    if diff < 0 {
        return Err("Maturity before next coupon".to_string());
    }
    Ok((diff / months_per) as u32 + 1)
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
        1 => {
            let prev = from_excel_date(prev_coupon_date(settlement, maturity, frequency)?)?;
            let next = from_excel_date(next_coupon_date(settlement, maturity, frequency)?)?;
            Ok(actual_days(prev, next))
        }
        3 => Ok(365.0 / frequency as f64),
        _ => Err("Invalid basis".to_string()),
    }
}

// COUPDAYS on basis 1. When the coupon period around settlement starts or ends on a 29 February
// that is not the maturity, Excel counts the period of the schedule walked back from the maturity
// without the end-of-month rule (every date clamped from the one after it): with maturity
// 2025-02-28 and settlement 2023-11-15 it is 2023-08-28 to 2024-02-28, 184 days, although
// COUPPCD/COUPNCD are 2023-08-31/2024-02-29. PRICE and the other readers keep the 182-day period.
// Excel: see docs/excel-semantics/financial.md#coupdays-basis-1-around-29-february.
pub(in crate::functions::financial) fn coupdays_actual(
    settlement: i64,
    maturity: i64,
    frequency: u32,
) -> Result<f64, String> {
    let (prev, next) = find_pcd_ncd(settlement, maturity, frequency)?;
    let on_leap_day = |serial: i64| -> Result<bool, String> {
        let d = from_excel_date(serial)?;
        Ok(serial != maturity && d.month() == 2 && d.day() == 29)
    };
    let (prev, next) = if on_leap_day(prev)? || on_leap_day(next)? {
        walk_coupon_schedule(settlement, maturity, frequency, false)?
    } else {
        (prev, next)
    };
    Ok(actual_days(from_excel_date(prev)?, from_excel_date(next)?))
}

// Accrued days from the previous coupon to settlement (A).
pub(in crate::functions::financial) fn coupon_days_a(
    settlement: i64,
    maturity: i64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let prev = from_excel_date(prev_coupon_date(settlement, maturity, frequency)?)?;
    let set = from_excel_date(settlement)?;
    Ok(basis_days(prev, set, basis))
}
