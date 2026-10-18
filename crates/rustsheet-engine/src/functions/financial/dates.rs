//! Date arguments of the financial functions: Excel reads a 1904 serial as the 1900 serial
//! s + 2 * 1462 (`DateSystem::financial_1900_serial`; docs/excel-semantics/dates.md "The 1904 date
//! system: financial day counts read a 1904 serial as the 1900 serial + 2 * 1462").
//! Also the coupon functions' day counts per basis and the coupon schedule Excel breaks in a 1904
//! workbook (`Model::coupon_schedule_breaks`).

use chrono::{Datelike, NaiveDate};

use crate::{
    calc_result::CalcResult,
    expressions::{token::Error, types::CellReferenceIndex},
    formatter::dates::{from_excel_date, EPOCH_1904_IN_1900_SERIALS},
    functions::lift::num_error,
    model::Model,
};

/// Whether `end_date` is at most a year after `start_date` (1900 serials), the TBILL* limit.
pub(super) fn is_less_than_one_year(start_date: i64, end_date: i64) -> Result<bool, String> {
    let end = from_excel_date(end_date)?;
    let start = from_excel_date(start_date)?;
    if end_date - start_date < 365 {
        return Ok(true);
    }
    let end_year = end.year();
    let start_year = start.year();
    if end_year == start_year {
        return Ok(true);
    }
    if end_year != start_year + 1 {
        return Ok(false);
    }
    let start_month = start.month();
    let end_month = end.month();
    if end_month < start_month {
        return Ok(true);
    }
    if end_month > start_month {
        return Ok(false);
    }
    // we are one year later same month
    let start_day = start.day();
    let end_day = end.day();
    Ok(end_day <= start_day)
}

impl Model {
    /// The floored 1900 serial a financial function's day counts read for the date argument
    /// `serial`: 2 * 1462 days on in a 1904 workbook (`DateSystem::financial_1900_serial`).
    pub(super) fn financial_date(&self, serial: f64) -> i64 {
        self.workbook
            .settings
            .date_system
            .financial_1900_serial(serial)
            .floor() as i64
    }

    /// Whether Excel's coupon schedule breaks for `maturity`, as `financial_date` read it
    /// (`DateSystem::coupon_schedule_breaks`): only in a 1904 workbook, from 9995-12-31 on.
    pub(super) fn coupon_schedule_breaks(&self, maturity: i64) -> bool {
        self.workbook
            .settings
            .date_system
            .coupon_schedule_breaks(maturity)
    }

    /// A date argument of a financial function as a kernel reads it (SPEC-0011): the number of
    /// `value` (a boolean is #VALUE!) as `financial_date` reads it.
    pub(super) fn financial_date_of(
        &self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<i64, CalcResult> {
        let serial = self.cast_number_no_bools_ref(value, cell)?;
        Ok(self.financial_date(serial))
    }

    /// The settlement check of the COUP* functions, PRICE, YIELD and DURATION: #NUM! unless the
    /// settlement is a calendar date before the maturity. Excel is #NUM! for a settlement below
    /// serial 1 in a 1900 workbook (0, 0.5, -0.5, -1, -1E15: probes/fin-coupon-walk-bounds A1-A13,
    /// A19, A20, fin-coupon-walk-bounds-b A11) and for a negative one in a 1904 workbook, whose 0
    /// reads as the 1900 serial 2924 (probes/date-system-1904-financial A35-A40). The check also
    /// keeps the coupon schedule walk back from the maturity inside the calendar: without it
    /// `COUPNUM(-1E15,43831,4)` stepped about 10^6 quarters back to chrono's first year
    /// (directive 5, T-1822; docs/excel-semantics/financial.md "Coupon settlement before the
    /// calendar").
    pub(super) fn check_settlement(
        &self,
        settlement: i64,
        maturity: i64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        if settlement < 1 || settlement >= maturity {
            return Err(num_error(cell));
        }
        Ok(())
    }

    /// The date checks PRICE, YIELD and DURATION make before computing: [`Model::check_settlement`]
    /// and a coupon schedule that does not break (`coupon_schedule_breaks`); #NUM! otherwise.
    pub(super) fn check_bond_dates(
        &self,
        settlement: i64,
        maturity: i64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.check_settlement(settlement, maturity, cell)?;
        if self.coupon_schedule_breaks(maturity) {
            return Err(broken_schedule_error(cell));
        }
        Ok(())
    }

    /// A date a financial function computed in the `financial_date` space (COUPNCD, COUPPCD), as
    /// this workbook's serial: 2 * 1462 days earlier in a 1904 workbook.
    pub(super) fn financial_date_result(&self, serial: i64) -> CalcResult {
        let date_system = self.workbook.settings.date_system;
        CalcResult::Number(date_system.serial_from_financial_1900_serial(serial) as f64)
    }
}

// Day counts of the coupon functions (bases 0-4).

pub(super) fn last_day_of_feb(year: i32) -> u32 {
    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        29
    } else {
        28
    }
}

