//! Date arguments of the financial functions: Excel reads a 1904 serial as the 1900 serial
//! s + 2 * 1462 (`DateSystem::financial_1900_serial`; docs/excel-semantics/dates.md "The 1904 date
//! system: financial day counts read a 1904 serial as the 1900 serial + 2 * 1462").
//! Also the coupon functions' day counts per basis and the coupon schedule Excel breaks in a 1904
//! workbook (`Model::coupon_schedule_breaks`).

use chrono::{Datelike, NaiveDate};

use crate::{
    calc_result::CalcResult,
    constants::MAXIMUM_DATE_SERIAL_NUMBER,
    expressions::{token::Error, types::CellReferenceIndex},
    formatter::dates::{excel_serial_to_ymd, EPOCH_1904_IN_1900_SERIALS},
    functions::lift::num_error,
    model::Model,
};

/// Whether `end_date` is at most a year after `start_date` (1900 serials), the TBILL* limit,
/// `None` off the calendar. It compares the dates Excel shows, so serial 0 is 1900-01-00 and
/// serial 1 1900-01-01: `TBILLPRICE(0,366,0.1)` and `TBILLPRICE(1,367,0.1)` are numbers and
/// `TBILLPRICE(0,367,0.1)` is `#NUM!`. A 29 February settlement, the fictitious 1900-02-29
/// (serial 60) too, reaches a year on at 1 March: `TBILLPRICE(DATE(2024,2,29),DATE(2025,3,1),0.1)`
/// and `TBILLPRICE(60,426,0.1)` are numbers (probes/tbill-settlement-below-one B3-B18;
/// docs/excel-semantics/financial.md "T-bill settlement below serial 1"). A settlement in 9999,
/// whose anniversary would be in 10000, is `None`, `#NUM!` however short the bill: in both date
/// systems `TBILLPRICE(DATE(9999,1,1),DATE(9999,1,10),0.05)` is `#NUM!` and 9998-12-31 to
/// 9999-12-31 a number (probes/fin-1900-tbill-last-day, fin-1904-calendar-readers A2, A21:A25;
/// financial.md "T-bills settled in 9999").
pub(super) fn is_less_than_one_year(start_date: i64, end_date: i64) -> Option<bool> {
    let (start_year, start_month, start_day) = excel_serial_to_ymd(start_date)?;
    if start_year >= 9999 {
        return None;
    }
    if end_date - start_date < 365 {
        return Some(true);
    }
    let (end_year, end_month, end_day) = excel_serial_to_ymd(end_date)?;
    if end_year == start_year {
        return Some(true);
    }
    let anniversary = if (start_month, start_day) == (2, 29) {
        (3, 1)
    } else {
        (start_month, start_day)
    };
    // One year later: on or before the settlement's anniversary.
    Some(end_year == start_year + 1 && (end_month, end_day) <= anniversary)
}

/// How a financial function reads a date argument of a 1904 workbook. Excel for the web changed
/// five functions on 2026-10-21 and left the rest (docs/excel-semantics/dates.md "The 1904 date
/// system: ACCRINT, COUPNUM and the TBILL functions read the date itself"); in a 1900 workbook
/// both readings are the serial itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DateReading {
    /// Serial + 2 * 1462 (`DateSystem::financial_1900_serial`): YEARFRAC, the other COUP*
    /// functions, PRICE, YIELD, DURATION, the discount securities, the odd-period functions,
    /// AMORLINC and AMORDEGRC.
    Shifted,
    /// The date itself, serial + 1462 (`DateSystem::calendar_1900_serial`): ACCRINT, COUPNUM,
    /// TBILLEQ, TBILLPRICE and TBILLYIELD.
    Calendar,
}

impl Model {
    /// The floored 1900 serial a financial function's day counts read for the date argument
    /// `serial`: 2 * 1462 days on in a 1904 workbook (`DateSystem::financial_1900_serial`).
    pub(super) fn financial_date(&self, serial: f64) -> i64 {
        self.financial_date_as(DateReading::Shifted, serial)
    }

