use chrono::{Datelike, NaiveDate};
use std::cmp::Ordering;

use crate::formatter::dates::real_date_to_serial;

mod coupon_schedule;

use super::dates::{
    basis_days, broken_schedule_days, broken_schedule_error, days_30_360_us, days_30_360_us_both,
    BROKEN_SCHEDULE_PCD,
};
use coupon_schedule::{
    change_month, coupdays_actual, coupon_count, coupon_days_a, coupon_days_e, find_pcd_ncd,
    is_last_day_of_month, last_day_of_month, next_coupon_date, prev_coupon_date,
};

use super::number_or_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    functions::lift::{div_error, num_error},
    functions::Function,
    model::Model,
};

// #NUM! unless ODDFPRICE and ODDFYIELD take this first coupon: it must be on the maturity's
// coupon schedule (probes/date-system-1900-oddf-schedule; T-1457). A first period exactly one
// period long is #NUM! too, in `OddFirstSchedule::new`.
fn check_odd_first_coupon(
    issue: i64,
    first_coupon: i64,
    maturity: i64,
    frequency: u32,
    cell: CellReferenceIndex,
) -> Result<(), CalcResult> {
    // Off-calendar dates are #NUM! anyway (`OddFirstSchedule::new`); checking them first keeps the
    // schedule walk back from the maturity within the calendar for a far-negative serial.
    let (_, next_coupon) = from_excel_date(issue)
        .and_then(|_| from_excel_date(first_coupon))
        .and_then(|_| find_pcd_ncd(first_coupon.saturating_sub(1), maturity, frequency))
        .map_err(|_| num_error(cell))?;
    if next_coupon != first_coupon {
        return Err(num_error(cell));
    }
    Ok(())
}

// ============================================================
// Validation helpers
// ============================================================

fn validate_frequency(frequency: u32, cell: CellReferenceIndex) -> Result<(), CalcResult> {
    if frequency == 1 || frequency == 2 || frequency == 4 {
        Ok(())
    } else {
        Err(num_error(cell))
    }
}

fn validate_basis(basis: u32, cell: CellReferenceIndex) -> Result<(), CalcResult> {
    if basis <= 4 {
        Ok(())
    } else {
        Err(num_error(cell))
    }
}

// ============================================================
// Core bond price computation (standard coupon bond)
// ============================================================

/// The yield-independent part of a standard coupon bond's price: the coupon count and the day
/// counts of the period around settlement, each found by a walk of the coupon schedule.
struct CouponSchedule {
    n: f64,
    e: f64,
    a: f64,
}

impl CouponSchedule {
    fn new(settlement: i64, maturity: i64, frequency: u32, basis: u32) -> Result<Self, String> {
        Ok(CouponSchedule {
            n: coupon_count(settlement, maturity, frequency)? as f64,
            e: coupon_days_e(settlement, maturity, frequency, basis)?,
            a: coupon_days_a(settlement, maturity, frequency, basis)?,
        })
    }

    /// The clean price (per `redemption` par) at the yield `yld`: a sum over every coupon, and no
    /// schedule walk, so YIELD's solver walks the schedule once, not on every step (T-1842).
    fn price(&self, rate: f64, yld: f64, redemption: f64, frequency: u32) -> f64 {
        let CouponSchedule { n, e, a } = *self;
        let dsc_over_e = (e - a) / e;

        let coupon = 100.0 * rate / frequency as f64;
        let accrued = coupon * (a / e);

        // A single remaining coupon is discounted with simple (linear) interest
        if n == 1.0 {
            return (redemption + coupon) / (1.0 + dsc_over_e * yld / frequency as f64) - accrued;
        }

        let v = 1.0 + yld / frequency as f64;
        let first_term = redemption / v.powf(dsc_over_e + n - 1.0);

        let mut second_term = 0.0;
        for k in 1..=(n as u32) {
            let t = dsc_over_e + k as f64 - 1.0;
            second_term += coupon / v.powf(t);
        }

        first_term + second_term - accrued
    }
}

// Returns the clean price (per `redemption` par) for a standard coupon bond.
fn compute_price(
    settlement: i64,
    maturity: i64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
) -> Result<f64, String> {
    let schedule = CouponSchedule::new(settlement, maturity, frequency, basis)?;
    Ok(schedule.price(rate, yld, redemption, frequency))
}

// ============================================================
// Yield solver (Newton-Raphson + bisection)
// ============================================================

