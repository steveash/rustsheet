//! CUMIPMT and CUMPRINC: the interest and the principal paid from `start_period` to
//! `end_period`. They are Analysis `ToolPak` functions, so every argument lifts over an array and a
//! multi-cell range is `#VALUE!` in it, with no intersection (`ArgKind::LiftArrays`, SPEC-0011;
//! probes/fin-argument-modes H72-H87, probes/fin-remaining-later-argument-ranges H65-H84). Excel
//! sums any number of periods in constant time (`CUMIPMT(0.01,1E308,1,1,1E308,0)` is -1E306,
//! probes/fin-cumipmt-period-bound A26), so the kernel does too: a short range is the per-period
//! sum main always computed and a long one a closed form (docs/excel-semantics/financial.md
//! "CUMIPMT and CUMPRINC over many periods"). Moved out of `financial/mod.rs` (T-1845).

use super::annuity::{compute_ipmt, compute_ppmt};
use super::number_or_error;
use crate::functions::lift::num_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// The most periods summed one by one. Up to here the kernel keeps main's per-period IPMT/PPMT
/// sum, which matches Excel-cached cells the closed form misses by an ulp or two
/// (`ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx` G3, G9, G15, G26: 1 and 8 periods). Past it the
/// sum drifts from Excel (83 ulps over 1,024 periods, probes/fin-cumipmt-period-bound A11, and no
/// correct digit over 10^5 periods of type 1, A5), while the closed form is within 6 ulps there
/// (48 to 64 near period 1E15). The bound also caps the kernel's work: at most 32 PPMT steps an
/// element (`CUMULATIVE_WEIGHT` in functions/lift/kinds.rs).
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
        // The per-period sum only where every period is an exact integer: past 2^53 `period + 1`
        // is `period`, and the closed form takes over.
        if end - start < CUMULATIVE_LOOP_PERIODS && end < EXACT_INTEGER_LIMIT {
            if let Some(sum) = periods.summed(paid) {
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
    /// The per-period IPMT or PPMT sum, `None` when a period's `(1+r)^n` overflows (nper 1E6 or
    /// 1E15, a rate of 1E10), where Excel still has a value (probes/fin-cumipmt-period-bound
    /// A13-A19) and the closed form takes over.
    fn summed(&self, paid: Paid) -> Option<f64> {
        let step = match paid {
            Paid::Interest => compute_ipmt,
            Paid::Principal => compute_ppmt,
        };
        let mut sum = 0.0;
        // A bounded count, not `period += 1.0`, so the loop ends whatever the period values: the
        // caller passes `end - start < CUMULATIVE_LOOP_PERIODS`, so at most 32 steps.
        let last = (self.end - self.start).clamp(0.0, CUMULATIVE_LOOP_PERIODS) as u32;
        for offset in 0..=last {
            let period = self.start + f64::from(offset);
            sum += step(
                self.rate,
                period,
                self.nper,
                self.pv,
                0.0,
                self.period_start,
            )
            .ok()?;
        }
        // 32 finite terms can still add up to an infinity: the closed form then decides
        // (`#NUM!` when it is not finite either; unprobed, T-1890).
        sum.is_finite().then_some(sum)
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
    /// argument-ranges K3, A18; TODO(T-1890)). `None` when the result is not finite.
    fn closed_form(&self, paid: Paid) -> Option<f64> {
        let log_growth = self.rate.ln_1p();
        let discount = |periods: f64| (periods * log_growth).exp();
        let mut payment = -self.pv * self.rate / -(-self.nper * log_growth).exp_m1();
        if self.period_start {
            payment /= 1.0 + self.rate;
        }
        // Type 1 pays no interest in period 1, so its principal is the whole payment.
        let first = if self.period_start {
            self.start.max(2.0)
        } else {
            self.start
        };
        let mut principal = if self.end >= first {
            payment
                * ((discount(self.end - self.nper) - discount(first - 1.0 - self.nper))
                    / log_growth.exp_m1())
        } else {
            0.0
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
