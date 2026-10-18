use chrono::{Datelike, NaiveDate};

use super::dates::broken_schedule_error;
use crate::{
    calc_result::CalcResult,
    constants::MAXIMUM_DATE_SERIAL_NUMBER,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::{
        from_excel_date, from_excel_date_up_to, real_date_to_serial, EPOCH_1904_IN_1900_SERIALS,
    },
    functions::lift::num_error,
    functions::Function,
    model::Model,
};

fn last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // Gregorian leap-year rule
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

// ---------------------------------------------------------------------------
// ACCRINT day-count helpers
//
// Port of the ACCRINT algorithm as implemented in the
// `ExcelFinancialFunctions` library (bonds.fs / daycountbasis.fs)
// See https://github.com/fsprojects/ExcelFinancialFunctions
// ---------------------------------------------------------------------------

fn acc_is_last_day(d: NaiveDate) -> bool {
    d.day() == last_day_of_month(d.year(), d.month())
}

fn acc_is_last_feb(d: NaiveDate) -> bool {
    d.month() == 2 && acc_is_last_day(d)
}

// .NET DateTime.AddMonths semantics: shift the month, clamping the
// day-of-month to the last day of the target month when it overflows
// (e.g. Jan 31 + 1 month = Feb 28).
fn acc_add_months(d: NaiveDate, n: i32) -> Result<NaiveDate, String> {
    let total = d.year() * 12 + (d.month() as i32 - 1) + n;
    let y = total.div_euclid(12);
    let m = (total.rem_euclid(12) + 1) as u32;
    let day = d.day().min(last_day_of_month(y, m));
    NaiveDate::from_ymd_opt(y, m, day).ok_or_else(|| "Invalid date".to_string())
}

// EFF `changeMonth`: AddMonths, then optionally snap to the last day of the
// target month. `return_last_day` is fixed for the whole coupon schedule
// (= whether `first_interest` is an end-of-month date).
fn acc_change_month(d: NaiveDate, n: i32, return_last_day: bool) -> Result<NaiveDate, String> {
    let nd = acc_add_months(d, n)?;
    if return_last_day {
        NaiveDate::from_ymd_opt(
            nd.year(),
            nd.month(),
            last_day_of_month(nd.year(), nd.month()),
        )
        .ok_or_else(|| "Invalid date".to_string())
    } else {
        Ok(nd)
    }
}

fn acc_days(after: NaiveDate, before: NaiveDate) -> f64 {
    (after - before).num_days() as f64
}

