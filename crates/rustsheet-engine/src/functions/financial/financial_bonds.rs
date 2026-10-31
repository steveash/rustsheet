use chrono::{Datelike, NaiveDate};
use std::cmp::Ordering;

pub(super) mod coupon_schedule;
mod odd_last;
mod yield_closed;
mod yield_solver;

use super::dates::{
    basis_days, broken_schedule_days, day_count_basis, DateReading, BROKEN_SCHEDULE_PCD,
};
use coupon_schedule::{
    change_month, coupdays_actual, coupon_count, coupon_days_a, coupon_days_e, coupon_days_nc,
    find_pcd_ncd, is_last_day_of_month, last_day_of_month, next_coupon_date, prev_coupon_date,
};
use odd_last::odd_l_func;
use yield_solver::{excel_yield_newton, odd_first_yield, yield_start, yield_start_term};

use super::number_or_error;
use crate::{
    arithmetic::power,
    calc_result::CalcResult,
    constants::MAXIMUM_DATE_SERIAL_NUMBER,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    functions::lift::{div_error, num_error},
    functions::Function,
    model::Model,
};

/// `v^t`, the growth a bond price divides a cash flow `t` periods out by, with Excel's `^`: with
/// libm `powf`, PRICE misses Excel by ulps on every integer-exponent cell of ironcalc-calc
/// `FINANCIAL/PRICE_YIELD.xlsx` (`PRICE_YIELD!H2` 108.22689722310892) and visibly on
/// probes/fin-coupon-walk-bounds-c C10, where `^` gives Excel's bits (T-2207). Where `^` is an
/// error (an overflow, or a negative base under a fractional exponent) the value is libm's
/// infinity or NaN, so those prices come out as before.
fn discount_growth(v: f64, t: f64) -> f64 {
    power(v, t).unwrap_or_else(|_| v.powf(t))
}

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

