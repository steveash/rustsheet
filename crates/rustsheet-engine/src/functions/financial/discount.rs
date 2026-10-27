//! The discount securities and T-bills: ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED,
//! YIELDDISC, YIELDMAT, TBILLEQ, TBILLPRICE and TBILLYIELD. They are Analysis `ToolPak` functions,
//! so every argument lifts over an array and a multi-cell range is `#VALUE!` in it, with no
//! intersection (`ArgKind::LiftArrays`, SPEC-0011). Evidence: probes/fin-argument-modes (every
//! argument over an array, the first over a range) and probes/fin-later-argument-ranges H1-H82
//! (every later argument over a range on its own rows), docs/excel-semantics/financial.md
//! "Financial function argument modes". Each kernel reads its arguments one by one, returning
//! the first error, text or boolean, and checks only the basis and the dates as it reads them;
//! the date order and the amounts are checked after every read, so an error argument wins over an
//! invalid amount (probes/discount-argument-check-order, T-2484; financial.md "Discount securities
//! check order"). Moved out of `financial/mod.rs` (T-1372).

use super::dates::{is_less_than_one_year, DateReading};
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
    basis: u32,
}

/// `(settlement, maturity, issue, rate, last, basis)` of PRICEMAT and YIELDMAT.
struct MaturityArgs {
    settlement: i64,
    maturity: i64,
    issue: i64,
    rate: f64,
    last: f64,
    basis: u32,
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
        // Unlike the other discount securities, ACCRINTM reads its basis last; both dates are
        // #NUM! off the calendar as they are read (`ACCRINTM(-1,39448,0.1,1000,NA())` is #NUM!,
        // `ACCRINTM(NA(),2958466,..)` #N/A: probes/discount-argument-check-order).
        let issue_serial = self.discount_date_of(issue, DateReading::Shifted, cell)?;
        let settlement_serial = self.discount_date_of(settlement, DateReading::Shifted, cell)?;
        // A boolean rate or par is `#VALUE!` (`ACCRINTM(34825,37622,1E-9,TRUE,4)`, a TRUE cell
        // too: probes/t2455-transpose-single-value A41, A54); text "2" is 2 (A55).
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let par = self.cast_number_no_bools_or(rest.first(), 1000.0, cell)?;
        // A boolean basis is #VALUE!, text "1" is 1, 1.9 truncates to 1, 5 and -1 are #NUM!
        // (fin-later-argument-ranges J1-J11), and it is read before any value is checked (J20).
        let basis = self.day_count_basis_of(rest.get(1), cell)?;
        if rate <= 0.0 || par <= 0.0 || issue_serial >= settlement_serial {
            return Err(num_error(cell));
        }
        let year_fraction =
            self.get_yearfrac(issue_serial, settlement_serial, f64::from(basis), cell)?;
        Ok(par * rate * year_fraction)
    }

    /// A date argument of the discount securities as read: its whole-day serial, #NUM! unless it
    /// floors to a serial from 0 to 9999-12-31 (`DISC(39448,-1,NA(),..)` and
    /// `DISC(2958466,"x",..)` are #NUM!, `DISC(0.5,NA(),..)` #N/A:
    /// probes/discount-argument-check-order).
    fn discount_date_of(
        &self,
        value: &CalcResult,
        reading: DateReading,
        cell: CellReferenceIndex,
    ) -> Result<i64, CalcResult> {
        Ok(self
            .financial_calendar_date_of(value, reading, cell)?
            .floor() as i64)
    }

    /// Reads [`DiscountArgs`] in Excel's order: the basis first, then the rest left to right, each
    /// a number (a boolean is `#VALUE!`) and each date on the calendar.
    fn discount_args(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<DiscountArgs, CalcResult> {
        let [settlement, maturity, first, second, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // The basis is read and checked first: `RECEIVED(37025,2.5,1E+300,1/0,"abc")` and
        // `DISC(37025,2.5,1/0,100,"abc")` are `#VALUE!`, `RECEIVED(..,NA(),1/0)` `#DIV/0!`
        // (probes/t2455-transpose-single-value A21, A64, A65), `DISC(NA(),..,5)` #NUM!
        // (probes/discount-argument-check-order; financial.md "Discount securities check order").
        let basis = self.day_count_basis_of(rest.first(), cell)?;
        Ok(DiscountArgs {
            settlement: self.discount_date_of(settlement, DateReading::Shifted, cell)?,
            maturity: self.discount_date_of(maturity, DateReading::Shifted, cell)?,
            first: self.cast_number_no_bools_ref(first, cell)?,
            second: self.cast_number_no_bools_ref(second, cell)?,
            basis,
        })
    }

    /// The basis `(day count, year length)` from settlement to maturity of [`DiscountArgs`] whose
    /// two amounts are positive, `#NUM!` otherwise or when settlement is not before maturity.
    fn discount_days(
        &self,
        args: &DiscountArgs,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        if args.first <= 0.0 || args.second <= 0.0 || args.settlement >= args.maturity {
            return Err(num_error(cell));
        }
        self.year_parts(args.settlement, args.maturity, f64::from(args.basis), cell)
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
        let (dsm, b) = self.discount_days(&args, cell)?;
        if dsm == 0.0 {
            return Err(div_error(cell));
        }
        Ok((1.0 - pr / redemption) * (b / dsm))
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
        let (dim, b) = self.discount_days(&args, cell)?;
        if dim == 0.0 {
            return Err(div_error(cell));
        }
        Ok((redemption - investment) / investment * (b / dim))
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
        let (dsm, b) = self.discount_days(&args, cell)?;
        Ok(redemption - discount * redemption * dsm / b)
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
        let (dim, b) = self.discount_days(&args, cell)?;
        // `discount*DIM/B` left to right, not `discount*(DIM/B)`: `RECEIVED(39448,43831,"1",0.05,0)`
        // is 2.5, not 2.5000000000000004 (probes/discount-argument-check-order A758).
        let denom = 1.0 - discount * dim / b;
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
        let (dsm, b) = self.discount_days(&args, cell)?;
        if dsm == 0.0 {
            return Err(div_error(cell));
        }
        Ok((redemption - pr) / pr * (b / dsm))
    }

    /// Reads [`MaturityArgs`] in the order of [`Model::discount_args`]: the basis first, then the
    /// rest left to right (probes/discount-argument-check-order).
    fn maturity_args(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<MaturityArgs, CalcResult> {
        let [settlement, maturity, issue, rate, last, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let basis = self.day_count_basis_of(rest.first(), cell)?;
        Ok(MaturityArgs {
            settlement: self.discount_date_of(settlement, DateReading::Shifted, cell)?,
            maturity: self.discount_date_of(maturity, DateReading::Shifted, cell)?,
            issue: self.discount_date_of(issue, DateReading::Shifted, cell)?,
            rate: self.cast_number_no_bools_ref(rate, cell)?,
            last: self.cast_number_no_bools_ref(last, cell)?,
            basis,
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
        let (b, dim, a, dsm) = self.get_mat_factors(
            args.issue,
            args.settlement,
            args.maturity,
            f64::from(args.basis),
            cell,
        )?;
        let denom = 1.0 + dsm / b * yld;
        if denom == 0.0 {
            return Err(div_error(cell));
        }
        Ok((100.0 + dim / b * rate * 100.0) / denom - a / b * rate * 100.0)
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
        let (b, dim, a, dsm) = self.get_mat_factors(
            args.issue,
            args.settlement,
            args.maturity,
            f64::from(args.basis),
            cell,
        )?;
        let denom = price / 100.0 + a / b * rate;
        if denom == 0.0 || dsm == 0.0 {
            return Err(div_error(cell));
        }
        Ok((1.0 + dim / b * rate - denom) / denom * (b / dsm))
    }

    // The next three functions deal with Treasury Bills, or T-Bills: zero-coupon bonds that
    // mature in one year or less. Let r be the discount rate, v the face value, p the price and
    // d_m the days from settlement to maturity. Then p = v*(1 - d_m*r/360). For d_m < 183 the
    // bond equivalent yield is y = 365*r/(360 - r*d_m); from 183 days on, with
    // d_e = d_m - 365/2 (183 for d_m = 366), v = p*(1 + y/2)*(1 + d_e*y/365), which with the
    // relation of p and v is a quadratic equation for y.

    /// The checked `(days from settlement to maturity, rate or price)` of TBILLEQ, TBILLPRICE and
    /// TBILLYIELD.
    fn tbill_arguments(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        let [settlement, maturity, rate] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // The date order and the one-year limit are checked after the discount is read. Both
        // dates floor to whole days, so a settlement from 0 to below 1 is serial 0
        // (`TBILLPRICE(0.5,1.9,0.1)` = `TBILLPRICE(0,1,0.1)` and `TBILLPRICE(0.5,0.9,0.1)` is
        // #NUM!: probes/tbill-settlement-below-one A1:A5; financial.md "T-bill settlement below
        // serial 1"). The TBILL functions read the dates themselves (`DateReading::Calendar`), so
        // in a 1904 workbook 2023-02-28 to 2024-02-29 is more than a year
        // (probes/date-system-1904-financial-shift A31:A33).
        let reading = DateReading::Calendar; // TBILLEQ, TBILLPRICE, TBILLYIELD: `DateReading::of`
        let settlement = self.discount_date_of(settlement, reading, cell)?;
        let maturity = self.discount_date_of(maturity, reading, cell)?;
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let less_than_one_year = is_less_than_one_year(
            self.financial_date_as(reading, settlement as f64),
            self.financial_date_as(reading, maturity as f64),
        )
        .ok_or_else(|| num_error(cell))?;
        if settlement >= maturity || !less_than_one_year || rate <= 0.0 {
            return Err(num_error(cell));
        }
        Ok(((maturity - settlement) as f64, rate))
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
        let (d_m, discount) = self.tbill_arguments(values, cell)?;
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
        let (d_m, discount) = self.tbill_arguments(values, cell)?;
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
        let (days, pr) = self.tbill_arguments(values, cell)?;
        // Excel's order: `(100 - pr)/pr * (360/days)` (ironcalc-calc TBILLs.xlsx R6:R30, bit-exact;
        // docs/excel-semantics/financial.md "TBILLEQ and TBILLYIELD").
        Ok((100.0 - pr) / pr * (360.0 / days))
    }
}
