//! The discount securities and T-bills: ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED,
//! YIELDDISC, YIELDMAT, TBILLEQ, TBILLPRICE and TBILLYIELD. They are Analysis `ToolPak` functions,
//! so every argument lifts over an array and a multi-cell range is `#VALUE!` in it, with no
//! intersection (`ArgKind::LiftArrays`, SPEC-0011). Evidence: probes/fin-argument-modes (every
//! argument over an array, the first over a range) and probes/fin-later-argument-ranges H1-H82
//! (every later argument over a range on its own rows), docs/excel-semantics/financial.md
//! "Financial function argument modes". Each kernel reads all its arguments, in order, before it
//! checks their values, so an error argument wins over an invalid value: ACCRINTM with a negative
//! rate and an `NA()` basis is `#N/A` (fin-later-argument-ranges J17, J20). Moved out of
//! `financial/mod.rs` (T-1372).

use super::dates::is_less_than_one_year;
use super::number_or_error;
use crate::functions::lift::{div_error, num_error};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// `(settlement, maturity, first, second, basis)` of DISC, INTRATE, PRICEDISC, RECEIVED and
/// YIELDDISC: whole-day dates, two amounts and the day-count basis (0 when omitted).
struct DiscountArgs {
    settlement: i64,
    maturity: i64,
    first: f64,
    second: f64,
    basis: f64,
}

/// `(settlement, maturity, issue, rate, last, basis)` of PRICEMAT and YIELDMAT.
struct MaturityArgs {
    settlement: i64,
    maturity: i64,
    issue: i64,
    rate: f64,
    last: f64,
    basis: f64,
}

