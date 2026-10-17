//! The annuity functions PMT, PV, FV, NPER, RATE, IPMT, PPMT and ISPMT, and RRI and PDURATION:
//! core worksheet functions, so every argument lifts over an array and over a multi-cell range
//! (`ArgKind::Lift`, SPEC-0011). Each `fn_x` runs its scalar kernel through `lift_call`; a kernel
//! reads its arguments in the order the scalar function always did, which decides an element's
//! error (`INDEX(PMT({#NUM!},{#N/A},100),1)` is `#NUM!`, probes/scalar-lift-open-a W14, W15).
//! Evidence: probes/fin-argument-modes, probes/fin-later-argument-ranges K1-K25 and
//! probes/scalar-lift-shapes H13-H48 (docs/excel-semantics/financial.md "Financial function
//! argument modes"). Moved out of `financial/mod.rs` (T-1372).

use super::financial_util::compute_rate;
use super::{formula_error, number_or_error};
use crate::{
    arithmetic::{extended_quotient, extended_sum, power},
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// `P = (1+r)^n` as Excel's PV and FV compute it: `1+r` with Excel's `+` and the power with its `^`
/// (right-to-left square-and-multiply for an integer `n`). With [`annuity_factor`] this matches all
/// 308 PV and 308 FV cells of probes-numeric/m9-fin-pv-fv (docs/excel-semantics/financial.md
/// "Annuity evaluation order"; fit lab `annuity-pv`, `annuity-fv`, T-1524).
fn annuity_power(rate: f64, nper: f64) -> Result<f64, (Error, String)> {
    power(extended_sum(1.0, rate), nper).map_err(|error| (error, "Invalid result".to_string()))
}

/// The annuity factor `k*((P-1)/r)`, `k = 1+r*type`, evaluated in doubles in exactly this
/// parenthesization: `(pmt*k)*((P-1)/r)` instead misses 38 of the 308 PV cells (financial.md).
fn annuity_factor(rate: f64, p: f64, period_start: bool) -> f64 {
    let k = if period_start { 1.0 + rate } else { 1.0 };
    k * ((p - 1.0) / rate)
}

fn compute_payment(
    rate: f64,
    nper: f64,
    pv: f64,
    fv: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    if rate == 0.0 {
        if nper == 0.0 {
            return Err((Error::NUM, "Period count must be non zero".to_string()));
        }
        return Ok(-(pv + fv) / nper);
    }
    if rate <= -1.0 {
        return Err((Error::NUM, "Rate must be > -1".to_string()));
    };
    let rate_nper = if nper == 0.0 {
        1.0
    } else {
        (1.0 + rate).powf(nper)
    };
    let result = if period_start {
        // type = 1
        (fv + pv * rate_nper) * rate / ((1.0 + rate) * (1.0 - rate_nper))
    } else {
        (fv * rate + pv * rate * rate_nper) / (1.0 - rate_nper)
    };
    if result.is_nan() || result.is_infinite() {
        return Err((Error::NUM, "Invalid result".to_string()));
    }
    Ok(result)
}

/// Which formulation [`compute_future_value`] uses.
#[derive(Clone, Copy)]
enum FutureValueForm {
    /// FV itself: financial.md's fitted `-(pv*P + pmt*(k*((P-1)/r)))`.
    Fitted,
    /// IPMT's FV recurrence keeps the earlier `powf` form: the fitted form alone regresses 4
    /// Excel-cached IPMT/CUMIPMT cells (ironcalc-calc `IPMT_PPMT.xlsx` H4, H31, G24 and
    /// `CUMPRINC_CUMIPMT.xlsx` G9), because IPMT's own last digits are still unexplained.
    // TODO(T-1528): fold `IpmtRecurrence` into `Fitted` once IPMT is fitted.
    IpmtRecurrence,
}

fn compute_future_value(
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    period_start: bool,
    form: FutureValueForm,
) -> Result<f64, (Error, String)> {
    if rate == 0.0 {
        return Ok(-pv - pmt * nper);
    }
    if rate == -1.0 && nper < 0.0 {
        return Err((Error::DIV, "Divide by zero".to_string()));
    }

    let fv = match form {
        // FV = -(pv*P + pmt*(k*((P-1)/r))), every operation in doubles (financial.md)
        FutureValueForm::Fitted => {
            let p = annuity_power(rate, nper)?;
            -(pv * p + pmt * annuity_factor(rate, p, period_start))
        }
        FutureValueForm::IpmtRecurrence => {
            let rate_nper = (1.0 + rate).powf(nper);
            if period_start {
                -pv * rate_nper - pmt * (1.0 + rate) * (rate_nper - 1.0) / rate
            } else {
                -pv * rate_nper - pmt * (rate_nper - 1.0) / rate
            }
        }
    };
    if fv.is_nan() {
        return Err((Error::NUM, "Invalid result".to_string()));
    }
    if !fv.is_finite() {
        return Err((Error::DIV, "Divide by zero".to_string()));
    }
    Ok(fv)
}

pub(super) fn compute_ipmt(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    // http://www.staff.city.ac.uk/o.s.kerr/CompMaths/WSheet4.pdf
    // https://www.experts-exchange.com/articles/1948/A-Guide-to-the-PMT-FV-IPMT-and-PPMT-Functions.html
    // type = 0 (end of period)
    // impt = -[(1+rate)^(period-1)*(pv*rate+pmt)-pmt]
    // ipmt = FV(rate, period-1, payment, pv, type) * rate
    // type = 1 (beginning of period)
    // ipmt = (FV(rate, period-2, payment, pv, type) - payment) * rate
    let payment = compute_payment(
        rate,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    if period < 1.0 || period >= period_count + 1.0 {
        return Err((
            Error::NUM,
            format!("Period must be between 1 and {}", period_count + 1.0),
        ));
    }
    if period == 1.0 && period_start {
        Ok(0.0)
    } else {
        let p = if period_start {
            period - 2.0
        } else {
            period - 1.0
        };
        let c = if period_start { -payment } else { 0.0 };
        let fv = compute_future_value(
            rate,
            p,
            payment,
            present_value,
            period_start,
            FutureValueForm::IpmtRecurrence,
        )?;
        Ok((fv + c) * rate)
    }
}

pub(super) fn compute_ppmt(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    let payment = compute_payment(
        rate,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    // It's a bit unfortunate that the first thing compute_ipmt does is compute_payment again
    let ipmt = compute_ipmt(
        rate,
        period,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    Ok(payment - ipmt)
}

// These formulas revolve around compound interest and annuities.
// The financial functions pv, rate, nper, pmt and fv:
// rate = interest rate per period
// nper (number of periods) = loan term
// pv (present value) = loan amount
// fv (future value) = cash balance after last payment. Default is 0
// type = the annuity type indicates when payments are due
//         * 0 (default) Payments are made at the end of the period
//         * 1 Payments are made at the beginning of the period (like a lease or rent)
// The variable period_start is true if type is 1
// They are linked by the formulas:
// If rate != 0
//   $pv*(1+rate)^nper+pmt*(1+rate*type)*((1+rate)^nper-1)/rate+fv=0$
// If rate = 0
//   $pmt*nper+pv+fv=0$
// All, except for rate are easily solvable in terms of the others.
// In these formulas the payment (pmt) is normally negative

/// The `#NUM!` a kernel's own check gives; it allocates no message.
fn num_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, String::new())
}

/// The `#DIV/0!` a kernel's own check gives.
fn div_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::DIV, cell, String::new())
}

