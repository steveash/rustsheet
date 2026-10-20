//! NETWORKDAYS and NETWORKDAYS.INTL: count the working days between two dates. The dates lift
//! over arrays and refuse a range, the weekend lifts over both, and the holidays are read once
//! per call (`Function::arg_kinds`, SPEC-0011; T-1236).

use chrono::Datelike;

use super::holidays::{days_per_weekday, working_days, Holidays};
use super::{bare_error, DEFAULT_WEEKEND};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::{
        lift::{number_or_error, LiftCost},
        Function,
    },
    model::Model,
};

impl Model {
    /// `NETWORKDAYS(start_date, end_date, [holidays])`.
    pub(crate) fn fn_networkdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.networkdays_call(Function::Networkdays, args, 2, cell)
    }

    /// `NETWORKDAYS.INTL(start_date, end_date, [weekend], [holidays])`.
    pub(crate) fn fn_networkdays_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.networkdays_call(Function::NetworkdaysIntl, args, 3, cell)
    }

    /// NETWORKDAYS or NETWORKDAYS.INTL, whose holidays are argument `holidays_at`: read and
    /// prepared once, as WORKDAY's are (`Model::workday_call`), numbered by workbook serial.
    fn networkdays_call(
        &mut self,
        kind: Function,
        args: &[Node],
        holidays_at: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=holidays_at + 1).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let holidays = match args.get(holidays_at) {
            Some(arg) => self
                .holiday_serials(arg, cell)
                .map(Holidays::new)
                .map_err(|error| bare_error(&error, cell)),
            None => Ok(Holidays::none()),
        };
        let read = holidays.as_ref().map_or(0, |holidays| holidays.read);
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Prepared(read),
            cell,
            &mut |model, values| {
                number_or_error(model.networkdays_of(values, holidays_at, &holidays, cell))
            },
        )
    }

    /// One element of NETWORKDAYS(.INTL): the weekend (NETWORKDAYS.INTL only), the dates (booleans
    /// are rejected), then the holidays. An end before the start counts the same days,
    /// negated. Both dates must be dates (`#NUM!` otherwise); the dates of a date system are one
    /// run of serials, so every day between is one too, and its weekday follows the first day's.
    /// The days are counted in closed form (T-1808).
    fn networkdays_of(
        &self,
        values: &[CalcResult],
        holidays_at: usize,
        holidays: &Result<Holidays, CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [start, end, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // NETWORKDAYS.INTL reads the weekend before the dates: `NETWORKDAYS.INTL(NA(),45030,99)` is
        // `#NUM!` and `NETWORKDAYS.INTL(1/0,45030,NA())` `#VALUE!` (probes/date-text-weekend-3d
        // Q2, Q32, Q33).
        let weekend = match rest.first() {
            Some(weekend) if holidays_at == 3 => Self::weekend_mask(weekend, Error::VALUE, cell)?,
            _ => DEFAULT_WEEKEND,
        };
        let start = self.cast_number_no_bools_ref(start, cell)?.floor() as i64;
        let end = self.cast_number_no_bools_ref(end, cell)?.floor() as i64;
        let holidays = holidays.as_ref().map_err(Clone::clone)?;
        let (from, to, sign) = if start <= end {
            (start, end, 1.0)
        } else {
            (end, start, -1.0)
        };
        let first_weekday = self
            .excel_date(from, cell)?
            .weekday()
            .num_days_from_monday() as usize;
        self.excel_date(to, cell)?;
        let days = working_days(&days_per_weekday(from, to, first_weekday), &weekend)
            - working_days(&holidays.per_weekday(from, to, first_weekday), &weekend);
        Ok(days as f64 * sign)
    }
}
