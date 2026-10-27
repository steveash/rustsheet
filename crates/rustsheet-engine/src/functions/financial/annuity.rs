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
use crate::x87_exp::{dd_add, dd_div, dd_mul, excel_exp, excel_expm1, two_sum};
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;
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
pub(super) fn annuity_power(rate: f64, nper: f64) -> Result<f64, Error> {
    power(extended_sum(1.0, rate), nper)
}

/// The annuity factor `k*((P-1)/r)`, `k = 1+r*type`, evaluated in doubles in exactly this
/// parenthesization: `(pmt*k)*((P-1)/r)` instead misses 38 of the 308 PV cells (financial.md).
pub(super) fn annuity_factor(rate: f64, p: f64, period_start: bool) -> f64 {
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

/// Below this |rate| PPMT's `ln(1+rate)` is the atanh series, from it on Excel's LN of `1+rate`:
/// probes-numeric/ppmt-log-rate-sweep has the series through 0.37 and the log from 0.38 on (0.375
/// is a round point between them, not pinned).
const GROWTH_LOG_SERIES_TO: f64 = 0.375;

/// At or below this rate PPMT keeps the exact form, which matches `PPMT(-0.5,1,277,73630,609)`
/// (-37119.5) where the discounted one is 25 ulp off; other -0.5 cells fit neither (T-2588).
const DISCOUNTED_RATE_FROM: f64 = -0.375;

/// The series stops once a term leaves the sum unchanged; for |rate| < 0.375 each term is under
/// 0.025 of the one before, so about 11 terms. This only bounds the loop.
const GROWTH_LOG_TERMS: u32 = 64;

/// The `L = ln(1+rate)` of Excel's PPMT discount: for |rate| < [`GROWTH_LOG_SERIES_TO`],
/// `2*(s + s^3/3 + s^5/5 + ...)` with `s = rate/(2+rate)` a double and `s^2`, the running power,
/// each term and each partial sum kept in an x87 register, until a term leaves the sum unchanged;
/// above it, Excel's LN of the double `1+rate`. Either way L is often an ulp or two from the
/// correctly rounded `ln(1+rate)` (-1 at 0.1, +1 at 0.05, +2 at 0.6), and the discount follows it
/// (docs/excel-semantics/financial.md "PPMT discounts with an atanh-series log", T-2242). ATANH's
/// own series (`hyperbolic::atanh_series`) stores each term and partial sum to a double instead,
/// which misses this L, so the two loops stay separate.
fn growth_log(rate: f64) -> f64 {
    if rate.abs() >= GROWTH_LOG_SERIES_TO {
        return excel_ln(1.0 + rate);
    }
    let s = Register::load(rate / (2.0 + rate));
    let square = s.mul(s);
    let mut power = s;
    let mut sum = s;
    for k in 1..=GROWTH_LOG_TERMS {
        power = power.mul(square);
        let next = sum.add(power.div(Register::load(f64::from(2 * k + 1))));
        if next == sum {
            break;
        }
        sum = next;
    }
    2.0 * sum.store()
}

/// Below this |nper*L| PPMT keeps the exact form: L's ulp-level error moves the power by under an
/// ulp there, and the exact form matches Excel's last bit where the discounted one does not
/// (`PPMT(1E-10,2,124,191996,822,1)` is -1554.9838614045912, gen-m9 5ded7594e597e31e A49).
const SHORT_GROWTH: f64 = 1e-6;

/// PPMT (financial.md "PPMT discounts with an atanh-series log"): with `Q(x) = EXP(x*L)` (Excel's
/// EXP, [`growth_log`]'s L), `-(pv+fv)*rate*Q(period-1-nper)/(1-Q(-nper))`, the denominator
/// Excel's expm1 of `-nper*L` and the whole divided by `1+rate` for type 1, rounded once. It is the
/// exact `-(pv+fv)*rate*P_e/(P-1)` with Excel's own power, several ulp off the exact one for long
/// terms (`PPMT(0.1,1,360,10000)` is -1.2549699967849914E-12, the exact value
/// -1.254969996784983E-12). `None` for a rate at or below [`DISCOUNTED_RATE_FROM`], where Excel is
/// the exact form instead (`PPMT(-0.5,1,277,73630,609)` is -37119.5, gen-m9 762cd0e4ec9a7aa9 A5),
/// below [`SHORT_GROWTH`], and where a factor or the result is not finite and normal.
///
/// `past_range` is the form where the exact one declines, `(1+rate)^nper` past about 1E+270: a
/// discount below the normal range is then 0 and so is PPMT, as Excel's is (`PPMT(42,10,303,8292,
/// 1,1)` and `PPMT(10,1,296,1)` are 0, the discount 11^-296 subnormal and PPMT itself not;
/// `PPMT(10,11,304,44626,826)` is -3.0768304255991313E-301; gen-m9 31474c20fc1dafd6 A235,
/// probes/fin-ipmt-ppmt-huge-rate B4, gen-m9 6376e27be7a05871 A32), even where `(pv+fv)*rate`
/// overflows (`PPMT(1E+300,1.5,180,194733,-1E+300)` is 0, B13).
fn discounted_principal(
    rate: f64,
    nper: f64,
    period: f64,
    (pv, fv): (f64, f64),
    period_start: bool,
    past_range: bool,
) -> Option<f64> {
    if rate <= DISCOUNTED_RATE_FROM {
        return None;
    }
    let log = growth_log(rate);
    if (nper * log).abs() < SHORT_GROWTH {
        return None;
    }
    let discount = excel_exp((period - 1.0 - nper) * log);
    let periods = -excel_expm1(-nper * log);
    if !periods.is_normal() {
        return None;
    }
    if !discount.is_normal() {
        return past_range.then_some(0.0);
    }
    let owed = dd_mul(dd_mul(two_sum(pv, fv), (rate, 0.0)), (discount, 0.0));
    let mut principal = negated(dd_div(owed, (periods, 0.0)));
    if period_start {
        principal = dd_div(principal, two_sum(1.0, rate));
    }
    let principal = principal.0;
    let usable = if past_range {
        principal.is_finite()
    } else {
        principal.is_normal()
    };
    usable.then_some(principal)
}

/// `(1+rate)^x` for `x <= 0` as [`accurate_growth`] has it, and 0 where that declines for a
/// positive rate (the power is then below 1E-270, which a double near 1 cannot see).
fn accurate_discount(rate: f64, x: f64) -> Option<Pair> {
    match accurate_growth(rate, x) {
        None if rate > 0.0 && x < 0.0 => Some((0.0, 0.0)),
        growth => growth,
    }
}

/// IPMT as `-rate*B` (divided by `1+rate` for type 1) on the balance after period-1 periods,
/// `B = (pv+fv)*Y - fv` with `Y = (1-(1+rate)^(period-1-nper))/(1-(1+rate)^-nper)`, and whether
/// that balance cancels (`|B| < |fv|`). Where it cancels Excel has `B` in doubles on the rounded
/// `Y`, `(pv+fv)*Y - fv`, so its IPMT carries `fv`'s rounding: `IPMT(0.5,2,124,0,315)` is 0,
/// `IPMT(50,2,10,0,315)` 5.6843418860808015E-12 = 25*2^-42, `IPMT(1E+10,10,10,0,315)`
/// 315.00007935392205 (probes/fin-ipmt-ppmt-huge-rate A27, A291, A403); otherwise `B` is exact
/// and the caller keeps [`accurate_period_split`]'s interest where it has one. Needing no
/// `(1+rate)^nper`, it has the huge rates' `-pv*rate` (`IPMT(1E+300,3,2000,452017,905)` is
/// -4.52017E+305, gen-m9 abb7d565af89b849 A112). docs/excel-semantics/financial.md "IPMT
/// cancels on fv". `None` where a power declines or the result is not finite.
fn balance_interest(
    rate: f64,
    nper: f64,
    period: f64,
    (pv, fv): (f64, f64),
    period_start: bool,
) -> Option<(f64, bool)> {
    let one_minus = |x: Pair| dd_add((1.0, 0.0), negated(x));
    let remaining = one_minus(accurate_discount(rate, period - 1.0 - nper)?);
    let periods = one_minus(accurate_discount(rate, -nper)?);
    if periods.0 == 0.0 {
        return None;
    }
    let share = dd_div(remaining, periods);
    let exact = dd_add(dd_mul(two_sum(pv, fv), share), (-fv, 0.0));
    let cancels = exact.0.abs() < fv.abs();
    let balance = if cancels {
        ((pv + fv) * share.0 - fv, 0.0)
    } else {
        exact
    };
    let mut interest = negated(dd_mul(balance, (rate, 0.0)));
    if period_start {
        interest = dd_div(interest, two_sum(1.0, rate));
    }
    interest.0.is_finite().then_some((interest.0, cancels))
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

/// `(IPMT, PPMT)` of one period, for IPMT, PPMT, CUMIPMT and CUMPRINC, each with its own error:
/// past PMT's overflow at a positive rate IPMT and PPMT keep their values (`PPMT(1E+307,7,292,
/// 176644,89)` is 0, gen-m9 3aa168259335be54 A340), and IPMT is 0 in period 1 of type 1 even where
/// PMT is `#NUM!` (`IPMT(-0.5,1,2000,1,0,1)`, probes/fin-ipmt-ppmt-huge-rate B1).
pub(super) fn period_payments(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> (Result<f64, Error>, Result<f64, Error>) {
    if period < 1.0 || period >= period_count + 1.0 {
        return (Err(Error::NUM), Err(Error::NUM));
    }
    let payment = match payment_pair(
        rate,
        period_count,
        present_value,
        future_value,
        period_start,
    ) {
        Ok(payment) => Some(payment),
        // For a positive rate PMT errs only when its value overflows (`(1+rate)^nper` cannot
        // underflow there), which IPMT and PPMT do not need.
        Err(_) if rate > 0.0 => None,
        Err(error) => {
            let interest = if period == 1.0 && period_start {
                Ok(0.0)
            } else {
                Err(error.clone())
            };
            return (interest, Err(error));
        }
    };
    if period == 1.0 && period_start {
        return (Ok(0.0), payment.map(|p| p.0).ok_or(Error::NUM));
    }
    let values = (present_value, future_value);
    let (mut interest, mut principal) = (None, None);
    if rate != 0.0 {
        let e = if period_start {
            period - 2.0
        } else {
            period - 1.0
        };
        let split = payment.and_then(|payment| {
            accurate_period_split(rate, period_count, e, values, payment, period_start)
        });
        principal = discounted_principal(rate, period_count, period, values, period_start, false)
            .or(split.map(|(_, principal)| principal.0))
            .or_else(|| {
                discounted_principal(rate, period_count, period, values, period_start, true)
            });
        // A fractional nper past the range keeps the FV recurrence below: it has Excel's last
        // bit on the one such cell probed, where the exact balance is an ulp off (gen-m9
        // 0a957119475760c5 A153, nper about 1.85E+7; TODO(T-2665)).
        let whole_term = period_count.fract() == 0.0;
        interest = match balance_interest(rate, period_count, period, values, period_start) {
            Some((interest, true)) => Some(interest),
            balance => split
                .map(|(interest, _)| interest.0)
                .or(balance.filter(|_| whole_term).map(|(interest, _)| interest)),
        };
    }
    if let (Some(interest), Some(principal)) = (interest, principal) {
        return (Ok(interest), Ok(principal));
    }
    let fallback = payment.ok_or(Error::NUM).and_then(|payment| {
        double_period_payments(rate, period, present_value, payment.0, period_start)
    });
    match fallback {
        Ok((fallback_interest, fallback_principal)) => (
            Ok(interest.unwrap_or(fallback_interest)),
            Ok(principal.unwrap_or(fallback_principal)),
        ),
        Err(error) => (
            interest.ok_or_else(|| error.clone()),
            principal.ok_or(error),
        ),
    }
}

/// IPMT and PPMT from FV's double recurrence on the rounded PMT, where the closed forms decline:
/// type 0 `IPMT = FV(rate, period-1, pmt, pv)*rate`, type 1 `(FV(rate, period-2, pmt, pv, 1) -
/// pmt)*rate`, and `PPMT = PMT - IPMT`.
fn double_period_payments(
    rate: f64,
    period: f64,
    present_value: f64,
    payment: f64,
    period_start: bool,
) -> Result<(f64, f64), Error> {
    let (e, c) = if period_start {
        (period - 2.0, -payment)
    } else {
        (period - 1.0, 0.0)
    };
    let fv = compute_future_value(rate, e, payment, present_value, period_start)?;
    let interest = (fv + c) * rate;
    Ok((interest, payment - interest))
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
        // PV = -(fv + pmt*(k*((P-1)/r)))/P, every operation in doubles (financial.md). P is Excel's
        // `^`, with its errors: a rate of -1 with nper 0 is 0^0, `#NUM!` (`PV(-1,0,-713)`), and a P
        // of 0 (a rate of -1, or one so near it that P underflows) is the division's `#DIV/0!`
        // (`PV(-1+2^-52,165,-978,224,0)`; probes/t2475-stat-subexpressions A39, A136).
        let p = annuity_power(rate, period_count).map_err(formula_error(cell))?;
        if p == 0.0 {
            return Err(div_error(cell));
        }
        let result = -(future_value + payment * annuity_factor(rate, p, period_start)) / p;
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `RATE(nper, pmt, pv, [fv], [type], [guess])`. The secant stops after `RATE_MAX_STEPS`
    /// (200) steps, so the table charges each element the time of a run to that cap
    /// (`RATE_WEIGHT`, `KernelWork::Weighted`, SPEC-0011 security rule).
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
        // A rate of -1 is `#NUM!`: ln(1+rate) is LN(0) (`NPER(-1,-31,1)`, gen-m9/3c06f9298f65a3ba A50).
        if rate <= -1.0 {
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
        // A rate so small that 1+rate is 1 divides by ln 1 = 0: `#DIV/0!` (`NPER(1E-300,-236,2766,945)`,
        // gen-m9/3c06f9298f65a3ba A148; probes/t2475-stat-subexpressions A92).
        let growth_log = (1.0 + rate).ln();
        if growth_log == 0.0 {
            return Err(div_error(cell));
        }
        // The final division is Excel's `/`, rounded twice (A9 `NPER(0.2,-1000,-1000,20000)`)
        let result = extended_quotient(ratio.ln(), growth_log);
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
                period_payments(a.0, a.1, a.2, a.3, a.4, a.5)
                    .0
                    .map_err(formula_error(cell))
            }))
        })
    }

    /// `PPMT(rate, per, nper, pv, [fv], [type])`.
    pub(crate) fn fn_ppmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Ppmt, args, cell, &mut |model, values| {
            number_or_error(model.period_payment_args(values, cell).and_then(|a| {
                period_payments(a.0, a.1, a.2, a.3, a.4, a.5)
                    .1
                    .map_err(formula_error(cell))
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

#[cfg(test)]
mod tests {
    use super::{growth_log, period_payments};
    use crate::expressions::token::Error;

    /// IPMT and PPMT past `(1+rate)^nper`'s range and where IPMT's balance cancels on fv
    /// (probes/fin-ipmt-ppmt-huge-rate, gen-m9; financial.md "IPMT cancels on fv").
    #[test]
    fn period_payments_at_huge_rates() {
        let ipmt = |r, per, n, pv, fv, t| period_payments(r, per, n, pv, fv, t).0;
        let ppmt = |r, per, n, pv, fv, t| period_payments(r, per, n, pv, fv, t).1;
        assert_eq!(ppmt(42.0, 10.0, 303.0, 8292.0, 1.0, true), Ok(0.0));
        assert_eq!(ppmt(10.0, 1.0, 296.0, 1.0, 0.0, false), Ok(0.0));
        assert_eq!(
            ppmt(10.0, 11.0, 304.0, 44626.0, 826.0, false),
            Ok(-3.076_830_425_599_131_3e-301)
        );
        assert_eq!(ppmt(1e300, 1.5, 180.0, 194_733.0, -1e300, false), Ok(0.0));
        assert_eq!(ppmt(1e307, 7.0, 292.0, 176_644.0, 89.0, false), Ok(0.0));
        assert_eq!(ppmt(2.0, 1100.0, 1100.0, 1.0, 5.0, false), Ok(-4.0));
        assert_eq!(
            ipmt(1e300, 3.0, 2000.0, 452_017.0, 905.0, false),
            Ok(-4.520_170_000_000_000_4e305)
        );
        assert_eq!(
            ipmt(1e10, 8.0, 226.0, 439_138.0, 43.0, false),
            Ok(-4.391_38e15)
        );
        assert_eq!(ipmt(2.0, 1100.0, 1100.0, 1.0, 5.0, false), Ok(2.0));
        assert_eq!(ipmt(0.5, 2.0, 124.0, 0.0, 315.0, false), Ok(0.0));
        assert_eq!(
            ipmt(50.0, 2.0, 10.0, 0.0, 315.0, false),
            Ok(25.0 * 2f64.powi(-42))
        );
        assert_eq!(
            ipmt(1e10, 10.0, 10.0, 0.0, 315.0, false),
            Ok(315.000_079_353_922_05)
        );
        // Period 1 of type 1 is 0 even where PMT is #NUM!; PPMT is PMT there.
        assert_eq!(ipmt(-0.5, 1.0, 2000.0, 1.0, 0.0, true), Ok(0.0));
        assert_eq!(ppmt(-0.5, 1.0, 2000.0, 1.0, 0.0, true), Err(Error::NUM));
    }

    /// PPMT's log is the series below 0.375 and Excel's LN above it, an ulp or two from
    /// `ln_1p` in places (probes-numeric/ppmt-first-period-sweep, ppmt-log-rate-sweep).
    #[test]
    fn growth_log_is_excels() {
        assert_eq!(growth_log(0.1), 0.095_310_179_804_324_85); // ln_1p: ...87
        assert_eq!(growth_log(0.05), 0.048_790_164_169_432_014); // ln_1p: ...01
        assert_eq!(growth_log(-0.02), -0.020_202_707_317_519_45);
        assert_eq!(growth_log(0.37), 0.314_810_739_840_033_5);
        assert_eq!(growth_log(0.38), 0.322_083_499_169_113_2); // LN(1.38)
        assert_eq!(growth_log(0.6), 0.470_003_629_245_735_63);
    }
}
