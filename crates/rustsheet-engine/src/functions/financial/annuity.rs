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
use crate::functions::lift::{div_error, num_error};
use crate::x87_exp::{dd_add, dd_div, dd_mul, two_sum};
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
fn annuity_power(rate: f64, nper: f64) -> Result<f64, Error> {
    power(extended_sum(1.0, rate), nper)
}

/// The annuity factor `k*((P-1)/r)`, `k = 1+r*type`, evaluated in doubles in exactly this
/// parenthesization: `(pmt*k)*((P-1)/r)` instead misses 38 of the 308 PV cells (financial.md).
fn annuity_factor(rate: f64, p: f64, period_start: bool) -> f64 {
    let k = if period_start { 1.0 + rate } else { 1.0 };
    k * ((p - 1.0) / rate)
}

/// PMT's `P = (1+rate)^nper` (1 for no periods) for its overflow guards and fallback forms:
/// Excel's `^` on Excel's `1+rate`, infinite where `^` errs (it errs only past the double range
/// for a positive base).
fn payment_growth(rate: f64, nper: f64) -> f64 {
    if nper == 0.0 {
        1.0
    } else {
        annuity_power(rate, nper).unwrap_or(f64::INFINITY)
    }
}

/// A double-double, `hi + lo` with `hi` the rounded sum (as the `x87_exp` helpers return it).
type Pair = (f64, f64);

/// The largest |exponent| [`accurate_growth`] takes: 32 squarings.
const ACCURATE_EXPONENT_LIMIT: f64 = 4_294_967_296.0; // 2^32

/// |P| bounds for [`accurate_growth`]: far enough inside the double range that the products and
/// quotients PMT, IPMT and PPMT build on P keep their low parts normal.
const ACCURATE_GROWTH_MIN: f64 = 1e-270;
const ACCURATE_GROWTH_MAX: f64 = 1e270;

/// `(1+rate)^n` in double-double arithmetic, for |n| up to [`ACCURATE_EXPONENT_LIMIT`]: the
/// integer part of `n` by right-to-left square-and-multiply from the exact `1+rate` (relative
/// error about n·2^-104), times `1 + expm1(f·ln(1+rate))` for its fraction `f`, whose error is
/// a few ulp of the expm1 term, not of 1. `None` past the limit or when P leaves the bounds above.
/// Excel's PMT, IPMT and PPMT are within a few ulp of the exact payment on the double arguments,
/// for integer and fractional periods alike, which the double forms miss for long terms and
/// fractional periods (financial.md "PMT, IPMT and PPMT are the exact payment", T-2130).
fn accurate_growth(rate: f64, n: f64) -> Option<Pair> {
    if n.is_nan() || n.abs() > ACCURATE_EXPONENT_LIMIT {
        return None;
    }
    let whole = n.trunc();
    let mut base = two_sum(1.0, rate);
    let mut result = (1.0, 0.0);
    let mut exponent = whole.abs();
    while exponent > 0.0 {
        if exponent % 2.0 == 1.0 {
            result = dd_mul(result, base);
        }
        exponent = (exponent / 2.0).floor();
        if exponent > 0.0 {
            base = dd_mul(base, base);
        }
    }
    if whole < 0.0 {
        result = dd_div((1.0, 0.0), result);
    }
    let fraction = n - whole;
    if fraction != 0.0 {
        result = dd_mul(result, two_sum(1.0, (fraction * rate.ln_1p()).exp_m1()));
    }
    (ACCURATE_GROWTH_MIN..=ACCURATE_GROWTH_MAX)
        .contains(&result.0.abs())
        .then_some(result)
}

/// `k = 1+rate*type` as a double-double.
fn accurate_type_factor(rate: f64, period_start: bool) -> Pair {
    if period_start {
        two_sum(1.0, rate)
    } else {
        (1.0, 0.0)
    }
}

fn negated((hi, lo): Pair) -> Pair {
    (-hi, -lo)
}

/// PMT `-(fv + pv*P)*rate / (k*(P-1))` in double-double arithmetic, or `None` where
/// [`accurate_growth`] declines or the payment is not finite (the double forms then decide).
fn accurate_payment(rate: f64, nper: f64, pv: f64, fv: f64, period_start: bool) -> Option<Pair> {
    let p = accurate_growth(rate, nper)?;
    let numerator = dd_mul(dd_add((fv, 0.0), dd_mul((pv, 0.0), p)), (rate, 0.0));
    let denominator = dd_mul(
        accurate_type_factor(rate, period_start),
        dd_add(p, (-1.0, 0.0)),
    );
    if denominator.0 == 0.0 {
        return None;
    }
    let payment = negated(dd_div(numerator, denominator));
    payment.0.is_finite().then_some(payment)
}