/// YIELD's solver: at most 100 Newton steps of three prices and 97 bisection prices (the
/// bracket [-0.9999, 100] is below 1e-12 wide after 47 halvings), each a sum over the coupons of
/// `schedule`, which the caller walks once.
fn compute_yield_from_price(
    target: f64,
    schedule: &CouponSchedule,
    rate: f64,
    redemption: f64,
    frequency: u32,
) -> Result<f64, String> {
    let obj =
        |y: f64| -> Option<f64> { Some(schedule.price(rate, y, redemption, frequency) - target) };

    let h = 1e-5;
    let eps = 1e-8;

    // Newton-Raphson starting near the coupon rate.
    let mut y = rate;
    for _ in 0..100 {
        let fv = match obj(y) {
            Some(v) if v.is_finite() => v,
            _ => break,
        };
        let fp = match (obj(y + h), obj(y - h)) {
            (Some(a), Some(b)) => (a - b) / (2.0 * h),
            _ => break,
        };
        if !fp.is_finite() || fp.abs() < 1e-15 {
            break;
        }
        let yn = y - fv / fp;
        if yn <= -1.0 {
            break;
        }
        if (yn - y).abs() < eps {
            return Ok(yn);
        }
        y = yn;
    }

    // Bisection fallback.
    let x1 = -0.9999_f64;
    let x2 = 100.0_f64;
    let f1 = obj(x1).ok_or("bisection failed")?;
    let f2 = obj(x2).ok_or("bisection failed")?;
    if f1 * f2 > 0.0 {
        return Err("YIELD: failed to converge".to_string());
    }
    let (mut lo, mut hi, fl_sign) = if f1 < 0.0 {
        (x1, x2, -1.0_f64)
    } else {
        (x2, x1, 1.0_f64)
    };
    let _ = fl_sign; // used only implicitly through lo/hi ordering

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = obj(mid).ok_or("bisection failed")?;
        if fm.abs() < eps || (hi - lo).abs() < 1e-12 {
            return Ok(mid);
        }
        let f_lo = obj(lo).ok_or("bisection failed")?;
        if f_lo * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Err("YIELD: failed to converge".to_string())
}

// ============================================================
// Odd-coupon helpers (port of ExcelFinancialFunctions oddbonds.fs)
// ============================================================

// `DaysBetween` (Numerator position), clamped to be non-negative.
fn days_between_not_neg(d1: NaiveDate, d2: NaiveDate, basis: u32) -> f64 {
    basis_days(d1, d2, basis).max(0.0)
}

// As above, but basis 0 uses the 30/360 ModifyBothDates rule.
fn days_between_not_neg_with_hack(d1: NaiveDate, d2: NaiveDate, basis: u32) -> f64 {
    if basis == 0 {
        days_30_360_us_both(d1, d2).max(0.0)
    } else {
        days_between_not_neg(d1, d2, basis)
    }
}

// Port of `coupNumber`: counts whole quasi-coupon periods between
// `settl` and the anchor date `mat`, walking forward by `num_months` from a
// (possibly month-end-adjusted) settlement.
fn coup_number(
    mat: i64,
    settl: i64,
    num_months: i32,
    is_whole_number: bool,
) -> Result<f64, String> {
    let mat_d = from_excel_date(mat)?;
    let settl_d = from_excel_date(settl)?;

    let coupons_temp = if is_whole_number { 0.0 } else { 1.0 };
    let end_of_month_temp = is_last_day_of_month(mat_d);
    let end_of_month = if !end_of_month_temp
        && mat_d.month() != 2
        && mat_d.day() > 28
        && mat_d.day() < last_day_of_month(mat_d.year(), mat_d.month())
    {
        is_last_day_of_month(settl_d)
    } else {
        end_of_month_temp
    };

    let start_date = change_month(settl_d, 0, end_of_month)?;
    let mut coupons = if settl_d < start_date {
        coupons_temp + 1.0
    } else {
        coupons_temp
    };

    let mut front = change_month(start_date, num_months, end_of_month)?;
    while front < mat_d {
        coupons += 1.0;
        front = change_month(front, num_months, end_of_month)?;
    }
    Ok(coupons)
}

// ============================================================
// Odd-first-period price
// ============================================================

/// The yield-independent part of the price of a bond with an odd first coupon period: every
/// schedule walk and day count, so ODDFYIELD's solver walks the schedule once (T-1842). The
/// price is `redemption / x^front_exponent + lead / x^lead_exponent + sum - accrued`, the sum
/// running `coupon / x^(index + offset + y)` over `first_index..=last_index`, with
/// `x = yld / m + 1`: the short and the long first period of oddbonds.fs in one form, with
/// each term's operations in the order the port had them.
struct OddFirstSchedule {
    m: f64,
    redemption: f64,
    front_exponent: f64,
    lead: f64,
    lead_exponent: f64,
    coupon: f64,
    first_index: i64,
    last_index: i64,
    offset: f64,
    y: f64,
    accrued: f64,
}