    /// The floored 1900 serial a financial function that reads dates by `reading` takes for the
    /// date argument `serial`.
    pub(super) fn financial_date_as(&self, reading: DateReading, serial: f64) -> i64 {
        let date_system = self.workbook.settings.date_system;
        let read = match reading {
            DateReading::Shifted => date_system.financial_1900_serial(serial),
            DateReading::Calendar => date_system.calendar_1900_serial(serial),
        };
        read.floor() as i64
    }

    /// Whether Excel's coupon schedule breaks for `maturity`, as `financial_date` read it
    /// (`DateSystem::coupon_schedule_breaks`): only in a 1904 workbook, from 9995-12-31 on.
    pub(super) fn coupon_schedule_breaks(&self, maturity: i64) -> bool {
        self.workbook
            .settings
            .date_system
            .coupon_schedule_breaks(maturity)
    }

    /// The settlement check of PRICE, YIELD and DURATION: #NUM! unless the settlement is a
    /// calendar date before the maturity. Excel is #NUM! for a settlement below serial 1 in a 1900
    /// workbook (0, 0.5, -1: probes/fin-coupon-walk-bounds A10-A13, A20, fin-coupon-walk-bounds-c
    /// M3-N5) and for a negative one in a 1904 workbook, whose 0 reads as the 1900 serial 2924
    /// (probes/date-system-1904-financial A35-A40). The COUP* functions check their own settlement
    /// (`Model::coupon_args`: 0 is a date to them). The check also keeps the coupon schedule
    /// walk back from the maturity inside the calendar: without it
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

    /// A date argument of the TBILL, discount-security, AMORLINC and AMORDEGRC functions as read:
    /// its number, #NUM! unless `reading` floors it to a serial from 0 to 9999-12-31, checked before the
    /// next argument is read (`TBILLEQ(-1,1/0,0.05)` and `TBILLEQ(39000,2958466,1/0)`
    /// are #NUM!, `TBILLEQ(0,39100,1/0)` #DIV/0!; probes/securities-argument-error-order C1:F16;
    /// financial.md, T-1406).
    pub(super) fn financial_calendar_date_of(
        &self,
        value: &CalcResult,
        reading: DateReading,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let serial = self.cast_number_no_bools_ref(value, cell)?;
        self.financial_calendar_serial(serial, reading, cell)
    }

    /// The range check of [`Model::financial_calendar_date_of`] for a serial already read as a
    /// number, for a date argument that takes booleans (ACCRINT's `first_interest`).
    pub(super) fn financial_calendar_serial(
        &self,
        serial: f64,
        reading: DateReading,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let calendar = 0..=i64::from(MAXIMUM_DATE_SERIAL_NUMBER);
        if !calendar.contains(&self.financial_date_as(reading, serial)) {
            return Err(num_error(cell));
        }
        Ok(serial)
    }

    /// A date argument of the COUP* functions as read by `reading` (`DateReading`): #NUM!
    /// unless it floors to a day of this workbook's calendar, checked before the next argument is
    /// read (`COUPDAYS(2958466,NA(),2)` and `COUPNUM(39000,2958466,2,NA())` are #NUM!,
    /// `COUPNCD(2958465.5,NA(),2)` #N/A; probes/coup-calendar-edges A1-A8, T-2486). In a 1904
    /// workbook the bound is its own last day, 9999-12-31 (2957003), though `financial_date` reads
    /// that past it (`COUPPCD(39000,2957004,2)` is #NUM!; probes/coup-calendar-edges-1904).
    pub(super) fn coupon_date_of(
        &self,
        value: &CalcResult,
        reading: DateReading,
        cell: CellReferenceIndex,
    ) -> Result<i64, CalcResult> {
        let serial = self.cast_number_no_bools_ref(value, cell)?;
        let calendar = 0..=i64::from(MAXIMUM_DATE_SERIAL_NUMBER);
        let day = self
            .workbook
            .settings
            .date_system
            .to_1900_serial(serial.floor() as i64);
        if !day.is_some_and(|day| calendar.contains(&day)) {
            return Err(num_error(cell));
        }
        Ok(self.financial_date_as(reading, serial))
    }

