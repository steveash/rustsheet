//! The odd-last-period kernel of ODDLPRICE and ODDLYIELD, a port of `oddLFunc` from
//! `ExcelFinancialFunctions` `oddbonds.fs`.

use chrono::NaiveDate;

use super::coupon_schedule::{change_month, coupon_count};
use super::days_between_not_neg;
use crate::formatter::dates::{from_excel_date, real_date_to_serial};
use crate::functions::financial::dates::days_30_360_us_both;

// `days_between_not_neg`, but basis 0 uses the 30/360 ModifyBothDates rule.
fn days_between_not_neg_with_hack(d1: NaiveDate, d2: NaiveDate, basis: u32) -> f64 {
    if basis == 0 {
        days_30_360_us_both(d1, d2).max(0.0)
    } else {
        days_between_not_neg(d1, d2, basis)
    }
}

// Port of `oddLFunc` from oddbonds.fs. With `is_l_price` it returns the
// ODDLPRICE clean price; otherwise it returns the ODDLYIELD yield in closed form.
// last_interest: last regular interest (coupon) date before the odd last period.
#[allow(clippy::too_many_arguments)]
pub(super) fn odd_l_func(
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
