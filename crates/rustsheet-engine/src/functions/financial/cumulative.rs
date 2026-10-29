//! CUMIPMT and CUMPRINC: the interest and the principal paid from `start_period` to
//! `end_period`. They are Analysis `ToolPak` functions, so every argument lifts over an array and a
//! multi-cell range is `#VALUE!` in it, with no intersection (`ArgKind::LiftArrays`, SPEC-0011;
//! probes/fin-argument-modes H72-H87, probes/fin-remaining-later-argument-ranges H65-H84). Excel
//! sums any number of periods in constant time (`CUMIPMT(0.01,1E308,1,1,1E308,0)` is -1E306,
//! probes/fin-cumipmt-period-bound A26), so the kernel does too: a short range is a per-period
//! sum (type 0's own forms, type 1's interest the IPMT sum), anything else a closed form
//! (docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods"). Moved out of
//! `financial/mod.rs` (T-1845).

use super::annuity::{growth_log, payment_overflows, period_payments};
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
/// at most 32 `exp` steps. Type 1's principal is the geometric sum or the closed form (T-2514).
const CUMULATIVE_LOOP_PERIODS: f64 = 32.0;

/// 2^53: below it every period is an exact integer, so `start + offset` is a distinct period.
const EXACT_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;

/// The growth `(1+r)^(end-1) - 1` over which type 0's short interest keeps the balance form where
/// the principal is larger than the interest ([`Periods::short_interest`]): from 2^-10 (below it
/// `q - 1` cancels, `CUMIPMT(1E-9,55,157761,1,18,0)`) to 2^10 (above it `pv·q·r` swamps the
/// interest, `CUMIPMT(0.5,96,265678,1,23,0)` is 80 ulps off). Neither edge decides a probed cell:
/// probes/fin-cumipmt-gate-edges A1-A8 cross 2^-10 and are 14 to 512 ulps off with either form, and
/// every cell probed across 2^10 (B1-B11) has a principal smaller than its interest (T-2521).
const BALANCE_GROWTH_MIN: f64 = 1.0 / 1024.0;
/// See [`BALANCE_GROWTH_MIN`].
const BALANCE_GROWTH_MAX: f64 = 1024.0;

/// The leading discount's exponent `(nper-end+1)·L`, the one power of
/// [`Periods::geometric_principal`], from which a short principal is that geometric sum. Below it
/// the range ends near nper and the per-period sum (type 0) or the closed form (type 1) is as
/// close or closer: `CUMPRINC(1E-9,192,2707,8,24,0)` is -239.68227237224258 to the bit with the
/// sum and 2 ulps off with the geometric form (exponent 2E-7, gen-m9 d912671baa1364aa A300),
/// and so are the last periods of nper 1E15 at 0.1 (exponent 0.095, probes/fin-cumprinc-near-1e15
/// C6, C10) and `CUMPRINC(0.1,6,100000,2,6,0)` (probes/fin-cumprinc-form-boundaries G1). From
/// 0.398 up the geometric sum is exact on 22 of the 32 cells of E1-H8 (rates 0.01 and 0.1,
/// leading exponents 0.4 to 3.6) and within 2 ulps on the rest, where the other forms are exact
/// on 7 and up to 5 ulps off; between 0.095 and 0.398 the cut is a fit (T-3067). The deepest
/// discount `(nper-first+1)·L`, the cut before T-3067, misses those near-1E15 cells.
const GEOMETRIC_MIN_LEAD: f64 = 0.3;

