//! YIELD's Newton as Excel runs it ([`yield_start`], [`excel_yield_newton`]), and ODDFYIELD's,
//! the same Newton with its own slope ([`odd_first_yield`]).

use super::super::dates::{days_30_360_eu_ymd, days_30_360_us_ymd};
use super::{discount_growth, OddArgs, OddFirstSchedule};
use crate::formatter::dates::{date_to_serial_number, excel_serial_to_ymd};
use crate::functions::date_and_time::is_leap_year;
use crate::x87_exp::{dd_add, dd_div, dd_mul, two_sum};

/// The settlement-to-maturity term of [`yield_start`]: `whole_years + days / year_days` years,
/// kept as its parts so the start can take it exactly ([`yield_start_term`]).
pub(super) struct StartTerm {
    pub(super) whole_years: f64,
    pub(super) days: f64,
    pub(super) year_days: f64,
}

/// The term [`yield_start`] takes, as its parts ([`StartTerm`]), from the `settlement` and
/// `maturity` serials: the whole years from settlement to maturity, then the basis' days from the
/// last anniversary of the settlement to maturity over 360 (bases 0, 2 and 4), 365 (basis 3) or the
/// length of the maturity's calendar year (basis 1). A 29 February settlement has its anniversaries
/// on 1 March in common years; the 30/360 count's end-of-February rule looks at the anniversary.
/// Fitted on probes/yield-start-term-basis (T-2777, lab arithmetic: 153 of 159 cells within 4e-16
/// relative, 5 of the other 6 at PRICE's root, against 34 for the basis' days over its year):
/// `YIELD(DATE(2010,1,31),DATE(2020,1,30),..,1)` starts from 9 + 364/366 (2020 is a leap year), and
/// `YIELD(DATE(2012,2,29),DATE(2019,8,15),..,0)` from 7 + DAYS360(1 March 2019, maturity)/360, not
/// DAYS360 over the whole term (financial.md "YIELD stops short of PRICE's root"). The dates are
/// the ones Excel shows, so serial 60 is 29 February 1900 (`YIELD(60,36585,0.05,90,100,4)` starts
/// from 100 years; probes/fin-coupon-walk-bounds-c H12). `None` off the calendar.
pub(super) fn yield_start_term(settlement: i64, maturity: i64, basis: u32) -> Option<StartTerm> {
    let (year, month, day) = excel_serial_to_ymd(settlement)?;
    let end = excel_serial_to_ymd(maturity)?;
    let anniversary = |y: i32| {
        if month == 2 && day == 29 && !(y == 1900 || is_leap_year(y)) {
            (y, 3, 1)
        } else {
            (y, month, day)
        }
    };
    let mut whole_years = end.0 - year;
    let mut last = anniversary(end.0);
    if last > end {
        whole_years -= 1;
        last = anniversary(end.0 - 1);
    }
    let days = match basis {
        0 => days_30_360_us_ymd(last, end),
        4 => days_30_360_eu_ymd(last, end),
        _ if whole_years == 0 => (maturity - settlement) as f64,
        // An anniversary after 1900 is a real date; counting from the month's first day keeps a
        // serial-0 settlement's day 0 (1900-01-00).
        _ => {
            let first = date_to_serial_number(1, last.1, last.0).ok()?;
            (maturity - i64::from(first) - i64::from(last.2) + 1) as f64
        }
    };
    let year_days = match basis {
        1 if is_leap_year(end.0) => 366.0,
        1 | 3 => 365.0,
        _ => 360.0,
    };
    Some(StartTerm {
        whole_years: f64::from(whole_years),
        days,
        year_days,
    })
}