impl OddFirstSchedule {
    // settlement:   settlement date
    // maturity:     maturity date
    // issue:        issue date (defines start of odd first period)
    // first_coupon: first coupon date
    // rate, redemption, frequency, basis: standard
    #[allow(clippy::too_many_arguments)]
    fn new(
        settlement: i64,
        maturity: i64,
        issue: i64,
        first_coupon: i64,
        rate: f64,
        redemption: f64,
        frequency: u32,
        basis: u32,
    ) -> Result<Self, String> {
        let m = frequency as f64;
        let num_months = (12 / frequency) as i32;

        let settlement_d = from_excel_date(settlement)?;
        let issue_d = from_excel_date(issue)?;
        let first_coupon_d = from_excel_date(first_coupon)?;

        // E: length of the quasi-coupon period that ends on the first coupon.
        let e = coupon_days_e(settlement, first_coupon, frequency, basis)?;
        let dfc = days_between_not_neg(issue_d, first_coupon_d, basis);
        // Excel: a first period neither shorter nor longer than E is #NUM!, counted on the basis: issue
        // 2023-07-01 with first coupon 2023-12-31 (180 days 30/360) on basis 0 but not 1 (183 of 184
        // days) or 4 (179), issue 2023-07-04 (180 actual days) on basis 2, while the quasi-coupon date
        // 2023-06-30 is a value on bases 2 and 3 (probes/date-system-1900-oddf-issue-after-quasi;
        // docs/excel-semantics/financial.md; T-1490). Both are whole day counts (E is 182.5 or 91.25 on
        // basis 3), so the comparison is exact by design.
        if dfc.partial_cmp(&e) == Some(Ordering::Equal) {
            return Err("first period is one regular period".to_string());
        }

        if dfc < e {
            // --- Short odd first coupon period ---
            let n = coupon_count(settlement, maturity, frequency)? as f64;
            let dsc = days_between_not_neg(settlement_d, first_coupon_d, basis);
            let a = days_between_not_neg(issue_d, settlement_d, basis);
            let y = dsc / e;
            Ok(OddFirstSchedule {
                m,
                redemption,
                front_exponent: n - 1.0 + y,
                lead: 100.0 * rate / m * dfc / e,
                lead_exponent: y,
                coupon: 100.0 * rate / m,
                first_index: 2,
                last_index: n as i64,
                // `index - 1` as `index + (-1)`: the same double.
                offset: -1.0,
                y,
                accrued: a / e * (rate / m) * 100.0,
            })
        } else {
            // --- Long odd first coupon period (spans several quasi-coupon periods) ---
            let nc = coupon_count(issue, first_coupon, frequency)? as i64;

            let mut late_coupon = first_coupon_d;
            let mut dcnl = 0.0_f64;
            let mut anl = 0.0_f64;
            for index in (1..=nc).rev() {
                let early_coupon = change_month(late_coupon, -num_months, false)?;
                let nl = if basis == 1 {
                    days_between_not_neg(early_coupon, late_coupon, basis)
                } else {
                    e
                };
                let dci = if index > 1 {
                    nl
                } else {
                    days_between_not_neg(issue_d, late_coupon, basis)
                };
                let start_date = issue_d.max(early_coupon);
                let end_date = settlement_d.min(late_coupon);
                let a = days_between_not_neg(start_date, end_date, basis);
                late_coupon = early_coupon;
                dcnl += dci / nl;
                anl += a / nl;
            }

            let dsc = if basis == 2 || basis == 3 {
                let ncd = from_excel_date(next_coupon_date(settlement, first_coupon, frequency)?)?;
                days_between_not_neg(settlement_d, ncd, basis)
            } else {
                let pcd = from_excel_date(prev_coupon_date(settlement, first_coupon, frequency)?)?;
                e - basis_days(pcd, settlement_d, basis)
            };

            let nq = coup_number(first_coupon, settlement, num_months, true)?;
            let n = coupon_count(first_coupon, maturity, frequency)? as f64;
            let y = dsc / e;
            Ok(OddFirstSchedule {
                m,
                redemption,
                front_exponent: y + nq + n,
                lead: 100.0 * rate / m * dcnl,
                lead_exponent: nq + y,
                coupon: 100.0 * rate / m,
                first_index: 1,
                last_index: n as i64,
                offset: nq,
                y,
                accrued: 100.0 * rate / m * anl,
            })
        }
    }

    /// The clean price at the yield `yld`: a sum over the coupons after the first, no walk.
    fn price(&self, yld: f64) -> f64 {
        let x = yld / self.m + 1.0;
        let term1 = self.redemption / x.powf(self.front_exponent);
        let term2 = self.lead / x.powf(self.lead_exponent);
        let mut term3 = 0.0;
        for index in self.first_index..=self.last_index {
            term3 += self.coupon / x.powf(index as f64 + self.offset + self.y);
        }
        term1 + term2 + term3 - self.accrued
    }
}