fn validate_basis(basis: f64, cell: CellReferenceIndex) -> Result<u32, CalcResult> {
    day_count_basis(basis).ok_or_else(|| num_error(cell))
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

    /// Whether `(1 + yld/frequency)^t` overflows at the redemption's term `t`, the longest one:
    /// PRICE, DURATION and MDURATION are then #NUM!, where the overflowing flows would discount to
    /// 0. `PRICE(43831,47484,0.05,5.12154E+15,100,2,0)` (t = 20) is 9.76e-16 and 5.22501E+15
    /// #NUM!, and with t = 19.5 1.27233E+16 is a number and 1.29803E+16 #NUM!
    /// (probes/bond-huge-yield-overflow; financial.md "PRICE, DURATION and MDURATION at a huge
    /// yield or coupon", T-2488). A term of at most 1 cannot overflow, so a single remaining
    /// coupon, which PRICE discounts linearly, is never #NUM! here (`PRICE(..,1E+300,..)` is
    /// 2.05e-298).
    fn discount_overflows(&self, yld: f64, frequency: u32) -> bool {
        let dsc_over_e = (self.e - self.a) / self.e;
        power(1.0 + yld / frequency as f64, dsc_over_e + self.n - 1.0).is_err()
    }

    /// The clean price (per `redemption` par) at the yield `yld`: a sum over every coupon, and no
    /// schedule walk, so YIELD's solver walks the schedule once, not on every step (T-1842).
    fn price(
        &self,
        rate: f64,
        yld: f64,
        redemption: f64,
        frequency: u32,
        growth: fn(f64, f64) -> f64,
    ) -> f64 {
        let CouponSchedule { n, e, a } = *self;
        let dsc_over_e = (e - a) / e;

        let coupon = 100.0 * rate / frequency as f64;
        // The accrued interest is `rate/f · A/E · 100`, not `coupon · A/E`, and each coupon is
        // discounted at `k + DSC/E` counted from k = 0 (not `DSC/E + k - 1` from k = 1): with it
        // all 240 PRICE cells of probes/bond-extreme-coupon-yield are bit-exact in the eval, and
        // the single-coupon branch below uses the same accrued order without regressions
        // (financial.md "PRICE, DURATION and MDURATION sum order at a huge coupon rate and yield",
        // T-2733).
        let accrued = rate / frequency as f64 * (a / e) * 100.0;

        // A single remaining coupon is discounted with simple (linear) interest
        if n == 1.0 {
            return (redemption + coupon) / (1.0 + dsc_over_e * yld / frequency as f64) - accrued;
        }

        let v = 1.0 + yld / frequency as f64;
        let first_term = redemption / growth(v, dsc_over_e + n - 1.0);

        let mut second_term = 0.0;
        for k in 0..(n as u32) {
            second_term += coupon / growth(v, k as f64 + dsc_over_e);
        }

        first_term + second_term - accrued
    }

    /// d/d`yld` of [`CouponSchedule::price`] with more than one coupon left, analytic, with the
    /// same `growth`: YIELD's Newton steps on it (T-1592).
    fn dprice(&self, rate: f64, yld: f64, redemption: f64, frequency: u32) -> f64 {
        let CouponSchedule { n, e, a } = *self;
        let dsc_over_e = (e - a) / e;
        let f = frequency as f64;
        let v = 1.0 + yld / f;
        let mut slope = 0.0;
        for k in 1..=(n as u32) {
            let t = k as f64 - 1.0 + dsc_over_e;
            slope -= t * 100.0 * rate / f / discount_growth(v, t + 1.0) / f;
        }
        let t = n - 1.0 + dsc_over_e;
        slope - t * redemption / discount_growth(v, t + 1.0) / f
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
    let price = schedule.price(rate, yld, redemption, frequency, discount_growth);
    // A coupon so large that the sum overflows is #NUM! too (`PRICE(..,1E+306,0.05,..)`;
    // probes/bond-huge-yield-overflow A22, A23).
    if schedule.discount_overflows(yld, frequency) || !price.is_finite() {
        return Err("#NUM!".to_string());
    }
    Ok(price)
}

// ============================================================
// Yield solver (Newton)
// ============================================================

/// YIELD with more than one coupon left: Excel's Newton ([`excel_yield_newton`]) on PRICE as PRICE
/// computes it, from its closed-form start over the term in years ([`yield_start_term`]). Where
/// that Newton leaves PRICE's domain YIELD is #NUM!, with no fallback, even where PRICE has a root
/// (`YIELD(36665,44491,42,106,100,4,0)`, `YIELD(39448,43831,0.05,1E+5,100,2)`; T-2391).
fn compute_yield(bond: &BondArgs, schedule: &CouponSchedule) -> Result<f64, String> {
    let term = yield_start_term(bond.settlement, bond.maturity, bond.basis)
        .ok_or_else(|| "#NUM!".to_string())?;
    let (rate, redemption, frequency) = (bond.rate, bond.redemption, bond.frequency);
    excel_yield_newton(
        bond.yld_or_pr,
        yield_start(rate, bond.yld_or_pr, &term),
        |y| schedule.price(rate, y, redemption, frequency, discount_growth),
        |y| schedule.dprice(rate, y, redemption, frequency),
    )
    .ok_or_else(|| "#NUM!".to_string())
}

// ============================================================
// Odd-coupon helpers (port of ExcelFinancialFunctions oddbonds.fs)
// ============================================================

// `DaysBetween` (Numerator position), clamped to be non-negative.
pub(super) fn days_between_not_neg(d1: NaiveDate, d2: NaiveDate, basis: u32) -> f64 {
    basis_days(d1, d2, basis).max(0.0)
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

    /// Whether the redemption's discount power overflows, so ODDFPRICE is #NUM! (T-2659).
    fn discount_overflows(&self, yld: f64) -> bool {
        power(yld / self.m + 1.0, self.front_exponent).is_err()
    }

    /// The clean price at the yield `yld`: a sum over the coupons after the first, no walk.
    fn price(&self, yld: f64, growth: fn(f64, f64) -> f64) -> f64 {
        let x = yld / self.m + 1.0;
        let term1 = self.redemption / growth(x, self.front_exponent);
        let term2 = self.lead / growth(x, self.lead_exponent);
        let mut term3 = 0.0;
        for index in self.first_index..=self.last_index {
            term3 += self.coupon / growth(x, index as f64 + self.offset + self.y);
        }
        term1 + term2 + term3 - self.accrued
    }
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
    /// Reads the COUP* arguments in order, each checked (#NUM!) before the next is read: settlement,
    /// date order, frequency, basis (probes/securities-argument-error-order; financial.md, T-1406).
    /// A lifted call is charged before it runs by `ValueWork::CouponWalks` (lift/kinds), which
    /// mirrors these reads, and `bond_args`' and `accrint_of`'s: keep it in step with them.
    fn coupon_args(
        &self,
        values: &[CalcResult],
        reading: DateReading,
        cell: CellReferenceIndex,
    ) -> Result<CouponArgs, CalcResult> {
        let [settlement, maturity, frequency, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // Settlement 0 is a date to the COUP* functions (`COUPPCD(0,43831,2)` is 0); COUPNUM is
        // #NUM! there because its previous coupon date falls before the calendar (`coupon_count`).
        let settlement = self.coupon_date_of(settlement, reading, cell)?;
        let maturity = self.coupon_date_of(maturity, reading, cell)?;
        if settlement >= maturity {
            return Err(num_error(cell));
        }
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        validate_frequency(frequency, cell)?;
        let basis = self.cast_number_no_bools_or(rest.first(), 0.0, cell)?;
        let basis = validate_basis(basis, cell)?;
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
        let reading = DateReading::of(&kind);
        self.lift_call(kind, args, cell, &mut |model, values| match model
            .coupon_args(values, reading, cell)
        {
            Ok(coupon) => body(model, coupon, cell),
            Err(error) => error,
        })
    }

    /// Reads and checks `(settlement, maturity, rate, yld, frequency, basis)` of DURATION and
    /// MDURATION, and of PRICE and YIELD (`yld` is the price) with their redemption at
    /// `values[4]`, in Excel's order (probes/bond-argument-check-order, T-2482; financial.md
    /// "PRICE, YIELD, DURATION and MDURATION check order"). PRICE and YIELD read and check the
    /// basis first, DURATION and MDURATION last. A settlement or maturity off the calendar is
    /// #NUM! as it is read, and DURATION's frequency as it is read; the date order and the
    /// rate, yld, price and redemption checks come after every argument is read, and settlement
    /// 0 passes the read (`PRICE(0,43831,NA(),..)` is #N/A).
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
        // `PRICE(-1,..,NA())` is #N/A and `PRICE(NA(),..,5)` #NUM!; DURATION's basis comes last.
        let early_basis = if with_redemption {
            Some(self.day_count_basis_of(basis, cell)?)
        } else {
            None
        };
        let settlement = self.financial_calendar_date_of(settlement, DateReading::Shifted, cell)?;
        let maturity = self.financial_calendar_date_of(maturity, DateReading::Shifted, cell)?;
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let yld = self.cast_number_no_bools_ref(yld, cell)?;
        let redemption = match redemption {
            Some(redemption) => self.cast_number_no_bools_ref(redemption, cell)?,
            None => 100.0,
        };
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        validate_frequency(frequency, cell)?;
        let basis = match early_basis {
            Some(basis) => basis,
            None => self.day_count_basis_of(basis, cell)?,
        };
        let (settlement, maturity) = (
            self.financial_date(settlement),
            self.financial_date(maturity),
        );
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
        let schedule = CouponSchedule::new(settlement, maturity, frequency, basis)
            .map_err(|_| num_error(cell))?;
        if schedule.discount_overflows(bond.yld_or_pr, frequency) {
            return Err(num_error(cell));
        }
        let CouponSchedule { n, e, a } = schedule;
        let dsc_over_e = (e - a) / e;

        let redemption = bond.redemption;
        let coupon = redemption * bond.rate / frequency as f64;
        let v = 1.0 + bond.yld_or_pr / frequency as f64;

        // Each cash flow is discounted by Excel's `^` and its weighted term is `(t·cf)/v^t`, not
        // `t·(cf/v^t)`: with libm `powf` 6 of 12 DURATION cells miss by an ulp, and the zero-coupon
        // `DURATION(DATE(2020,3,15),DATE(2025,9,30),0,0.06,2,0)` is 5.541666666666668, one ulp above
        // t/2 (probes/fin-mduration-rri-fit A1-A36, fin-mduration; T-1975).
        let mut num = 0.0; // sum of t * PV(CF)
        let mut den = 0.0; // dirty price (sum of PV(CF))
        let mut add_cash_flow = |t: f64, cash_flow: f64| {
            // `v` > 1 and `t` > 0 here, and the longest term does not overflow.
            let growth = discount_growth(v, t);
            num += t * cash_flow / growth;
            den += cash_flow / growth;
        };
        // Coupon terms `k + DSC/E` from k = 0, as PRICE's ([`CouponSchedule::price`], T-2733).
        for k in 0..(n as u32) {
            add_cash_flow(k as f64 + dsc_over_e, coupon);
        }
        add_cash_flow(dsc_over_e + n - 1.0, redemption);

        if den == 0.0 {
            return Err(div_error(cell));
        }
        // A coupon so large that the sums overflow is #NUM! (`DURATION(..,1E+306,0.05,2,0)`;
        // probes/bond-huge-yield-overflow B22, B23).
        let duration = num / den / frequency as f64;
        if !duration.is_finite() {
            return Err(num_error(cell));
        }
        Ok(duration)
    }

    // MDURATION(settlement, maturity, coupon, yld, frequency, [basis])
    pub(crate) fn fn_mduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Mduration, args, cell, &mut |model, values| {
            number_or_error(model.mduration_of(values, cell))
        })
    }

    /// DURATION, then the Modified Duration adjustment. DURATION reads every argument in order
    /// first, so a settlement error comes before a yld or frequency one
    /// (`MDURATION(NA(),43831,0.08,"x",2)` is #N/A: probes/fin-coupon-walk-bounds-c M8, N8).
    fn mduration_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let duration = self.duration_of(values, cell)?;
        let [_, _, _, yld, frequency, ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let yld = self.cast_number_no_bools_ref(yld, cell)?;
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
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
                        if schedule.n == 1.0 {
                            return yield_closed::yield_one_period(&bond, &schedule)
                                .map_err(|_| num_error(cell));
                        }
                        compute_yield(&bond, &schedule).map_err(|_| num_error(cell))
                    }),
            )
        })
    }

    /// Reads and checks the arguments of ODDFPRICE and ODDFYIELD (`N` = 4 dates: settlement,
    /// maturity, issue, `first_coupon`) or ODDLPRICE and ODDLYIELD (`N` = 3: settlement, maturity,
    /// `last_interest`), then rate, yld or pr (`price_given`), redemption, frequency and [basis],
    /// in Excel's order (probes/odd-bond-argument-check-order, T-2483; financial.md "ODDFPRICE,
    /// ODDFYIELD, ODDLPRICE and ODDLYIELD check order"). A date below 0 or past 9999-12-31 is
    /// #NUM! as it is read, and so is the frequency, before the basis is read; then a date a 1904
    /// workbook's reading moves off the calendar, the basis, `check_dates` (the dates' order) and
    /// the values are #NUM! in that order.
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
            let serial = self.cast_number_no_bools_ref(value, cell)?;
            // A date off the calendar is `#NUM!` as it is read, ahead of an error in a later
            // argument (probes/fin-odd-date-before-error-arg A1:A7, A11, A14:A16; a date of 0 is
            // fine, A17; odd-bond-argument-check-order: -0.5 and 2958466 too, 2958465 is fine).
            let calendar = 0.0..f64::from(MAXIMUM_DATE_SERIAL_NUMBER) + 1.0;
            if !calendar.contains(&serial) {
                return Err(num_error(cell));
            }
            *date = date_system.odd_period_1900_serial(serial);
        }
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let yld_or_pr = self.cast_number_no_bools_ref(yld_or_pr, cell)?;
        let redemption = self.cast_number_no_bools_ref(redemption, cell)?;
        let frequency = self.cast_number_no_bools_ref(frequency, cell)?.floor() as u32;
        // `ODDFPRICE(..,100,3,NA())` is #NUM!: the frequency is checked before the basis is read.
        validate_frequency(frequency, cell)?;
        let basis = self.cast_number_no_bools_or(rest.first(), 0.0, cell)?;
        // A date Excel's reading moves off the calendar (`DateSystem::odd_period_1900_serial`).
        let mut dates = [0; N];
        for (date, read) in dates.iter_mut().zip(read) {
            *date = read.ok_or_else(|| num_error(cell))?;
        }
        let basis = validate_basis(basis, cell)?;
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

    /// The checked arguments of ODDLPRICE (`price_given` false) or ODDLYIELD. A settlement on or
    /// before the last coupon is #NUM!, after every argument is read
    /// (probes/odd-bond-argument-check-order A909:A925).
    fn odd_last_of(
        &self,
        values: &[CalcResult],
        price_given: bool,
        cell: CellReferenceIndex,
    ) -> Result<OddArgs<3>, CalcResult> {
        self.odd_args::<3>(values, price_given, cell, |dates, _| {
            let [settlement, maturity, last_interest] = *dates;
            if settlement <= last_interest || last_interest >= maturity || settlement >= maturity {
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
                    .and_then(|(odd, schedule)| {
                        (!schedule.discount_overflows(odd.yld_or_pr))
                            .then(|| schedule.price(odd.yld_or_pr, discount_growth))
                            .ok_or_else(|| num_error(cell))
                    }),
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
                        odd_first_yield(&odd, &schedule).ok_or_else(|| num_error(cell))
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
            date_number(coupon_days_nc(settlement, maturity, frequency, basis), cell)
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
            if model.coupon_before_calendar(settlement, maturity, frequency) {
                return num_error(cell);
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
