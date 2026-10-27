use chrono::Datelike;

use super::num_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    functions::Function,
    model::Model,
};

/// WEEKNUM's stale `return_type` state: the start weekday the last WEEKNUM or ISOWEEKNUM of the
/// recalc kept, read by a `return_type` of 22+ or negative (`None` when empty; SPEC-0017, dates.md
/// "WEEKNUM's stale `return_type` state").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WeeknumStart {
    Sunday,
    Monday,
}

impl Model {
    /// `WEEKNUM(serial, [return_type])`; 1900 weeks follow Excel's Sunday serial 1 (docs/excel-semantics/dates.md).
    /// Both arguments lift over arrays and refuse a multi-cell range (SPEC-0011 `LiftArrays`,
    /// probes/date-argument-modes).
    pub(crate) fn fn_weeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let result = self.lift_call(Function::Weeknum, args, cell, &mut |model, values| {
            model.weeknum_of(values, cell)
        });
        if matches!(result, CalcResult::Array(_)) {
            // A WEEKNUM that lifts over an array empties the state, whatever its types
            // (`SUM(WEEKNUM(100,{2,2}))`; dates.md "WEEKNUM's stale return_type state").
            self.evaluation.order_state.weeknum_start = None;
        }
        result
    }

    /// WEEKNUM of one element. Once both arguments convert and the serial is valid, it updates
    /// the stale `return_type` state, or reads it for a `return_type` of 22+ or negative (SPEC-0017
    /// rules 2, 4 and 5).
    fn weeknum_of(&mut self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let (serial, return_type) = match values {
            [serial] => (serial, None),
            [serial, return_type] => (serial, Some(return_type)),
            _ => return CalcResult::new_args_number_error(cell),
        };
        // A boolean serial is #VALUE!, before the return_type is read (`WEEKNUM(<FALSE cell>,1)`;
        // dates.md "WEEKNUM of a boolean serial is #VALUE!", gen-m6, T-2276); WEEKDAY and
        // ISOWEEKNUM read it as 1. A typed or array-constant boolean is `#VALUE!` too
        // (probes/weeknum-boolean-serial A1:A8, A11).
        let serial = match self.cast_number_no_bools_ref(serial, cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let out_of_range = || num_error(cell);
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
        let return_type = match return_type {
            Some(value) => match self.cast_number_no_bools_ref(value, cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            },
            None => 1,
        };
        // Types 1 and 17 keep a Sunday start, 2, 11 and 21 a Monday one, and every other type
        // below 22 empties the state, the always-#NUM! ones included (probes/weeknum-stale-setters).
        let state = &mut self.evaluation.order_state.weeknum_start;
        match return_type {
            1 | 17 => *state = Some(WeeknumStart::Sunday),
            2 | 11 | 21 => *state = Some(WeeknumStart::Monday),
            0..=20 => *state = None,
            _ => {}
        }
        if return_type == 21 {
            return match iso_week(serial) {
                Ok(week) => CalcResult::Number(week),
                Err(error) => CalcResult::new_error(error, cell, String::new()),
            };
        }
        let start_offset = match return_type {
            1 | 17 => chrono::Weekday::Sun,
            2 | 11 => chrono::Weekday::Mon,
            12 => chrono::Weekday::Tue,
            13 => chrono::Weekday::Wed,
            14 => chrono::Weekday::Thu,
            15 => chrono::Weekday::Fri,
            16 => chrono::Weekday::Sat,
            // 0, 3-10 and 18-20 are #NUM! (probes/weeknum-serial-0 C3, C4, C18, C19, C23-C26).
            0..=20 => return num_error(cell),
            // 22+ and negatives read the state left by the WEEKNUM and ISOWEEKNUM calls Excel ran
            // before (in its fresh-load order, `excel_order.rs`): weeks numbered as type 1 does,
            // from that weekday, and #NUM! when it is empty (docs/excel-semantics/dates.md
            // "WEEKNUM's stale return_type state", probes/weeknum-stale-*, SPEC-0017).
            _ => {
                self.evaluation.order_state_read = true;
                match self.evaluation.order_state.weeknum_start {
                    Some(WeeknumStart::Sunday) => chrono::Weekday::Sun,
                    Some(WeeknumStart::Monday) => chrono::Weekday::Mon,
                    None => return num_error(cell),
                }
            }
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