// ============================================================
// Odd-last-period price / yield
// ============================================================

// Port of `oddLFunc` from oddbonds.fs. With `is_l_price` it returns the
// ODDLPRICE clean price; otherwise it returns the ODDLYIELD yield in closed form.
// last_interest: last regular interest (coupon) date before the odd last period.
#[allow(clippy::too_many_arguments)]
fn odd_l_func(
    settlement: i64,
    maturity: i64,
    last_interest: i64,
    rate: f64,
    pr_or_yld: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
    is_l_price: bool,
) -> Result<f64, String> {
    let m = frequency as f64;
    let num_months = (12 / frequency) as i32;

    let settlement_d = from_excel_date(settlement)?;
    let maturity_d = from_excel_date(maturity)?;
    let last_interest_d = from_excel_date(last_interest)?;

    let nc = coupon_count(last_interest, maturity, frequency)? as i64;

    let mut early_coupon = last_interest_d;
    let mut dcnl = 0.0_f64;
    let mut anl = 0.0_f64;
    let mut dscnl = 0.0_f64;
    for index in 1..=nc {
        let late_coupon = change_month(early_coupon, num_months, false)?;
        // A quasi-coupon date past 9999-12-31 is #NUM!, even the one after the maturity
        // (`ODDLPRICE(DATE(9999,8,1),DATE(9999,12,31),DATE(9999,5,15),...)`,
        // probes/date-system-1900-odd-accrint-bound D7-D10; T-1457).
        if real_date_to_serial(late_coupon).is_none() {
            return Err("quasi-coupon date past the last date".to_string());
        }
        let nl = days_between_not_neg_with_hack(early_coupon, late_coupon, basis);
        let dci = if index < nc {
            nl
        } else {
            days_between_not_neg_with_hack(early_coupon, maturity_d, basis)
        };
        let a = if late_coupon < settlement_d {
            dci
        } else if early_coupon < settlement_d {
            days_between_not_neg(early_coupon, settlement_d, basis)
        } else {
            0.0
        };
        let start_date = settlement_d.max(early_coupon);
        let end_date = maturity_d.min(late_coupon);
        let dsc = days_between_not_neg(start_date, end_date, basis);
        early_coupon = late_coupon;
        dcnl += dci / nl;
        anl += a / nl;
        dscnl += dsc / nl;
    }

    let x = 100.0 * rate / m;
    let term1 = dcnl * x + redemption;
    if is_l_price {
        let term2 = dscnl * pr_or_yld / m + 1.0;
        let term3 = anl * x;
        Ok(term1 / term2 - term3)
    } else {
        let term2 = anl * x + pr_or_yld;
        let term3 = m / dscnl;
        Ok((term1 - term2) / term2 * term3)
    }
}

// ============================================================
// Generic yield solver (wraps any price function)
// ============================================================

fn solve_yield<F>(target_price: f64, rate: f64, price_fn: F) -> Result<f64, String>
where
    F: Fn(f64) -> Option<f64>,
{
    let obj = |y: f64| price_fn(y).map(|p| p - target_price);

    let h = 1e-5;
    let eps = 1e-8;

    let mut y = rate;
    for _ in 0..100 {
        let fv = match obj(y) {
            Some(v) if v.is_finite() => v,
            _ => break,
        };
        let fp = match (obj(y + h), obj(y - h)) {
            (Some(a), Some(b)) => (a - b) / (2.0 * h),
            _ => break,
        };
        if !fp.is_finite() || fp.abs() < 1e-15 {
            break;
        }
        let yn = y - fv / fp;
        if yn <= -1.0 {
            break;
        }
        if (yn - y).abs() < eps {
            return Ok(yn);
        }
        y = yn;
    }

    // Bisection fallback
    let (x1, x2) = (-0.9999_f64, 100.0_f64);
    let f1 = obj(x1).ok_or("bisection failed")?;
    let f2 = obj(x2).ok_or("bisection failed")?;
    if f1 * f2 > 0.0 {
        return Err("yield solver: failed to converge".to_string());
    }
    let (mut lo, mut hi) = if f1 < 0.0 { (x1, x2) } else { (x2, x1) };

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = obj(mid).ok_or("bisection failed")?;
        if fm.abs() < eps || (hi - lo).abs() < 1e-12 {
            return Ok(mid);
        }
        let f_lo = obj(lo).ok_or("bisection failed")?;
        if f_lo * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Err("yield solver: failed to converge".to_string())
}

