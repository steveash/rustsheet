use chrono::NaiveDate;

use super::dates::{day_count_basis, days_30_360_us_both_ymd, DateReading};
use super::financial_bonds::coupon_schedule::{
    change_month, coupon_date, coupon_serial, is_last_day_of_month, serial_basis_days, serial_ymd,
};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::{from_excel_date, real_date_to_serial},
    functions::lift::{num_error, value_error},
    functions::Function,
    model::Model,
};

/// The serial of an ACCRINT coupon date, `#NUM!` before the workbook's first coupon date `first`
/// (`DateSystem::first_coupon_1900_serial`). Excel reads the schedule in its 1900 calendar, where
/// a coupon date before 1900-01-01 is serial 0, and ACCRINT is `#NUM!` for it: an issue whose
/// previous coupon date is 1899-12-31 has no accrual (`ACCRINT(DATE(1900,3,1),DATE(1900,6,30),
/// DATE(1950,6,30),0.05,1000,4,0,TRUE)`, probes/fin-coupon-walk-bounds-c O1-O13,
/// fin-accrint-1900-calendar; financial.md "ACCRINT at the calendar's start"). In a 1904 workbook
/// the cut is 1904-01-01 (`ACCRINT(10,DATE(1904,6,30),DATE(1904,3,1),0.06,1000,2,0)`;
/// probes/fin-coupnum-1904-maturity-sweep C10).
fn accrint_coupon_serial(date: NaiveDate, first: i64) -> Result<i64, String> {
    let serial = coupon_serial(date);
    if serial < first {
        return Err("Coupon date before the calendar".to_string());
    }
    Ok(serial)
}

/// The days of a stub period from `start` to `end` in the denominator position of `DaysBetween`:
/// the US 30/360 count that modifies both dates on basis 0, the US count on basis 2, the
/// European one on basis 4 and the serial difference on basis 1 (basis 3 is `365/frequency`).
fn stub_denominator(start: i64, end: i64, basis: u32) -> Result<f64, String> {
    match basis {
        0 => Ok(days_30_360_us_both_ymd(
            serial_ymd(start)?,
            serial_ymd(end)?,
        )),
        // Actual/360 counts its numerator in actual days but its denominator in 30/360 days.
        2 => serial_basis_days(start, end, 0),
        _ => serial_basis_days(start, end, basis),
    }
}

