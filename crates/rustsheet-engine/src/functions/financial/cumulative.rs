//! CUMIPMT and CUMPRINC: the interest and the principal paid from `start_period` to
//! `end_period`. They are Analysis `ToolPak` functions, so every argument lifts over an array and a
//! multi-cell range is `#VALUE!` in it, with no intersection (`ArgKind::LiftArrays`, SPEC-0011;
//! probes/fin-argument-modes H72-H87, probes/fin-remaining-later-argument-ranges H65-H84). Excel
//! sums any number of periods in constant time (`CUMIPMT(0.01,1E308,1,1,1E308,0)` is -1E306,
//! probes/fin-cumipmt-period-bound A26), so the kernel does too: a short range is a per-period
//! sum (type 0's own forms, type 1's interest the IPMT sum), anything else a closed form
//! (docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods"). Moved out of
//! `financial/mod.rs` (T-1845).

use super::annuity::{payment_overflows, period_payments};
use super::number_or_error;
use crate::functions::lift::num_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// The most periods summed one by one. Up to here type 1's interest is main's per-period IPMT
/// sum, which matches Excel-cached cells the closed form misses by an ulp or two
/// (`ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx` G3, G9, G15, G26: 1 and 8 periods). Past it the
/// sum drifts from Excel (83 ulps over 1,024 periods, probes/fin-cumipmt-period-bound A11, and no
/// correct digit over 10^5 periods of type 1, A5), while the closed form is within 6 ulps there
/// (48 to 64 near period 1E15). The bound also caps the kernel's work: at most 32 IPMT steps an
/// element (`CUMULATIVE_WEIGHT` in functions/lift/kinds/financial.rs), and the type 0 forms take
/// at most 32 `exp` steps. Type 1's principal is always the closed form (T-1890).
const CUMULATIVE_LOOP_PERIODS: f64 = 32.0;

/// 2^53: below it every period is an exact integer, so `start + offset` is a distinct period.
const EXACT_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;

/// Which sum the kernel returns.
#[derive(Clone, Copy)]
enum Paid {
    Interest,
    Principal,
}

impl Model {
    /// `CUMIPMT(rate, nper, pv, start_period, end_period, type)`.
    pub(crate) fn fn_cumipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Cumipmt, args, cell, &mut |model, values| {
            number_or_error(model.cumulative_of(values, Paid::Interest, cell))
        })
    }

    /// `CUMPRINC(rate, nper, pv, start_period, end_period, type)`.
    pub(crate) fn fn_cumprinc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Cumprinc, args, cell, &mut |model, values| {
            number_or_error(model.cumulative_of(values, Paid::Principal, cell))
        })
    }

    /// Reads every argument in order (a boolean is `#VALUE!`, `CUMPRINC_CUMIPMT.xlsx` G27, G28),
    /// then checks them: the start period rounds up and the end period down (G6, G7), the type is
    /// 0 or 1 (G23, G24), and the rate, nper and pv are positive.
    fn cumulative_of(
        &self,
        values: &[CalcResult],
        paid: Paid,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [rate, nper, pv, start, end, period_type] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_no_bools_ref(rate, cell)?;
        let nper = self.cast_number_no_bools_ref(nper, cell)?;
        let pv = self.cast_number_no_bools_ref(pv, cell)?;
        let start = self.cast_number_no_bools_ref(start, cell)?.ceil();
        let end = self.cast_number_no_bools_ref(end, cell)?.trunc();
        // 0 at the end of the period, 1 at the beginning of the period
        let period_start = match self.cast_number_no_bools_ref(period_type, cell)? {
            0.0 => false,
            1.0 => true,
            _ => return Err(num_error(cell)),
        };
        // An end period past nper is #NUM!, also below nper + 1: `CUMIPMT(0.01,100.5,1000,1,101,0)`
        // (probes/fin-cumipmt-period-bound A20-A22).
        // Written as negated `>=`/`<=` so a NaN argument fails a check too.
        let valid =
            start <= end && rate > 0.0 && nper > 0.0 && pv > 0.0 && start >= 1.0 && end <= nper;
        if !valid {
            return Err(num_error(cell));
        }
        let periods = Periods {
            rate,
            nper,
            pv,
            start,
            end,
            period_start,
        };
        // Per-period sums only over a short range, and only where every period is an exact
        // integer: past 2^53 `period + 1` is `period`, and the closed form takes over. Type 0 has
        // its own per-period forms, within 4 ulps of Excel on every probed cell where the closed
        // form and the IPMT sum were up to 343 ulps off (T-2192). Type 1's interest is the IPMT sum, which is
        // bit-exact on CUMPRINC_CUMIPMT.xlsx G3, G4, G9, G15 where those forms are not, and its
        // principal the closed form: summing PPMT loses a small principal's digits to the payment
        // it is taken from (H5 had 8 right digits, H7 7; T-1890), and the per-period `exp` form misses H22
        // and H25 by an ulp or two.
        let short = end - start < CUMULATIVE_LOOP_PERIODS && end < EXACT_INTEGER_LIMIT;
        if short && !period_start {
            let sum = match paid {
                Paid::Interest => periods.balance_interest(),
                Paid::Principal => periods.discounted_principal(),
            };
            if let Some(sum) = sum {
                return Ok(sum);
            }
        }
        if short && matches!(paid, Paid::Interest) {
            if let Some(sum) = periods.summed_interest() {
                return Ok(sum);
            }
        }
        periods.closed_form(paid).ok_or_else(|| num_error(cell))
    }
}