// ============================================================
// Shared argument reader for COUP* functions
// (settlement, maturity, frequency, [basis])
// ============================================================

/// The checked `(settlement, maturity, frequency, basis)` of a COUP* function.
type CouponArgs = (i64, i64, u32, u32);

/// Which function's arguments [`Model::bond_args`] reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BondFunction {
    /// DURATION and MDURATION: no redemption (it is 100).
    Duration,
    /// PRICE: the fourth argument is the yield, at least 0.
    Price,
    /// YIELD: the fourth argument is the price, above 0.
    Yield,
}

/// The checked arguments of DURATION, MDURATION, PRICE and YIELD (redemption 100 for the
/// durations).
struct BondArgs {
    settlement: i64,
    maturity: i64,
    rate: f64,
    /// The yield, or YIELD's price.
    yld_or_pr: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
}

/// The checked arguments of an ODD* function: its `N` dates as 1900 serials, then the rest.
struct OddArgs<const N: usize> {
    dates: [i64; N],
    rate: f64,
    /// The yield, or ODDFYIELD's and ODDLYIELD's price.
    yld_or_pr: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
}

/// `odd_l_func` over the checked arguments of ODDLPRICE (`is_l_price`) or ODDLYIELD.
fn odd_l_function(odd: &OddArgs<3>, is_l_price: bool) -> Result<f64, String> {
    let [settlement, maturity, last_interest] = odd.dates;
    odd_l_func(
        settlement,
        maturity,
        last_interest,
        odd.rate,
        odd.yld_or_pr,
        odd.redemption,
        odd.frequency,
        odd.basis,
        is_l_price,
    )
}

/// A COUP* kernel over its checked arguments.
type CouponBody = fn(&Model, CouponArgs, CellReferenceIndex) -> CalcResult;

// ============================================================
// Model implementations
// ============================================================

impl Model {
    /// Reads and checks the COUP* arguments: every one is read, in order, before any value is
    /// checked. A settlement before the calendar is #NUM! (`Model::check_settlement`).
    fn coupon_args(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<CouponArgs, CalcResult> {
        let [settlement, maturity, frequency, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let settlement = self.financial_date_of(settlement, cell)?;
        let maturity = self.financial_date_of(maturity, cell)?;
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        let basis = self
            .cast_number_no_bools_or(rest.first(), 0.0, cell)?
            .floor() as u32;
        validate_frequency(frequency, cell)?;
        validate_basis(basis, cell)?;
        self.check_settlement(settlement, maturity, cell)?;
        Ok((settlement, maturity, frequency, basis))
    }

    /// Runs a COUP* function through the lifting driver: `body` over the checked arguments of
    /// each element.
    fn coupon_call(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        body: CouponBody,
    ) -> CalcResult {
        self.lift_call(
            kind,
            args,
            cell,
            &mut |model, values| match model.coupon_args(values, cell) {
                Ok(coupon) => body(model, coupon, cell),
                Err(error) => error,
            },
        )
    }

    /// Reads and checks `(settlement, maturity, rate, yld, frequency, basis)` of DURATION and
    /// MDURATION, and of PRICE and YIELD (`yld` is the price) with their redemption at
    /// `values[4]`: every argument is read, in order, before any value is checked.
    fn bond_args(
        &self,
        values: &[CalcResult],
        function: BondFunction,
        cell: CellReferenceIndex,
    ) -> Result<BondArgs, CalcResult> {
        let with_redemption = function != BondFunction::Duration;
        let (settlement, maturity, rate, yld, redemption, frequency, basis) =
            match (with_redemption, values) {
                (true, [s, m, r, y, red, f, rest @ ..]) => (s, m, r, y, Some(red), f, rest.first()),
                (false, [s, m, r, y, f, rest @ ..]) => (s, m, r, y, None, f, rest.first()),
                _ => return Err(CalcResult::new_args_number_error(cell)),
            };
        let settlement = self.financial_date_of(settlement, cell)?;
        let maturity = self.financial_date_of(maturity, cell)?;
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let yld = self.cast_number_no_bools_ref(yld, cell)?;
        let redemption = match redemption {
            Some(redemption) => self.cast_number_no_bools_ref(redemption, cell)?,
            None => 100.0,
        };
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        let basis = self.cast_number_no_bools_or(basis, 0.0, cell)?.floor() as u32;
        validate_frequency(frequency, cell)?;
        validate_basis(basis, cell)?;
        // YIELD's price must be positive, a yield may be 0.
        let bad_fourth = match function {
            BondFunction::Yield => yld <= 0.0,
            BondFunction::Duration | BondFunction::Price => yld < 0.0,
        };
        if rate < 0.0 || bad_fourth || redemption <= 0.0 {
            return Err(num_error(cell));
        }
        self.check_bond_dates(settlement, maturity, cell)?;
        Ok(BondArgs {
            settlement,
            maturity,
            rate,
            yld_or_pr: yld,
            redemption,
            frequency,
            basis,
        })
    }

    // DURATION(settlement, maturity, coupon, yld, frequency, [basis])
    pub(crate) fn fn_duration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Duration, args, cell, &mut |model, values| {
            number_or_error(model.duration_of(values, cell))
        })
    }

