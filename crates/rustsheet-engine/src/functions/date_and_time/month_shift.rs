//! EDATE and EOMONTH: a date shifted by whole months, and the last day of that month. Both lift
//! over arrays and refuse a multi-cell range (`ArgKind::LiftArrays`, SPEC-0011): the scalar
//! function is the kernel `lift_call` runs per element. Moved out of `date_and_time.rs` (T-1770).

use chrono::Datelike;

use super::num_error;
use crate::{
    calc_result::CalcResult,
    constants::MAXIMUM_DATE_SERIAL_NUMBER,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::{month_arithmetic_date, real_date_to_serial, shift_months},
    functions::Function,
    model::Model,
};

impl Model {
    /// `EDATE(start_date, months)`.
    pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Edate, args, cell, &mut |model, values| {
            model.edate_of(values, cell)
        })
    }

    /// `EOMONTH(start_date, months)`.
    pub(crate) fn fn_eomonth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Eomonth, args, cell, &mut |model, values| {
            model.eomonth_of(values, cell)
        })
    }

    /// EDATE of one element. A boolean date or month count is `#VALUE!` (`{TRUE;45000}` gives
    /// `#VALUE!`, 45031: docs/excel-semantics/arrays.md "Scalar arguments lift").
    fn edate_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [start, months] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let serial_number = match self.cast_number_no_bools_ref(start, cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        // As in EOMONTH: 1900-calendar arithmetic on a 1904 serial shifted to it (A6).
        let date_system = self.workbook.settings.date_system;
        let Some((serial_number, date)) = date_system
            .to_1900_serial(serial_number)
            .and_then(|serial| month_arithmetic_date(serial).map(|date| (serial, date)))
        else {
            return num_error(cell);
        };
        let months = match self.cast_number_no_bools_ref(months, cell) {
            Ok(c) => c.trunc() as i32,
            Err(s) => return s,
        };
        let Some(native_date) = shift_months(date, months) else {
            return num_error(cell);
        };
        // Serial 0 is day 0 of January 1900, so its result is day 0 of the shifted month: one
        // serial before that month's first day (EDATE(0,1) = 31, EDATE(0,12) = 366).
        let day_offset = i32::from(serial_number == 0);
        match real_date_to_serial(native_date) {
            Some(serial) => CalcResult::Number(
                date_system.signed_serial_from_1900_serial(i64::from(serial - day_offset)) as f64,
            ),
            None => num_error(cell),
        }
    }

    /// EOMONTH of one element. A negative date is `#NUM!`; a boolean is `#VALUE!`, as for EDATE.
    fn eomonth_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [start, months] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let serial_number = match self.cast_number_no_bools_ref(start, cell) {
            Ok(c) if c.floor() < 0.0 => return num_error(cell),
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        // The month arithmetic runs on the 1900 calendar; in a 1904 workbook the serial counts
        // from 1904-01-01 (probes/date-system-1904-readers A7).
        let date_system = self.workbook.settings.date_system;
        let Some(serial_number) = date_system.to_1900_serial(serial_number) else {
            return num_error(cell);
        };
        if serial_number > i64::from(MAXIMUM_DATE_SERIAL_NUMBER) {
            return num_error(cell);
        }
        let Some(date) = month_arithmetic_date(serial_number)
            .and_then(|d| chrono::NaiveDate::from_ymd_opt(d.year(), d.month(), 1))
        else {
            return num_error(cell);
        };
        let months = match self.cast_number_no_bools_ref(months, cell) {
            Ok(c) => c.trunc() as i32,
            Err(s) => return s,
        };
        let Some(native_date) = shift_months(date, months) else {
            return num_error(cell);
        };
        // The last day of the month is a real calendar date: EOMONTH(31,1) is 28 February 1900
        // (serial 59), not the fictitious serial 60.
        let (mut month, mut year) = (native_date.month() + 1, native_date.year());
        if month == 13 {
            month = 1;
            year += 1;
        }
        let last_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).and_then(|d| d.pred_opt());
        match last_day.and_then(real_date_to_serial) {
            Some(serial) => CalcResult::Number(
                date_system.signed_serial_from_1900_serial(i64::from(serial)) as f64,
            ),
            None => num_error(cell),
        }
    }
}
