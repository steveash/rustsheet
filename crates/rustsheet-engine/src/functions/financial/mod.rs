use crate::{
    arithmetic::{extended_quotient, extended_sum, power},
    calc_result::{CalcResult, Range},
    constants::{LAST_COLUMN, LAST_ROW, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

mod accrint;
mod dates;
mod financial_bonds;
mod financial_depreciation;
mod financial_util;
mod mirr;

use dates::is_less_than_one_year;
use financial_util::{compute_irr, compute_npv, compute_rate, compute_xirr, compute_xnpv};

// See:
// https://github.com/apache/openoffice/blob/c014b5f2b55cff8d4b0c952d5c16d62ecde09ca1/main/scaddins/source/analysis/financial.cxx

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

fn compute_ipmt(
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

fn compute_ppmt(
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

impl Model {
    fn get_array_of_numbers_generic(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        accept_number_node: bool,
        handle_empty_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
        handle_non_number_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        match self.evaluate_node_in_context(arg, *cell) {
            CalcResult::Number(value) if accept_number_node => values.push(value),
            CalcResult::Number(_) => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected range of numbers".to_string(),
                ));
            }
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        *cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                let sheet = left.sheet;
                let row1 = left.row;
                let mut row2 = right.row;
                let column1 = left.column;
                let mut column2 = right.column;
                let (used_row, used_column) = self
                    .workbook
                    .worksheet(sheet)
                    .map_err(|_| {
                        CalcResult::new_error(
                            Error::ERROR,
                            *cell,
                            format!("Invalid worksheet index: '{sheet}'"),
                        )
                    })?
                    .used_area_end();
                if row1 == 1 && row2 == LAST_ROW {
                    row2 = used_row;
                }
                if column1 == 1 && column2 == LAST_COLUMN {
                    column2 = used_column;
                }
                for row in row1..=row2 {
                    for column in column1..=column2 {
                        let cell_ref = CellReferenceIndex { sheet, row, column };
                        match self.evaluate_cell(cell_ref) {
                            CalcResult::Number(value) => values.push(value),
                            error @ CalcResult::Error { .. } => return Err(error),
                            CalcResult::EmptyCell => {
                                if let Some(value) = handle_empty_cell()? {
                                    values.push(value);
                                }
                            }
                            _ => {
                                if let Some(value) = handle_non_number_cell()? {
                                    values.push(value);
                                }
                            }
                        }
                    }
                }
                // The clip skips empty cells Excel still walks, so an empty-cell error applies after
                // the used area: `XNPV(0.1,A:A,B:B)` is #NUM! (probes/xirr-xnpv-whole-column-nogaps A1).
                if row2 < right.row || column2 < right.column {
                    handle_empty_cell()?;
                }
            }
            CalcResult::Array(arr) => {
                for row in arr {
                    for node in row {
                        match node {
                            ArrayNode::Number(f) => values.push(f),
                            ArrayNode::Error(e) => {
                                return Err(CalcResult::new_error(
                                    e,
                                    *cell,
                                    "Error in array".to_string(),
                                ));
                            }
                            ArrayNode::Empty => {
                                if let Some(value) = handle_empty_cell()? {
                                    values.push(value);
                                }
                            }
                            _ => {
                                if let Some(value) = handle_non_number_cell()? {
                                    values.push(value);
                                }
                            }
                        }
                    }
                }
            }
            error @ CalcResult::Error { .. } => return Err(error),
            _ => {
                handle_non_number_cell()?;
            }
        }
        Ok(values)
    }

    fn get_array_of_numbers(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true,        // accept_number_node
            || Ok(None), // Ignore empty cells
            || Ok(None), // Ignore non-number cells
        )
    }

    fn get_array_of_numbers_xpnv(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        error: Error,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true, // accept_number_node
            || {
                Err(CalcResult::new_error(
                    Error::NUM,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
            || {
                Err(CalcResult::new_error(
                    error.clone(),
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    fn get_array_of_numbers_xirr(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            false,            // Do not accept a single number node
            || Ok(Some(0.0)), // Treat empty cells as zero
            || {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    /// PMT(rate, nper, pv, [fv], [type])
    pub(crate) fn fn_pmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // number of periods
        let nper = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // present value
        let pv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // future_value
        let fv = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        match compute_payment(rate, nper, pv, fv, period_start) {
            Ok(p) => CalcResult::Number(p),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // PV(rate, nper, pmt, [fv], [type])
    pub(crate) fn fn_pv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // nper
        let period_count = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pmt
        let payment = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        if rate == 0.0 {
            return CalcResult::Number(-future_value - payment * period_count);
        }
        if rate == -1.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Rate must be != -1".to_string(),
            };
        };
        // PV = -(fv + pmt*(k*((P-1)/r)))/P, every operation in doubles (financial.md)
        let p = match annuity_power(rate, period_count) {
            Ok(p) => p,
            Err((error, message)) => {
                return CalcResult::Error {
                    error,
                    origin: cell,
                    message,
                }
            }
        };
        let result = -(future_value + payment * annuity_factor(rate, p, period_start)) / p;
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // RATE(nper, pmt, pv, [fv], [type], [guess])
    pub(crate) fn fn_rate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let nper = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pmt = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let fv = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let annuity_type = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => i32::from(f != 0.0),
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            0
        };

        let guess = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };

        match compute_rate(pv, fv, nper, pmt, annuity_type, guess) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // NPER(rate,pmt,pv,[fv],[type])
    pub(crate) fn fn_nper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pmt
        let payment = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pv
        let present_value = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        if rate == 0.0 {
            if payment == 0.0 {
                return CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "Divide by zero".to_string(),
                };
            }
            return CalcResult::Number(-(future_value + present_value) / payment);
        }
        if rate < -1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Rate must be > -1".to_string(),
            };
        };
        // NPER = ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r), with plain logarithms of
        // the double 1+r, not log1p: all 95 numeric cells of probes-numeric/m9-fin-nper-npv, where
        // log1p gives 11 (financial.md "Annuity evaluation order"; fit lab `annuity-nper`, T-1524).
        let k = if period_start { 1.0 + rate } else { 1.0 };
        let payment_k = payment * k;
        let ratio = (payment_k - future_value * rate) / (payment_k + present_value * rate);
        if ratio.is_nan() || ratio <= 0.0 || ratio.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Cannot compute.".to_string(),
            };
        }
        // The final division is Excel's `/`, rounded twice (A9 `NPER(0.2,-1000,-1000,20000)`)
        let result = extended_quotient(ratio.ln(), (1.0 + rate).ln());
        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Cannot compute.".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    // FV(rate, nper, pmt, [pv], [type])
    pub(crate) fn fn_fv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // number of periods
        let nper = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // payment
        let pmt = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // present value
        let pv = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        match compute_future_value(rate, nper, pmt, pv, period_start, FutureValueForm::Fitted) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // IPMT(rate, per, nper, pv, [fv], [type])
    pub(crate) fn fn_ipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // per
        let period = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // nper
        let period_count = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pv
        let present_value = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        let ipmt = match compute_ipmt(
            rate,
            period,
            period_count,
            present_value,
            future_value,
            period_start,
        ) {
            Ok(f) => f,
            Err(error) => {
                return CalcResult::Error {
                    error: error.0,
                    origin: cell,
                    message: error.1,
                }
            }
        };
        CalcResult::Number(ipmt)
    }

    // PPMT(rate, per, nper, pv, [fv], [type])
    pub(crate) fn fn_ppmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // per
        let period = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // nper
        let period_count = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pv
        let present_value = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };

        let ppmt = match compute_ppmt(
            rate,
            period,
            period_count,
            present_value,
            future_value,
            period_start,
        ) {
            Ok(f) => f,
            Err(error) => {
                return CalcResult::Error {
                    error: error.0,
                    origin: cell,
                    message: error.1,
                }
            }
        };
        CalcResult::Number(ppmt)
    }

    // NPV(rate, value1, [value2],...)
    // npv = Sum[value[i]/(1+rate)^i, {i, 1, n}]
    pub(crate) fn fn_npv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let mut values = Vec::new();
        for arg in &args[1..] {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(value),
                CalcResult::Range { left, right } => {
                    let areas = [Range { left, right }];
                    if let Err(e) = self.for_each_number_in_areas(&areas, cell, |v| values.push(v))
                    {
                        return e;
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // We ignore booleans and strings
                }
            };
        }
        match compute_npv(rate, &values) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::new_error(error.0, cell, error.1),
        }
    }

    // Returns the internal rate of return for a series of cash flows represented by the numbers
    // in values.
    // These cash flows do not have to be even, as they would be for an annuity.
    // However, the cash flows must occur at regular intervals, such as monthly or annually.
    // The internal rate of return is the interest rate received for an investment consisting
    // of payments (negative values) and income (positive values) that occur at regular periods

    // IRR(values, [guess])
    pub(crate) fn fn_irr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 2 || arg_count == 0 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let guess = if arg_count == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };
        match compute_irr(&values, guess) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // XNPV(rate, values, dates)
    pub(crate) fn fn_xnpv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // All three are required (docs/excel-semantics/syntax.md, "Minimum argument counts").
        let [rate, values, dates] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        let rate = match self.get_number(rate, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let values = match self.get_array_of_numbers_xpnv(values, &cell, Error::NUM) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let dates = match self.get_array_of_numbers_xpnv(dates, &cell, Error::VALUE) {
            Ok(s) => s,
            Err(error) => return error,
        };
        // Decimal points on dates are truncated
        let dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
        let values_count = values.len();
        // If values and dates contain a different number of values, XNPV returns the #NUM! error value.
        if values_count != dates.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Values and dates must be the same length".to_string(),
            );
        }
        if values_count == 0 {
            return CalcResult::new_error(Error::NUM, cell, "Not enough values".to_string());
        }
        let first_date = dates[0];
        for date in &dates {
            if *date < MINIMUM_DATE_SERIAL_NUMBER as f64
                || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64
            {
                // Excel docs claim that if any number in dates is not a valid date,
                // XNPV returns the #VALUE! error value, but it seems to return #VALUE!
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid number for date".to_string(),
                );
            }
            // If any number in dates precedes the starting date, XNPV returns the #NUM! error value.
            if date < &first_date {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Date precedes the starting date".to_string(),
                );
            }
        }
        // It seems Excel returns #NUM! if rate < 0, this is only necessary if r <= -1
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate needs to be > 0".to_string());
        }
        match compute_xnpv(rate, &values, &dates) {
            Ok(f) => CalcResult::Number(f),
            Err((error, message)) => CalcResult::new_error(error, cell, message),
        }
    }

    // XIRR(values, dates, [guess])
    pub(crate) fn fn_xirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_xirr(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let dates = match self.get_array_of_numbers_xirr(&args[1], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let guess = if arg_count == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };
        // Decimal points on dates are truncated
        let dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
        let values_count = values.len();
        // If values and dates contain a different number of values, XNPV returns the #NUM! error value.
        if values_count != dates.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Values and dates must be the same length".to_string(),
            );
        }
        if values_count == 0 {
            return CalcResult::new_error(Error::NUM, cell, "Not enough values".to_string());
        }
        // Excel reads an empty value or date cell as 0 and accepts date 0 and dates before the
        // first one, but is #NUM! for a negative date, a date past 9999-12-31, or when no date is
        // later than the first: {-1000,1100} on {0,45365} is 1.1^(365/45365) - 1,
        // {-1000,1100,-1000} on {45000,45365,44000} is -0.2467, and {-1000,1100} on
        // {45365,45000} is #NUM! although it has a root (financial.md, "XIRR reads empty cells as
        // 0 and accepts earlier dates"; probes/xirr-mismatched-empties, xirr-early-dates; T-1124).
        let first_date = dates[0];
        if dates
            .iter()
            .any(|date| *date < 0.0 || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)
        {
            return CalcResult::new_error(Error::NUM, cell, "Invalid number for date".to_string());
        }
        if !dates.iter().any(|date| *date > first_date) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "No date is later than the starting date".to_string(),
            );
        }
        match compute_xirr(&values, &dates, guess) {
            Ok(f) => CalcResult::Number(f),
            Err((error, message)) => CalcResult::Error {
                error,
                origin: cell,
                message,
            },
        }
    }

    // ISPMT(rate, per, nper, pv)
    // Formula is:
    // $$pv*rate*\left(\frac{per}{nper}-1\right)$$
    pub(crate) fn fn_ispmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let per = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if nper == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        CalcResult::Number(pv * rate * (per / nper - 1.0))
    }

    // RRI(nper, pv, fv)
    // Formula is
    // $$ \left(\frac{fv}{pv}\right)^{\frac{1}{nper}}-1  $$
    pub(crate) fn fn_rri(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let nper = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let fv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if nper <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "nper should be >0".to_string());
        }
        // When the value is unchanged the rate is zero (this also covers the
        // pv == fv == 0 case, which Excel returns as 0 rather than an error).
        if fv == pv {
            return CalcResult::Number(0.0);
        }
        if pv == 0.0 {
            // Note error is NUM not DIV/0 also bellow
            return CalcResult::new_error(Error::NUM, cell, "Division by 0".to_string());
        }
        let result = (fv / pv).powf(1.0 / nper) - 1.0;
        if result.is_infinite() {
            return CalcResult::new_error(Error::NUM, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // SLN(cost, salvage, life)
    // Formula is:
    // $$ \frac{cost-salvage}{life} $$
    pub(crate) fn fn_sln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if life == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        let result = (cost - salvage) / life;

        CalcResult::Number(result)
    }

    // SYD(cost, salvage, life, per)
    // Formula is:
    // $$ \frac{(cost-salvage)*(life-per+1)*2}{life*(life+1)} $$
    pub(crate) fn fn_syd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let per = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if life == 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Division by 0".to_string());
        }
        if per > life || per <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "per should be <= life".to_string());
        }
        let result = ((cost - salvage) * (life - per + 1.0) * 2.0) / (life * (life + 1.0));

        CalcResult::Number(result)
    }

    // NOMINAL(effective_rate, npery)
    // Formula is:
    // $$ n\times\left(\left(1+r\right)^{\frac{1}{n}}-1\right) $$
    // where:
    //   $r$ is the effective interest rate
    //   $n$ is the number of periods per year
    pub(crate) fn fn_nominal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let effect_rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let npery = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if effect_rate <= 0.0 || npery < 1.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let result = ((1.0 + effect_rate).powf(1.0 / npery) - 1.0) * npery;
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // EFFECT(nominal_rate, npery)
    // Formula is:
    // $$ \left(1+\frac{r}{n}\right)^n-1 $$
    // where:
    //   $r$ is the nominal interest rate
    //   $n$ is the number of periods per year
    pub(crate) fn fn_effect(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let nominal_rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let npery = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if nominal_rate <= 0.0 || npery < 1.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let result = (1.0 + nominal_rate / npery).powf(npery) - 1.0;
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // PDURATION(rate, pv, fv)
    // Formula is:
    // $$ \frac{log(fv) - log(pv)}{log(1+r)} $$
    // where:
    //   * $r$ is the interest rate per period
    //   * $pv$ is the present value of the investment
    //   * $fv$ is the desired future value of the investment
    pub(crate) fn fn_pduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let fv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if fv <= 0.0 || pv <= 0.0 || rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let result = (fv.ln() - pv.ln()) / ((1.0 + rate).ln());
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // This next three functions deal with Treasure Bills or T-Bills for short
    // They are zero-coupon that mature in one year or less.
    //  Definitions:
    //    $r$ be the discount rate
    //    $v$ the face value of the Bill
    //    $p$ the price of the Bill
    //    $d_m$ is the number of days from the settlement to maturity
    // Then:
    //   $$ p = v \times\left(1-\frac{d_m}{r}\right) $$
    // If d_m is less than 183 days the he Bond Equivalent Yield (BEY, here $y$) is given by:
    // $$ y = \frac{F - B}{M}\times \frac{365}{d_m} = \frac{365\times r}{360-r\times d_m}
    // If d_m>= 183 days things are a bit more complicated.
    // Let $d_e = d_m - 365/2$ if $d_m <= 365$ or $d_e = 183$ if $d_m = 366$.
    // $$ v = p\times \left(1+\frac{y}{2}\right)\left(1+d_e\times\frac{y}{365}\right) $$
    // Together with the previous relation of $p$ and $v$ gives us a quadratic equation for $y$.

    /// The checked `(settlement, maturity, rate or price)` of TBILLEQ, TBILLPRICE and TBILLYIELD.
    fn tbill_arguments(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64), CalcResult> {
        if args.len() != 3 {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let settlement = self.get_number_no_bools(&args[0], cell)?;
        let maturity = self.get_number_no_bools(&args[1], cell)?;
        let rate = self.get_number_no_bools(&args[2], cell)?;
        let less_than_one_year = is_less_than_one_year(
            self.financial_date(settlement),
            self.financial_date(maturity),
        )
        .map_err(|_| CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()))?;
        if settlement >= maturity {
            let message = "settlement should be < maturity".to_string();
            return Err(CalcResult::new_error(Error::NUM, cell, message));
        }
        if !less_than_one_year {
            let message = "maturity <= settlement + year".to_string();
            return Err(CalcResult::new_error(Error::NUM, cell, message));
        }
        if rate <= 0.0 {
            let message = "discount should be >0".to_string();
            return Err(CalcResult::new_error(Error::NUM, cell, message));
        }
        Ok((settlement, maturity, rate))
    }

    // TBILLEQ(settlement, maturity, discount)
    pub(crate) fn fn_tbilleq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, discount) = match self.tbill_arguments(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let d_m = maturity - settlement;
        let result = if d_m < 183.0 {
            // A zero denominator (price 0) is 0 and a negative one `#NUM!`, not `#DIV/0!`
            // (probes/tbilleq-zero-price A3, probes/tbilleq-nonpositive-price; financial.md
            // "TBILLEQ and TBILLYIELD").
            let denominator = 360.0 - discount * d_m;
            if denominator < 0.0 {
                return CalcResult::new_error(Error::NUM, cell, "Price is negative".to_string());
            }
            if denominator == 0.0 {
                return CalcResult::Number(0.0);
            }
            365.0 * discount / denominator
        } else {
            // The root of (1 + y/2)(1 + (d_m - year/2) y / year) = 100/price in Excel's form and
            // order, fitted bit-exact on ironcalc-calc TBILLs.xlsx D4:D10, D24:D30; a price <= 0 is
            // `#NUM!` (probes/tbilleq-zero-price A1:A2; financial.md "TBILLEQ and TBILLYIELD").
            let year = if d_m == 366.0 { 366.0 } else { 365.0 };
            let price = 100.0 - discount * d_m * 100.0 / 360.0;
            if price <= 0.0 {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Price is not positive".to_string(),
                );
            }
            let t = d_m / year;
            (-t + (t * t - (2.0 * t - 1.0) * (1.0 - 100.0 / price)).sqrt()) / (t - 0.5)
        };
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // TBILLPRICE(settlement, maturity, discount)
    pub(crate) fn fn_tbillprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, discount) = match self.tbill_arguments(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        // days to maturity
        let d_m = maturity - settlement;
        let result = 100.0 * (1.0 - discount * d_m / 360.0);
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() || result < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // TBILLYIELD(settlement, maturity, pr)
    pub(crate) fn fn_tbillyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (settlement, maturity, pr) = match self.tbill_arguments(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let days = maturity - settlement;
        // Excel's order: `(100 - pr)/pr * (360/days)` (ironcalc-calc TBILLs.xlsx R6:R30, bit-exact;
        // docs/excel-semantics/financial.md "TBILLEQ and TBILLYIELD").
        let result = (100.0 - pr) / pr * (360.0 / days);

        CalcResult::Number(result)
    }

    // DOLLARDE(fractional_dollar, fraction)
    pub(crate) fn fn_dollarde(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let fractional_dollar = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let fraction = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if fraction < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "fraction should be >= 1".to_string());
        }
        // `fraction` is truncated to an integer; if that is 0 it is a DIV/0.
        let a_base = fraction.trunc();
        if a_base < 1.0 {
            return CalcResult::new_error(Error::DIV, cell, "fraction should be >= 1".to_string());
        }
        // See LibreOffice/ExcelFinancialFunctions: the fractional part is scaled
        // by 10^ceil(log10(a_base)) and divided by the fraction base.
        let dollar = fractional_dollar.trunc();
        let remainder = fractional_dollar - dollar;
        let digits = 10f64.powf(a_base.log10().ceil());
        CalcResult::Number(remainder * digits / a_base + dollar)
    }

    // DOLLARFR(decimal_dollar, fraction)
    pub(crate) fn fn_dollarfr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let decimal_dollar = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let fraction = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if fraction < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "fraction should be >= 1".to_string());
        }
        // `fraction` is truncated to an integer; if that is 0 it is a DIV/0.
        let a_base = fraction.trunc();
        if a_base < 1.0 {
            return CalcResult::new_error(Error::DIV, cell, "fraction should be >= 1".to_string());
        }
        // Inverse of DOLLARDE: the decimal remainder is expressed in fractional
        // units (a_base) and scaled down by 10^ceil(log10(a_base)).
        let dollar = decimal_dollar.trunc();
        let remainder = decimal_dollar - dollar;
        let digits = 10f64.powf(a_base.log10().ceil());
        CalcResult::Number(remainder * a_base / digits.abs() + dollar)
    }

    // CUMIPMT(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.ceil() as i32,
            Err(s) => return s,
        };
        let end_period = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // 0 at the end of the period, 1 at the beginning of the period
        let period_type = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => {
                if f == 0.0 {
                    false
                } else if f == 1.0 {
                    true
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "invalid period type".to_string(),
                    );
                }
            }
            Err(s) => return s,
        };
        if start_period > end_period {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "start period should come before end period".to_string(),
            );
        }
        if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let mut result = 0.0;
        for period in start_period..=end_period {
            result += match compute_ipmt(rate, period as f64, nper, pv, 0.0, period_type) {
                Ok(f) => f,
                Err(error) => {
                    return CalcResult::Error {
                        error: error.0,
                        origin: cell,
                        message: error.1,
                    }
                }
            }
        }
        CalcResult::Number(result)
    }

    // CUMPRINC(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumprinc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.ceil() as i32,
            Err(s) => return s,
        };
        let end_period = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // 0 at the end of the period, 1 at the beginning of the period
        let period_type = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => {
                if f == 0.0 {
                    false
                } else if f == 1.0 {
                    true
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "invalid period type".to_string(),
                    );
                }
            }
            Err(s) => return s,
        };
        if start_period > end_period {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "start period should come before end period".to_string(),
            );
        }
        if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let mut result = 0.0;
        for period in start_period..=end_period {
            result += match compute_ppmt(rate, period as f64, nper, pv, 0.0, period_type) {
                Ok(f) => f,
                Err(error) => {
                    return CalcResult::Error {
                        error: error.0,
                        origin: cell,
                        message: error.1,
                    }
                }
            }
        }
        CalcResult::Number(result)
    }

    // DDB(cost, salvage, life, period, [factor])
    pub(crate) fn fn_ddb(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let period = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // The rate at which the balance declines.
        let factor = if arg_count > 4 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            // If factor is omitted, it is assumed to be 2 (the double-declining balance method).
            2.0
        };
        if period > life || cost < 0.0 || salvage < 0.0 || period <= 0.0 || factor <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        };
        let result = financial_depreciation::ddb_depreciation(cost, salvage, life, period, factor);
        CalcResult::Number(result)
    }

    // DB(cost, salvage, life, period, [month])
    pub(crate) fn fn_db(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let period = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let month = if arg_count > 4 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f.trunc(),
                Err(s) => return s,
            }
        } else {
            12.0
        };
        if month == 12.0 && period > life
            || (period > life + 1.0)
            || month <= 0.0
            || month > 12.0
            || period <= 0.0
            || cost < 0.0
        {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        };
        if cost == 0.0 {
            return CalcResult::Number(0.0);
        }
        let result = financial_depreciation::db_depreciation(cost, salvage, life, period, month);
        CalcResult::Number(result)
    }

    // ACCRINTM(issue, settlement, rate, [par], [basis])
    pub(crate) fn fn_accrintm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let issue_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let settlement_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let par = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            1000.0
        };
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate must be > 0".to_string());
        }
        if par <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "par must be > 0".to_string());
        }
        if issue_serial >= settlement_serial {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "issue must be before settlement".to_string(),
            );
        }
        // The basis is read as a scalar, as YEARFRAC read it before it lifted (T-1770):
        // ACCRINTM's own lifting is T-1372's.
        let basis = if arg_count > 4 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        match self.get_yearfrac(issue_serial, settlement_serial, basis, cell) {
            Ok(yf) => CalcResult::Number(par * rate * yf),
            Err(error) => error,
        }
    }

    fn get_yearfrac(
        &mut self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = [
            Node::NumberKind(start as f64),
            Node::NumberKind(end as f64),
            Node::NumberKind(basis),
        ];
        match self.fn_yearfrac(&args, cell) {
            CalcResult::Number(yf) => Ok(yf),
            error => Err(error),
        }
    }

    // Day-count factors shared by PRICEMAT and YIELDMAT, mirroring
    // `getMatFactors` in ExcelFinancialFunctions: all three terms share a single
    // denominator `b = DaysInYear(issue, settlement)`, and the settlement→maturity
    // span is `dim - a` (issue-anchored), not an independent year fraction.
    //
    // Returns `(a, dim, dsm)` already divided by `b`, i.e. in year units:
    //   a   = DaysBetween(issue, settlement) / b   (= YEARFRAC(issue, settlement))
    //   dim = DaysBetween(issue, maturity)   / b
    //   dsm = dim - a
    fn get_mat_factors(
        &mut self,
        issue: i64,
        settlement: i64,
        maturity: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64), CalcResult> {
        let a = self.get_yearfrac(issue, settlement, basis, cell)?;
        let dim = if basis == 1.0 {
            // Actual/Actual has a date-dependent DaysInYear, so YEARFRAC(issue,
            // maturity) would use a different denominator. Re-base onto `b` via
            // the actual day counts: dim = dim_days * a / a_days = dim_days / b.
            let a_days = (settlement - issue) as f64;
            let dim_days = (maturity - issue) as f64;
            // Callers guarantee `issue < settlement`, so `a_days > 0`; this guard
            // is defensive only.
            if a_days == 0.0 {
                return Err(CalcResult::new_error(
                    Error::DIV,
                    cell,
                    "Division by zero".to_string(),
                ));
            }
            dim_days * a / a_days
        } else {
            // Constant DaysInYear, so the common denominator is automatic.
            self.get_yearfrac(issue, maturity, basis, cell)?
        };
        Ok((a, dim, dim - a))
    }

    // DISC(settlement, maturity, pr, redemption, [basis])
    pub(crate) fn fn_disc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let pr = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "pr and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if yf == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number((1.0 - pr / redemption) / yf)
    }

    // FVSCHEDULE(principal, schedule)
    pub(crate) fn fn_fvschedule(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let principal = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let schedule = match self.get_array_of_numbers(&args[1], &cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let mut result = principal;
        for rate in schedule {
            result *= 1.0 + rate;
        }
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(result)
    }

    // INTRATE(settlement, maturity, investment, redemption, [basis])
    pub(crate) fn fn_intrate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let investment = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if investment <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "investment and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if yf == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number((redemption / investment - 1.0) / yf)
    }

    // PRICEDISC(settlement, maturity, discount, redemption, [basis])
    pub(crate) fn fn_pricedisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let discount = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if discount <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "discount and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        CalcResult::Number(redemption * (1.0 - discount * yf))
    }

    // PRICEMAT(settlement, maturity, issue, rate, yld, [basis])
    pub(crate) fn fn_pricemat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let issue = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let yld = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if rate < 0.0 || yld < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "rate and yld must be >= 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        if issue >= settlement {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "issue must be before settlement".to_string(),
            );
        }
        let (a, dim, dsm) = match self.get_mat_factors(issue, settlement, maturity, basis, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let denom = 1.0 + yld * dsm;
        if denom == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(100.0 * ((1.0 + rate * dim) / denom - rate * a))
    }

    // RECEIVED(settlement, maturity, investment, discount, [basis])
    pub(crate) fn fn_received(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let investment = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let discount = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if investment <= 0.0 || discount <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "investment and discount must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let denom = 1.0 - discount * yf;
        if denom <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "discount * yearfrac >= 1".to_string());
        }
        CalcResult::Number(investment / denom)
    }

    // YIELDDISC(settlement, maturity, pr, redemption, [basis])
    pub(crate) fn fn_yielddisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let pr = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "pr and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if yf == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number((redemption / pr - 1.0) / yf)
    }

    // YIELDMAT(settlement, maturity, issue, rate, price, [basis])
    pub(crate) fn fn_yieldmat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let issue = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let price = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if price <= 0.0 || rate < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "price must be > 0 and rate >= 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        if issue >= settlement {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "issue must be before settlement".to_string(),
            );
        }
        let (a, dim, dsm) = match self.get_mat_factors(issue, settlement, maturity, basis, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let denom = price / 100.0 + rate * a;
        if denom == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        if dsm == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(((1.0 + rate * dim) / denom - 1.0) / dsm)
    }
}
