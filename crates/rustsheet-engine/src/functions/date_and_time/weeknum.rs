use chrono::Datelike;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::{from_excel_date, DATE_OUT_OF_RANGE_MESSAGE},
    model::Model,
};

impl Model {
    /// `WEEKNUM(serial, [return_type])`; 1900 weeks follow Excel's Sunday serial 1 (docs/excel-semantics/dates.md).
    pub(crate) fn fn_weeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_array_args(args, &[0, 1], cell, |_| 1, Self::weeknum_scalar)
    }

    fn weeknum_scalar(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let out_of_range =
            || CalcResult::new_error(Error::NUM, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string());
        // The weeks are numbered on the 1900 calendar: a 1904 serial is shifted to it first, so
        // `WEEKNUM(0)` is 1 there (1904-01-01; probes/date-system-1904-readers A5).
        let Some(serial) = self.workbook.settings.date_system.to_1900_serial(serial) else {
            return out_of_range();
        };
        // Serial 0 is "Saturday 0 January 1900": no calendar date, but WEEKNUM numbers it.
        let date = match from_excel_date(serial) {
            Ok(d) => Some(d),
            Err(_) if serial == 0 => None,
            Err(_) => return out_of_range(),
        };
        // A boolean return_type is #VALUE! (`WEEKNUM(100,TRUE)`); text that reads as a number
        // is accepted (`WEEKNUM(100,"2")` = 16; probes/weeknum-serial-0 C35-C38, T-1023).
        let return_type = if args.len() == 2 {
            match self.get_number_no_bools(&args[1], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        if return_type == 21 {
            return match iso_week(serial) {
                Ok(week) => CalcResult::Number(week),
                Err(error) => {
                    CalcResult::new_error(error, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string())
                }
            };
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
            // 0, 3-10 and 18-20 are #NUM! (probes/weeknum-serial-0 C3, C4, C18, C19, C23-C26).
            // Excel gives 22+ and negatives the start weekday of the last valid WEEKNUM evaluated
            // earlier in the recalc, #NUM! only when none ran (probes/weeknum-stale-return-type,
            // docs/excel-semantics/dates.md). That value depends on Excel's calc order; we give
            // #NUM!, Excel's no-state value, until TODO(T-1484) reproduces the state.
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

/// The ISO week of `serial` (WEEKNUM type 21, ISOWEEKNUM). Serial 0 ("Saturday 0 January 1900")
/// shares serial 1's ISO week: `WEEKNUM(0,21)` = `ISOWEEKNUM(0)` = 52 (probes/weeknum-serial-0
/// C1, C2, C17, C21; T-1023); a negative serial is #NUM!.
pub(super) fn iso_week(serial: i64) -> Result<f64, Error> {
    let serial = if serial == 0 { 1 } else { serial };
    let date = from_excel_date(serial).map_err(|_| Error::NUM)?;
    Ok(f64::from(date.iso_week().week()))
}
