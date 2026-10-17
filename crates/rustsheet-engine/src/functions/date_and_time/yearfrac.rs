//! YEARFRAC: the year fraction between two dates under a day-count basis, read the way the
//! financial functions read dates (`DateSystem::financial_1900_serial`).

use chrono::Datelike;

use super::{
    days360_eu, days360_serial, is_feb29_between_consecutive_years, is_leap_year, num_error,
};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::{excel_serial_to_ymd, from_excel_date},
    functions::Function,
    model::Model,
};

/// Leap years in 1..=`year` (Gregorian), for the average year length of basis 1 in O(1).
fn leap_years_through(year: i32) -> i64 {
    let year = i64::from(year);
    year / 4 - year / 100 + year / 400
}

/// YEARFRAC basis 0, the financial US 30/360 convention (`dateDiff360Us`, `ModifyStartDate`), over
/// `(year, month, day)` triples, which differs from the DAYS360 worksheet function for February
/// month-ends: a February-end *end* date only collapses to 30 when the *start* is also a February
/// month-end, and the `end == 31` rule tests the original (un-normalised) start day.
fn us_30_360(start: (i32, u32, u32), end: (i32, u32, u32)) -> f64 {
    let (start_year, start_month, start_day) = start;
    let (end_year, end_month, end_day) = end;
    let last_day_feb = |year: i32| if is_leap_year(year) { 29 } else { 28 };
    let sd_is_feb_last = start_month == 2 && start_day == last_day_feb(start_year);
    let ed_is_feb_last = end_month == 2 && end_day == last_day_feb(end_year);
    let mut sd_day = start_day as i32;
    let mut ed_day = end_day as i32;

    if ed_is_feb_last && sd_is_feb_last {
        ed_day = 30;
    }
    if ed_day == 31 && sd_day >= 30 {
        ed_day = 30;
    }
    if sd_day == 31 {
        sd_day = 30;
    }
    if sd_is_feb_last {
        sd_day = 30;
    }
    let d360 = (end_year - start_year) * 360
        + (end_month as i32 - start_month as i32) * 30
        + (ed_day - sd_day);
    f64::from(d360) / 360.0
}

impl Model {
    /// `YEARFRAC(start_date, end_date, [basis])`. Every argument lifts over arrays and refuses a
    /// multi-cell range (SPEC-0011 `LiftArrays`; probes/scalar-lift-shapes H6,
    /// probes/date-argument-modes).
    pub(crate) fn fn_yearfrac(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Yearfrac, args, cell, &mut |model, values| {
            model.yearfrac_of(values, cell)
        })
    }

    /// YEARFRAC of one element.
    fn yearfrac_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let (start, end, basis) = match values {
            [start, end] => (start, end, None),
            [start, end, basis] => (start, end, Some(basis)),
            _ => return CalcResult::new_args_number_error(cell),
        };
        // Excel reads `basis` first and range-checks it, then the start date, then the end date:
        // `YEARFRAC(1/0,45000,NA())` is #N/A and `YEARFRAC(NA(),45000,5)` is #NUM!
        // (probes/scalar-lift-open-d W1-W7, probes/yearfrac-basis-order; dates.md "Date
        // function argument modes"). Per element, that order also decides between an error and
        // a refused range (-a W6, W7).
        let basis = match basis {
            Some(value) => match self.cast_number_no_bools_ref(value, cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            },
            None => 0,
        };
        if !(0..=4).contains(&basis) {
            return num_error(cell);
        }
        // The dates as the 1900 serials the financial day counts read: 2 * 1462 days on in a 1904
        // workbook (`DateSystem::financial_1900_serial`).
        let date_system = self.workbook.settings.date_system;
        let start_raw = match self.cast_number_no_bools_ref(start, cell) {
            Ok(c) => date_system.financial_1900_serial(c),
            Err(s) => return s,
        };
        let end_raw = match self.cast_number_no_bools_ref(end, cell) {
            Ok(c) => date_system.financial_1900_serial(c),
            Err(s) => return s,
        };
        let (start_serial, end_serial) = (start_raw.floor() as i64, end_raw.floor() as i64);
        // Serial 0 (1900-01-00, so any date in [0, 1)) is a date to the 30/360 day count of basis 0
        // only, as day 0 of January 1900; other bases keep #NUM! until probed (gen-m9/a2e51f48717af2a1
        // Calc!A32; dates.md "YEARFRAC accepts serial 0", T-1409).
        if basis == 0 && (start_serial == 0 || end_serial == 0) {
            let (Some(start), Some(end)) = (
                excel_serial_to_ymd(start_serial),
                excel_serial_to_ymd(end_serial),
            ) else {
                return num_error(cell);
            };
            return CalcResult::Number(us_30_360(start, end).abs());
        }
        let (Ok(start_date), Ok(end_date)) =
            (from_excel_date(start_serial), from_excel_date(end_serial))
        else {
            return num_error(cell);
        };
        let days = (end_date - start_date).num_days() as f64;
        let result = match basis {
            0 => us_30_360(
                (start_date.year(), start_date.month(), start_date.day()),
                (end_date.year(), end_date.month(), end_date.day()),
            ),
            1 => {
                // Actual/Actual. Port of ExcelFinancialFunctions
                // `ActualActual.DaysInYear`: yearfrac = actual_days / DaysInYear.
                // Excel swaps reversed dates for basis 1 (gen-m7/c263b43217a5898b
                // Calc!A38; dates.md "YEARFRAC swaps reversed dates for basis 1"); `days`
                // is made positive by the final `abs`.
                let (start_date, end_date) = if start_date > end_date {
                    (end_date, start_date)
                } else {
                    (start_date, end_date)
                };
                let (y1, m1, d1) = (start_date.year(), start_date.month(), start_date.day());
                let (y2, m2, d2) = (end_date.year(), end_date.month(), end_date.day());

                // lessOrEqualToAYearApart
                let within_a_year =
                    y1 == y2 || (y2 == y1 + 1 && (m1 > m2 || (m1 == m2 && d1 >= d2)));

                let days_in_year = if !within_a_year {
                    // Dates more than a year apart: average the calendar-year
                    // lengths spanned, inclusive of both end years. Counted in
                    // closed form, so the kernel is O(1) (SPEC-0011 `KernelWork`).
                    let years = i64::from(y2 - y1 + 1);
                    let tot_days =
                        365 * years + leap_years_through(y2) - leap_years_through(y1 - 1);
                    tot_days as f64 / years as f64
                } else {
                    // considerAsBisestile
                    let consider = (y1 == y2 && is_leap_year(y1))
                        || (m2 == 2 && d2 == 29)
                        || is_feb29_between_consecutive_years(start_date, end_date);
                    if consider {
                        366.0
                    } else {
                        365.0
                    }
                };
                days / days_in_year
            }
            2 => days / 360.0,
            3 => days / 365.0,
            // DAYS360(start, end, TRUE) over the same (shifted) dates.
            4 => match (
                excel_serial_to_ymd(days360_serial(start_raw)),
                excel_serial_to_ymd(days360_serial(end_raw)),
            ) {
                (Some(start), Some(end)) => days360_eu(start, end) / 360.0,
                _ => return num_error(cell),
            },
            // Range-checked before the dates were read.
            _ => return num_error(cell),
        };
        CalcResult::Number(result.abs())
    }
}