fn acc_date_diff_360(sd: i32, sm: i32, sy: i32, ed: i32, em: i32, ey: i32) -> f64 {
    ((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64
}

// US (NASD) 30/360 day count, basis 0 and the denominator convention of the
// Actual/360 basis. `modify_both` selects Excel's `ModifyBothDates` variant.
fn acc_date_diff_360_us(start: NaiveDate, end: NaiveDate, modify_both: bool) -> f64 {
    let (mut sd, sm, sy) = (start.day() as i32, start.month() as i32, start.year());
    let (mut ed, em, ey) = (end.day() as i32, end.month() as i32, end.year());
    if acc_is_last_feb(end) && (acc_is_last_feb(start) || modify_both) {
        ed = 30;
    }
    if ed == 31 && (sd >= 30 || modify_both) {
        ed = 30;
    }
    if sd == 31 {
        sd = 30;
    }
    if acc_is_last_feb(start) {
        sd = 30;
    }
    acc_date_diff_360(sd, sm, sy, ed, em, ey)
}

// European 30/360 day count, basis 4.
fn acc_date_diff_360_eu(start: NaiveDate, end: NaiveDate) -> f64 {
    let sd = if start.day() == 31 {
        30
    } else {
        start.day() as i32
    };
    let ed = if end.day() == 31 {
        30
    } else {
        end.day() as i32
    };
    acc_date_diff_360(
        sd,
        start.month() as i32,
        start.year(),
        ed,
        end.month() as i32,
        end.year(),
    )
}

// Excel's Actual/365 denominator convention (basis 3).
fn acc_date_diff_365(start: NaiveDate, end: NaiveDate) -> Result<f64, String> {
    let sd = if start.day() > 28 && start.month() == 2 {
        28
    } else {
        start.day()
    };
    let ed = if end.day() > 28 && end.month() == 2 {
        28
    } else {
        end.day()
    };
    let startd = NaiveDate::from_ymd_opt(start.year(), start.month(), sd)
        .ok_or_else(|| "Invalid date".to_string())?;
    let endd = NaiveDate::from_ymd_opt(end.year(), end.month(), ed)
        .ok_or_else(|| "Invalid date".to_string())?;
    Ok(((end.year() - start.year()) * 365) as f64 + acc_days(endd, startd))
}

// EFF `findPcdNcd`: step coupon boundaries from `start` by `num_months` until
// `end` is reached, returning (front, trailing) at the stopping point.
fn acc_find_pcd_ncd(
    start: NaiveDate,
    end: NaiveDate,
    num_months: i32,
    return_last: bool,
) -> Result<(NaiveDate, NaiveDate), String> {
    let mut front = start;
    let mut trailing = end;
    loop {
        let stop = if num_months > 0 {
            front >= end
        } else {
            front <= end
        };
        if stop {
            return Ok((front, trailing));
        }
        trailing = front;
        front = acc_change_month(front, num_months, return_last)?;
    }
}

// EFF `DaysBetween` for a given basis. `numerator` selects the numerator vs
// denominator convention used by the Actual/360 and Actual/365 bases.
pub(super) fn acc_days_between(
    start: NaiveDate,
    end: NaiveDate,
    numerator: bool,
    basis: i32,
) -> Result<f64, String> {
    let days = match basis {
        0 => acc_date_diff_360_us(start, end, false),
        4 => acc_date_diff_360_eu(start, end),
        2 => {
            if numerator {
                acc_days(end, start)
            } else {
                acc_date_diff_360_us(start, end, false)
            }
        }
        3 => {
            if numerator {
                acc_days(end, start)
            } else {
                acc_date_diff_365(start, end)?
            }
        }
        _ => acc_days(end, start), // Actual/Actual (basis 1)
    };
    Ok(days)
}

// EFF `CoupDays`: the normal length of a coupon period. For Actual/Actual it
// is the actual length of the period bracketing `settl`, anchored on `mat`.
fn acc_coup_days(settl: NaiveDate, mat: NaiveDate, freq: i32, basis: i32) -> Result<f64, String> {
    let coup_days = match basis {
        0 | 2 | 4 => 360.0 / freq as f64,
        3 => 365.0 / freq as f64,
        _ => {
            let num_months = -(12 / freq);
            let (pcd, ncd) = acc_find_pcd_ncd(mat, settl, num_months, acc_is_last_day(mat))?;
            acc_days(ncd, pcd)
        }
    };
    Ok(coup_days)
}

// ACCRINT
//
// Accrued interest = par * rate / frequency * Σ contributionᵢ, summed over the
// quasi-coupon periods spanned by the accrual interval. The schedule is anchored
// on `first_interest` and stepped by 12/frequency months. The settlement (last)
// period contributes a prorated slice; each earlier full period contributes
// `calc_method` (1 for the default issue→settlement, 0 for
// first_interest→settlement); the issue-stub period contributes its own prorated
// slice. `settlement_days`, when given, replaces the day count of the settlement slice.
#[allow(clippy::too_many_arguments)]
fn compute_accrint(
    issue: NaiveDate,
    first_interest: NaiveDate,
    settlement: NaiveDate,
    rate: f64,
    par: f64,
    frequency: i32,
    basis: i32,
    calc_method: bool,
    settlement_days: Option<f64>,
) -> Result<f64, String> {
    let num_months = 12 / frequency;
    let freq = frequency as f64;
    let end_month_bond = acc_is_last_day(first_interest);
    let calc = if calc_method { 1.0 } else { 0.0 };

    // Regular coupon period [reg_pcd, first_interest]; its length is the
    // denominator used for the settlement slice.
    let reg_pcd = acc_change_month(first_interest, -num_months, end_month_bond)?;
    // Coupon date immediately before settlement. When settlement is past
    // first_interest (only meaningful for the default calc_method) walk the
    // schedule forward to find it; otherwise it is the regular coupon start.
    let pcd = if settlement > first_interest && calc_method {
        let (ncd, pcd) = acc_find_pcd_ncd(first_interest, settlement, num_months, end_month_bond)?;
        // Excel: the coupon date that ends the settlement's period must be a date; one past
        // 9999-12-31 is #NUM! in either date system (`ACCRINT(2956276,2956456,2958100,0.05,1000,
        // 2,0,TRUE)`, next coupon 10000-01-01; probes/date-system-1900-accrint-settlement-bound
        // A23:A28, A39:A49; docs/excel-semantics/financial.md).
        if real_date_to_serial(ncd).is_none() {
            return Err("Next coupon date is past 9999-12-31".to_string());
        }
        pcd
    } else {
        reg_pcd
    };

    let first_date = if issue > pcd { issue } else { pcd };
    let days0 = match settlement_days {
        Some(days) => days,
        None => acc_days_between(first_date, settlement, true, basis)?,
    };
    let coup_days0 = acc_coup_days(reg_pcd, first_interest, frequency, basis)?;
    let mut acc = days0 / coup_days0;

    // Walk coupon periods backward from `pcd` toward `issue`.
    let mut front = pcd;
    while front > issue {
        let period_end = front;
        front = acc_change_month(front, -num_months, end_month_bond)?;
        let period_start = front;
        if issue <= period_start {
            // Full coupon period inside the accrual span.
            acc += calc;
        } else {
            // The issue falls inside this period (the issue stub).
            let numerator = if basis == 0 {
                acc_date_diff_360_us(issue, period_end, false)
            } else {
                acc_days_between(issue, period_end, true, basis)?
            };
            let denominator = if basis == 0 {
                acc_date_diff_360_us(period_start, period_end, true)
            } else if basis == 3 {
                365.0 / freq
            } else {
                acc_days_between(period_start, period_end, false, basis)?
            };
            acc += numerator / denominator;
        }
    }

    Ok(par * rate / freq * acc)
}

impl Model {
    // ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])
    //
    // Canonical specification: Mayle, *Standard Securities Calculation Methods*
    // (SIA / SIFMA), generalized by the Microsoft DAX `ACCRINT` documentation:
    //
    //   AI = par * (rate / frequency) * Σᵢ (Aᵢ / NLᵢ),  i = 1..NC
    //
    // where the sum runs over the NC quasi-coupon periods spanned by the
    // accrual interval. For each period i, `Aᵢ` is the number of accrued
    // days that fall inside the period under the given day-count basis,
    // and `NLᵢ` is the normal length of the period under the same basis.
    //
    // The quasi-coupon period boundaries are derived by walking backward
    // from `first_interest` (= Mayle's `CPNDT_1`, the first coupon date
    // after settlement) at intervals of 12/frequency months, snapped to
    // the end of the month when first_interest is itself an end-of-month
    // date. The walk stops once the period containing the accrual start
    // is reached.
    //
    // `calc_method` selects the accrual start:
    //   - TRUE  (default): accrue from `issue` to `settlement`
    //   - FALSE          : accrue from `first_interest` to `settlement`
    //
    // Boundary case: when `settlement` coincides with a coupon boundary
    // (issue = settlement, or accrual reduces to zero days), return 0.
    // The `issue >= settlement` case still errors per the MS spec.
    pub(crate) fn fn_accrint(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Accrint, args, cell, &mut |model, values| {
            model.accrint_of(values, cell)
        })
    }

    /// ACCRINT's kernel: every argument is read, in order, before any value is checked. Its
    /// dates and numbers take booleans (as `get_number` did), `calc_method` reads as a boolean.
    fn accrint_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [issue, first_interest, settlement, rate, par, frequency, rest @ ..] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let read = || -> Result<_, CalcResult> {
            Ok((
                self.financial_date(self.cast_number_ref(issue, cell)?),
                self.financial_date(self.cast_number_ref(first_interest, cell)?),
                self.financial_date(self.cast_number_ref(settlement, cell)?),
                self.cast_number_ref(rate, cell)?,
                self.cast_number_ref(par, cell)?,
                self.cast_number_ref(frequency, cell)?.floor() as i32,
                self.cast_number_or(rest.first(), 0.0, cell)?.floor() as i32,
                rest.get(1)
                    .map_or(Ok(true), |value| Self::cast_bool_ref(value, cell))?,
            ))
        };
        let (
            issue_serial,
            first_interest_serial,
            settlement_serial,
            rate,
            par,
            frequency,
            basis,
            calc_method,
        ) = match read() {
            Ok(read) => read,
            Err(error) => return error,
        };
        if rate <= 0.0
            || par <= 0.0
            || !matches!(frequency, 1 | 2 | 4)
            || !(0..=4).contains(&basis)
            || issue_serial >= settlement_serial
        {
            return num_error(cell);
        }

        // ACCRINT walks the coupon schedule of `first_interest`, which in a 1904 workbook breaks
        // from 9995-12-31 on as a maturity's does: #NUM! for any settlement and calc_method,
        // while 9995-12-30 still accrues (probes/date-system-1904-odd-accrint-bound A11:D15;
        // docs/excel-semantics/dates.md; T-1447).
        if self.coupon_schedule_breaks(first_interest_serial) {
            return broken_schedule_error(cell);
        }
        let issue = match from_excel_date(issue_serial) {
            Ok(d) => d,
            Err(_) => return num_error(cell),
        };
        let first_interest = match from_excel_date(first_interest_serial) {
            Ok(d) => d,
            Err(_) => return num_error(cell),
        };
        let settlement_limit = self
            .workbook
            .settings
            .date_system
            .accrint_settlement_limit(calc_method);
        let settlement = match from_excel_date_up_to(settlement_serial, settlement_limit) {
            Ok(d) => d,
            Err(_) => return num_error(cell),
        };

        // Excel: a 1904 settlement read past 9999-12-31 (calc_method FALSE) has no 30/360 date;
        // bases 0 and 4 then accrue first_interest's 1904 serial plus 1462 days in the settlement
        // slice, whatever the settlement and the issue (410415.83 for first_interest 9990-06-30,
        // 1904 serial 2953532; probes/date-system-1904-accrint-settlement-bound A1:A17,
        // docs/excel-semantics/dates.md). Bases 1 to 3 count its actual days.
        let settlement_days = (settlement_serial > i64::from(MAXIMUM_DATE_SERIAL_NUMBER)
            && matches!(basis, 0 | 4))
        .then(|| (first_interest_serial - EPOCH_1904_IN_1900_SERIALS) as f64);
        match compute_accrint(
            issue,
            first_interest,
            settlement,
            rate,
            par,
            frequency,
            basis,
            calc_method,
            settlement_days,
        ) {
            Ok(value) => CalcResult::Number(value),
            Err(_) => num_error(cell),
        }
    }
}
