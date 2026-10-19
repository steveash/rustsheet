//! NETWORKDAYS and NETWORKDAYS.INTL: count the working days between two dates.

use std::collections::HashSet;

use chrono::Datelike;

use super::DEFAULT_WEEKEND;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    fn get_array_of_dates(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<i64>, CalcResult> {
        let mut values = Vec::new();
        self.collect_serial_numbers(arg, cell, |serial| {
            values.push(serial);
            Ok(())
        })?;
        Ok(values)
    }

    /// The start and end serials of NETWORKDAYS / NETWORKDAYS.INTL (booleans are rejected).
    fn networkdays_bounds(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(i64, i64), CalcResult> {
        let start = self.get_number_no_bools(&args[0], cell)?.floor() as i64;
        let end = self.get_number_no_bools(&args[1], cell)?.floor() as i64;
        Ok((start, end))
    }

    /// The holiday serials of NETWORKDAYS / NETWORKDAYS.INTL (none without the argument).
    fn networkdays_holidays(
        &mut self,
        arg: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<HashSet<i64>, CalcResult> {
        match arg {
            Some(arg) => Ok(self.get_array_of_dates(arg, cell)?.into_iter().collect()),
            None => Ok(HashSet::new()),
        }
    }

    pub(crate) fn fn_networkdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let (start_serial, end_serial) = match self.networkdays_bounds(args, cell) {
            Ok(bounds) => bounds,
            Err(e) => return e,
        };
        let holidays = match self.networkdays_holidays(args.get(2), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };

        let (from, to, sign) = Self::networkdays_span(start_serial, end_serial);
        match self.networkdays_per_weekday(from, to, &holidays, cell) {
            Ok(per_weekday) => {
                CalcResult::Number(Self::working_days(&per_weekday, &DEFAULT_WEEKEND) as f64 * sign)
            }
            Err(e) => e,
        }
    }

    /// The span of NETWORKDAYS / NETWORKDAYS.INTL as `(from, to, sign)` with `from <= to`: an end
    /// before the start counts the same days, negated.
    fn networkdays_span(start_serial: i64, end_serial: i64) -> (i64, i64, f64) {
        if start_serial <= end_serial {
            (start_serial, end_serial, 1.0)
        } else {
            (end_serial, start_serial, -1.0)
        }
    }

    /// The days of `from..=to` (`from <= to`) per weekday, Monday first, without the distinct
    /// `holidays` among them. Counted in closed form, whole weeks plus the remainder, so a span of
    /// the whole date range costs O(holidays), not one step per day (T-1808). Both endpoints must
    /// be dates (#NUM! otherwise); the dates of a date system are one run of serials, so every
    /// day between is one too, and its weekday follows the first day's.
    fn networkdays_per_weekday(
        &self,
        from: i64,
        to: i64,
        holidays: &HashSet<i64>,
        cell: CellReferenceIndex,
    ) -> Result<[i64; 7], CalcResult> {
        let first_weekday = i64::from(
            self.excel_date(from, cell)?
                .weekday()
                .num_days_from_monday(),
        );
        self.excel_date(to, cell)?;
        let weekday_of = |serial: i64| ((first_weekday + serial - from) % 7) as usize;
        let days = to - from + 1;
        let mut per_weekday = [days / 7; 7];
        for serial in from..from + days % 7 {
            per_weekday[weekday_of(serial)] += 1;
        }
        for &holiday in holidays {
            if (from..=to).contains(&holiday) {
                per_weekday[weekday_of(holiday)] -= 1;
            }
        }
        Ok(per_weekday)
    }

    /// The days of `per_weekday` (Monday first) that are not weekend days of `weekend_mask`.
    fn working_days(per_weekday: &[i64; 7], weekend_mask: &[bool; 7]) -> i64 {
        per_weekday
            .iter()
            .zip(weekend_mask)
            .filter(|(_, &weekend)| !weekend)
            .map(|(days, _)| days)
            .sum()
    }

    pub(crate) fn fn_networkdays_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let (start_serial, end_serial) = match self.networkdays_bounds(args, cell) {
            Ok(bounds) => bounds,
            Err(e) => return e,
        };

        let weekend = match self.weekend_arg(args.get(2), cell) {
            Ok(w) => w,
            Err(e) => return e,
        };

        let holidays = match self.networkdays_holidays(args.get(3), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };

        let (from, to, sign) = Self::networkdays_span(start_serial, end_serial);
        // The days are counted per weekday once, so each mask of a lifted weekend costs seven
        // additions.
        let per_weekday = match self.networkdays_per_weekday(from, to, &holidays, cell) {
            Ok(per_weekday) => per_weekday,
            Err(e) => return e,
        };
        weekend
            .lift(|weekend_mask| Ok(Self::working_days(&per_weekday, &weekend_mask) as f64 * sign))
    }
}
