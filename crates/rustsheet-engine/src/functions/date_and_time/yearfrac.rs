//! YEARFRAC: the year fraction between two dates under a day-count basis, read the way the
//! financial functions read dates (`DateSystem::financial_1900_serial`).

use chrono::Datelike;

use super::{days360_eu, days360_serial, is_feb29_between_consecutive_years, is_leap_year};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::{excel_serial_to_ymd, from_excel_date, DATE_OUT_OF_RANGE_MESSAGE},
    model::Model,
};

impl Model {
    pub(crate) fn fn_yearfrac(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_array_args(args, &[0, 1, 2], cell, |_| 1, Self::yearfrac_scalar)
    }

    pub(crate) fn yearfrac_scalar(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        // The dates as the 1900 serials the financial day counts read: 2 * 1462 days on in a 1904
        // workbook (`DateSystem::financial_1900_serial`).
        let date_system = self.workbook.settings.date_system;
        let start_raw = match self.get_number_no_bools(&args[0], cell) {
            Ok(c) => date_system.financial_1900_serial(c),
            Err(s) => return s,
        };
        let end_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(c) => date_system.financial_1900_serial(c),
            Err(s) => return s,
        };
        let basis = if args.len() == 3 {
            match self.get_number_no_bools(&args[2], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            }
        } else {
            0
        };
        let (start_date, end_date) = match (
            from_excel_date(start_raw.floor() as i64),
            from_excel_date(end_raw.floor() as i64),
        ) {
            (Ok(start), Ok(end)) => (start, end),
            (Err(message), _) | (_, Err(message)) => {
                return CalcResult::new_error(Error::NUM, cell, message)
            }
        };
        let days = (end_date - start_date).num_days() as f64;
        let result = match basis {
            0 => {
                // YEARFRAC basis 0 uses the financial US 30/360 convention
                // (`dateDiff360Us`, ModifyStartDate), which differs from the
                // DAYS360 worksheet function for February month-ends: a
                // February-end *end* date only collapses to 30 when the *start*
                // is also a February month-end, and the `end == 31` rule tests
                // the original (un-normalised) start day.
                let last_day_feb = |year: i32| if is_leap_year(year) { 29 } else { 28 };
                let sd_is_feb_last =
                    start_date.month() == 2 && start_date.day() == last_day_feb(start_date.year());
                let ed_is_feb_last =
                    end_date.month() == 2 && end_date.day() == last_day_feb(end_date.year());
                let mut sd_day = start_date.day() as i32;
                let mut ed_day = end_date.day() as i32;

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
                let d360 = (end_date.year() - start_date.year()) * 360
                    + (end_date.month() as i32 - start_date.month() as i32) * 30
                    + (ed_day - sd_day);
                d360 as f64 / 360.0
            }
            1 => {
                // Actual/Actual. Port of ExcelFinancialFunctions
                // `ActualActual.DaysInYear`: yearfrac = actual_days / DaysInYear.
                let (y1, m1, d1) = (start_date.year(), start_date.month(), start_date.day());
                let (y2, m2, d2) = (end_date.year(), end_date.month(), end_date.day());

                // lessOrEqualToAYearApart
                let within_a_year =
                    y1 == y2 || (y2 == y1 + 1 && (m1 > m2 || (m1 == m2 && d1 >= d2)));

                let days_in_year = if !within_a_year {
                    // Dates more than a year apart: average the calendar-year
                    // lengths spanned, inclusive of both end years.
                    let tot_days: i64 = (y1..=y2)
                        .map(|y| if is_leap_year(y) { 366 } else { 365 })
                        .sum();
                    tot_days as f64 / (y2 - y1 + 1) as f64
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
                _ => {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        DATE_OUT_OF_RANGE_MESSAGE.to_string(),
                    )
                }
            },
            _ => return CalcResult::new_error(Error::NUM, cell, "Invalid basis".to_string()),
        };
        CalcResult::Number(result.abs())
    }
}