/// Validated arguments: `1 <= start <= end <= nper`, whole periods, and a positive rate and pv.
struct Periods {
    rate: f64,
    nper: f64,
    pv: f64,
    start: f64,
    end: f64,
    period_start: bool,
}

impl Periods {
    /// The per-period IPMT sum, `None` when PMT's `pv·(1+r)^nper` overflows (nper 1E6 or
    /// 1E15, a rate of 1E10) or a period fails, where Excel still has a value (probes/fin-cumipmt-period-bound
    /// A13-A19) and the closed form takes over.
    fn summed_interest(&self) -> Option<f64> {
        // IPMT/PPMT have a value past PMT's overflow since T-1405, but Excel's sums there are the
        // closed form's (A15 is -8910.89108910891, not the summed -8910.891089108913).
        if payment_overflows(self.rate, self.nper, self.pv, self.period_start) {
            return None;
        }
        let mut sum = 0.0;
        for offset in 0..=self.offsets() {
            let period = self.start + f64::from(offset);
            let (interest, _) = period_payments(
                self.rate,
                period,
                self.nper,
                self.pv,
                0.0,
                self.period_start,
            )
            .ok()?;
            sum += interest;
        }
        // 32 finite terms can still add up to an infinity: the closed form then decides
        // (`#NUM!` when it is not finite either; unprobed, T-1890).
        sum.is_finite().then_some(sum)
    }

    /// The payment at the end of each period as a positive amount, `pv·r/(1 - v^nper)` with
    /// `1 - v^nper = -expm1(-nper·L)` and `L = log1p(r)`, as [`Periods::closed_form`] has it.
    fn end_payment(&self, log_growth: f64) -> f64 {
        self.pv * self.rate / self.unpaid_share(log_growth)
    }

    /// `1 - v^nper = -expm1(-nper·L)`, the share of the loan the payments' discounted sum covers
    /// per unit of interest: the payment's denominator.
    fn unpaid_share(&self, log_growth: f64) -> f64 {
        -(-self.nper * log_growth).exp_m1()
    }

    /// Type 0's interest over a short range, period by period from the balance after `k-1`
    /// payments: `-(pv·q·r - pmt·(q-1))` with `q = exp((k-1)·L)`. Excel's interest is this form,
    /// not the exact sum: `CUMIPMT(0.01,100,100000,100,100,0)` is -15.708656559792416 to the bit
    /// where the IPMT sum gives ...592 (probes/fin-cumipmt-overflow-and-forms A12, A14;
    /// fin-argument-modes H72 and `CUMPRINC_CUMIPMT.xlsx` G29 within 4 ulps, T-2192). `None` where
    /// `q` overflows (a rate of 1E10, A18, A19; periods near 1E15, A9-A11), which the other forms
    /// take.
    fn balance_interest(&self) -> Option<f64> {
        let log_growth = self.rate.ln_1p();
        let payment = self.end_payment(log_growth);
        let mut sum = 0.0;
        for offset in 0..=self.offsets() {
            let growth = ((self.start - 1.0 + f64::from(offset)) * log_growth).exp();
            sum += -(self.pv * growth * self.rate - payment * (growth - 1.0));
        }
        sum.is_finite().then_some(sum)
    }

    /// Type 0's principal over a short range, period by period: `-pmt·exp((k-1-nper)·L)`, each
    /// power by `exp` of a rounded product. Excel's errors follow that product's rounding: a
    /// rate of 1E10 is within an ulp of Excel on every probed cell, where the closed form's
    /// difference or product of powers was 33 to 343 ulps off (`CUMPRINC(1E10,40,1,1,20,0)`
    /// -9.999999980000501E-201, probes/fin-cumipmt-overflow-and-forms A16, A20-A24; T-2192).
    /// Other probed type 0 cells are 0 to 4 ulps off (fin-argument-modes H80 4, `CUMPRINC_CUMIPMT.xlsx`
    /// H29 3 where the closed form was 1; T-2362).
    /// `None` when the sum is not finite.
    fn discounted_principal(&self) -> Option<f64> {
        let log_growth = self.rate.ln_1p();
        let payment = -self.end_payment(log_growth);
        let mut sum = 0.0;
        for offset in 0..=self.offsets() {
            let period = self.start + f64::from(offset);
            sum += payment * ((period - 1.0 - self.nper) * log_growth).exp();
        }
        // The sum starts at +0, so an underflowed principal is Excel's 0, not -0
        // (fin-cumipmt-period-bound A14).
        sum.is_finite().then_some(sum)
    }

