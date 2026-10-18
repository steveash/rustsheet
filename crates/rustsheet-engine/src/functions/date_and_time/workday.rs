//! WORKDAY and WORKDAY.INTL: step a date over working days, skipping weekends and holidays.

use chrono::Datelike;

use super::DEFAULT_WEEKEND;
use crate::{
    calc_result::CalcResult,
    constants::{EXCEL_DATE_BASE, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::DATE_OUT_OF_RANGE_MESSAGE,
    model::Model,
};

impl Model {
    fn is_weekend(day: chrono::Weekday, weekend_mask: &[bool; 7]) -> bool {
        match day {
            chrono::Weekday::Mon => weekend_mask[0],
            chrono::Weekday::Tue => weekend_mask[1],
            chrono::Weekday::Wed => weekend_mask[2],
            chrono::Weekday::Thu => weekend_mask[3],
            chrono::Weekday::Fri => weekend_mask[4],
            chrono::Weekday::Sat => weekend_mask[5],
            chrono::Weekday::Sun => weekend_mask[6],
        }
    }

    pub(crate) fn fn_workday(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let (date, days) = match self.workday_start_and_days(args, cell) {
            Ok(start) => start,
            Err(error) => return error,
        };
        let weekend = DEFAULT_WEEKEND;
        let holiday_set = match self.get_holiday_set(args.get(2), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };
        match self.step_workdays(date, days, &weekend, &holiday_set, cell) {
            Ok(serial) => CalcResult::Number(serial),
            Err(error) => error,
        }
    }

    /// The start date and the signed working-day count (the first two arguments) of WORKDAY and
    /// WORKDAY.INTL.
    fn workday_start_and_days(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(chrono::NaiveDate, i32), CalcResult> {
        let start_serial = self.get_number_no_bools(&args[0], cell)?.floor() as i64;
        let date = self.excel_date(start_serial, cell)?;
        let days = self.get_number_no_bools(&args[1], cell)?.floor() as i32;
        Ok((date, days))
    }

    /// Steps `date` over `days` working days (forward when positive), skipping the weekend days
    /// of `weekend_mask` and `holidays`, and returns the result as a serial of the workbook's
    /// date system. Out-of-range steps and results before the 1904 epoch are `#NUM!`
    /// (`WORKDAY(0,-1)`, `WORKDAY.INTL(1,-2)`; probes/date-system-1904-readers A43, A44).
    fn step_workdays(
        &self,
        mut date: chrono::NaiveDate,
        mut days: i32,
        weekend_mask: &[bool; 7],
        holidays: &std::collections::HashSet<chrono::NaiveDate>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let out_of_range =
            || CalcResult::new_error(Error::NUM, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string());
        // Skip whole weeks first so large day counts cost O(holidays), not O(days). The skip stops
        // short of the target and of the range limits, so the loop below still yields the same
        // result and the same range errors.
        let per_week = 7 - weekend_mask.iter().filter(|&&w| w).count() as i64;
        // Unreachable today (callers reject the all-weekend mask with #VALUE!); guards the division.
        if per_week == 0 {
            return Err(out_of_range());
        }
        // Holiday serials that fall on a working day, sorted so each skipped span is counted by
        // binary search (dense holidays make many passes possible).
        let mut workday_holidays: Vec<i64> = holidays
            .iter()
            .filter(|h| !Self::is_weekend(h.weekday(), weekend_mask))
            .map(|h| i64::from(h.num_days_from_ce() - EXCEL_DATE_BASE))
            .collect();
        workday_holidays.sort_unstable();
        loop {
            let weeks = (i64::from(days.unsigned_abs()) - 1).max(0) / per_week;
            if weeks == 0 {
                break;
            }
            let forward = days > 0;
            let from = i64::from(date.num_days_from_ce() - EXCEL_DATE_BASE);
            let to = if forward {
                from + 7 * weeks
            } else {
                from - 7 * weeks
            };
            if to >= i64::from(MAXIMUM_DATE_SERIAL_NUMBER)
                || to <= i64::from(MINIMUM_DATE_SERIAL_NUMBER)
            {
                // The target lies strictly beyond `to`, so the day loop would end in this error.
                return Err(out_of_range());
            }
            let (lo, hi) = if forward {
                (from + 1, to)
            } else {
                (to, from - 1)
            };
            let skipped_holidays = (workday_holidays.partition_point(|&h| h <= hi)
                - workday_holidays.partition_point(|&h| h < lo))
                as i64;
            let consumed = i32::try_from(per_week * weeks - skipped_holidays).unwrap_or(0);
            days += if forward { -consumed } else { consumed };
            date += chrono::Duration::days(to - from);
        }
        while days != 0 {
            if days > 0 {
                if date.num_days_from_ce() - EXCEL_DATE_BASE >= MAXIMUM_DATE_SERIAL_NUMBER {
                    return Err(out_of_range());
                }
                date += chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), weekend_mask) && !holidays.contains(&date) {
                    days -= 1;
                }
            } else {
                if date.num_days_from_ce() - EXCEL_DATE_BASE <= MINIMUM_DATE_SERIAL_NUMBER {
                    return Err(out_of_range());
                }
                date -= chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), weekend_mask) && !holidays.contains(&date) {
                    days += 1;
                }
            }
        }
        let serial = i64::from(date.num_days_from_ce() - EXCEL_DATE_BASE);
        self.workbook
            .settings
            .date_system
            .serial_from_1900_serial(serial)
            .map(|serial| serial as f64)
            .ok_or_else(out_of_range)
    }

    fn get_holiday_set(
        &mut self,
        arg_option: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<std::collections::HashSet<chrono::NaiveDate>, CalcResult> {
        let mut holiday_set = std::collections::HashSet::new();

        if let Some(arg) = arg_option {
            let date_system = self.workbook.settings.date_system;
            self.collect_serial_numbers(arg, cell, |serial| {
                match date_system.date_of_serial(serial) {
                    Some(date) => {
                        holiday_set.insert(date);
                        Ok(())
                    }
                    None => Err(CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Invalid holiday date".to_string(),
                    }),
                }
            })?;
        }

        Ok(holiday_set)
    }

    pub(crate) fn fn_workday_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let (date, days) = match self.workday_start_and_days(args, cell) {
            Ok(start) => start,
            Err(error) => return error,
        };
        let weekend = match self.weekend_arg(args.get(2), cell) {
            Ok(w) => w,
            Err(e) => return e,
        };
        let holiday_set = match self.get_holiday_set(args.get(3), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };
        weekend.lift(|weekend_mask| {
            // Checks if all days are weekends
            if weekend_mask == [true; 7] {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid weekend mask".to_string(),
                ));
            }
            self.step_workdays(date, days, &weekend_mask, &holiday_set, cell)
        })
    }
}