// Days d1→d2 via the US (NASD) 30/360 convention (basis 0).
//
// Port of `dateDiff360Us` with the `ModifyStartDate` method from
// ExcelFinancialFunctions (the variant used by `DaysBetween` in the
// Numerator position). The order matters: the `day == 31 → 30` adjustment for
// the end date tests the *original* start day, before the start day is itself
// normalised.
pub(super) fn days_30_360_us(d1: NaiveDate, d2: NaiveDate) -> f64 {
    let y1 = d1.year();
    let m1 = d1.month() as i32;
    let mut day1 = d1.day() as i32;
    let y2 = d2.year();
    let m2 = d2.month() as i32;
    let mut day2 = d2.day() as i32;

    let start_is_feb_last = m1 == 2 && d1.day() == last_day_of_feb(y1);
    let end_is_feb_last = m2 == 2 && d2.day() == last_day_of_feb(y2);

    if end_is_feb_last && start_is_feb_last {
        day2 = 30;
    }
    if day2 == 31 && day1 >= 30 {
        day2 = 30;
    }
    if day1 == 31 {
        day1 = 30;
    }
    if start_is_feb_last {
        day1 = 30;
    }
    (360 * (y2 - y1) + 30 * (m2 - m1) + (day2 - day1)) as f64
}

// Days d1→d2 via US 30/360 with the `ModifyBothDates` rule — the "PSA hack"
// the odd-last-coupon code uses for basis 0.
pub(super) fn days_30_360_us_both(d1: NaiveDate, d2: NaiveDate) -> f64 {
    let y1 = d1.year();
    let m1 = d1.month() as i32;
    let mut day1 = d1.day() as i32;
    let y2 = d2.year();
    let m2 = d2.month() as i32;
    let mut day2 = d2.day() as i32;

    let start_is_feb_last = m1 == 2 && d1.day() == last_day_of_feb(y1);
    let end_is_feb_last = m2 == 2 && d2.day() == last_day_of_feb(y2);

    if end_is_feb_last {
        day2 = 30;
    }
    if day2 == 31 {
        day2 = 30;
    }
    if day1 == 31 {
        day1 = 30;
    }
    if start_is_feb_last {
        day1 = 30;
    }
    (360 * (y2 - y1) + 30 * (m2 - m1) + (day2 - day1)) as f64
}

// Days d1→d2 via the EU 30/360 convention (basis 4).
pub(super) fn days_30_360_eu(d1: NaiveDate, d2: NaiveDate) -> f64 {
    let y1 = d1.year();
    let m1 = d1.month() as i32;
    let day1 = (d1.day() as i32).min(30);
    let y2 = d2.year();
    let m2 = d2.month() as i32;
    let day2 = (d2.day() as i32).min(30);
    (360 * (y2 - y1) + 30 * (m2 - m1) + (day2 - day1)) as f64
}

pub(super) fn actual_days(d1: NaiveDate, d2: NaiveDate) -> f64 {
    (d2 - d1).num_days() as f64
}

// Basis-aware days from d1 to d2.
pub(super) fn basis_days(d1: NaiveDate, d2: NaiveDate, basis: u32) -> f64 {
    match basis {
        0 => days_30_360_us(d1, d2),
        1..=3 => actual_days(d1, d2),
        4 => days_30_360_eu(d1, d2),
        _ => 0.0,
    }
}

// The coupon schedule Excel breaks in a 1904 workbook (`Model::coupon_schedule_breaks`).

/// The previous coupon date (a 1900 serial) Excel takes when the coupon schedule breaks
/// (`Model::coupon_schedule_breaks`); the next one is then the maturity itself. It reads as
/// 1904 serial -1462 (`COUPPCD` is -1462), 1900 serial 1462 in the doubled-offset reading of
/// `DateSystem::financial_1900_serial` (T-1492).
pub(super) const BROKEN_SCHEDULE_PCD: i64 = EPOCH_1904_IN_1900_SERIALS;

/// Days between two serials of a broken coupon schedule: their difference on the actual bases
/// 1-3 (`COUPDAYS(DATE(9999,1,1),DATE(9999,12,31),2,1)` is 2958465 in a 1904 workbook), #NUM! on
/// the 30/360 bases 0 and 4 (probes/date-system-1904-coupon-bound A4-A6, K1-L8).
pub(super) fn broken_schedule_days(
    from: i64,
    to: i64,
    basis: u32,
    cell: CellReferenceIndex,
) -> CalcResult {
    if (1..=3).contains(&basis) {
        CalcResult::Number(to.saturating_sub(from) as f64)
    } else {
        broken_schedule_error(cell)
    }
}

/// COUPNUM, PRICE, YIELD, DURATION, MDURATION and ACCRINT (on its `first_interest`) are #NUM! when
/// the coupon schedule breaks (probes/date-system-1904-coupon-bound A3, A7, A19, A20, H3, I3, M1;
/// date-system-1904-odd-accrint-bound B11:D15).
pub(super) fn broken_schedule_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, "no coupon schedule".to_string())
}
