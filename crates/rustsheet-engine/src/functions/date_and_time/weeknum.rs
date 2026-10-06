use chrono::Datelike;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::DATE_OUT_OF_RANGE_MESSAGE,
    model::Model,
};

impl Model {
    /// `WEEKNUM(serial, [return_type])`; 1900 weeks follow Excel's Sunday serial 1 (docs/excel-semantics/dates.md).
    pub(crate) fn fn_weeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        // Serial 0 is "Saturday 0 January 1900": no calendar date, but WEEKNUM numbers it.
        let date = match self.excel_date(serial, cell) {
            Ok(d) => Some(d),
            Err(_) if serial == 0 => None,
            Err(e) => return e,
        };
        let out_of_range =
            || CalcResult::new_error(Error::NUM, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string());
        let return_type = if args.len() == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        if return_type == 21 {
            let Some(date) = date else {
                return out_of_range();
            };
            let w = date.iso_week().week();
            return CalcResult::Number(w as f64);
        }
        let start_offset = match return_type {
            1 => chrono::Weekday::Sun,
            2 | 11 => chrono::Weekday::Mon,
            12 => chrono::Weekday::Tue,
            13 => chrono::Weekday::Wed,
            14 => chrono::Weekday::Thu,
            15 => chrono::Weekday::Fri,
            16 => chrono::Weekday::Sat,
            17 => chrono::Weekday::Sun,
            x if x <= 0 || x == 3 => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid return_type".to_string())
            }
            _ => return CalcResult::new_error(Error::NUM, cell, "Invalid return_type".to_string()),
        };
        // Excel numbers the weeks of 1900 as if serial 1 were a Sunday (the real 1 Jan 1900 is a
        // Monday): the week-1 anchor is the last serial <= 1 on the start weekday, and serials
        // 0 and 1 may share a week. Verified for serials 0..=366, later years use the real
        // calendar (docs/excel-semantics/dates.md, probes/dates-early-1900-edges,
        // probes/weeknum-1900-later; T-0995).
        if (0..=366).contains(&serial) {
            let start = i64::from(start_offset.num_days_from_sunday());
            let first = 1 - (-start).rem_euclid(7);
            return CalcResult::Number(((serial - first).div_euclid(7) + 1) as f64);
        }
        let Some(date) = date else {
            return out_of_range();
        };
        // 1 January of `date`'s year, then back to the start weekday.
        let Some(mut first) = date.with_ordinal(1) else {
            return out_of_range();
        };
        while first.weekday() != start_offset {
            first -= chrono::Duration::days(1);
        }
        let week = (date - first).num_days() / 7 + 1;
        CalcResult::Number(week as f64)
    }
}