/// The yield YIELD's Newton starts from with more than one coupon left, over the term `T`:
/// `(100 rate + (100 - pr)/T) / ((100 + pr)/2 - (100 - pr)/(4T))`. Fitted (T-1592) on the starts
/// two analytic-Newton steps back from Excel's values on probes/yield-start-one-input: the
/// denominator is `pr + w (100 - pr)` with `(1/2 - w) T` = 0.25000 on every row whose values sit
/// below PRICE's root (rows 22-96 but 67: prices 90.5 to 99.5, rates 0 to 0.12, terms 2.3 to
/// 30.3 years), and it moves with neither the redemption nor the frequency (financial.md "YIELD
/// stops short of PRICE's root").
///
/// Excel's start is this value rounded once, with the term exact (T-2974): one Newton step from
/// it shows its last bit on the cells that take one step, and the correctly rounded start makes
/// 503 of the lab's 761 YIELD probe cells bit-exact, against 441 for the formula in doubles
/// (yield-start-rate-term B49 from 11 ulps to 0). So the start is
/// `(400 rate N + 4 (100 - pr) Y) / (2 N (100 + pr) - (100 - pr) Y)`, the formula times `4N/4N`
/// with `T = N / Y` (`N = whole_years Y + days`, an integer), in double-double arithmetic. A zero
/// term, or a start past the double range, keeps the formula in doubles.
pub(super) fn yield_start(rate: f64, pr: f64, term: &StartTerm) -> f64 {
    let years = term.whole_years + term.days / term.year_days;
    let plain =
        (100.0 * rate + (100.0 - pr) / years) / ((100.0 + pr) / 2.0 - (100.0 - pr) / (4.0 * years));
    // Whole years, days and a year's days are integers well below 2^53, so N and the scaled
    // integers below are exact.
    let n = term.whole_years * term.year_days + term.days;
    if n == 0.0 {
        return plain;
    }
    let discount = two_sum(100.0, -pr);
    let numerator = dd_add(
        dd_mul((rate, 0.0), (400.0 * n, 0.0)),
        dd_mul(discount, (4.0 * term.year_days, 0.0)),
    );
    let premium = dd_mul(two_sum(100.0, pr), (2.0 * n, 0.0));
    let shortfall = dd_mul(discount, (term.year_days, 0.0));
    let start = dd_div(numerator, dd_add(premium, (-shortfall.0, -shortfall.1))).0;
    if start.is_finite() {
        start
    } else {
        plain
    }
}

/// Newton on `price(y) - target` with the analytic slope `dprice`, from `start`: each step is
/// `(price(y) - target) / dprice(y)`, and the first step below 1e-10 is not taken, so the value
/// is the iterate before it, short of the root (T-1592: 240 of the lab's 370 YIELD cells
/// bit-exact and 361 within 4 ulp on `^` prices; the cut lies between 9.67e-11, a step
/// yield-start-sweep C18 does not take, and 1.01e-10, one yield-start-one-input B21 takes). `None`
/// when an iterate leaves PRICE's domain (a non-finite step or iterate). After 100 steps the iterate is the
/// value, root or not: with a redemption of 1E+300 Excel returns a yield far below PRICE's root,
/// what the 100th step reaches (`YIELD(37341,45513,1E-9,105,1E+300,1)` = 78.13551092802992,
/// bit-exact; probes/yield-newton-domain C, T-2391).
pub(super) fn excel_yield_newton(
    target: f64,
    start: f64,
    price: impl Fn(f64) -> f64,
    dprice: impl Fn(f64) -> f64,
) -> Option<f64> {
    let mut y = start;
    for _ in 0..100 {
        let step = (price(y) - target) / dprice(y);
        if !step.is_finite() {
            return None;
        }
        if step.abs() < 1e-10 {
            return Some(y);
        }
        y -= step;
    }
    // The 100th step can still overflow the iterate; inside the loop the next step catches that.
    y.is_finite().then_some(y)
}

/// ODDFYIELD: [`excel_yield_newton`] on ODDFPRICE from YIELD's start over the settlement-to-maturity
/// term ([`yield_start_term`]), with the slope of [`OddFirstSchedule::dprice`]. `None` (#NUM!) off
/// the calendar or where the Newton leaves ODDFPRICE's domain.
pub(super) fn odd_first_yield(odd: &OddArgs<4>, schedule: &OddFirstSchedule) -> Option<f64> {
    let [settlement, maturity, ..] = odd.dates;
    let term = yield_start_term(settlement, maturity, odd.basis)?;
    excel_yield_newton(
        odd.yld_or_pr,
        yield_start(odd.rate, odd.yld_or_pr, &term),
        |y| schedule.price(y, discount_growth),
        |y| schedule.dprice(y),
    )
}

