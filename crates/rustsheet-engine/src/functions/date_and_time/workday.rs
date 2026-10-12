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
        let start_serial = match self.get_number_no_bools(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let mut date = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let mut days = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i32,
            Err(s) => return s,
        };
        let weekend = DEFAULT_WEEKEND;
        let holiday_set = match self.get_holiday_set(args.get(2), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };
        while days != 0 {
            if days > 0 {
                if date.num_days_from_ce() - EXCEL_DATE_BASE >= MAXIMUM_DATE_SERIAL_NUMBER {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: DATE_OUT_OF_RANGE_MESSAGE.to_string(),
                    };
                }
                date += chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), &weekend) && !holiday_set.contains(&date) {
                    days -= 1;
                }
            } else {
                if date.num_days_from_ce() - EXCEL_DATE_BASE <= MINIMUM_DATE_SERIAL_NUMBER {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: DATE_OUT_OF_RANGE_MESSAGE.to_string(),
                    };
                }
                date -= chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), &weekend) && !holiday_set.contains(&date) {
                    days += 1;
                }
            }
        }
        // Unlike EDATE, a WORKDAY result before the 1904 epoch is #NUM! (`WORKDAY(0,-1)`,
        // probes/date-system-1904-readers A43).
        let serial = i64::from(date.num_days_from_ce() - EXCEL_DATE_BASE);
        match self
            .workbook
            .settings
            .date_system
            .serial_from_1900_serial(serial)
        {
            Some(serial) => CalcResult::Number(serial as f64),
            None => CalcResult::new_error(Error::NUM, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string()),
        }
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
        let start_serial = match self.get_number_no_bools(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let days = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i32,
            Err(s) => return s,
        };
        let weekend = match self.weekend_arg(args.get(2), cell) {
            Ok(w) => w,
            Err(e) => return e,
        };
        let holiday_set = match self.get_holiday_set(args.get(3), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let date_system = self.workbook.settings.date_system;

        weekend.lift(|weekend_mask| {
            // Checks if all days are weekends
            if weekend_mask == [true; 7] {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid weekend mask".to_string(),
                ));
            }
            let out_of_range =
                || CalcResult::new_error(Error::NUM, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string());
            let (mut date, mut days) = (date, days);
            while days != 0 {
                if days > 0 {
                    if date.num_days_from_ce() - EXCEL_DATE_BASE >= MAXIMUM_DATE_SERIAL_NUMBER {
                        return Err(out_of_range());
                    }
                    date += chrono::Duration::days(1);
                    if !Self::is_weekend(date.weekday(), &weekend_mask)
                        && !holiday_set.contains(&date)
                    {
                        days -= 1;
                    }
                } else {
                    if date.num_days_from_ce() - EXCEL_DATE_BASE <= MINIMUM_DATE_SERIAL_NUMBER {
                        return Err(out_of_range());
                    }
                    date -= chrono::Duration::days(1);
                    if !Self::is_weekend(date.weekday(), &weekend_mask)
                        && !holiday_set.contains(&date)
                    {
                        days += 1;
                    }
                }
            }
            // Before the 1904 epoch is #NUM!, as in WORKDAY (`WORKDAY.INTL(1,-2)`, A44).
            let serial = i64::from(date.num_days_from_ce() - EXCEL_DATE_BASE);
            date_system
                .serial_from_1900_serial(serial)
                .map(|serial| serial as f64)
                .ok_or_else(out_of_range)
        })
    }
}
