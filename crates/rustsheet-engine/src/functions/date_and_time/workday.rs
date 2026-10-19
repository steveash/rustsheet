//! WORKDAY and WORKDAY.INTL: step a date over working days, skipping weekends and holidays.
//! The dates lift over arrays and refuse a range, the weekend lifts over both, and the holidays
//! are read once per call (`Function::arg_kinds`, SPEC-0011; T-1236).

use std::cmp::Ordering;

use chrono::Datelike;

use super::holidays::{days_per_weekday, weekday_of, working_days, Holidays};
use super::{bare_error, DEFAULT_WEEKEND};
use crate::{
    calc_result::CalcResult,
    constants::{EXCEL_DATE_BASE, MAXIMUM_DATE_SERIAL_NUMBER},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::DATE_OUT_OF_RANGE_MESSAGE,
    functions::{
        lift::{number_or_error, LiftCost},
        Function,
    },
    model::Model,
};

/// The calendar day number of `date`, the one WORKDAY steps over: consecutive days are
/// consecutive numbers, and the 1900 serial limits bound it.
fn day_number(date: chrono::NaiveDate) -> i64 {
    i64::from(date.num_days_from_ce() - EXCEL_DATE_BASE)
}

/// The least offset `k` in `1..=max_offset` with `count(k) >= target` (`target >= 1`), for a
/// `count` that does not decrease as `k` grows and grows by at most one per step; `None` when
/// there is none. With `per_week` working days a week, the offset that `target` working days
/// need without holidays is tried first; holidays double it until it is enough (galloping), then
/// a bisection finds the least one, so a small `target` costs a few counts.
fn nearest_offset(
    count: impl Fn(i64) -> i64,
    target: i64,
    per_week: i64,
    max_offset: i64,
) -> Option<i64> {
    if max_offset < target || per_week <= 0 {
        return None;
    }
    // `count` grows by at most one per step, so no offset below `target` reaches it.
    let mut low = target;
    let mut span = 7 * (target / per_week + 1);
    let mut high = span.min(max_offset);
    while count(high) < target {
        if high == max_offset {
            return None;
        }
        low = high + 1;
        span = span.saturating_mul(2);
        high = span.min(max_offset);
    }
    while low < high {
        let middle = low + (high - low) / 2;
        if count(middle) >= target {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    Some(low)
}

impl Model {
    /// `WORKDAY(start_date, days, [holidays])`.
    pub(crate) fn fn_workday(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.workday_call(Function::Workday, args, 2, cell)
    }

    /// `WORKDAY.INTL(start_date, days, [weekend], [holidays])`.
    pub(crate) fn fn_workday_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.workday_call(Function::WorkdayIntl, args, 3, cell)
    }

    /// WORKDAY or WORKDAY.INTL, whose holidays are argument `holidays_at`. The holidays are read
    /// and prepared once, so the call is charged their cells once (`LiftCost::Prepared`) and an
    /// element costs O(log holidays). An error among them is each element's, after the element's
    /// own argument errors, as the scalar call reads them.
    fn workday_call(
        &mut self,
        kind: Function,
        args: &[Node],
        holidays_at: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=holidays_at + 1).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let holidays = self
            .workday_holidays(args.get(holidays_at), cell)
            .map_err(|error| bare_error(&error, cell));
        let read = holidays.as_ref().map_or(0, |holidays| holidays.read);
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Prepared(read),
            cell,
            &mut |model, values| {
                number_or_error(model.workday_of(values, holidays_at, &holidays, cell))
            },
        )
    }

    /// The holidays of WORKDAY(.INTL), numbered by [`day_number`].
    fn workday_holidays(
        &mut self,
        arg: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<Holidays, CalcResult> {
        let Some(arg) = arg else {
            return Ok(Holidays::none());
        };
        let days = self
            .holiday_serials(arg, cell)?
            .into_iter()
            .map(|serial| self.excel_date(serial, cell).map(day_number))
            .collect::<Result<Vec<i64>, CalcResult>>()?;
        Ok(Holidays::new(days))
    }

    /// One element of WORKDAY(.INTL): the start date, the day count, then the weekend (WORKDAY.INTL
    /// only), then the holidays; an all-weekend mask is `#VALUE!`. A day count of 0 is the start
    /// date, whatever the weekend and the holidays are.
    fn workday_of(
        &self,
        values: &[CalcResult],
        holidays_at: usize,
        holidays: &Result<Holidays, CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [start, days, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let start = self.cast_number_no_bools_ref(start, cell)?.floor() as i64;
        let date = self.excel_date(start, cell)?;
        let days = self.cast_number_no_bools_ref(days, cell)?.floor() as i32;
        if days == 0 {
            // Excel returns the start date without reading the weekend or the holidays, so their
            // errors do not apply: `WORKDAY(33393,0,{"a"})` and `WORKDAY.INTL(33393,0,"x")` are
            // 33393 (probes/workday-serial-zero F31, F45:F58; dates.md "WORKDAY and NETWORKDAYS
            // from serial 0").
            return Ok(start as f64);
        }
        let weekend = match rest.first() {
            Some(weekend) if holidays_at == 3 => Self::weekend_mask(weekend, cell)?,
            _ => DEFAULT_WEEKEND,
        };
        let holidays = holidays.as_ref().map_err(Clone::clone)?;
        if weekend == [true; 7] {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid weekend mask".to_string(),
            ));
        }
        self.step_workdays(date, days, &weekend, holidays, cell)
    }

    /// Steps `date` over `days` working days (forward when positive), skipping the weekend days
    /// of `weekend_mask` and `holidays`, and returns the result as a serial of the workbook's
    /// date system. The result is the nearest day past `date` with `days` working days up to it,
    /// found by [`nearest_offset`] over closed-form counts, so a call costs
    /// O(log(days + holidays) * log holidays) whatever `days` is, and a few counts for a small
    /// one. A result beyond the date range and one before the 1904 epoch are `#NUM!`
    /// (`WORKDAY(0,-1)`, `WORKDAY.INTL(1,-2)`; probes/date-system-1904-readers A43, A44).
    fn step_workdays(
        &self,
        date: chrono::NaiveDate,
        days: i32,
        weekend_mask: &[bool; 7],
        holidays: &Holidays,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let out_of_range =
            || CalcResult::new_error(Error::NUM, cell, DATE_OUT_OF_RANGE_MESSAGE.to_string());
        let start = day_number(date);
        let start_weekday = date.weekday().num_days_from_monday() as usize;
        // The working days of `from..=to` (`from <= to`).
        let working = |from: i64, to: i64| {
            let first_weekday = weekday_of(from, start, start_weekday);
            working_days(&days_per_weekday(from, to, first_weekday), weekend_mask)
                - working_days(&holidays.per_weekday(from, to, first_weekday), weekend_mask)
        };
        let target = i64::from(days.unsigned_abs());
        let per_week = weekend_mask.iter().filter(|&&weekend| !weekend).count() as i64;
        let day = match days.cmp(&0) {
            Ordering::Equal => Some(start),
            // Offset `k` reaches `start + k`, counting the working days of `start + 1..=start + k`.
            Ordering::Greater => nearest_offset(
                |k| working(start + 1, start + k),
                target,
                per_week,
                i64::from(MAXIMUM_DATE_SERIAL_NUMBER) - start,
            )
            .map(|k| start + k),
            // Offset `k` reaches `start - k`, counting the working days of `start - k..=start - 1`.
            Ordering::Less => nearest_offset(
                |k| working(start - k, start - 1),
                target,
                per_week,
                // Serial 0 is the first day (`WORKDAY.INTL(1,-1,"0000000")` is 0;
                // probes/workday-serial-zero F30).
                start,
            )
            .map(|k| start - k),
        };
        let day = day.ok_or_else(out_of_range)?;
        self.workbook
            .settings
            .date_system
            .serial_from_1900_serial(day)
            .map(|serial| serial as f64)
            .ok_or_else(out_of_range)
    }
}