/// [`GEOMETRIC_MIN_LEAD`] for a long range (more than 32 periods), where the other form is the
/// closed form's difference of two powers. From a leading exponent of 4.3 up that difference is
/// 46 to 1,431 ulps off Excel and the geometric sum within 2: `CUMPRINC(2,600,319288,3,42,1)` is
/// -6.214428680002485E-262 to the bit with it (probes/fin-cumprinc-form-boundaries A1-D14, 33 to
/// 512 periods at rates 2 and 0.1, I9-I11, J9-J11; gen-m9 b73c6cc96e093f03 A62). Up to 2.9 the
/// difference is as close and bit-exact on the whole-loan cells the geometric sum misses by an
/// ulp (`CUMPRINC(0.005,100000,1000,2,99999,1)` -990.0745031063589, fin-cumipmt-period-bound A6,
/// A11, A30; form-boundaries I1-I8, K1-K11, L1-L11). Between 2.9 and 4.3 the cut is a fit, and
/// [`GEOMETRIC_MIN_LEAD`]'s 0.3 here leaves 5 more probe cells off the bit (T-3067).
const LONG_GEOMETRIC_MIN_LEAD: f64 = 3.6;

/// The growth `nper·L` below which type 1's short interest is the payments less the principal
/// ([`Periods::tiny_rate_interest`]): the rates of 1E-9 and 1E-10 that need it are far below
/// it, the ordinary rates of `CUMPRINC_CUMIPMT.xlsx` (0.0083 over 36 periods, 0.3) far above.
const TINY_RATE_GROWTH: f64 = 1.0 / 1024.0;

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
        let given_start = self.cast_number_no_bools_ref(start, cell)?;
        let start = given_start.ceil();
        let end = self.cast_number_no_bools_ref(end, cell)?.trunc();
        // 0 at the end of the period, 1 at the beginning of the period
        let period_start = match self.cast_number_no_bools_ref(period_type, cell)? {
            0.0 => false,
            1.0 => true,
            _ => return Err(num_error(cell)),
        };
        // An end period past nper is #NUM!, also below nper + 1: `CUMIPMT(0.01,100.5,1000,1,101,0)`
        // (probes/fin-cumipmt-period-bound A20-A22).
        // The start period is compared with 1 as given, before it rounds up: 0.3, 0.5 and 1E-15 are
        // `#NUM!` although they round up to 1 (gen-m9 0a957119475760c5 A40, 3aa168259335be54 A10,
        // 6376e27be7a05871 A306, bc04c1f478c9896c A260; financial.md "gen-m9 financial sweep").
        // Written as negated `>=`/`<=` so a NaN argument fails a check too.
        let valid = start <= end
            && rate > 0.0
            && nper > 0.0
            && pv > 0.0
            && given_start >= 1.0
            && end <= nper;
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
        // bit-exact on CUMPRINC_CUMIPMT.xlsx G3, G4, G9, G15 where those forms are not (but at a
        // tiny rate), and its principal the geometric sum at a large discount, else the closed form:
        // summing PPMT loses a small principal's digits to the payment it is taken from (H5 had 8
        // right digits, H7 7; T-1890), and the per-period `exp` form misses H22 and H25 by an ulp or
        // two (T-2514).
        let short = end - start < CUMULATIVE_LOOP_PERIODS && end < EXACT_INTEGER_LIMIT;
        if !period_start {
            let sum = match (paid, short) {
                (Paid::Interest, true) => periods.short_interest(),
                (Paid::Interest, false) => {
                    periods.payments_less_principal(periods.remaining_principal())
                }
                (Paid::Principal, true) => periods.short_principal(),
                // Type 0's long principal is the geometric sum or the closed form, below.
                (Paid::Principal, false) => None,
            };
            if let Some(sum) = sum {
                return Ok(sum);
            }
        }
        // Type 1 from here on, and type 0's long principal (its short sums returned above).
        let sum = match (paid, short) {
            (Paid::Interest, true) => periods
                .tiny_rate_interest()
                .or_else(|| periods.summed_interest()),
            (Paid::Interest, false) => None,
            (Paid::Principal, true) => periods.geometric_principal(GEOMETRIC_MIN_LEAD),
            // A long range that ends a deep discount before nper is the geometric sum too (T-3067).
            (Paid::Principal, false) => periods.geometric_principal(LONG_GEOMETRIC_MIN_LEAD),
        };
        if let Some(sum) = sum {
            return Ok(sum);
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
            let interest = period_payments(
                self.rate,
                period,
                self.nper,
                self.pv,
                0.0,
                self.period_start,
            )
            .0
            .ok()?;
            sum += interest;
        }
        // 32 finite terms can still add up to an infinity: the closed form then decides
        // (`#NUM!` when it is not finite either; unprobed, T-1890).
        sum.is_finite().then_some(sum)
    }

    /// The payment at the end of each period as a positive amount, `pv·r/(1 - v^nper)` with
    /// `1 - v^nper = -expm1(-nper·L)` and `L = log1p(r)`, as [`Periods::closed_form`] has it.
    /// The interest's `-pmt·m - principal` takes [`Periods::interest_payment`] instead.
    fn end_payment(&self, log_growth: f64) -> f64 {
        self.pv * self.rate / self.unpaid_share(log_growth)
    }

    /// The same payment rounded as Excel's type 0 interest has it, `(pv/(1 - v^nper))·r`: the
    /// operation order is the fit, not a style choice. It is to the bit on all 13 type 0 cells
    /// probed with both functions where [`Periods::end_payment`]'s order is not (T-2192, T-2363).
    fn interest_payment(&self, log_growth: f64) -> f64 {
        self.pv / self.unpaid_share(log_growth) * self.rate
    }

    /// `1 - v^nper = -expm1(-nper·L)`, the share of the loan the payments' discounted sum covers
    /// per unit of interest: the payment's denominator.
    fn unpaid_share(&self, log_growth: f64) -> f64 {
        -(-self.nper * log_growth).exp_m1()
    }

    /// Type 0's interest over a short range. Excel's interest is the payments less the principal,
    /// `-pmt·m - CUMPRINC` in doubles, with the cancellation that brings: `CUMIPMT(1E-9,55,157761,1,18,0)`
    /// is -0.002400835590378847 to the bit where the exact sum is -0.0024008355913268677, and
    /// `CUMIPMT(10,89,267388,3,13,0)` is -29412680 where the balance form loses its digits to
    /// `pv·q·r` (gen-m9 5c6a23c4f6597f1c A392, 1cd3096e3e87c965 A316; T-2363). Its digits are then
    /// the principal's, and where ours is an ulp off Excel's (`CUMPRINC_CUMIPMT.xlsx` H29 3 ulps,
    /// the one-period principal of probes/fin-cumipmt-overflow-and-forms A12) the difference is
    /// visibly off. The balance form is bit-exact there (A12, A14, G21), so it is kept while the
    /// growth `q - 1` over the range stays within [2^-10, 2^10], where it neither cancels in
    /// `q - 1` nor in `pv·q·r - pmt·(q-1)`; the payments less the principal take the rest (T-2363).
    /// But where the principal is no larger than the interest, the payments less the principal
    /// lose no digits and are Excel's inside the band too: `CUMIPMT(0.5,60,100000,10,18,0)` is
    /// -449999.99609733943 to the bit with them and 74 ulps off with the balance form
    /// (probes/fin-cumipmt-gate-edges B1-B11, fin-argument-modes H72; T-2521).
    fn short_interest(&self) -> Option<f64> {
        let principal = self.short_principal();
        let payments_less = self.payments_less_principal(principal);
        if let (Some(principal), Some(sum)) = (principal, payments_less) {
            if principal.abs() <= sum.abs() {
                return Some(sum);
            }
        }
        let growth = ((self.end - 1.0) * self.rate.ln_1p()).exp_m1();
        if (BALANCE_GROWTH_MIN..=BALANCE_GROWTH_MAX).contains(&growth) {
            if let Some(sum) = self.balance_interest() {
                return Some(sum);
            }
        }
        payments_less
    }

    /// Type 0's principal over a short range, which its interest's `-pmt·m - principal` shares
    /// (Excel's CUMIPMT is that of its own CUMPRINC, T-2363). A rate lost in `1 + rate` (1E-300)
    /// has the remaining-balance principal: Excel's `CUMIPMT(1E-300,243,80923,12,22,0)` is
    /// -4.547473508864641E-13 and `(1E-300,83,156759,5,17,0)` 0 to the bit with it, where the
    /// per-period sum adds the payment m times (gen-m9 81bcdc0bec0c8e6b A274, abb7d565af89b849
    /// A84). Elsewhere the geometric sum, then the per-period sum, then the closed form.
    fn short_principal(&self) -> Option<f64> {
        let principal = if 1.0 + self.rate == 1.0 {
            self.remaining_principal()
        } else {
            self.geometric_principal(GEOMETRIC_MIN_LEAD)
                .or_else(|| self.discounted_principal())
        };
        principal.or_else(|| self.closed_form(Paid::Principal))
    }

    /// Type 0's interest from a principal: `-pmt·m - principal` with [`Periods::interest_payment`]
    /// on Excel's own L ([`growth_log`], as the principal's). `None` without a principal or when
    /// the result is not finite.
    fn payments_less_principal(&self, principal: Option<f64>) -> Option<f64> {
        let payment = self.interest_payment(growth_log(self.rate));
        let sum = -payment * (self.end - self.start + 1.0) - principal?;
        // `+ 0.0` turns a -0 into Excel's 0 (`CUMIPMT(1E-300,83,156759,5,17,0)`).
        sum.is_finite().then_some(sum + 0.0)
    }

    /// Type 0's principal as the drop in the balance: `-(pv/(1-v^nper))·v^(nper-start+1)·expm1(m·L)`.
    /// It is the principal of the interest's `-pmt·m - principal` past 32 periods and where `1 + rate`
    /// is 1: `CUMIPMT(1E-15,2E16,1,1E16,1E16+8,0)` is -8.999591419181679E-15 to the bit with it, where
    /// the closed form gave ...674757897966E-15 (probes/fin-cumipmt-overflow-and-forms A5), and the
    /// long-range probe cells stay bit-exact (fin-cumipmt-period-bound A1, A9, A10, A16, A24, A26; K1).
    /// `None` where the leading power is not normal (it has lost its digits) or the result is not
    /// finite, which the closed form then takes.
    fn remaining_principal(&self) -> Option<f64> {
        let log_growth = self.rate.ln_1p();
        let leading = ((self.start - 1.0 - self.nper) * log_growth).exp();
        if !leading.is_normal() {
            return None;
        }
        let drop = ((self.end - self.start + 1.0) * log_growth).exp_m1();
        let principal = -(self.pv / self.unpaid_share(log_growth) * (leading * drop));
        principal.is_finite().then_some(principal)
    }

    /// Type 0's interest over a short range, period by period from the balance after `k-1`
    /// payments: `-(pv·q·r - pmt·(q-1))` with `q = exp((k-1)·L)`. Bit-exact where it is
    /// well-conditioned: `CUMIPMT(0.01,100,100000,100,100,0)` is -15.708656559792416 to the bit
    /// where the IPMT sum gives ...592 (probes/fin-cumipmt-overflow-and-forms A12, A14;
    /// fin-argument-modes H72 and `CUMPRINC_CUMIPMT.xlsx` G29 within 4 ulps, T-2192). `None` where
    /// the sum is not finite.
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
    /// H29 3 where the closed form was 1; T-2363).
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

    /// The principal as one geometric sum from the last period back:
    /// `-((pmt·((1-v^m)/r·(1+r)))·v^(nper-end+1))` with `pmt = (pv/(1-v^nper))·r`, `v^j = exp(-j·L)`
    /// and `1-v^m = -expm1(-m·L)`; type 1 sums from period 2 (period 1 pays no interest) and
    /// divides that by `1+r`, then adds period 1's whole payment `pmt/(1+r)` when the range starts
    /// there. Its one power is the smallest discount, so the per-period powers' own rounding
    /// (hundreds of ulps at integer rates) does not reach it. Within an ulp of Excel on the
    /// integer-rate principals of type 0 (`CUMPRINC(10,261,484823,4,24,0)` -7.507871520276804E-242
    /// to the bit, 311 ulps from the exact sum; gen-m9 9a1fa5366bc50918 A361, A18, 67a4c03afd63f988
    /// A110, 1cd3096e3e87c965 A147, d0854366830dcdee A98) and bit-exact on type 1's (`CUMPRINC(10,121,308352,2,16,1)`
    /// -1.263054652346343E-105, where the closed form was 104 ulps off; 0a957119475760c5 A383,
    /// 5ded7594e597e31e A22, a91bad4e84f49739 A193, b6a02893a8d6b151 A398; T-2514). The operation
    /// order is the fit. L is Excel's own `ln(1+rate)`, PPMT's [`growth_log`], not `log1p`: Excel's is
    /// an ulp off the correctly rounded log at some rates (-1 at 0.1, +1 at 2 against libm's
    /// `log1p`), and a deep discount carries that, so `CUMPRINC(0.1,1023,478581,7,24,1)` and
    /// `CUMPRINC(2,151,319288,3,42,1)` are to the bit with it and 67 and 245 ulps off with `log1p`
    /// (gen-m9 9d9ceba257f8001b A372, b73c6cc96e093f03 A62; T-2521). Over a long range it is
    /// Excel's too where the range ends a deep discount before nper (T-3067). `None` where the
    /// leading exponent `(nper-end+1)·L` is below `min_lead` ([`GEOMETRIC_MIN_LEAD`],
    /// [`LONG_GEOMETRIC_MIN_LEAD`]), the leading power is not normal or the result is not finite:
    /// the per-period sum or the closed form decides there.
    fn geometric_principal(&self, min_lead: f64) -> Option<f64> {
        let log_growth = growth_log(self.rate);
        // Type 1 pays no interest in period 1, so its principal is the whole payment.
        let first = if self.period_start {
            self.start.max(2.0)
        } else {
            self.start
        };
        let lead = (self.nper - self.end + 1.0) * log_growth;
        if self.end < first || lead < min_lead {
            return None;
        }
        let payment = self.interest_payment(log_growth);
        let count = self.end - first + 1.0;
        let series = -(-count * log_growth).exp_m1() / self.rate * (1.0 + self.rate);
        let leading = (-lead).exp();
        if !leading.is_normal() {
            return None;
        }
        let mut principal = -(payment * series * leading);
        if self.period_start {
            principal /= 1.0 + self.rate;
            if self.start == 1.0 {
                principal -= payment / (1.0 + self.rate);
            }
        }
        principal.is_finite().then_some(principal + 0.0)
    }

    /// Type 1's interest at a tiny rate, as Excel has it: the payments less the principal,
    /// `-pmt·m - CUMPRINC` with `pmt = pv·r/(1-v^nper)/(1+r)`. `CUMIPMT(1E-10,296,197540,13,19,1)` is
    /// -1.3127066813467536E-4 to the bit with it, 6.7E-9 relative off the exact sum that the IPMT
    /// sum gives (gen-m9 b72d952c5f52cb4d A257; T-2514). Only where `nper·L` is below
    /// [`TINY_RATE_GROWTH`]: at ordinary rates the IPMT sum is bit-exact on
    /// `CUMPRINC_CUMIPMT.xlsx` G3, G9, G15, G12, G16, G17 and G25, which this form misses by 1 to 3
    /// ulps (T-2363). `None` elsewhere, or when the result is not finite.
    fn tiny_rate_interest(&self) -> Option<f64> {
        let log_growth = self.rate.ln_1p();
        if !self.period_start || self.nper * log_growth >= TINY_RATE_GROWTH {
            return None;
        }
        let payment = self.end_payment(log_growth) / (1.0 + self.rate);
        let principal = self.closed_form(Paid::Principal)?;
        let sum = -payment * (self.end - self.start + 1.0) - principal;
        sum.is_finite().then_some(sum + 0.0)
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
    /// argument-ranges K3, A18; TODO(T-3067)). It keeps `log1p`, not the geometric sum's
    /// `growth_log`: on Excel's L over every long range fin-cumipmt-period-bound A6, A11 and A30
    /// leave the bit (T-2521). `None` when the result is not finite.
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