/// IPMT and PPMT of the period after `e` whole periods (e = period-1, or period-2 for type 1) as
/// double-doubles, from closed forms that do not cancel: with the balance
/// `B = (pv*P_e*(P_(n-e)-1) - fv*(P_e-1))/(P-1)`, IPMT is `-rate*B` (`-rate*(B+pmt)` for type 1)
/// and PPMT `-(pv+fv)*rate*P_e/(P-1)`. IPMT's FV recurrence and PMT - IPMT are the same values,
/// but they cancel to nothing for long terms even in double-double (`PPMT(0.2,1,360,10000)` is
/// -6.248581209907205E-26, `PPMT(0.3,360,360,10000,-500)` -2192.3076923076924;
/// probes-numeric/m9-fin-pmt A82, A504). `None` where [`accurate_growth`] declines.
fn accurate_period_split(
    rate: f64,
    nper: f64,
    e: f64,
    (pv, fv): (f64, f64),
    payment: Pair,
    period_start: bool,
) -> Option<(Pair, Pair)> {
    let p = accurate_growth(rate, nper)?;
    let p_e = accurate_growth(rate, e)?;
    let p_rest = accurate_growth(rate, nper - e)?;
    let one_less = |x: Pair| dd_add(x, (-1.0, 0.0));
    let periods = one_less(p);
    if periods.0 == 0.0 {
        return None;
    }
    let balance = dd_div(
        dd_add(
            dd_mul(dd_mul((pv, 0.0), p_e), one_less(p_rest)),
            negated(dd_mul((fv, 0.0), one_less(p_e))),
        ),
        periods,
    );
    let owed = if period_start {
        dd_add(balance, payment)
    } else {
        balance
    };
    let interest = negated(dd_mul(owed, (rate, 0.0)));
    let principal = negated(dd_div(
        dd_mul(dd_mul(two_sum(pv, fv), (rate, 0.0)), p_e),
        periods,
    ));
    (interest.0.is_finite() && principal.0.is_finite()).then_some((interest, principal))
}

/// PMT's fitted quotient for `P = rate_nper`; not finite when `pv*P` overflows.
fn fitted_payment(rate: f64, rate_nper: f64, pv: f64, fv: f64, period_start: bool) -> f64 {
    if period_start {
        // type = 1
        (fv + pv * rate_nper) * rate / ((1.0 + rate) * (1.0 - rate_nper))
    } else {
        (fv * rate + pv * rate * rate_nper) / (1.0 - rate_nper)
    }
}

/// Whether PMT's fitted quotient overflows for a positive `rate` and `fv` 0, where
/// [`compute_payment`] falls back to its divided-through form. CUMIPMT and CUMPRINC then take
/// their closed form instead of summing IPMT/PPMT, as Excel does (probes/fin-cumipmt-period-bound
/// A13-A19).
pub(super) fn payment_overflows(rate: f64, nper: f64, pv: f64, period_start: bool) -> bool {
    !fitted_payment(rate, payment_growth(rate, nper), pv, 0.0, period_start).is_finite()
}

fn compute_payment(
    rate: f64,
    nper: f64,
    pv: f64,
    fv: f64,
    period_start: bool,
) -> Result<f64, Error> {
    payment_pair(rate, nper, pv, fv, period_start).map(|payment| payment.0)
}

/// PMT as a double-double: [`accurate_payment`] where it applies, else a double form (low part 0).
fn payment_pair(rate: f64, nper: f64, pv: f64, fv: f64, period_start: bool) -> Result<Pair, Error> {
    if rate == 0.0 {
        if nper == 0.0 {
            return Err(Error::NUM);
        }
        return Ok((-(pv + fv) / nper, 0.0));
    }
    if rate <= -1.0 {
        return Err(Error::NUM);
    };
    let rate_nper = payment_growth(rate, nper);
    // A `(1+rate)^nper` so small that its reciprocal overflows (0 or 2^-1070) is #NUM!, whatever
    // fv and type are, for PMT and so IPMT, PPMT and CUMIPMT (`PMT(-0.5,2000,1)`,
    // `PMT(-0.5,1070,0,5)`; probes/pmt-overflow A14, A15, E1:E8, excel-graph, T-1405).
    let inverse = 1.0 / rate_nper;
    if !inverse.is_finite() {
        return Err(Error::NUM);
    }
    if let Some(payment) = accurate_payment(rate, nper, pv, fv, period_start) {
        return Ok(payment);
    }
    let result = fitted_payment(rate, rate_nper, pv, fv, period_start);
    if result.is_finite() {
        return Ok((result, 0.0));
    }
    // When `pv*P` overflows (P itself infinite or not), Excel gives the same quotient divided
    // through by P, whose limit is `-pv*rate` (`/(1+rate)` for type 1) and in which fv vanishes:
    // `PMT(100,184,439103)` is -43910300 and `PMT(100,153,1)` is -100 (probes/pmt-overflow,
    // excel-graph, T-1405). The fitted form above stays first, so finite results keep its digits.
    let result = if period_start {
        (fv * inverse + pv) * rate / ((1.0 + rate) * (inverse - 1.0))
    } else {
        (fv * rate * inverse + pv * rate) / (inverse - 1.0)
    };
    if result.is_finite() {
        Ok((result, 0.0))
    } else {
        Err(Error::NUM)
    }
}