impl OddFirstSchedule {
    /// The slope ODDFYIELD's Newton takes at `yld`: d/dy of [`Self::price`], except that the odd
    /// first coupon weighs as a regular one (`coupon` at the lead's exponent, not `lead`), as if the
    /// bond's first period were regular. Excel: fitted on probes/oddfyield-solver-sweep, where this
    /// slope from YIELD's start puts 323 of 324 coupon-bond cells within 1e-13 of Excel and the exact
    /// slope 15 (financial.md "ODDFYIELD stops short of ODDFPRICE's root", T-2007).
    fn dprice(&self, yld: f64) -> f64 {
        let x = yld / self.m + 1.0;
        let mut slope = 0.0;
        // The lead at its own exponent, then the regular coupons, one period apart: index
        // `first_index - 1` is the lead's (`lead_exponent` is `offset + y` there, short or long).
        for index in (self.first_index - 1)..=self.last_index {
            let t = index as f64 + self.offset + self.y;
            slope -= t * self.coupon / discount_growth(x, t + 1.0) / self.m;
        }
        let t = self.front_exponent;
        slope - t * self.redemption / discount_growth(x, t + 1.0) / self.m
    }
}

#[cfg(test)]
mod tests {
    use super::{discount_growth, yield_start, OddFirstSchedule, StartTerm};

    /// The probes/oddfyield-solver-sweep bond (settlement 2020-02-01, maturity 2025-06-30, issue
    /// 2020-01-01, first coupon 2020-06-30, semiannual, basis 0) at the coupon `rate`.
    fn sweep_bond(rate: f64) -> OddFirstSchedule {
        OddFirstSchedule::new(43862, 45838, 43831, 44012, rate, 100.0, 2, 0).unwrap()
    }

    /// `dprice` against a central difference of `price` at 5%, as a relative gap.
    fn slope_gap(schedule: &OddFirstSchedule) -> f64 {
        let (y, h) = (0.05, 1e-6);
        let numeric = (schedule.price(y + h, discount_growth)
            - schedule.price(y - h, discount_growth))
            / (2.0 * h);
        ((schedule.dprice(y) - numeric) / numeric).abs()
    }

    #[test]
    fn the_start_is_rounded_once_over_the_exact_term() {
        // yield-start-rate-term B49's bond: 30 + 119/360 years, rate 0.08, price 90. The exact
        // start (a rational, rounded once) is 0x1.677417c665554p-4; the formula in doubles
        // rounds to the double below it (T-2974).
        let term = StartTerm {
            whole_years: 30.0,
            days: 119.0,
            year_days: 360.0,
        };
        assert_eq!(yield_start(0.08, 90.0, &term), 0.087_757_199_164_419_32);
        let years = 30.0 + 119.0 / 360.0;
        let plain = (100.0 * 0.08 + 10.0 / years) / (95.0 - 10.0 / (4.0 * years));
        assert_eq!(plain, 0.087_757_199_164_419_3);
        // A zero term keeps the formula in doubles (not a finite start).
        let zero = StartTerm {
            whole_years: 0.0,
            days: 0.0,
            year_days: 360.0,
        };
        assert!(yield_start(0.05, 98.0, &zero).is_nan());
    }

    #[test]
    fn zero_coupon_slope_is_the_price_derivative() {
        assert!(slope_gap(&sweep_bond(0.0)) < 1e-8);
    }

    #[test]
    fn odd_first_coupon_weighs_as_a_regular_one_in_the_slope() {
        // The short first coupon is 150/180 of a regular one, so the fitted slope is not the
        // derivative (financial.md "ODDFYIELD stops short of ODDFPRICE's root").
        assert!(slope_gap(&sweep_bond(0.05)) > 1e-6);
    }
}
