//! YIELD's Newton as Excel runs it ([`yield_start`], [`excel_yield_newton`]), and ODDFYIELD's,
//! the same Newton with its own slope ([`odd_first_yield`]).

use super::{discount_growth, yield_start_years, OddArgs, OddFirstSchedule};

/// The yield YIELD's Newton starts from with more than one coupon left, `years` being the
/// settlement-to-maturity term in years:
/// `(100 rate + (100 - pr)/T) / ((100 + pr)/2 - (100 - pr)/(4T))`. Fitted (T-1592) on the starts
/// two analytic-Newton steps back from Excel's values on probes/yield-start-one-input: the
/// denominator is `pr + w (100 - pr)` with `(1/2 - w) T` = 0.25000 on every row whose values sit
/// below PRICE's root (rows 22-96 but 67: prices 90.5 to 99.5, rates 0 to 0.12, terms 2.3 to
/// 30.3 years), and it moves with neither the redemption nor the frequency (financial.md "YIELD
/// stops short of PRICE's root").
pub(super) fn yield_start(rate: f64, pr: f64, years: f64) -> f64 {
    (100.0 * rate + (100.0 - pr) / years) / ((100.0 + pr) / 2.0 - (100.0 - pr) / (4.0 * years))
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
/// term ([`yield_start_years`]), with the slope of [`OddFirstSchedule::dprice`]. `None` (#NUM!) off
/// the calendar or where the Newton leaves ODDFPRICE's domain.
pub(super) fn odd_first_yield(odd: &OddArgs<4>, schedule: &OddFirstSchedule) -> Option<f64> {
    let [settlement, maturity, ..] = odd.dates;
    let years = yield_start_years(settlement, maturity, odd.basis)?;
    excel_yield_newton(
        odd.yld_or_pr,
        yield_start(odd.rate, odd.yld_or_pr, years),
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
    use super::{discount_growth, OddFirstSchedule};

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