    fn duration_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let bond = self.bond_args(values, BondFunction::Duration, cell)?;
        let (settlement, maturity, frequency, basis) =
            (bond.settlement, bond.maturity, bond.frequency, bond.basis);
        let date_error = |_| num_error(cell);
        let n = f64::from(coupon_count(settlement, maturity, frequency).map_err(date_error)?);
        let e = coupon_days_e(settlement, maturity, frequency, basis).map_err(date_error)?;
        let a = coupon_days_a(settlement, maturity, frequency, basis).map_err(date_error)?;
        let dsc_over_e = (e - a) / e;

        let redemption = bond.redemption;
        let coupon = redemption * bond.rate / frequency as f64;
        let v = 1.0 + bond.yld_or_pr / frequency as f64;

        let mut num = 0.0; // sum of t * PV(CF)
        let mut den = 0.0; // dirty price (sum of PV(CF))

        for k in 1..=(n as u32) {
            let t = dsc_over_e + k as f64 - 1.0;
            let pv = coupon / v.powf(t);
            num += t * pv;
            den += pv;
        }
        let t_final = dsc_over_e + n - 1.0;
        let pv_red = redemption / v.powf(t_final);
        num += t_final * pv_red;
        den += pv_red;

        if den == 0.0 {
            return Err(div_error(cell));
        }
        Ok(num / den / frequency as f64)
    }

    // MDURATION(settlement, maturity, coupon, yld, frequency, [basis])
    pub(crate) fn fn_mduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Mduration, args, cell, &mut |model, values| {
            number_or_error(model.mduration_of(values, cell))
        })
    }

    /// DURATION, then the Modified Duration adjustment. It reads yld and frequency before
    /// DURATION reads every argument, so their errors come first, as they did before the lift.
    fn mduration_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [_, _, _, yld, frequency, ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let yld = self.cast_number_no_bools_ref(yld, cell)?;
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        let duration = self.duration_of(values, cell)?;
        Ok(duration / (1.0 + yld / frequency as f64))
    }

    // PRICE(settlement, maturity, rate, yld, redemption, frequency, [basis])
    pub(crate) fn fn_price(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Price, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .bond_args(values, BondFunction::Price, cell)
                    .and_then(|bond| {
                        compute_price(
                            bond.settlement,
                            bond.maturity,
                            bond.rate,
                            bond.yld_or_pr,
                            bond.redemption,
                            bond.frequency,
                            bond.basis,
                        )
                        .map_err(|_| num_error(cell))
                    }),
            )
        })
    }

    // YIELD(settlement, maturity, rate, pr, redemption, frequency, [basis])
    pub(crate) fn fn_yield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Yield, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .bond_args(values, BondFunction::Yield, cell)
                    .and_then(|bond| {
                        let schedule = CouponSchedule::new(
                            bond.settlement,
                            bond.maturity,
                            bond.frequency,
                            bond.basis,
                        )
                        .map_err(|_| num_error(cell))?;
                        compute_yield_from_price(
                            bond.yld_or_pr,
                            &schedule,
                            bond.rate,
                            bond.redemption,
                            bond.frequency,
                        )
                        .map_err(|_| num_error(cell))
                    }),
            )
        })
    }

    /// Reads and checks the arguments of ODDFPRICE and ODDFYIELD (`N` = 4 dates: settlement,
    /// maturity, issue, `first_coupon`) or ODDLPRICE and ODDLYIELD (`N` = 3: settlement, maturity,
    /// `last_interest`), then rate, yld or pr (`price_given`), redemption, frequency and [basis].
    /// Every argument is read, in order, before any value is checked; then a date off the
    /// calendar, the frequency, the basis, `check_dates` (the dates' order) and the values are
    /// #NUM! in that order.
    fn odd_args<const N: usize>(
        &self,
        values: &[CalcResult],
        price_given: bool,
        cell: CellReferenceIndex,
        check_dates: impl Fn(&[i64; N], u32) -> Result<(), CalcResult>,
    ) -> Result<OddArgs<N>, CalcResult> {
        let (Some(date_values), Some(rest)) = (values.get(..N), values.get(N..)) else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let [rate, yld_or_pr, redemption, frequency, rest @ ..] = rest else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let date_system = self.workbook.settings.date_system;
        let mut read = [None; N];
        for (date, value) in read.iter_mut().zip(date_values) {
            *date = date_system.odd_period_1900_serial(self.cast_number_no_bools_ref(value, cell)?);
        }
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let yld_or_pr = self.cast_number_no_bools_ref(yld_or_pr, cell)?;
        let redemption = self.cast_number_no_bools_ref(redemption, cell)?;
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        let basis = self
            .cast_number_no_bools_or(rest.first(), 0.0, cell)?
            .floor() as u32;
        // A date Excel's reading moves off the calendar (`DateSystem::odd_period_1900_serial`).
        let mut dates = [0; N];
        for (date, read) in dates.iter_mut().zip(read) {
            *date = read.ok_or_else(|| num_error(cell))?;
        }
        validate_frequency(frequency, cell)?;
        validate_basis(basis, cell)?;
        check_dates(&dates, frequency)?;
        let bad_fourth = if price_given {
            yld_or_pr <= 0.0
        } else {
            yld_or_pr < 0.0
        };
        if rate < 0.0 || bad_fourth || redemption <= 0.0 {
            return Err(num_error(cell));
        }
        Ok(OddArgs {
            dates,
            rate,
            yld_or_pr,
            redemption,
            frequency,
            basis,
        })
    }

    /// The checked arguments of ODDFPRICE (`price_given` false) or ODDFYIELD, and the schedule of
    /// their odd first period.
    fn odd_first_of(
        &self,
        values: &[CalcResult],
        price_given: bool,
        cell: CellReferenceIndex,
    ) -> Result<(OddArgs<4>, OddFirstSchedule), CalcResult> {
        let odd = self.odd_args::<4>(values, price_given, cell, |dates, frequency| {
            let [settlement, maturity, issue, first_coupon] = *dates;
            if issue >= first_coupon || first_coupon >= maturity || settlement >= maturity {
                return Err(num_error(cell));
            }
            check_odd_first_coupon(issue, first_coupon, maturity, frequency, cell)
        })?;
        let [settlement, maturity, issue, first_coupon] = odd.dates;
        let schedule = OddFirstSchedule::new(
            settlement,
            maturity,
            issue,
            first_coupon,
            odd.rate,
            odd.redemption,
            odd.frequency,
            odd.basis,
        )
        .map_err(|_| num_error(cell))?;
        Ok((odd, schedule))
    }

    /// The checked arguments of ODDLPRICE (`price_given` false) or ODDLYIELD.
    fn odd_last_of(
        &self,
        values: &[CalcResult],
        price_given: bool,
        cell: CellReferenceIndex,
    ) -> Result<OddArgs<3>, CalcResult> {
        self.odd_args::<3>(values, price_given, cell, |dates, _| {
            let [settlement, maturity, last_interest] = *dates;
            if last_interest >= maturity || settlement >= maturity {
                return Err(num_error(cell));
            }
            Ok(())
        })
    }

    // ODDFPRICE(settlement, maturity, issue, first_coupon, rate, yld, redemption, frequency, [basis])
    pub(crate) fn fn_oddfprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oddfprice, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .odd_first_of(values, false, cell)
                    .map(|(odd, schedule)| schedule.price(odd.yld_or_pr)),
            )
        })
    }

    // ODDFYIELD(settlement, maturity, issue, first_coupon, rate, pr, redemption, frequency, [basis])
    pub(crate) fn fn_oddfyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oddfyield, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .odd_first_of(values, true, cell)
                    .and_then(|(odd, schedule)| {
                        solve_yield(odd.yld_or_pr, odd.rate, |yld| Some(schedule.price(yld)))
                            .map_err(|_| num_error(cell))
                    }),
            )
        })
    }

    // ODDLPRICE(settlement, maturity, last_interest, rate, yld, redemption, frequency, [basis])
    pub(crate) fn fn_oddlprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oddlprice, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .odd_last_of(values, false, cell)
                    .and_then(|odd| odd_l_function(&odd, true).map_err(|_| num_error(cell))),
            )
        })
    }

    // ODDLYIELD(settlement, maturity, last_interest, rate, pr, redemption, frequency, [basis])
    pub(crate) fn fn_oddlyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oddlyield, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .odd_last_of(values, true, cell)
                    .and_then(|odd| odd_l_function(&odd, false).map_err(|_| num_error(cell))),
            )
        })
    }

    // COUPDAYBS(settlement, maturity, frequency, [basis])
    // Days from the beginning of the coupon period to settlement.
    pub(crate) fn fn_coupdaybs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.coupon_call(Function::Coupdaybs, args, cell, |model, coupon, cell| {
            let (settlement, maturity, frequency, basis) = coupon;
            if model.coupon_schedule_breaks(maturity) {
                return broken_schedule_days(BROKEN_SCHEDULE_PCD, settlement, basis, cell);
            }
            date_number(coupon_days_a(settlement, maturity, frequency, basis), cell)
        })
    }

    // COUPDAYS(settlement, maturity, frequency, [basis])
    // Total days in the coupon period that contains settlement.
    pub(crate) fn fn_coupdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.coupon_call(Function::Coupdays, args, cell, |model, coupon, cell| {
            let (settlement, maturity, frequency, basis) = coupon;
            // Bases 0, 2, 3 and 4 keep their fixed period length (J1-J8).
            if basis == 1 && model.coupon_schedule_breaks(maturity) {
                return broken_schedule_days(BROKEN_SCHEDULE_PCD, maturity, basis, cell);
            }
            let days = if basis == 1 {
                coupdays_actual(settlement, maturity, frequency)
            } else {
                coupon_days_e(settlement, maturity, frequency, basis)
            };
            date_number(days, cell)
        })
    }

    // COUPDAYSNC(settlement, maturity, frequency, [basis])
    // Days from settlement to the next coupon date.
    pub(crate) fn fn_coupdaysnc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.coupon_call(Function::Coupdaysnc, args, cell, |model, coupon, cell| {
            let (settlement, maturity, frequency, basis) = coupon;
            if model.coupon_schedule_breaks(maturity) {
                return broken_schedule_days(settlement, maturity, basis, cell);
            }
            date_number(coupdaysnc(settlement, maturity, frequency, basis), cell)
        })
    }

    // COUPNCD(settlement, maturity, frequency, [basis])
    // Next coupon date after settlement (returned as an Excel serial date).
    pub(crate) fn fn_coupncd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.coupon_call(Function::Coupncd, args, cell, |model, coupon, cell| {
            let (settlement, maturity, frequency, _basis) = coupon;
            if model.coupon_schedule_breaks(maturity) {
                return model.financial_date_result(maturity);
            }
            match next_coupon_date(settlement, maturity, frequency) {
                Ok(serial) => model.financial_date_result(serial),
                Err(_) => num_error(cell),
            }
        })
    }

    // COUPNUM(settlement, maturity, frequency, [basis])
    // Number of coupons payable between settlement and maturity.
    pub(crate) fn fn_coupnum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.coupon_call(Function::Coupnum, args, cell, |model, coupon, cell| {
            let (settlement, maturity, frequency, _basis) = coupon;
            if model.coupon_schedule_breaks(maturity) {
                return broken_schedule_error(cell);
            }
            date_number(
                coupon_count(settlement, maturity, frequency).map(f64::from),
                cell,
            )
        })
    }

    // COUPPCD(settlement, maturity, frequency, [basis])
    // Most recent coupon date at or before settlement (returned as an Excel serial date).
    pub(crate) fn fn_couppcd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.coupon_call(Function::Couppcd, args, cell, |model, coupon, cell| {
            let (settlement, maturity, frequency, _basis) = coupon;
            if model.coupon_schedule_breaks(maturity) {
                return model.financial_date_result(BROKEN_SCHEDULE_PCD);
            }
            match prev_coupon_date(settlement, maturity, frequency) {
                Ok(serial) => model.financial_date_result(serial),
                Err(_) => num_error(cell),
            }
        })
    }
}

