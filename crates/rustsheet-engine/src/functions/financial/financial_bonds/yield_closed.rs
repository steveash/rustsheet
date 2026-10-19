use super::coupon_schedule::coupon_period_days;
use super::{BondArgs, CouponSchedule};
use crate::{formatter::dates::from_excel_date, functions::financial::dates::basis_days};

/// YIELD with one coupon period left: Excel does not iterate but evaluates the documented closed
/// form, in this order of operations, with DSR the basis' settlement-to-maturity count (not PRICE's
/// E - A) and E the actual period length on bases 1-3 (not 360/f or 365/f on bases 2 and 3).
/// Excel: see docs/excel-semantics/financial.md#yield-with-one-coupon-period-left-is-closed-form.
pub(super) fn yield_one_period(bond: &BondArgs, schedule: &CouponSchedule) -> Result<f64, String> {
    let (settlement, maturity) = (bond.settlement, bond.maturity);
    let e = if matches!(bond.basis, 1..=3) {
        coupon_period_days(settlement, maturity, bond.frequency)?
    } else {
        schedule.e
    };
    let dsr = basis_days(
        from_excel_date(settlement)?,
        from_excel_date(maturity)?,
        bond.basis,
    );
    let f = bond.frequency as f64;
    let (rate, pr, red) = (bond.rate, bond.yld_or_pr, bond.redemption);
    let result = ((red / 100.0 + rate / f) - (pr / 100.0 + schedule.a / e * rate / f))
        / (pr / 100.0 + (schedule.a / e) * (rate / f))
        * (f * e / dsr);
    if result.is_finite() {
        Ok(result)
    } else {
        Err("YIELD: no finite result".to_string())
    }
}