impl Model {
    /// `PMT(rate, nper, pv, [fv], [type])`.
    pub(crate) fn fn_pmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Pmt, args, cell, &mut |model, values| {
            number_or_error(model.pmt_of(values, cell))
        })
    }

    fn pmt_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [rate, nper, pv, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let nper = self.cast_number_ref(nper, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        let fv = self.cast_number_or(rest.first(), 0.0, cell)?;
        // 0 or omitted: at the end of the period.
        let period_start = self.cast_number_or(rest.get(1), 0.0, cell)? != 0.0;
        compute_payment(rate, nper, pv, fv, period_start).map_err(formula_error(cell))
    }

    /// `PV(rate, nper, pmt, [fv], [type])`.
    pub(crate) fn fn_pv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Pv, args, cell, &mut |model, values| {
            number_or_error(model.pv_of(values, cell))
        })
    }

    fn pv_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [rate, period_count, payment, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let period_count = self.cast_number_ref(period_count, cell)?;
        let payment = self.cast_number_ref(payment, cell)?;
        let future_value = self.cast_number_or(rest.first(), 0.0, cell)?;
        let period_start = self.cast_number_or(rest.get(1), 0.0, cell)? != 0.0;
        if rate == 0.0 {
            return Ok(-future_value - payment * period_count);
        }
        if rate == -1.0 {
            return Err(div_error(cell));
        };
        // PV = -(fv + pmt*(k*((P-1)/r)))/P, every operation in doubles (financial.md)
        let p = annuity_power(rate, period_count).map_err(formula_error(cell))?;
        let result = -(future_value + payment * annuity_factor(rate, p, period_start)) / p;
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `RATE(nper, pmt, pv, [fv], [type], [guess])`. The solver stops after 50 steps, so the
    /// table charges each element that weight (`KernelWork::Weighted`, SPEC-0011 security rule).
    pub(crate) fn fn_rate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Rate, args, cell, &mut |model, values| {
            number_or_error(model.rate_of(values, cell))
        })
    }

    fn rate_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [nper, pmt, pv, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let nper = self.cast_number_ref(nper, cell)?;
        let pmt = self.cast_number_ref(pmt, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        let fv = self.cast_number_or(rest.first(), 0.0, cell)?;
        let annuity_type = i32::from(self.cast_number_or(rest.get(1), 0.0, cell)? != 0.0);
        let guess = self.cast_number_or(rest.get(2), 0.1, cell)?;
        compute_rate(pv, fv, nper, pmt, annuity_type, guess).map_err(formula_error(cell))
    }

    /// `NPER(rate, pmt, pv, [fv], [type])`.
    pub(crate) fn fn_nper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Nper, args, cell, &mut |model, values| {
            number_or_error(model.nper_of(values, cell))
        })
    }

    fn nper_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [rate, payment, present_value, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let payment = self.cast_number_ref(payment, cell)?;
        let present_value = self.cast_number_ref(present_value, cell)?;
        let future_value = self.cast_number_or(rest.first(), 0.0, cell)?;
        let period_start = self.cast_number_or(rest.get(1), 0.0, cell)? != 0.0;
        if rate == 0.0 {
            if payment == 0.0 {
                return Err(div_error(cell));
            }
            return Ok(-(future_value + present_value) / payment);
        }
        if rate < -1.0 {
            return Err(num_error(cell));
        };
        // NPER = ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r), with plain logarithms of
        // the double 1+r, not log1p: all 95 numeric cells of probes-numeric/m9-fin-nper-npv, where
        // log1p gives 11 (financial.md "Annuity evaluation order"; fit lab `annuity-nper`, T-1524).
        let k = if period_start { 1.0 + rate } else { 1.0 };
        let payment_k = payment * k;
        let ratio = (payment_k - future_value * rate) / (payment_k + present_value * rate);
        if ratio.is_nan() || ratio <= 0.0 || ratio.is_infinite() {
            return Err(num_error(cell));
        }
        // The final division is Excel's `/`, rounded twice (A9 `NPER(0.2,-1000,-1000,20000)`)
        let result = extended_quotient(ratio.ln(), (1.0 + rate).ln());
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `FV(rate, nper, pmt, [pv], [type])`.
    pub(crate) fn fn_fv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Fv, args, cell, &mut |model, values| {
            number_or_error(model.fv_of(values, cell))
        })
    }

    fn fv_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [rate, nper, pmt, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let nper = self.cast_number_ref(nper, cell)?;
        let pmt = self.cast_number_ref(pmt, cell)?;
        let pv = self.cast_number_or(rest.first(), 0.0, cell)?;
        let period_start = self.cast_number_or(rest.get(1), 0.0, cell)? != 0.0;
        compute_future_value(rate, nper, pmt, pv, period_start, FutureValueForm::Fitted)
            .map_err(formula_error(cell))
    }

    /// `IPMT(rate, per, nper, pv, [fv], [type])`.
    pub(crate) fn fn_ipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Ipmt, args, cell, &mut |model, values| {
            number_or_error(model.period_payment_args(values, cell).and_then(|a| {
                compute_ipmt(a.0, a.1, a.2, a.3, a.4, a.5).map_err(formula_error(cell))
            }))
        })
    }

    /// `PPMT(rate, per, nper, pv, [fv], [type])`.
    pub(crate) fn fn_ppmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Ppmt, args, cell, &mut |model, values| {
            number_or_error(model.period_payment_args(values, cell).and_then(|a| {
                compute_ppmt(a.0, a.1, a.2, a.3, a.4, a.5).map_err(formula_error(cell))
            }))
        })
    }

    /// The `(rate, per, nper, pv, fv, period_start)` of IPMT and PPMT, read in that order.
    fn period_payment_args(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64, f64, f64, bool), CalcResult> {
        let [rate, period, period_count, present_value, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let period = self.cast_number_ref(period, cell)?;
        let period_count = self.cast_number_ref(period_count, cell)?;
        let present_value = self.cast_number_ref(present_value, cell)?;
        let future_value = self.cast_number_or(rest.first(), 0.0, cell)?;
        let period_start = self.cast_number_or(rest.get(1), 0.0, cell)? != 0.0;
        Ok((
            rate,
            period,
            period_count,
            present_value,
            future_value,
            period_start,
        ))
    }

    /// `ISPMT(rate, per, nper, pv)`: `pv*rate*(per/nper - 1)`.
    pub(crate) fn fn_ispmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Ispmt, args, cell, &mut |model, values| {
            number_or_error(model.ispmt_of(values, cell))
        })
    }

    fn ispmt_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [rate, per, nper, pv] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let per = self.cast_number_ref(per, cell)?;
        let nper = self.cast_number_ref(nper, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        if nper == 0.0 {
            return Err(div_error(cell));
        }
        Ok(pv * rate * (per / nper - 1.0))
    }

    /// `RRI(nper, pv, fv)`: `(fv/pv)^(1/nper) - 1`.
    pub(crate) fn fn_rri(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Rri, args, cell, &mut |model, values| {
            number_or_error(model.rri_of(values, cell))
        })
    }

    fn rri_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [nper, pv, fv] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let nper = self.cast_number_ref(nper, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        let fv = self.cast_number_ref(fv, cell)?;
        if nper <= 0.0 {
            return Err(num_error(cell));
        }
        // When the value is unchanged the rate is zero (this also covers the
        // pv == fv == 0 case, which Excel returns as 0 rather than an error).
        if fv == pv {
            return Ok(0.0);
        }
        if pv == 0.0 {
            // Note error is NUM not DIV/0 also below
            return Err(num_error(cell));
        }
        let result = (fv / pv).powf(1.0 / nper) - 1.0;
        if result.is_infinite() || result.is_nan() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `PDURATION(rate, pv, fv)`: `(ln(fv) - ln(pv))/ln(1+r)`.
    pub(crate) fn fn_pduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Pduration, args, cell, &mut |model, values| {
            number_or_error(model.pduration_of(values, cell))
        })
    }

    fn pduration_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [rate, pv, fv] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        let fv = self.cast_number_ref(fv, cell)?;
        if fv <= 0.0 || pv <= 0.0 || rate <= 0.0 {
            return Err(num_error(cell));
        }
        let result = (fv.ln() - pv.ln()) / ((1.0 + rate).ln());
        if result.is_infinite() {
            return Err(div_error(cell));
        }
        if result.is_nan() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}