fn compute_future_value(
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    period_start: bool,
) -> Result<f64, Error> {
    if rate == 0.0 {
        return Ok(-pv - pmt * nper);
    }
    if rate == -1.0 && nper < 0.0 {
        return Err(Error::DIV);
    }

    // FV = -(pv*P + pmt*(k*((P-1)/r))), every operation in doubles (financial.md)
    let p = annuity_power(rate, nper)?;
    let fv = -(pv * p + pmt * annuity_factor(rate, p, period_start));
    if fv.is_nan() {
        return Err(Error::NUM);
    }
    if !fv.is_finite() {
        return Err(Error::DIV);
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
) -> Result<f64, Error> {
    period_payments(
        rate,
        period,
        period_count,
        present_value,
        future_value,
        period_start,
    )
    .map(|(interest, _)| interest)
}

pub(super) fn compute_ppmt(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> Result<f64, Error> {
    period_payments(
        rate,
        period,
        period_count,
        present_value,
        future_value,
        period_start,
    )
    .map(|(_, principal)| principal)
}

/// IPMT and PPMT of one period.
fn period_payments(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> Result<(f64, f64), Error> {
    // http://www.staff.city.ac.uk/o.s.kerr/CompMaths/WSheet4.pdf
    // https://www.experts-exchange.com/articles/1948/A-Guide-to-the-PMT-FV-IPMT-and-PPMT-Functions.html
    // type = 0 (end of period)
    // impt = -[(1+rate)^(period-1)*(pv*rate+pmt)-pmt]
    // ipmt = FV(rate, period-1, payment, pv, type) * rate
    // type = 1 (beginning of period)
    // ipmt = (FV(rate, period-2, payment, pv, type) - payment) * rate
    let payment = payment_pair(
        rate,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    if period < 1.0 || period >= period_count + 1.0 {
        return Err(Error::NUM);
    }
    if period == 1.0 && period_start {
        return Ok((0.0, payment.0));
    }
    let p = if period_start {
        period - 2.0
    } else {
        period - 1.0
    };
    if rate != 0.0 {
        if let Some((interest, principal)) = accurate_period_split(
            rate,
            period_count,
            p,
            (present_value, future_value),
            payment,
            period_start,
        ) {
            return Ok((interest.0, principal.0));
        }
    }
    let c = if period_start { -payment.0 } else { 0.0 };
    let fv = compute_future_value(rate, p, payment.0, present_value, period_start)?;
    let interest = (fv + c) * rate;
    Ok((interest, payment.0 - interest))
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
        compute_future_value(rate, nper, pmt, pv, period_start).map_err(formula_error(cell))
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

    /// `ISPMT(rate, per, nper, pv)`: `(per - nper) * pv * rate / nper`, in Excel's order (`ispmt_of`).
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
        // Excel's operation order, `(per - nper) * pv * rate / nper`, left to right: it alone fits
        // all 24 cells of probes/fin-ispmt-order (inputs where the candidate orders differ in the
        // last bit) and ISPMT.xlsx E2 (`ISPMT(0.2,1,3,100)` = -13.333333333333334, where
        // `pv * rate * (per / nper - 1)` gives ...336; financial.md "ISPMT's operation order").
        Ok((per - nper) * pv * rate / nper)
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
        // Excel computes `(fv/pv)^(1/nper)-1` with its own `/`, `^` and `-`:
        // `RRI(10,1,200)` is 0.6986464646342472 like `(200/1)^(1/10)-1`, where libm `powf` is
        // 2 ulps higher (probes/fin-rri A14, fin-mduration-rri-fit A37-A60, T-1975). Unlike `^`, a
        // negative ratio is always #NUM!: `RRI(1,-100,200)` is #NUM! where `(200/-100)^1-1` is -3,
        // and so are odd roots (probes/fin-rri-vdb-edges A1-A16).
        let ratio = extended_quotient(fv, pv);
        if ratio < 0.0 {
            return Err(num_error(cell));
        }
        let growth = power(ratio, extended_quotient(1.0, nper)).map_err(|_| num_error(cell))?;
        let result = extended_sum(growth, -1.0);
        if !result.is_finite() {
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