    /// A day-count basis argument as read: an error, text or boolean is returned, an omitted basis
    /// is 0, and a number that is not a basis (5, -1) is #NUM! before the next argument is read
    /// (`PRICE(NA(),..,5)` and `DISC(NA(),..,5)` are #NUM!; probes/bond-argument-check-order,
    /// probes/discount-argument-check-order; financial.md "PRICE, YIELD, DURATION and MDURATION
    /// check order", "Discount securities check order").
    pub(super) fn day_count_basis_of(
        &self,
        basis: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<u32, CalcResult> {
        day_count_basis(self.cast_number_no_bools_or(basis, 0.0, cell)?)
            .ok_or_else(|| num_error(cell))
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
    days_30_360_us_ymd(
        (d1.year(), d1.month(), d1.day()),
        (d2.year(), d2.month(), d2.day()),
    )
}

// `days_30_360_us` over `(year, month, day)` triples, so YEARFRAC can count from serial 0
// (1900-01-00), which is not a `NaiveDate`.
pub(crate) fn days_30_360_us_ymd(d1: (i32, u32, u32), d2: (i32, u32, u32)) -> f64 {
    let (y1, month1, day1_raw) = d1;
    let (y2, month2, day2_raw) = d2;
    let m1 = month1 as i32;
    let m2 = month2 as i32;
    let mut day1 = day1_raw as i32;
    let mut day2 = day2_raw as i32;

    let start_is_feb_last = month1 == 2 && day1_raw == last_day_of_feb(y1);
    let end_is_feb_last = month2 == 2 && day2_raw == last_day_of_feb(y2);

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
    days_30_360_us_both_ymd(
        (d1.year(), d1.month(), d1.day()),
        (d2.year(), d2.month(), d2.day()),
    )
}

// `days_30_360_us_both` over `(year, month, day)` triples, so the coupon day counts can read
// serial 0 (1900-01-00) and Excel's fictitious 1900-02-29.
pub(super) fn days_30_360_us_both_ymd(d1: (i32, u32, u32), d2: (i32, u32, u32)) -> f64 {
    let (y1, month1, day1_raw) = d1;
    let (y2, month2, day2_raw) = d2;
    let m1 = month1 as i32;
    let m2 = month2 as i32;
    let mut day1 = day1_raw as i32;
    let mut day2 = day2_raw as i32;

    let start_is_feb_last = month1 == 2 && day1_raw == last_day_of_feb(y1);
    let end_is_feb_last = month2 == 2 && day2_raw == last_day_of_feb(y2);

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
    days_30_360_eu_ymd(
        (d1.year(), d1.month(), d1.day()),
        (d2.year(), d2.month(), d2.day()),
    )
}

// `days_30_360_eu` over `(year, month, day)` triples (see `days_30_360_us_both_ymd`).
pub(super) fn days_30_360_eu_ymd(d1: (i32, u32, u32), d2: (i32, u32, u32)) -> f64 {
    let (y1, m1, day1) = d1;
    let (y2, m2, day2) = d2;
    let day1 = day1.min(30) as i32;
    let day2 = day2.min(30) as i32;
    (360 * (y2 - y1) + 30 * (m2 as i32 - m1 as i32) + (day2 - day1)) as f64
}

pub(super) fn actual_days(d1: NaiveDate, d2: NaiveDate) -> f64 {
    (d2 - d1).num_days() as f64
}

/// The day-count basis a number names: 0 to 4 after truncation, `None` (Excel's #NUM!) for any
/// other number. Every negative basis is #NUM!, -1E-300 and -0.5 too (not truncated toward 0),
/// while 4.9 is 4 (probes/fin-bond-negative-basis; financial.md "Day-count basis: a negative basis
/// is #NUM!").
pub(crate) fn day_count_basis(basis: f64) -> Option<u32> {
    // `as u32` truncates in range; out of range (NaN too) the saturated value is discarded.
    (0.0..5.0).contains(&basis).then_some(basis as u32)
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