/// A day count or coupon count of the coupon schedule as a cell value: `#NUM!` for a date off
/// the calendar.
fn date_number(days: Result<f64, String>, cell: CellReferenceIndex) -> CalcResult {
    match days {
        Ok(days) => CalcResult::Number(days),
        Err(_) => num_error(cell),
    }
}

/// COUPDAYSNC's days from settlement to the next coupon date.
fn coupdaysnc(settlement: i64, maturity: i64, frequency: u32, basis: u32) -> Result<f64, String> {
    let next_date = from_excel_date(next_coupon_date(settlement, maturity, frequency)?)?;
    let set_date = from_excel_date(settlement)?;
    if basis == 0 {
        // ExcelFinancialFunctions enforces coupDaysNC = coupDays - coupDaysBS
        // for the US 30/360 basis: the total 30/360 days in the coupon period
        // (ModifyBothDates) minus the days from its start to settlement
        // (ModifyStartDate). This is *not* simply 30/360(settlement, next).
        let prev_date = from_excel_date(prev_coupon_date(settlement, maturity, frequency)?)?;
        let total = days_30_360_us_both(prev_date, next_date);
        let to_settlement = days_30_360_us(prev_date, set_date);
        return Ok(total - to_settlement);
    }
    Ok(basis_days(set_date, next_date, basis))
}
