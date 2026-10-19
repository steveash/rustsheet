//! The holidays of WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL, read once per call
//! and prepared so that each lifted element counts them in O(log holidays) (T-1236).

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// The serials of a holidays argument, read whole: a range is not intersected
    /// (docs/excel-semantics/dates.md "WORKDAY holidays take the whole range"). Each must be a
    /// date (`#NUM!`), and a value that is not a number is `#VALUE!`.
    pub(super) fn holiday_serials(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<i64>, CalcResult> {
        let mut serials = Vec::new();
        self.collect_serial_numbers(arg, cell, |serial| {
            serials.push(serial);
            Ok(())
        })?;
        Ok(serials)
    }
}

/// Distinct holiday days, grouped by day number mod 7 and sorted. The numbering is the caller's
/// (workbook serials for NETWORKDAYS, calendar day numbers for WORKDAY); it only has to number
/// consecutive days consecutively.
pub(super) struct Holidays {
    by_residue: [Vec<i64>; 7],
    /// The days read, repeats included: what preparing them cost (`LiftCost::Prepared`).
    pub(super) read: u64,
}

impl Holidays {
    /// No holidays: the call has no holidays argument.
    pub(super) fn none() -> Self {
        Self::new(Vec::new())
    }

    pub(super) fn new(mut days: Vec<i64>) -> Self {
        let read = u64::try_from(days.len()).unwrap_or(u64::MAX);
        days.sort_unstable();
        days.dedup();
        let mut by_residue: [Vec<i64>; 7] = Default::default();
        for day in days {
            by_residue[residue(day)].push(day);
        }
        Self { by_residue, read }
    }

    /// The distinct holidays of `from..=to` per weekday, Monday first, where day `from` is
    /// weekday `first_weekday` (0 is Monday).
    pub(super) fn per_weekday(&self, from: i64, to: i64, first_weekday: usize) -> [i64; 7] {
        let mut per_weekday = [0; 7];
        for (day_residue, days) in self.by_residue.iter().enumerate() {
            let count =
                days.partition_point(|&day| day <= to) - days.partition_point(|&day| day < from);
            let weekday = weekday_of(day_residue as i64, from, first_weekday);
            per_weekday[weekday] += count as i64;
        }
        per_weekday
    }
}

/// `day` mod 7, in `0..7` for negative days too.
fn residue(day: i64) -> usize {
    day.rem_euclid(7) as usize
}

/// The weekday (0 is Monday) of `day`, where day `from` is weekday `first_weekday`.
pub(super) fn weekday_of(day: i64, from: i64, first_weekday: usize) -> usize {
    residue(first_weekday as i64 + day - from)
}

/// The days of `from..=to` (`from <= to`) per weekday, Monday first, where day `from` is weekday
/// `first_weekday`: whole weeks plus the remainder, in closed form (T-1808).
pub(super) fn days_per_weekday(from: i64, to: i64, first_weekday: usize) -> [i64; 7] {
    let days = to - from + 1;
    let mut per_weekday = [days / 7; 7];
    for day in from..from + days % 7 {
        per_weekday[weekday_of(day, from, first_weekday)] += 1;
    }
    per_weekday
}

/// The days of `per_weekday` (Monday first) that are not weekend days of `weekend_mask`.
pub(super) fn working_days(per_weekday: &[i64; 7], weekend_mask: &[bool; 7]) -> i64 {
    per_weekday
        .iter()
        .zip(weekend_mask)
        .filter(|(_, &weekend)| !weekend)
        .map(|(days, _)| days)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{days_per_weekday, working_days, Holidays};

    /// The closed-form count equals walking the days one by one.
    #[test]
    fn counts_match_a_day_walk() {
        let holidays = Holidays::new(vec![3, 3, 10, 11, -4, 40]);
        let mask = [false, false, false, false, false, true, true];
        for from in -10_i64..20 {
            for to in from..from + 30 {
                let first_weekday = from.rem_euclid(7) as usize;
                let days = days_per_weekday(from, to, first_weekday);
                let off = holidays.per_weekday(from, to, first_weekday);
                let counted = working_days(&days, &mask) - working_days(&off, &mask);
                let walked = (from..=to)
                    .filter(|day| !mask[day.rem_euclid(7) as usize])
                    .filter(|day| ![3, 10, 11, -4, 40].contains(day))
                    .count() as i64;
                assert_eq!(counted, walked, "{from}..={to}");
            }
        }
    }
}