impl Model {
    /// `ACCRINTM(issue, settlement, rate, [par], [basis])`.
    pub(crate) fn fn_accrintm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Accrintm, args, cell, &mut |model, values| {
            number_or_error(model.accrintm_of(values, cell))
        })
    }

    fn accrintm_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [issue, settlement, rate, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let issue_serial = self.cast_number_ref(issue, cell)?.floor() as i64;
        let settlement_serial = self.cast_number_ref(settlement, cell)?.floor() as i64;
        let rate = self.cast_number_ref(rate, cell)?;
        let par = self.cast_number_or(rest.first(), 1000.0, cell)?;
        // A boolean basis is #VALUE!, text "1" is 1, 1.9 truncates to 1, 5 and -1 are #NUM!
        // (fin-later-argument-ranges J1-J11), and it is read before any value is checked (J20).
        let basis = self.cast_number_no_bools_or(rest.get(1), 0.0, cell)?;
        if rate <= 0.0 || par <= 0.0 || issue_serial >= settlement_serial {
            return Err(num_error(cell));
        }
        let year_fraction = self.get_yearfrac(issue_serial, settlement_serial, basis, cell)?;
        Ok(par * rate * year_fraction)
    }

    /// Reads [`DiscountArgs`], each argument a number (a boolean is `#VALUE!`).
    fn discount_args(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<DiscountArgs, CalcResult> {
        let [settlement, maturity, first, second, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        Ok(DiscountArgs {
            settlement: self.cast_number_no_bools_ref(settlement, cell)?.floor() as i64,
            maturity: self.cast_number_no_bools_ref(maturity, cell)?.floor() as i64,
            first: self.cast_number_no_bools_ref(first, cell)?,
            second: self.cast_number_no_bools_ref(second, cell)?,
            basis: self.cast_number_no_bools_or(rest.first(), 0.0, cell)?,
        })
    }

    /// The year fraction from settlement to maturity of [`DiscountArgs`] whose two amounts are
    /// positive, `#NUM!` otherwise or when settlement is not before maturity.
    fn discount_year_fraction(
        &mut self,
        args: &DiscountArgs,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        if args.first <= 0.0 || args.second <= 0.0 || args.settlement >= args.maturity {
            return Err(num_error(cell));
        }
        self.get_yearfrac(args.settlement, args.maturity, args.basis, cell)
    }

    /// `DISC(settlement, maturity, pr, redemption, [basis])`.
    pub(crate) fn fn_disc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Disc, args, cell, &mut |model, values| {
            number_or_error(model.disc_of(values, cell))
        })
    }

    fn disc_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.discount_args(values, cell)?;
        let (pr, redemption) = (args.first, args.second);
        let yf = self.discount_year_fraction(&args, cell)?;
        if yf == 0.0 {
            return Err(div_error(cell));
        }
        Ok((1.0 - pr / redemption) / yf)
    }

    /// `INTRATE(settlement, maturity, investment, redemption, [basis])`.
    pub(crate) fn fn_intrate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Intrate, args, cell, &mut |model, values| {
            number_or_error(model.intrate_of(values, cell))
        })
    }

    fn intrate_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.discount_args(values, cell)?;
        let (investment, redemption) = (args.first, args.second);
        let yf = self.discount_year_fraction(&args, cell)?;
        if yf == 0.0 {
            return Err(div_error(cell));
        }
        Ok((redemption / investment - 1.0) / yf)
    }

    /// `PRICEDISC(settlement, maturity, discount, redemption, [basis])`.
    pub(crate) fn fn_pricedisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Pricedisc, args, cell, &mut |model, values| {
            number_or_error(model.pricedisc_of(values, cell))
        })
    }

    fn pricedisc_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.discount_args(values, cell)?;
        let (discount, redemption) = (args.first, args.second);
        let yf = self.discount_year_fraction(&args, cell)?;
        Ok(redemption * (1.0 - discount * yf))
    }

    /// `RECEIVED(settlement, maturity, investment, discount, [basis])`.
    pub(crate) fn fn_received(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Received, args, cell, &mut |model, values| {
            number_or_error(model.received_of(values, cell))
        })
    }

    fn received_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.discount_args(values, cell)?;
        let (investment, discount) = (args.first, args.second);
        let yf = self.discount_year_fraction(&args, cell)?;
        let denom = 1.0 - discount * yf;
        if denom <= 0.0 {
            return Err(num_error(cell));
        }
        Ok(investment / denom)
    }

    /// `YIELDDISC(settlement, maturity, pr, redemption, [basis])`.
    pub(crate) fn fn_yielddisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Yielddisc, args, cell, &mut |model, values| {
            number_or_error(model.yielddisc_of(values, cell))
        })
    }

    fn yielddisc_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.discount_args(values, cell)?;
        let (pr, redemption) = (args.first, args.second);
        let yf = self.discount_year_fraction(&args, cell)?;
        if yf == 0.0 {
            return Err(div_error(cell));
        }
        Ok((redemption / pr - 1.0) / yf)
    }

    /// Reads [`MaturityArgs`], each argument a number (a boolean is `#VALUE!`).
    fn maturity_args(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<MaturityArgs, CalcResult> {
        let [settlement, maturity, issue, rate, last, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        Ok(MaturityArgs {
            settlement: self.cast_number_no_bools_ref(settlement, cell)?.floor() as i64,
            maturity: self.cast_number_no_bools_ref(maturity, cell)?.floor() as i64,
            issue: self.cast_number_no_bools_ref(issue, cell)?.floor() as i64,
            rate: self.cast_number_no_bools_ref(rate, cell)?,
            last: self.cast_number_no_bools_ref(last, cell)?,
            basis: self.cast_number_no_bools_or(rest.first(), 0.0, cell)?,
        })
    }

    /// `PRICEMAT(settlement, maturity, issue, rate, yld, [basis])`.
    pub(crate) fn fn_pricemat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Pricemat, args, cell, &mut |model, values| {
            number_or_error(model.pricemat_of(values, cell))
        })
    }

    fn pricemat_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.maturity_args(values, cell)?;
        let (rate, yld) = (args.rate, args.last);
        if rate < 0.0
            || yld < 0.0
            || args.settlement >= args.maturity
            || args.issue >= args.settlement
        {
            return Err(num_error(cell));
        }
        let (a, dim, dsm) =
            self.get_mat_factors(args.issue, args.settlement, args.maturity, args.basis, cell)?;
        let denom = 1.0 + yld * dsm;
        if denom == 0.0 {
            return Err(div_error(cell));
        }
        Ok(100.0 * ((1.0 + rate * dim) / denom - rate * a))
    }

    /// `YIELDMAT(settlement, maturity, issue, rate, pr, [basis])`.
    pub(crate) fn fn_yieldmat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Yieldmat, args, cell, &mut |model, values| {
            number_or_error(model.yieldmat_of(values, cell))
        })
    }

    fn yieldmat_of(
        &mut self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = self.maturity_args(values, cell)?;
        let (rate, price) = (args.rate, args.last);
        if price <= 0.0
            || rate < 0.0
            || args.settlement >= args.maturity
            || args.issue >= args.settlement
        {
            return Err(num_error(cell));
        }
        let (a, dim, dsm) =
            self.get_mat_factors(args.issue, args.settlement, args.maturity, args.basis, cell)?;
        let denom = price / 100.0 + rate * a;
        if denom == 0.0 || dsm == 0.0 {
            return Err(div_error(cell));
        }
        Ok(((1.0 + rate * dim) / denom - 1.0) / dsm)
    }

    // The next three functions deal with Treasury Bills, or T-Bills: zero-coupon bonds that
    // mature in one year or less. Let r be the discount rate, v the face value, p the price and
    // d_m the days from settlement to maturity. Then p = v*(1 - d_m*r/360). For d_m < 183 the
    // bond equivalent yield is y = 365*r/(360 - r*d_m); from 183 days on, with
    // d_e = d_m - 365/2 (183 for d_m = 366), v = p*(1 + y/2)*(1 + d_e*y/365), which with the
    // relation of p and v is a quadratic equation for y.

    /// The checked `(settlement, maturity, rate or price)` of TBILLEQ, TBILLPRICE and TBILLYIELD.
    fn tbill_arguments(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64), CalcResult> {
        let [settlement, maturity, rate] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let settlement = self.cast_number_no_bools_ref(settlement, cell)?;
        let maturity = self.cast_number_no_bools_ref(maturity, cell)?;
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let less_than_one_year = is_less_than_one_year(
            self.financial_date(settlement),
            self.financial_date(maturity),
        )
        .map_err(|_| num_error(cell))?;
        if settlement >= maturity || !less_than_one_year || rate <= 0.0 {
            return Err(num_error(cell));
        }
        Ok((settlement, maturity, rate))
    }

    /// `TBILLEQ(settlement, maturity, discount)`.
    pub(crate) fn fn_tbilleq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Tbilleq, args, cell, &mut |model, values| {
            number_or_error(model.tbilleq_of(values, cell))
        })
    }

    fn tbilleq_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let (settlement, maturity, discount) = self.tbill_arguments(values, cell)?;
        let d_m = maturity - settlement;
        let result = if d_m < 183.0 {
            // A zero denominator (price 0) is 0 and a negative one `#NUM!`, not `#DIV/0!`
            // (probes/tbilleq-zero-price A3, probes/tbilleq-nonpositive-price; financial.md
            // "TBILLEQ and TBILLYIELD").
            let denominator = 360.0 - discount * d_m;
            if denominator < 0.0 {
                return Err(num_error(cell));
            }
            if denominator == 0.0 {
                return Ok(0.0);
            }
            365.0 * discount / denominator
        } else {
            // The root of (1 + y/2)(1 + (d_m - year/2) y / year) = 100/price in Excel's form and
            // order, fitted bit-exact on ironcalc-calc TBILLs.xlsx D4:D10, D24:D30; a price <= 0 is
            // `#NUM!` (probes/tbilleq-zero-price A1:A2; financial.md "TBILLEQ and TBILLYIELD").
            let year = if d_m == 366.0 { 366.0 } else { 365.0 };
            let price = 100.0 - discount * d_m * 100.0 / 360.0;
            if price <= 0.0 {
                return Err(num_error(cell));
            }
            let t = d_m / year;
            (-t + (t * t - (2.0 * t - 1.0) * (1.0 - 100.0 / price)).sqrt()) / (t - 0.5)
        };
        if result.is_infinite() {
            return Err(div_error(cell));
        }
        if result.is_nan() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `TBILLPRICE(settlement, maturity, discount)`.
    pub(crate) fn fn_tbillprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Tbillprice, args, cell, &mut |model, values| {
            number_or_error(model.tbillprice_of(values, cell))
        })
    }

    fn tbillprice_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let (settlement, maturity, discount) = self.tbill_arguments(values, cell)?;
        // days to maturity
        let d_m = maturity - settlement;
        let result = 100.0 * (1.0 - discount * d_m / 360.0);
        if result.is_infinite() {
            return Err(div_error(cell));
        }
        if result.is_nan() || result < 0.0 {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `TBILLYIELD(settlement, maturity, pr)`.
    pub(crate) fn fn_tbillyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Tbillyield, args, cell, &mut |model, values| {
            number_or_error(model.tbillyield_of(values, cell))
        })
    }

    fn tbillyield_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let (settlement, maturity, pr) = self.tbill_arguments(values, cell)?;
        let days = maturity - settlement;
        // Excel's order: `(100 - pr)/pr * (360/days)` (ironcalc-calc TBILLs.xlsx R6:R30, bit-exact;
        // docs/excel-semantics/financial.md "TBILLEQ and TBILLYIELD").
        Ok((100.0 - pr) / pr * (360.0 / days))
    }
}