// ACCRINT
//
// Accrued interest = par * rate / frequency * Σ contributionᵢ, summed over the
// quasi-coupon periods spanned by the accrual interval. The schedule is anchored
// on `first_interest` and stepped by 12/frequency months. The settlement (last)
// period contributes a prorated slice; each earlier full period contributes
// `calc_method` (1 for the default issue→settlement, 0 for
// first_interest→settlement); the issue-stub period contributes its own prorated
// slice. A coupon date the accrual reads before `first_coupon` is `#NUM!` (`accrint_coupon_serial`).
//
// The dates are Excel 1900 serials and the day counts read them as Excel does: the actual bases
// count serials (Excel's fictitious 1900-02-29 included) and the 30/360 bases the `(year, month,
// day)` Excel shows, so `ACCRINT(DATE(1900,2,15),DATE(1900,5,10),DATE(1900,4,1),0.05,1000,4,0)`
// is 6.388888888888888 (46 days; probes/fin-coupon-walk-bounds-c O14, fin-accrint-1900-calendar).
#[allow(clippy::too_many_arguments)]
fn compute_accrint(
    issue: i64,
    first_interest: i64,
    settlement: i64,
    rate: f64,
    par: f64,
    frequency: u32,
    basis: u32,
    calc_method: bool,
    first_coupon: i64,
) -> Result<f64, String> {
    let num_months = (12 / frequency) as i32;
    let freq = f64::from(frequency);
    let first_interest_date = coupon_date(first_interest)?;
    let end_month_bond = is_last_day_of_month(first_interest_date);
    let calc = if calc_method { 1.0 } else { 0.0 };

    // Regular coupon period [reg_pcd, first_interest]; its length is the
    // denominator used for the settlement slice.
    let reg_pcd_date = change_month(first_interest_date, -num_months, end_month_bond)?;
    // Excel: basis 1 reads that period's length, so its start must be in the calendar too, even
    // when the settlement's period is later: `ACCRINT(33853,1.005,37554,1,9888,1,1)` is #NUM! on
    // basis 1 at every frequency and a number on the other bases, and a first_interest of 367
    // (regular period from 1900-01-01) is a number on basis 1 (probes/accrint-calendar-start-1900
    // A1:A60; gen-m9 932b33b1ef8b7608 Calc!A118; financial.md "ACCRINT at the calendar's start").
    let reg_pcd = if basis == 1 {
        accrint_coupon_serial(reg_pcd_date, first_coupon)?
    } else {
        coupon_serial(reg_pcd_date)
    };
    // Coupon date immediately before settlement. When settlement is past
    // first_interest (only meaningful for the default calc_method) walk the
    // schedule forward to find it; otherwise it is the regular coupon start.
    let pcd_date = if settlement > first_interest && calc_method {
        let mut pcd = first_interest_date;
        let mut ncd = first_interest_date;
        while coupon_serial(ncd) < settlement {
            pcd = ncd;
            ncd = change_month(ncd, num_months, end_month_bond)?;
        }
        // Excel: the coupon date that ends the settlement's period must be a date; one past
        // 9999-12-31 is #NUM! in either date system (`ACCRINT(2956276,2956456,2958100,0.05,1000,
        // 2,0,TRUE)`, next coupon 10000-01-01; probes/date-system-1900-accrint-settlement-bound
        // A23:A28, A39:A49; docs/excel-semantics/financial.md).
        if real_date_to_serial(ncd).is_none() {
            return Err("Next coupon date is past 9999-12-31".to_string());
        }
        pcd
    } else {
        reg_pcd_date
    };
    // Only the coupon dates the accrual reads must be in the calendar: a regular period that
    // starts before 1900 is fine when the settlement's period is later (`ACCRINT(33448,1,37714,
    // 1E+300,0.3,1,2,TRUE)` is a number; gen-m9 3b7990bd4e3307ba Calc!A317).
    let mut pcd = accrint_coupon_serial(pcd_date, first_coupon)?;
    // Excel: first_interest 60 (the fictitious 1900-02-29) stays serial 60 as the settlement's
    // coupon date, although the schedule steps from 1900-02-28, so an issue of 59 still has a
    // period to walk back through and the walk reaches 1899: `ACCRINT(59,60,242,0.05,1000,2,0,
    // TRUE)` is #NUM! while first_interest 59 is a number (probes/accrint-eom-1900 A9, A13).
    if pcd_date == first_interest_date {
        pcd = pcd.max(first_interest);
    }

    let first_date = issue.max(pcd);
    let days0 = serial_basis_days(first_date, settlement, basis)?;
    let coup_days0 = match basis {
        0 | 2 | 4 => 360.0 / freq,
        3 => 365.0 / freq,
        _ => (first_interest - reg_pcd) as f64,
    };
    let mut acc = days0 / coup_days0;

    // Walk coupon periods backward from `pcd` toward `issue`.
    let mut front = pcd_date;
    let mut period_end = pcd;
    while period_end > issue {
        front = change_month(front, -num_months, end_month_bond)?;
        let period_start = accrint_coupon_serial(front, first_coupon)?;
        if issue <= period_start {
            // Full coupon period inside the accrual span.
            acc += calc;
        } else {
            // The issue falls inside this period (the issue stub).
            let numerator = serial_basis_days(issue, period_end, basis)?;
            let denominator = if basis == 3 {
                365.0 / freq
            } else {
                stub_denominator(period_start, period_end, basis)?
            };
            acc += numerator / denominator;
        }
        period_end = period_start;
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

    /// ACCRINT's kernel. Excel reads `calc_method` first: its error is the result and text, "TRUE"
    /// and "FALSE" too, is `#VALUE!` (probes/t2492-vdb-ddb-arguments A47:A54), whatever the other
    /// arguments hold. The other seven are then read in order: a date off the calendar is `#NUM!`
    /// as soon as it is read, before a later argument's error, text or boolean, and a boolean
    /// issue, rate or par is `#VALUE!` (`ACCRINT(TRUE,SQRT(-1),..)`: probes/t2455-transpose-
    /// single-value A52, A53, A56; gen-m9/932b33b1ef8b7608 A233 for par; financial.md "ACCRINT
    /// check order: `calc_method` first, dates as read"). The other numbers take booleans (as
    /// `get_number` did; unprobed).
    fn accrint_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [issue, first_interest, settlement, rate, par, frequency, rest @ ..] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let read = || -> Result<_, CalcResult> {
            let calc_method = rest.get(1).map_or(Ok(true), |value| match value {
                CalcResult::String(_) => Err(value_error(cell)),
                value => Self::cast_bool_ref(value, cell),
            })?;
            // Serial 0 still reads; below it, or past the calendar's end, is `#NUM!` at once. The
            // settlement is not checked as it is read (`ACCRINT(34022,36149,0.5,NA(),..)` is `#N/A`,
            // gen-m9/f81bd3b6dc373e5c A60 reads an issue of 0.5 past a text rate).
            let reading = DateReading::of(&Function::Accrint);
            let issue = self.financial_date_as(
                reading,
                self.financial_calendar_date_of(issue, reading, cell)?,
            );
            let first_interest = self.financial_date_as(
                reading,
                self.financial_calendar_serial(
                    self.cast_number_ref(first_interest, cell)?,
                    reading,
                    cell,
                )?,
            );
            let settlement =
                self.financial_date_as(reading, self.cast_number_ref(settlement, cell)?);
            Ok((
                issue,
                first_interest,
                settlement,
                self.cast_number_no_bools_ref(rate, cell)?,
                self.cast_number_no_bools_ref(par, cell)?,
                self.cast_number_ref(frequency, cell)?.floor(),
                day_count_basis(self.cast_number_or(rest.first(), 0.0, cell)?),
                calc_method,
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
            || ![1.0, 2.0, 4.0].contains(&frequency)
            || issue_serial >= settlement_serial
        {
            return num_error(cell);
        }
        let Some(basis) = basis else {
            return num_error(cell);
        };

        // The serials must name dates of the calendar. Issue and first_interest were only
        // range-checked from serial 0 as they were read; in a 1900 workbook a serial 0 still fails
        // here, as it did before the read-time check (no oracle cell reads a first_interest of 0
        // there). An issue on serial 0 is `#NUM!` in a 1904 workbook too, though 1904-01-01 is a
        // date there (`ACCRINT(0,DATE(1904,7,1),DATE(1904,3,1),0.06,1000,2,0)`, while an issue of
        // 1 computes; probes/fin-coupnum-1904-maturity-sweep C2, C3, C9).
        let date_system = self.workbook.settings.date_system;
        if issue_serial <= self.financial_date_as(DateReading::Calendar, 0.0)
            || from_excel_date(issue_serial).is_err()
            || from_excel_date(first_interest_serial).is_err()
            || from_excel_date(settlement_serial).is_err()
        {
            return num_error(cell);
        }
        match compute_accrint(
            issue_serial,
            first_interest_serial,
            settlement_serial,
            rate,
            par,
            frequency as u32,
            basis,
            calc_method,
            date_system.first_coupon_1900_serial(),
        ) {
            Ok(value) => CalcResult::Number(value),
            Err(_) => num_error(cell),
        }
    }
}