    /// The offsets `0..=end-start` of a short range's periods: a bounded count, so the loop ends
    /// whatever the period values (the caller passes `end - start < CUMULATIVE_LOOP_PERIODS`).
    fn offsets(&self) -> u32 {
        (self.end - self.start).clamp(0.0, CUMULATIVE_LOOP_PERIODS) as u32
    }

    /// The sums in constant time. With `v^m = exp(m·ln(1+r))` for `m <= 0` (`log1p`, so a rate of
    /// 1E-300 is not lost in `1+r`, A24), the payment is `pmt = -pv·r/(1 - v^nper)` (and over
    /// `1+r` for type 1), the principal of period k is `pmt·v^(nper-k+1)` (type 1: period 1 is
    /// the whole payment), so the principal from period a to b is
    /// `pmt·((v^(b-nper) - v^(a-1-nper))/((1+r)-1))`, and the interest is the payments less the
    /// principal. The denominators' `1 - v^nper` and `(1+r)-1` are `expm1`s of the logarithm:
    /// `CUMPRINC(1E10,100,1,1,100,0)` is -1.0000000000000013, `-1E10/expm1(log1p(1E10))` (A17).
    /// This form matches 19 of the 28 probe cells over many periods or past an overflow (22 of the
    /// case's 30 cells pass overall) bit for
    /// bit and the others within 6 ulps, but for 48 to 64 near period 1E15 (fin-remaining-later-
    /// argument-ranges K3, A18; TODO(T-2362)). `None` when the result is not finite.
    fn closed_form(&self, paid: Paid) -> Option<f64> {
        let log_growth = self.rate.ln_1p();
        let discount = |periods: f64| (periods * log_growth).exp();
        let mut payment = -self.end_payment(log_growth);
        if self.period_start {
            payment = if payment.is_finite() || matches!(paid, Paid::Interest) {
                payment / (1.0 + self.rate)
            } else {
                // `pv·r` overflows, but Excel's principal still has a value: the payment is
                // `pv·(r/(1+r))/(1 - v^nper)` there, so `CUMPRINC(1E300,10,1E300,1,10,1)` is
                // -1E300 (probes/fin-cumipmt-overflow-and-forms A2; the CUMIPMT of an
                // overflowing payment is `#NUM!`, A1, A3). Everywhere else that form misses
                // CUMPRINC_CUMIPMT.xlsx H25 by an ulp, so it is only the fallback (T-2192).
                -(self.pv * (self.rate / (1.0 + self.rate)) / self.unpaid_share(log_growth))
            };
        }
        // Type 1 pays no interest in period 1, so its principal is the whole payment.
        let first = if self.period_start {
            self.start.max(2.0)
        } else {
            self.start
        };
        let difference = || {
            payment
                * ((discount(self.end - self.nper) - discount(first - 1.0 - self.nper))
                    / log_growth.exp_m1())
        };
        let mut principal = if self.end < first {
            0.0
        } else if self.end - first < CUMULATIVE_LOOP_PERIODS && self.end < EXACT_INTEGER_LIMIT {
            // Over a short range `v^(b-nper) - v^(a-1-nper)` cancels, so it is written
            // `v^(a-1-nper)·expm1((b-a+1)·L)`: `CUMPRINC(0.01,360,100000,1,12,0)` is
            // -362.8793467231003, where the difference gives ...0994 and Excel -362.8793467231
            // (probes/fin-remaining-later-argument-ranges I75; CUMPRINC_CUMIPMT.xlsx H6, H22 and
            // H25 to the bit; T-1890). Where `expm1` overflows (a rate of 1E10 over 31 periods)
            // the difference has the value (probes/fin-cumipmt-overflow-and-forms A15-A19). Over
            // a long range the difference stays closer to Excel (fin-cumipmt-period-bound A6).
            let leading = discount(first - 1.0 - self.nper);
            let product = payment
                * (leading * ((self.end - first + 1.0) * log_growth).exp_m1()
                    / log_growth.exp_m1());
            // Only a normal `v^(a-1-nper)` keeps its digits. Subnormal, it has lost them, and the
            // product with it: `CUMPRINC(1E10,32,1,1,25,0)` would be -9.999888696826292E-71
            // where Excel has -9.99999999300011E-71; underflowed, the product is a silent 0
            // (`CUMPRINC(1E10,40,1,1,20,0)`). The difference decides then
            // (probes/fin-cumipmt-overflow-and-forms A20-A24).
            if leading.is_normal() && product.is_finite() {
                product
            } else {
                difference()
            }
        } else {
            difference()
        };
        if self.period_start && self.start == 1.0 {
            principal += payment;
        }
        let result = match paid {
            Paid::Principal => principal,
            Paid::Interest => payment * (self.end - self.start + 1.0) - principal,
        };
        // `+ 0.0` turns the -0 of an underflowed principal into Excel's 0 (A14).
        result.is_finite().then_some(result + 0.0)
    }
}
