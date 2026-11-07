use super::annuity::{annuity_factor_rounded, annuity_power};
use crate::{
    arithmetic::{extended_product, extended_quotient, extended_sum, flush_subnormal},
    expressions::token::Error,
};

// The solvers behind RATE, IRR and XIRR. IRR (a secant in d = 1 - v, v = 1/(1+r)) and XIRR (a
// bracket walk and bisection) follow models fitted to oracle sweeps in the fit lab (T-1525,
// T-1533, T-3120; see their doc comments). RATE is a fitted secant too (T-2005, financial.md "RATE is a secant on the FV
// residual").

/// RATE's residual f(r) = ((pv·P) + (pmt·(k·((P-1)/r)))) + fv, P = (1+r)^n by FV's power
/// (`annuity_power`), k = 1+r for type 1, with each `+`, `*` and `/` Excel's twice-rounded operator
/// ([`annuity_factor_rounded`]; a once-rounded factor misses 4 of the 782 fit-lab
/// cells); f(0) is the limit pv + pmt·n + fv. `None` where Excel's `^` errs (a power past the
/// double range). Fit lab `rate` (T-2527): bit-exact on all 782 RATE cells of the fit set, a
/// nested PMT at Excel's bits (financial.md "RATE is a secant on the FV residual";
/// probes/rate-excel-pmt-bits pins it).
fn rate_residual(
    rate: f64,
    (nper, pmt, pv, fv): (f64, f64, f64, f64),
    period_start: bool,
) -> Option<f64> {
    if rate == 0.0 {
        return Some(pv + pmt * nper + fv);
    }
    let p = annuity_power(rate, nper).ok()?;
    let factor = annuity_factor_rounded(rate, p, period_start);
    let f = extended_sum(
        extended_sum(extended_product(pv, p), extended_product(pmt, factor)),
        fv,
    );
    f.is_finite().then_some(f)
}

/// The most secant steps RATE takes, Excel's own cap: the probe guesses whose run needs 200
/// steps converge and those that need 201 to 216 are `#NUM!` (probes/rate-step-cap).
/// A lifted RATE is charged the time of a run of this many steps (`RATE_WEIGHT` in the lifting
/// table, T-2814): refit it if the cap or a step's work changes.
pub(crate) const RATE_MAX_STEPS: u32 = 200;

/// RATE as Excel computes it: a secant on the residual [`rate_residual`] from r0 = guess and
/// r1 = r0 - 0.001 if f(r0) > 0, else r0 + 0.001 (a zero residual steps up, T-3876), each step `r2 = r1 - f1·(r1-r0)/(f1-f0)` with Excel's
/// twice-rounded `-`, `*` and `/`. It stops at the first iterate with |r2 - r1| < 1e-7 and
/// |f(r2)| < 1e-7. The iterate of step [`RATE_MAX_STEPS`] is returned unconverged when it passes
/// a looser test: |r2 - r1|, |f(r2)| and 1 + r2 against 1e-4, all absolute
/// (probes/rate-cap-return-rule, rate-pmt-zero-cap, rate-step-cap A30). The modelled outcome of a
/// flat secant (f1 = f0) is a hold at r1, judged by the same two tests, and an iterate at or below
/// -1 runs on (T-3348, decision 0198; the rule, its evidence and its known misses are in
/// financial.md "RATE holds on a flat secant and runs on below -1"). An early flat secant on two
/// distinct iterates with a small |f1| restarts the run once from the older iterate instead
/// (T-3698, financial.md "RATE restarts once on an early flat secant"). A residual Excel's `^`
/// cannot form and a run that has not stopped by then are `#NUM!`. Microsoft documents Newton's
/// method and 20 steps; neither fits. The fit, its scores and the rules not yet probed are in
/// financial.md "RATE is a secant on the FV residual" (T-2005).
pub(crate) fn compute_rate(
    pv: f64,
    fv: f64,
    nper: f64,
    pmt: f64,
    annuity_type: i32,
    guess: f64,
) -> Result<f64, Error> {
    if guess <= -1.0 {
        return Err(Error::VALUE);
    }
    let args = (nper, pmt, pv, fv);
    rate_secant(args, annuity_type != 0, guess, true)
}

/// One secant run of [`compute_rate`] from `guess`; `restart` allows the one restart of an early
/// flat secant (T-3698), which the restarted run does not get again.
fn rate_secant(
    args: (f64, f64, f64, f64),
    period_start: bool,
    guess: f64,
    restart: bool,
) -> Result<f64, Error> {
    // The start offset was chosen by scoring (T-2005; fit lab `rate_candidates` scores +-1E-3 and
    // 1E-4): on the 782 cells of the fit lab's `rate` set the model is bit-exact on all, with
    // nper 1..1000, |pmt| 1E-6..1E+6, pv -100..1E+7, guess -0.5..10 and rates -0.75..2. T-3478
    // measured it past that box on a fresh 300-point sweep (probes-numeric/t3478-rate-r4-wide):
    // bit-exact on all 100 points where Excel returns a number, where 1E-4 or the other side's
    // -1E-3 gets 8 and 14 (financial.md "RATE and IRR secant constants past their fit sets").
    // The tag's second range is where those 100 roots lie; past it (rates to 100, nper to 1E+5)
    // the sweep only agrees with Excel's `#NUM!`. Its 14 cells off Excel were not this constant's:
    // 13 converged to the spurious root r = -1 (`#NUM!` since T-3585's floor on 1 + r) and one is
    // T-3424's.
    /// FITTED(T-2005, 782 cells, RATE nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2; T-3478 sweep, 100 returned roots: nper 1..22908, |pmt| 1.6E-47..1.9E+10, |pv| and |fv| 1.1E-8..9.7E+11, guess -0.97..980, rate -0.985..21.3)
    const OFFSET: f64 = 0.001;
    // Three uses. The step stop is not a fit: Microsoft's RATE documentation gives 0.0000001 as the
    // convergence bound. The residual stop |f| < 1e-7 is not documented and was chosen by scoring
    // (financial.md "RATE is a secant on the FV residual": |f| < 1e-6 scores 188, < 1e-8 181), as
    // IRR's is (`irr_secant`). On T-3478's sweep 1e-7 keeps 100 of 100, 1e-6 92 and 1e-8 93. The
    // third is the floor on 1 + r of a converged iterate (T-3585, below).
    /// FITTED(T-2005, 782 cells, RATE nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2; T-3478 sweep, 100 returned roots: nper 1..22908, |pmt| 1.6E-47..1.9E+10, |pv| and |fv| 1.1E-8..9.7E+11, guess -0.97..980, rate -0.985..21.3; the residual stop)
    const TOLERANCE: f64 = 1e-7;
    // Excel's looser test at the cap (T-2689): the 200th iterate is returned when its step and
    // |f| are below 1e-4 and it is above -1 + 1e-4. probes/rate-cap-return-rule: steps of 7.3e-5
    // (A68) and 7.8e-5 (A72) return and 1.3e-4 (A70) is `#NUM!`, also where the step is 4e-4 of
    // the rate (A72);
    // |f| 7.3e-5 returns (rate-step-cap A30) and 1.06e-4 is `#NUM!` (A53); 1 + r 1.02e-4 returns (A1)
    // and 9.4e-5 is `#NUM!` (A16). One value sits inside all three brackets, so it is a cut placed
    // on the cells of the three cap probes (T-3395 counted them).
    /// FITTED(T-2689, 178 cells, RATE at the 200-step cap; one cut for all three tests bracketed 9.4E-5..1.02E-4)
    const RATE_CAP_TOLERANCE: f64 = 1e-4;
    // A flat secant on two distinct iterates at steps RATE_RESTART_FIRST..=RATE_RESTART_STEPS, with
    // |f1| from RATE_RESTART_F_MIN up to RATE_RESTART_F_MAX, restarts the run from the older iterate
    // r0 (T-3698). The step bound and the |f1| cut were chosen on the train side of the fit lab's
    // hash split only (holdout.py, margin 5 points stated before scoring; docs/research/t3698):
    // over K in 3..200 and the cut in 1E-5..1E-3, K = 16 or 17 is best on train (1,395 of 1,477
    // bit-exact) and the smaller is taken. The evidence brackets the cut: probes/t3698-rate-flat-
    // restart restarts at |f1| up to 4.3E-5 (A3:A8) and holds at 1.7E-4 (A1). Out of sample
    // (probes/t3906-rate-restart-oos, 240 fresh calls where the rule and the hold differ) the rule
    // is bit-exact on 48 and the hold on 12, but the rule moves A152 and A194 out of the Gate A
    // envelope (A194 becomes #NUM!), so it is not cleared: T-3850 must narrow or revert it
    // (T-3906). Outside the evidenced range (the first two
    // steps, |f1| below the smallest probed flat secant) the run holds as before. `r0 > -1` is a
    // guard, not evidence: a restart from r0 <= -1 would be a guess that RATE refuses.
    /// FITTED(T-3698, 1477 cells, RATE flat secant at steps 3..16 with |f1| 1.3E-7..1E-4)
    const RATE_RESTART_STEPS: u32 = 16;
    /// The first step the restart is evidenced at (no probed flat secant comes earlier; T-3698).
    const RATE_RESTART_FIRST: u32 = 3;
    /// The evidence floor of the restart's |f1| (T-3698). It sits just above 1.27E-7, the smallest |f1| of a
    /// probed flat secant on distinct iterates (step 58). Inside the rule's own steps 3..16 the smallest probed
    /// |f1| is 1.67E-7 (the nper-1 family at step 3), so 1.3E-7..1.67E-7 there is unprobed (T-3906 review).
    const RATE_RESTART_F_MIN: f64 = 1.3e-7;
    /// The |f1| cut of the restart: part of T-3698's fit (the FITTED tag on `RATE_RESTART_STEPS` covers it,
    /// scored over 1E-5..1E-3). It equals the cap's bound `RATE_CAP_TOLERANCE` but is not tied to it, so
    /// a change to that fit does not move this one (T-3906).
    const RATE_RESTART_F_MAX: f64 = 1e-4;
    let residual = |rate: f64| rate_residual(rate, args, period_start).ok_or(Error::NUM);
    let (mut r0, mut f0) = (guess, residual(guess)?);
    // Only a positive residual steps down: a residual of exactly 0 at the guess steps up, to
    // guess + 0.001. With every residual rounding noise (nper 1, pv + pmt = 0, fv 0) Excel returns
    // the guess exactly where f(guess + 0.001) is not 0 (the secant steps back onto the guess) and
    // is `#NUM!` where it is 0 too (a flat secant on 0), on all 14 such cells of
    // probes/t3876-rate-noise-residual; stepping down gets 11 (financial.md "RATE steps up from a
    // zero residual", T-3876).
    let mut r1 = if f0 > 0.0 { r0 - OFFSET } else { r0 + OFFSET };
    let mut f1 = residual(r1)?;
    for step in 1..=RATE_MAX_STEPS {
        // A flat secant (f1 = f0) has no finite step: 0/0 after a zero-step stall (r1 = r0),
        // a nonzero number over 0 otherwise. Holding at r1 is the modelled outcome that fits the
        // evidence; the engine stopped with `#NUM!` here before T-3348. The held point is judged
        // by the convergence test and by the cap's test, as the 200th iterate would be. Near the
        // root of a long annuity |f| is rounding noise above 1e-7: `RATE(239,-14841.27102594,
        // 344293.4825124,-155496.6323522,0)` is held at |f| 3.2e-7 and is 0.04310546919282547
        // in Excel (probes/t3348-rate-fv-nonzero A1). A residual of exactly 0 on both sides is
        // `#NUM!` (`RATE(1,0,0,0)`, probes/rate-zero-residual-guess A1, A2). Excel errs on 13
        // holds the rule returns (decision 0198, T-3424).
        let flat = f1 == f0;
        let (r2, f2) = if flat {
            if f1 == 0.0 {
                return Err(Error::NUM);
            }
            // The restart (T-3698): a fresh run from r0, as if r0 were the guess
            // (probes/t3579-rate-residual-nper1: A1's run is A6's, whose guess is A1's r0).
            if restart
                && r1 != r0
                && (RATE_RESTART_FIRST..=RATE_RESTART_STEPS).contains(&step)
                && (RATE_RESTART_F_MIN..RATE_RESTART_F_MAX).contains(&f1.abs())
                && r0 > -1.0
            {
                return rate_secant(args, period_start, r0, false);
            }
            (r1, f1)
        } else {
            let r2 = extended_sum(
                r1,
                -extended_quotient(
                    extended_product(f1, extended_sum(r1, -r0)),
                    extended_sum(f1, -f0),
                ),
            );
            if !r2.is_finite() {
                return Err(Error::NUM);
            }
            // An iterate at or below -1 does not end the run (Excel's `^` takes the negative
            // base at an integer nper): only the result must be above -1
            // (probes/t3348-rate-below-minus-one, 50/50; T-3348). At a non-integer nper the
            // residual's `^` errs there and the run is `#NUM!`; Excel is unprobed (financial.md).
            (r2, residual(r2)?)
        };
        let step_size = (r2 - r1).abs();
        if step_size < TOLERANCE && f2.abs() < TOLERANCE {
            // A converged iterate is returned only when 1 + r is above 1e-7, the step bound: roots
            // with 1 + r from 1E-4 down to 1.0000034E-7 return and 1E-7 and below are `#NUM!`,
            // genuine roots and the spurious root -1 of a type-1, fv-0 residual alike
            // (probes/t3585-rate-num-near-minus-one; financial.md "RATE is #NUM! when 1 + r is
            // 1e-7 or less", T-3585).
            return if 1.0 + r2 > TOLERANCE {
                Ok(r2)
            } else {
                Err(Error::NUM)
            };
        }
        if (flat || step == RATE_MAX_STEPS)
            && step_size < RATE_CAP_TOLERANCE
            && f2.abs() < RATE_CAP_TOLERANCE
            && 1.0 + r2 > RATE_CAP_TOLERANCE
        {
            return Ok(r2);
        }
        if flat {
            return Err(Error::NUM);
        }
        (r0, f0, r1, f1) = (r1, f1, r2, f2);
    }
    Err(Error::NUM)
}

/// The worksheet NPV with Excel's arithmetic and range rules: (v1/d0 + v2/d1 + ... + vn/d(n-1))
/// / (1+rate), where d0 = 1 and d(i) = d(i-1)*(1+rate) is a running product, every step Excel's
/// twice-rounded operator and the quotients summed left to right (financial.md "NPV's residue at
/// IRR's root is not the double sum"; T-2671, probes/npv-term-order 382/382 and m9-fin-nper-npv
/// 88/88 in the fit lab, with plain or twice-rounded operators alike; IRR's NPV needs the latter,
/// T-3205).
/// The call is `#DIV/0!` when 1+rate is 0 (a rate of -1, also with no flow but 0s) or when a
/// discount a period divides by overflows or is below the smallest normal double (financial.md
/// "NPV's discount past the double range", probes/npv-power-edges and npv-power-edges-2):
/// `NPV(1E+300,1,1)` is 1E-300 but `NPV(1E+300,0,0,0)` is `#DIV/0!`, and `NPV(1,SEQUENCE(1024))`
/// is a number but 1025 flows are `#DIV/0!`. A subnormal quotient is 0, as for `/`
/// (`NPV(9E+307,1)` is 0).
pub(crate) fn excel_npv(rate: f64, values: &[f64]) -> Result<f64, Error> {
    let base = 1.0 + rate;
    if base == 0.0 {
        return Err(Error::DIV);
    }
    let sum = discounted_sum(base, values).ok_or(Error::DIV)?;
    Ok(flush_subnormal(extended_quotient(sum, base)))
}

/// `c0/d0 + c1/d1 + ... + cn/dn` with d0 = 1 and d(i) = d(i-1)*base a running product, every `*`,
/// `/` and `+` Excel's twice-rounded operator and the quotients summed left to right: the sum
/// worksheet NPV divides by `base` and the NPV IRR's secant evaluates at `base` = 1 + r. The
/// twice-rounded `*` and `+` decide IRR's last secant step: with plain doubles
/// `IRR({-1000,230,200,125,200,10},5)` and gen-m9's `IRR({0.5,0.1,-0.5},0.5)` stop one iterate
/// early, 16 ulp off (financial.md "IRR is a secant in d = 1 - v", T-3205). `None` when a discount
/// overflows or is below the smallest normal double; a subnormal quotient is 0, as for `/`.
fn discounted_sum(base: f64, values: &[f64]) -> Option<f64> {
    let mut sum = 0.0;
    // d(i), the running product this period divides by.
    let mut discount = 1.0_f64;
    for (i, item) in values.iter().enumerate() {
        if i > 0 {
            discount = flush_subnormal(extended_product(discount, base));
        }
        if discount == 0.0 || !discount.is_finite() {
            return None;
        }
        sum = extended_sum(sum, flush_subnormal(extended_quotient(*item, discount)));
    }
    Some(sum)
}

/// NPV at the discount factor `v` = 1/(1 + r), as IRR's search evaluates it: the
/// [`discounted_sum`] at x = 1/v, the running product worksheet NPV uses (financial.md "IRR is a
/// secant in d = 1 - v"; fit lab `irr`, T-3120). `None` when a discount overflows or underflows
/// or the sum is not finite: that ends the secant's call with `#DIV/0!` (T-3407).
fn irr_npv(values: &[f64], v: f64) -> Option<f64> {
    discounted_sum(extended_quotient(1.0, v), values).filter(|total| total.is_finite())
}

/// The smallest 1 + r IRR does not return: Excel's IRR is `#NUM!` at roots this far out, from
/// the secant or the fallback alike (financial.md "IRR's far-root bound"): `IRR({-1,3E+6})` and
/// `IRR({-1,0,1E+14})` (1 + r = 9999999.994) return, `IRR({-1,1E+7})` and every larger
/// two-flow root are `#NUM!` (probes/irr-fallback-roots A25-A39, irr-guess-edges-2 A15-A34).
/// The exact edge between 9999999.994 and 1E+7 is not pinned: the round value is placed in that
/// bracket (T-2727).
/// FITTED(T-2727, 35 cells, IRR roots with 1 + r 1E+4..2E+15; cut bracketed 9999999.994..1E+7)
const IRR_FAR_ROOT_BOUND: f64 = 1e7;

/// The most Newton steps of [`irr_fallback`]: a run that needs 119 is `#NUM!` in Excel
/// (gen-m9/abb7d565af89b849 A254) and the slowest probed root that returns needs 29 (`{42,1,1,-1}`).
const IRR_FALLBACK_STEPS: usize = 50;

/// The largest |NPV| (as [`irr_npv`] evaluates it, one Newton step past the stop) at which
/// [`irr_fallback`] returns the root it reached: a larger residual is `#NUM!` (financial.md "IRR's
/// fallback is `#NUM!` when NPV at its root is 1E-4 or more"; T-3411). Over every IRR cell in eval
/// (169 cases), the cut moves no cell anywhere in 9.16E-5..1.22E-4: Excel's largest returning
/// residual is 9.155E-5 (probes-numeric/t3411-irr-r4-fresh A37 and A140) and its smallest `#NUM!`
/// one 1.2207E-4 (t3478-irr-r4-wide A90). Those residuals are 3 and 4 times 2^-15, the rounding
/// step of NPV sums of flows near 1E+11, so the round value is placed between the two.
/// Excel follows the absolute cut across scales: probes-numeric/t3411-irr-residual-scale scales those
/// cells' flows by 1E-6..1E+4 and Excel turns `#NUM!` as the flows grow (A37's flows times 10, A79's
/// times 10) exactly where this cut does, on all 72 cells. Applied only within
/// [`IRR_ROOT_RESIDUAL_FLOWS`] (decision 0188 R4.3); outside it the root is returned, as before T-3411.
/// FITTED(T-3411, 59264 cells, the 170 IRR cases: fallback roots of 2..148 flows, largest |c| 6.4E-12..2.2E+82; cut bracketed 9.155E-5..1.2207E-4)
const IRR_ROOT_RESIDUAL: f64 = 1e-4;

/// The flow sets [`IRR_ROOT_RESIDUAL`] is evidenced on: at most 148 flows (the longest scored series,
/// probes-numeric/t3411-irr-r4-fresh*) with the largest |flow| from 6.4E-12
/// (t3478-irr-r4-wide A287) to 2.2E+82 (A25).
const IRR_ROOT_RESIDUAL_FLOWS: (usize, f64, f64) = (148, 6.4e-12, 2.2e82);

/// The largest |flow| of `values` (0 when empty): the scale the IRR solver's flow ranges and the
/// secant's stalled-step bound are measured on.
fn largest_flow(values: &[f64]) -> f64 {
    values.iter().fold(0.0_f64, |m, c| m.max(c.abs()))
}

/// Whether [`IRR_ROOT_RESIDUAL`] applies to `values` ([`IRR_ROOT_RESIDUAL_FLOWS`]).
fn irr_root_residual_applies(values: &[f64]) -> bool {
    let (most_flows, smallest, largest) = IRR_ROOT_RESIDUAL_FLOWS;
    let peak = largest_flow(values);
    values.len() <= most_flows && (smallest..=largest).contains(&peak)
}

/// The |NPV| above which a secant run from the default guess 0.1 that goes flat after a zero step
/// (d1 = d0 and f1 = f0, so the next step is 0/0) is `#DIV/0!` rather than failed, measured on the
/// largest |NPV| the run evaluated (financial.md "IRR is #DIV/0! on a flat secant after a far step";
/// T-3347, decision 0316). On long conventional series the first step from 0.1 jumps far toward
/// r = -1, where NPV is 1E+200 or more, comes back and stalls. Placed on the train side of the fit
/// lab's split (decision 0188 R4.1): the largest train `#NUM!` run reaches 1.556E+226
/// (probes/t3406-irr-num-boundary A65), the smallest train `#DIV/0!` run above it 6.57E+230 (A81),
/// and the round value sits in that bracket (the test side's smallest `#DIV/0!` above, 5.16E+226 at
/// probes-numeric/t3411-irr-r4-fresh-4 A86, is in it too). Over every IRR cell in eval it moves 19
/// cells to Excel's `#DIV/0!` (13 train, 6 test) and none away; on its fresh sweep, drawn after the
/// freeze (probes-numeric/t3347-irr-r4-fresh, R4.2), 8 more and none away. It is a proxy, not Excel's
/// mechanism: two `#DIV/0!` runs reach only 1.3E+215 and 3.0E+221 (t3411-irr-r4-fresh A85, -3 A24),
/// and on 11 runs past the cut Excel returns the root (roots -0.04 to -0.11): the engine's `#NUM!`
/// there becomes `#DIV/0!`, wrong either way (T-3548). Applied only within
/// [`IRR_FLAT_DIV_ZERO_FLOWS`] and to runs from 0.1 (decision 0188 R4.3).
/// FITTED(T-3347, 36 cells, flat zero-step IRR runs past 1E+200 of 27..144 flows, largest |c| 1.3E+4..6.7E+14, guess 0.1; cut bracketed 1.556E+226..6.57E+230 on train)
const IRR_FLAT_DIV_ZERO_NPV: f64 = 3e226;

/// The flow sets [`IRR_FLAT_DIV_ZERO_NPV`] is evidenced on: 27 to 144 flows with the largest |flow|
/// from 1.3E+4 to 6.7E+14 (the 36 flat zero-step runs from 0.1 above 1E+200 in eval). The shape
/// follows [`IRR_ROOT_RESIDUAL_FLOWS`]'s, with the fewest flows added.
const IRR_FLAT_DIV_ZERO_FLOWS: (usize, usize, f64, f64) = (27, 144, 1.3e4, 6.7e14);

/// Whether a secant run from `guess` that went flat after a zero step, having evaluated an |NPV| as
/// large as `largest_npv`, is `#DIV/0!` ([`IRR_FLAT_DIV_ZERO_NPV`]).
fn irr_flat_is_div_zero(values: &[f64], guess: f64, largest_npv: f64) -> bool {
    let (fewest, most, smallest, largest) = IRR_FLAT_DIV_ZERO_FLOWS;
    let peak = largest_flow(values);
    // Every evidenced run starts from the default 0.1 (a call's own guess or its retry from 0.1):
    // a run from another guess keeps the failed run's path (decision 0188 R4.3).
    guess == 0.1
        && largest_npv > IRR_FLAT_DIV_ZERO_NPV
        && (fewest..=most).contains(&values.len())
        && (smallest..=largest).contains(&peak)
}

/// The steps an IRR secant run takes before it fails, where its flows and guess are inside
/// [`IRR_LONG_RUN_RANGE`]; elsewhere a run fails after [`IRR_SHORT_RUN_STEPS`]. Excel finishes runs
/// longer than 50 steps on the run's own digits: every train and test run that reaches the residual
/// stop at its 51st to 200th step is Excel's value bit for bit (83 cells; t3411-irr-r4-fresh A53 at
/// the 90th step, t3411-irr-r4-fresh-3 A53 at the 200th). Past 200 steps Excel's outcome is no clean
/// function of the step count: on the fit lab's train side it returns the run's own value after 241
/// to 291 steps (t3411-irr-r4-fresh A2, t2927-irr-fallback-sweep A97 at 256, t3411-irr-r4-fresh-4
/// A63, t3411-irr-r4-fresh A70) and the
/// retry's after 217 to 287 (t3411-irr-r4-fresh-2 A63, A54, A57, A74; t3411-irr-r4-fresh A142;
/// t3478-irr-r4-wide A240; t3411-irr-r4-fresh-3 A74). So 200 is not Excel's cap: it is a point on
/// the train-score plateau of 200..216 steps (64 of the 87 train cells whose result depends on the
/// cap; 291 scores 61, 50 scores 12). `docs/research/t3408/bounds.py` prints the table from the
/// replay's cell lists (financial.md "IRR's secant runs longer than 50 steps"; T-3408, decision 0320).
/// FITTED(T-3408, 92 cells, IRR secant runs of 51..200 steps: 4..54 flows (4..5 the 12 T-4888 cells), largest |c| 5.6E-9..2.2E+8, guess -0.99934..9.81E+7; cap 200 = train-score plateau 200..216, no Excel cap)
const IRR_LONG_RUN_STEPS: usize = 200;

/// The steps of an IRR secant run outside [`IRR_LONG_RUN_RANGE`], and the steps within which a
/// stalled zero step still stops a run: a directive-5 bound (financial.md "The IRR and XIRR solver
/// caps"). As the stall bound it sits in a bracket of 41..69 steps: Excel returns every stalled stop
/// in eval up to the 41st step (t3478-irr-r4-wide A260) and is `#NUM!` on all six from there to
/// Excel's cap (70th to 189th: probes/irr-guess-edges-2 A25-A27, Gate A r4-T-3411 A228 and A215, t3478-irr-r4-wide
/// A227). Those six are all outside [`IRR_LONG_RUN_RANGE`], where runs stop at 50 steps anyway, so no
/// scored cell tests the stall bound inside the range; there it follows them (T-3408).
const IRR_SHORT_RUN_STEPS: usize = 50;

/// The calls an IRR secant cap is evidenced on: flow count, largest |flow| and guess, each inclusive.
struct IrrRunRange {
    flows: (usize, usize),
    largest_flow: (f64, f64),
    guess: (f64, f64),
}

/// The calls [`IRR_LONG_RUN_STEPS`] applies to (decision 0188 R4.3): the range of the 53 train runs
/// of 51..200 steps that return their own value and whose run and retry give different bits (a cell
/// where both give the same bits cannot tell the cap apart): 6 flows (t2927-irr-fallback-sweep A93) to
/// 54 (t3411-irr-r4-fresh-4 A148), widened down to 4 by T-4888: on 4- and 5-flow series whose run
/// goes on past the pole, Excel's value is the run's own at its 52nd to 197th step
/// (probes/t4838-irr-pole-runon A11, A25, A51, A67, A80, A87, A88, A90, A96, A97, A105, A113,
/// predicted with a 200-step cap before the fill). No cell evidences 2 or 3 flows. Largest |flow|
/// 5.6E-9 (t3478-irr-r4-wide A270) to 2.15E+8 (t2927-irr-fallback-sweep A95); guess -0.99933
/// (t3478-irr-r4-wide A230) to 9.81E+7 (A234). The other bounds are rounded outward (2.2E+8,
/// -0.99934), the rounding chosen before scoring held-out cells.
const IRR_LONG_RUN_RANGE: IrrRunRange = IrrRunRange {
    flows: (4, 54),
    largest_flow: (5.6e-9, 2.2e8),
    guess: (-0.99934, 9.81e7),
};

/// The step cap of an IRR secant run on `values` from `guess` ([`IRR_LONG_RUN_STEPS`]).
fn irr_secant_steps(values: &[f64], guess: f64) -> usize {
    let IrrRunRange {
        flows: (fewest, most),
        largest_flow: (smallest, largest),
        guess: (lowest, highest),
    } = IRR_LONG_RUN_RANGE;
    let peak = largest_flow(values);
    if (fewest..=most).contains(&values.len())
        && (smallest..=largest).contains(&peak)
        && (lowest..=highest).contains(&guess)
    {
        IRR_LONG_RUN_STEPS
    } else {
        IRR_SHORT_RUN_STEPS
    }
}

/// The rate 1/v - 1 at a converged discount factor `v`, or `None` past [`IRR_FAR_ROOT_BOUND`].
fn irr_from_discount(v: f64) -> Option<f64> {
    let x = extended_quotient(1.0, v);
    (x < IRR_FAR_ROOT_BOUND).then_some(x - 1.0)
}

/// One run of IRR's secant from `guess`, or why it stopped without a root: [`SecantStop::Failed`]
/// on a flat secant, an iterate that is not a positive discount factor or no stop in 50 steps;
/// [`SecantStop::DivZero`] at v = 0 or where NPV cannot be evaluated. The state is
/// d = 1 - v = r/(1 + r), so from v = 0.5 down the iterates' v = 1 - d sit on Excel's grid of multiples of 2^-53
/// (probes/t2565-irr-guess-sweeps), and each step is anchored on the newer iterate.
/// Chosen by scoring on the fit set (financial.md "IRR is a secant in d = 1 - v"): the start offset
/// of 1E-3 toward larger |NPV| (+1E-3 on both sides scores 121 of 263) and the 1E-7 residual stop
/// (1E-6 scores 188, 1E-8 180). Not fits: the 1E-7 step stop is Microsoft's documented 0.00001
/// percent, and the step cap is [`irr_secant_steps`]'s (financial.md "The IRR and XIRR solver caps").
/// T-3478 measured both past the fit set on a fresh 300-point sweep (probes-numeric/t3478-irr-r4-wide):
/// in aggregate the shipped pair scores 232 of 300 against Excel, against 146 and 126 for offsets
/// 1E-4 and 1E-2 and 224 and 217 for residuals 1E-6 and 1E-8 (financial.md "RATE and IRR secant
/// constants past their fit sets"). The tag covers these constants only: the sweep's 50 cells off
/// Excel come from the step cap, the fallback, the residual's absolute scale and Excel errors not
/// yet explained (T-3548).
/// FITTED(T-1533, 263 cells, IRR of 2..41 flows, |c| 1E-6..1E+9, guess -0.99..100, roots -0.99..99; T-3478 sweep, 231 Excel roots: 2..148 flows, |c| 2.3E-15..4.0E+12, guess -0.9997..9.8E+7, roots -0.9997..9.0E+5)
/// The zero-step stop's bound: [`irr_stall_stops`] (FITTED there, T-3120 and T-4888).
fn irr_secant(values: &[f64], guess: f64) -> Result<f64, SecantStop> {
    // An NPV the run cannot evaluate (a discount power out of the double range, or a sum that is
    // not finite) ends the call with `#DIV/0!`, worksheet NPV's error for the same sum, and without
    // the retry from 0.1: probes/t3406-irr-num-boundary A69-A72 and A84-A94, the 7 long-series
    // hold-out cells of probes-numeric/t3347-irr-conventional-holdout (A123..A143) and 28 cells of
    // the t3411 fresh sweeps are `#DIV/0!` in Excel exactly where the secant's step lands there
    // (financial.md "IRR is #DIV/0! where its secant cannot evaluate NPV", T-3407).
    let npv = |d: f64| irr_npv(values, 1.0 - d).ok_or(SecantStop::DivZero);
    let peak = largest_flow(values);
    let mut d0 = extended_quotient(guess, 1.0 + guess);
    let mut f0 = npv(d0)?;
    let mut d1 = d0 - if f0 > 0.0 { 1e-3 } else { -1e-3 };
    let mut f1 = npv(d1)?;
    let mut largest_npv = f0.abs().max(f1.abs());
    for step in 0..irr_secant_steps(values, guess) {
        if f1 == f0 {
            // After a zero step (d1 = d0) the next step is 0/0: `#DIV/0!` where the run went far
            // (probes-numeric/t3347-irr-conventional-holdout A131 and A138; T-3347).
            return Err(
                if d1 == d0 && irr_flat_is_div_zero(values, guess, largest_npv) {
                    SecantStop::DivZero
                } else {
                    SecantStop::Failed
                },
            );
        }
        let d2 = d1 - extended_quotient(f1 * (d1 - d0), f1 - f0);
        // An iterate of exactly d = 1 is v = 0, and NPV's 1/v divides by zero: Excel's
        // `#DIV/0!` (`IRR({-1,1E+300})`, probes/irr-guess-edges A39; financial.md "IRR's far-root
        // bound", T-2802). The call ends there, without the retry from 0.1.
        if d2 == 1.0 {
            return Err(SecantStop::DivZero);
        }
        if !d2.is_finite() {
            return Err(SecantStop::Failed);
        }
        // Past the pole (d > 1, so v < 0 and 1 + r < 0) the run goes on: NPV is evaluated at
        // the negative v and the secant carries on, as RATE's does below -1. Excel's value is the
        // run-on secant's, bit for bit, on all 130 cells of a fresh sweep where it converges
        // (probes/t4838-irr-pole-runon, predictions generated from its seed before the fill) and on 65 of
        // t2927-irr-fallback-sweep; financial.md "IRR's secant runs on past the pole" (T-4888).
        let f2 = npv(d2)?;
        largest_npv = largest_npv.max(f2.abs());
        // A zero step also stops when |NPV| is under 1e-4: on the 2^-53 grid
        // `IRR({-2500000,-1000000,123456789123000})` cannot get NPV under 1e-7 (1.2E-6 at
        // Excel's 7026.083663633989), yet Excel returns it (probes/t2565-irr-guess-sweeps
        // A107-C141). T-3120 found any bound from 1E-20 to 1E-7 of the largest flow, or an absolute
        // one from 1E-5 to 0.1, scores the same there. The bound is absolute, RATE's 1e-4 for a
        // held flat secant ("RATE holds on a flat secant"): a relative one returns
        // gen-m9/fec39efaba9284c3 Calc!A283, whose run on past the pole (T-4888) stalls at
        // |NPV| = 0.2 (1.6E-9 of its largest flow) where no root exists and Excel is `#NUM!`.
        // Excel's other stops are absolute too (financial.md "IRR's second stage ignores the guess").
        // Past [`IRR_SHORT_RUN_STEPS`] a stalled zero step is no stop: the run goes flat on its
        // next step and fails. Excel is `#NUM!` on all 6 stalled stops in eval and Gate A between the
        // 41st step and its cap; all of them lie outside [`IRR_LONG_RUN_RANGE`] (T-3408).
        let stalled = d2 == d1 && irr_stall_stops(f2.abs(), peak) && step < IRR_SHORT_RUN_STEPS;
        if (d2 - d1).abs() < 1e-7 && (f2.abs() < 1e-7 || stalled) {
            // A root past the pole (v <= 0, r <= -1) is no result: the run fails and the call goes
            // on to the retry and the fallback (t4838-irr-pole-runon A118, A131 and A144 converge
            // there, and Excel returns a root above -1).
            if d2 >= 1.0 {
                return Err(SecantStop::Failed);
            }
            return irr_from_discount(1.0 - d2).ok_or(SecantStop::Failed);
        }
        (d0, f0, d1, f1) = (d1, f1, d2, f2);
    }
    Err(SecantStop::Failed)
}

/// Why a run of [`irr_secant`] stopped without a root.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SecantStop {
    /// No root: the call retries from 0.1, then falls back ([`compute_irr`]).
    Failed,
    /// An iterate reached v = 0, or NPV could not be evaluated: the call is `#DIV/0!`.
    DivZero,
}

/// The |NPV| under which a zero step (d2 = d1) stops an IRR secant run that has not reached the
/// 1e-7 residual stop, with [`IRR_STALL_RELATIVE`]: Excel holds a stalled step only at the flows'
/// rounding noise, both under 1e-4 absolute and under 1e-12 of the largest flow (financial.md
/// "IRR's secant runs on past the pole", T-4888). 1e-4 is RATE's bound for a held flat secant,
/// frozen before the fresh sweep T-4888's own fresh sweep was drawn: probes/t4888-irr-stall-sweep (199
/// stalls over largest flows 1E-9..1E+14) brackets it between 8.75E-5 (held) and 1.28E-4 (not).
/// Gate A's tables, which no fit reads, agree: 16 accuracy/IRR points of flows near 1E+11..1E+14
/// move into the envelope with it.
/// FITTED(T-4888, 199 cells, IRR zero-step stalls of 3..40 flows, largest |c| 1.3E-7..1.3E+14 (t2565 1.23E+14 rounded out); absolute bracket 8.75E-5..1.28E-4, relative bracket 2.8E-16..1.6E-9)
const IRR_STALL_RESIDUAL: f64 = 1e-4;

/// [`IRR_STALL_RESIDUAL`]'s relative half: a stall stops only at |NPV| under this fraction of the
/// largest |flow|. The fresh stall sweep holds every stall at 2.8E-16 of the largest flow or less
/// (A147, A161) and rejects gen-m9 fec39efaba9284c3 Calc!A283's no-root flows times 10^k at
/// 1.6E-9 (A201:A204) and every stall far from a root on small flows (relative 0.95..1.05); T-3120's
/// t2565 cells need at least 1E-20. The round value in 2.8E-16..1.6E-9 is taken. Train/test (fit
/// lab split): the two-part rule matches Excel's hold-or-not on 132 of 135 and 63 of 64.
const IRR_STALL_RELATIVE: f64 = 1e-12;

/// The largest |flow| range the stall stop is evidenced on (t4888-irr-stall-sweep's stalls and
/// t2565-irr-guess-sweeps): outside it a zero step is no stop (decision 0188 R4.3).
const IRR_STALL_FLOWS: (f64, f64) = (1.3e-7, 1.3e14);

/// Whether a zero step with |NPV| = `residual` stops a run on flows whose largest |c| is `peak`.
fn irr_stall_stops(residual: f64, peak: f64) -> bool {
    let (smallest, largest) = IRR_STALL_FLOWS;
    (smallest..=largest).contains(&peak)
        && residual < IRR_STALL_RESIDUAL
        && residual < IRR_STALL_RELATIVE * peak
}

/// The smallest guess IRR refuses as too large: 2^53.
const IRR_GUESS_BOUND: f64 = 9_007_199_254_740_992.0;

/// IRR as Excel computes it: a secant on NPV in d = 1 - v = r/(1 + r), v = 1/(1 + r), from
/// d0 = guess/(1 + guess) and d1 = d0 -+ 0.001 (toward larger |NPV|), each step anchored on the
/// newer iterate. It stops at the first iterate with |d2 - d1| < 1e-7 and |NPV| < 1e-7 (or a zero
/// step, see [`irr_secant`]) and returns 1/(1 - d2) - 1; a run that fails is retried once from 0.1.
/// Leading zero flows are dropped first. This matches all 263 cells of the fit set
/// (probes/solver-iteration-traces sheet I, probes-numeric/m9-fin-irr-rate; fit lab `irr`,
/// `irr_secant_d`, T-3120, T-3205) and 180 of the 181 of probes/t2565-irr-guess-sweeps.
/// Not Excel facts, kept from the fit: the d >= 1 (v <= 0) guard, no retry when the guess is
/// already 0.1, and `#NUM!` for flows without a sign change. Where both runs fail,
/// [`irr_fallback`] runs a Newton search from v = 0 (fitted, T-2727). A root with 1 + r at or past
/// [`IRR_FAR_ROOT_BOUND`] is `#NUM!` from either. A run's step cap is [`irr_secant_steps`]'s: 200
/// steps where Excel's longer runs are evidenced, 50 elsewhere, both directive-5 bounds
/// (financial.md "The IRR and XIRR solver caps").
pub(crate) fn compute_irr(values: &[f64], guess: f64) -> Result<f64, (Error, String)> {
    // Excel: a guess at or above 2^53 is #VALUE! too, before the sign-change check:
    // `IRR({-100,60,60},2^53-1)` and `(..,9E+15)` converge, `(..,2^53)` and `(..,1E+16)` are #VALUE!,
    // and so is `IRR(COUNTBLANK(..),1E+300)` (probes/irr-guess-edges A1-A19,
    // irr-guess-edges-2 A1-A14; financial.md "IRR's guess range").
    if guess <= -1.0 || guess >= IRR_GUESS_BOUND {
        return Err((
            Error::VALUE,
            "Rate initial guess must be > -1 and < 2^53".to_string(),
        ));
    }
    // The values cannot be all positive or all negative
    if values.iter().all(|&x| x >= 0.0) || values.iter().all(|&x| x <= 0.0) {
        return Err((Error::NUM, "Failed to converge".to_string()));
    }
    // Leading zero flows are dropped: `IRR({0,-100,110},0.05)` and `IRR({0,0,-100,110},10)` are
    // `IRR({-100,110},..)`'s 0.09999999999999987, not the secant's value on the full flows
    // (probes/irr-guess-edges A33, m9-fin-irr-rate A173 and A175; T-3120). For the fallback they
    // make v = 0 a root of P: `IRR({0,2,-1})` returns about -0.5 (gen-m9/3c06f9298f65a3ba A394).
    let values = values
        .iter()
        .position(|&c| c != 0.0)
        .map_or(values, |first| &values[first..]);
    let div_zero = || (Error::DIV, "Division by zero".to_string());
    let mut result = irr_secant(values, guess);
    // A failed run is retried once from the default guess 0.1, unless that was the guess
    if result == Err(SecantStop::Failed) && guess != 0.1 {
        result = irr_secant(values, 0.1);
    }
    match result {
        Ok(root) => Some(root),
        Err(SecantStop::DivZero) => return Err(div_zero()),
        Err(SecantStop::Failed) => None,
    }
    .or_else(|| irr_fallback(values))
    .filter(|r| r.is_finite())
    .ok_or_else(|| (Error::NUM, "Failed to converge".to_string()))
}

/// Where both secant runs fail, Excel still finds some roots and not others: it behaves as
/// Newton's method on the NPV polynomial in v, `sum c_i v^i`, started at v = 0 (r = +inf), and is
/// `#NUM!` when the root it reaches is not a positive discount factor, when a negative root lies
/// nearer 0 (P changes sign on [-v, 0)), when NPV at the root is [`IRR_ROOT_RESIDUAL`] or more
/// in magnitude, or when the run does not settle in [`IRR_FALLBACK_STEPS`] steps. This fits every probed
/// cell (financial.md "IRR's fallback when the secant fails", probes/irr-fallback-roots A1-A24, 15
/// gen-m9 cells): `{42,1,1,-1}` (-0.7467) and `{-400,1,1,1,1,1}` (-0.6737) return from every
/// guess while `{1,3,-1}`, `{1,7,-1}` and `{1,20,-1}`, whose root nearest v = 0 is negative, are
/// `#NUM!`; `{1E+15,-0.0001,1,-2.5,-0.0001}` (gen-m9/abb7d565af89b849 A254) needs 119 steps to
/// reach its root and is `#NUM!`. The method is fitted, not Excel's code: the returned digits are
/// 15 to 35 ulp from Excel's on the probed roots (T-1686, T-2131).
/// `values` has no leading zero flow ([`compute_irr`] drops them).
/// Range (T-3385, flow sets read from the cases): the probe's A1-A24 are 3 to 6 flows of |c| 0.3 to
/// 400 with roots -0.75 and 6.91; every gen-m9 IRR that reaches this fallback (a superset of the 15
/// cells) has 2 to 29 flows of |c| 1E-15 to 1E+15, and only 4 of its 68 calls return a root.
/// FITTED(T-2727, 39 cells, within IRR of 2..29 flows, |c| 1E-15..1E+15, roots -0.93..661, mostly #NUM!)
fn irr_fallback(values: &[f64]) -> Option<f64> {
    // Horner for P(v) and P'(v), highest power first
    let polynomial = |v: f64| {
        let (mut p, mut dp) = (0.0_f64, 0.0_f64);
        for &c in values.iter().rev() {
            dp = dp * v + p;
            p = p * v + c;
        }
        (p, dp)
    };
    let mut v = 0.0_f64;
    for _ in 0..IRR_FALLBACK_STEPS {
        let (p, dp) = polynomial(v);
        if dp == 0.0 {
            return None;
        }
        let next = v - p / dp;
        if !next.is_finite() {
            return None;
        }
        if (next - v).abs() < 1e-7 * next.abs().max(1.0) {
            // A sign change of P on [-next, 0) is a negative root nearer v = 0 than the one
            // Newton reached: Excel is `#NUM!` there (gen-m9/1b60bf99d71424e7 A117, root v = 8.41
            // past one at -4.6E-5; 1cd3096e3e87c965 A277, 4.06 past -4.01).
            let nearer_negative_root =
                (polynomial(-next).0 > 0.0) != values.first().is_some_and(|&c| c > 0.0);
            // Excel is `#NUM!` at a root whose NPV is not within [`IRR_ROOT_RESIDUAL`] of 0,
            // measured one Newton step past the stop (T-3411 round 4: at the stop iterate itself
            // the check loses probes-numeric/t3478-irr-r4-wide A79 and A292, whose residuals fall
            // under the cut only one step later). The stop iterate is still what is returned.
            let (p_next, dp_next) = polynomial(next);
            let past = next - p_next / dp_next;
            let settled = !irr_root_residual_applies(values)
                || (past.is_finite()
                    && irr_npv(values, past).is_some_and(|f| f.abs() < IRR_ROOT_RESIDUAL));
            return if next > 0.0 && !nearer_negative_root && settled {
                irr_from_discount(next)
            } else {
                None
            };
        }
        v = next;
    }
    None
}

/// XNPV: the sum of `value_i / (1 + rate)^((date_i - date_0)/365)`, days counted from the first date;
/// `#NUM!` when the sum is not finite or a power is `#NUM!` for a base <= 0.
pub(crate) fn compute_xnpv(
    rate: f64,
    values: &[f64],
    dates: &[f64],
) -> Result<f64, (Error, String)> {
    let mut xnpv = values[0];
    let d0 = dates[0];
    let n = values.len();
    for i in 1..n {
        let vi = values[i];
        let di = dates[i];
        // Excel's own `^` (docs/excel-semantics/financial.md "XNPV discounts with Excel's `^`").
        // A positive base whose power overflows (`#NUM!`) still adds value/inf = 0, as `powf` did (unprobed, unchanged).
        match crate::arithmetic::power(1.0 + rate, (di - d0) / 365.0) {
            Ok(power) => xnpv += vi / power,
            Err(Error::NUM) if 1.0 + rate > 0.0 => {}
            Err(e) => return Err((e, "NaN".to_string())),
        }
    }
    if xnpv.is_infinite() || xnpv.is_nan() {
        return Err((Error::NUM, "NaN".to_string()));
    }
    Ok(xnpv)
}

/// XNPV at a trial rate of XIRR's search, or `None` where it has no value (`1+rate <= 0`, or the
/// sum overflows, or a power overflows or underflows): such a point ends a bracket walk.
fn xirr_xnpv(rate: f64, values: &[f64], dates: &[f64]) -> Option<f64> {
    let base = 1.0 + rate;
    if base <= 0.0 {
        return None;
    }
    let d0 = dates[0];
    let mut total = 0.0;
    for (&value, &date) in values.iter().zip(dates) {
        // As in the lab model, a power that overflows or underflows to 0 has no value (rather
        // than adding value/inf = 0): `XIRR({-1,1E+20,1},{45000,45365,63615},0.1)` is `#NUM!`.
        let power = crate::arithmetic::power(base, (date - d0) / 365.0).ok()?;
        if power == 0.0 {
            return None;
        }
        total += value / power;
    }
    total.is_finite().then_some(total)
}

/// The bracket XIRR bisects: the first point of a trial chain whose XNPV sign differs from
/// XNPV(0)'s (any nonzero XNPV when XNPV(0) is exactly 0), with the point before it. With `s`
/// the guess (1e-5 for a guess of 0) the chain is 0, s, 2s, 4s, ... for a guess >= 0, then, when
/// that finds nothing and `s` < 1, 0, -s and p → (p - 1)/2 toward -1; a negative guess walks
/// only 0, g, (g - 1)/2, ...
/// (docs/excel-semantics/financial.md "XIRR is a bracketing bisection, not Newton"; fit lab `xirr`, T-1525).
/// Excel's cap on the walk is unprobed: 100 points per walk is enough on every oracle cell.
/// A zero first value skips the walk: the bracket is [0, s] whatever the flows, even where XNPV is
/// flat or has no root there, and a negative guess has no bracket (probes/xirr-zero-first-bracket, T-2329).
fn xirr_bracket(values: &[f64], dates: &[f64], guess: f64) -> Option<(f64, f64)> {
    const WALK_CAP: usize = 100;
    let step = if guess == 0.0 { 1e-5 } else { guess };
    if values.first() == Some(&0.0) {
        return (guess >= 0.0).then_some((0.0, step));
    }
    let f0 = xirr_xnpv(0.0, values, dates)?;
    let walk = |first: f64, next: &dyn Fn(f64) -> f64| -> Option<(f64, f64)> {
        let (mut previous, mut point) = (0.0, first);
        for _ in 0..WALK_CAP {
            let fp = xirr_xnpv(point, values, dates)?;
            // An exact XNPV(0) = 0 (a root at 0) differs from any nonzero point, so the first
            // point brackets: `XIRR({-100,10,20,30,40},{0,1,59,60,61},G)` is G/2^k, not a
            // negative near 0 (probes/irr-guess-edges A41-A47, T-2728).
            if (fp > 0.0) != (f0 > 0.0) || (f0 == 0.0 && fp != 0.0) {
                return Some((previous, point));
            }
            (previous, point) = (point, next(point));
        }
        None
    };
    let toward_minus_one = |p: f64| (p - 1.0) / 2.0;
    if guess >= 0.0 {
        walk(step, &|p| p * 2.0).or_else(|| {
            if step < 1.0 {
                walk(-step, &toward_minus_one)
            } else {
                None
            }
        })
    } else {
        walk(guess, &toward_minus_one)
    }
}

/// XIRR as Excel computes it: not Newton, but a bisection of [`xirr_bracket`]. `a` starts at the
/// upper end; each step takes m = a + (b - a)/2 and replaces the upper end when
/// XNPV(m)·values[0] >= 0, else the lower one (an exact 0 does not stop the search). It returns m
/// once the half-width is below 5e-9 + 1e-8·|m| and, on a negative midpoint only, |XNPV(m)| is
/// below 2e-8·max|value| (no XNPV bound when the first value is 0). Every positive root in the
/// oracle cells stops at the first step-test pass and every negative one fits that bound (T-3268),
/// so the fitted rule matches all 227 cells of the fit set (probes/solver-iteration-traces sheet X,
/// probes-numeric/m9-fin-irr-rate) and 80 of 81 out of sample, the miss being the input check of
/// xirr-early-dates A7 (fit lab `xirr`, `xirr-oos`; T-1525, T-1583, T-3268).
/// Excel's bisection cap is unprobed: 200 steps is enough on every oracle cell.
/// Range (T-3385): the 267 are the numeric cells of the fit lab's `xirr` and `xirr-oos` sweeps (308
/// cells, 278 numeric) whose first value is nonzero: 2 to 6 flows over 0.5 to 124 years, |value| 5E-6 to 1.4E+21,
/// roots -0.93 to 22.05 (184 positive, 83 negative).
/// FITTED(T-3268, 267 cells, 2..6 flows over 0.5..124 years, |value| 5E-6..1.4E+21, root -0.93..22.05)
pub(crate) fn compute_xirr(
    values: &[f64],
    dates: &[f64],
    guess: f64,
) -> Result<f64, (Error, String)> {
    const BISECTION_CAP: usize = 200;
    let no_result = || (Error::NUM, "Failed to converge".to_string());
    // A guess at or below -1 is `#NUM!`: `XIRR({-100,110},{39448,39508},-1)`
    // (probes/t3312-xirr-guess-dates-weibull A9, T-3312).
    if guess <= -1.0 {
        return Err((Error::NUM, "Rate initial guess must be > -1".to_string()));
    }
    // The values cannot be all positive or all negative
    if values.iter().all(|&x| x >= 0.0) || values.iter().all(|&x| x <= 0.0) {
        return Err(no_result());
    }
    let (first, second) = xirr_bracket(values, dates, guess).ok_or_else(no_result)?;
    let (mut upper, mut lower) = if first > second {
        (first, second)
    } else {
        (second, first)
    };
    let first_value = values[0];
    let largest = values.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    // `a` (here `upper`) is the end the midpoint is measured from
    for _ in 0..BISECTION_CAP {
        let m = upper + (lower - upper) / 2.0;
        let fm = xirr_xnpv(m, values, dates).ok_or_else(no_result)?;
        let half_width = (lower - upper).abs() / 2.0;
        if half_width < 5e-9 + 1e-8 * m.abs()
            && (m >= 0.0 || first_value == 0.0 || fm.abs() < 2e-8 * largest)
        {
            return Ok(m);
        }
        if fm * first_value >= 0.0 {
            upper = m;
        } else {
            lower = m;
        }
    }
    Err(no_result())
}

#[cfg(test)]
mod tests {
    use super::{compute_irr, irr_secant, irr_secant_steps, SecantStop};

    /// t3347-irr-conventional-holdout P!A138: -11890081760.62, then 40 flows of 13.03.
    fn a138() -> Vec<f64> {
        let mut flows = vec![-11_890_081_760.62];
        flows.extend(std::iter::repeat_n(13.03, 40));
        flows
    }

    #[test]
    fn the_flat_secant_div_zero_cut_applies_only_to_a_run_from_the_default_guess() {
        // From 0.1 the run jumps far (|NPV| 1.8E+278), stalls and goes flat: `#DIV/0!` (T-3347).
        assert_eq!(irr_secant(&a138(), 0.1), Err(SecantStop::DivZero));
        // The same flat stall from guesses 0.05 and 0.2 is outside the evidence (decision 0188
        // R4.3): the run fails as before, and the call goes on to its retry from 0.1.
        assert_eq!(irr_secant(&a138(), 0.05), Err(SecantStop::Failed));
        assert_eq!(irr_secant(&a138(), 0.2), Err(SecantStop::Failed));
    }

    #[test]
    fn the_long_secant_cap_applies_only_inside_its_evidence() {
        // T-3408: 200 steps on 4..54 flows (4 and 5 since T-4888) of largest |c| 5.6E-9..2.2E+8 and a
        // guess -0.99934..9.81E+7.
        let flows = [-5170.93, 288.87, 570.22, 565.46, 610.52, 231.19];
        assert_eq!(irr_secant_steps(&flows, -0.99), 200);
        assert_eq!(irr_secant_steps(&flows, -0.9995), 50);
        assert_eq!(irr_secant_steps(&flows[..5], -0.99), 200);
        assert_eq!(irr_secant_steps(&flows[..4], -0.99), 200);
        assert_eq!(irr_secant_steps(&flows[..3], -0.99), 50);
        assert_eq!(
            irr_secant_steps(&[-3e8, 1e8, 2.5e8, 1.0, 1.0, 1.0], 0.1),
            50
        );
        assert_eq!(irr_secant_steps(&[-1.0; 55], 0.1), 50);
    }

    #[test]
    fn the_secant_runs_on_past_the_pole() {
        // probes/t4838-irr-pole-runon (T-4888): from 0.1 the secant steps past d = 1 (v < 0) and
        // carries on; Excel's value is the run-on secant's, bit for bit.
        let a1 = [0.344655, -57.62, 2.00041, 0.326446, -0.900787, 0.129903];
        assert_eq!(compute_irr(&a1, 0.1), Ok(166.14691657983454));
        let a71 = [176.028, -5.65972, 40.0811, -0.336625];
        assert_eq!(compute_irr(&a71, 0.1), Ok(-0.9915940340372632));
        // A113 needs 197 steps on 4 flows: the long-run cap covers 4 and 5 flows too.
        let a113 = [254.447, 36.7029, 36.6416, -1.26381];
        assert_eq!(compute_irr(&a113, 0.1), Ok(-0.9668615752328438));
    }

    #[test]
    fn a_root_past_the_pole_fails_the_run() {
        // A118 converges past the pole, to v < 0 (r = -5.38): no result, the run fails.
        let a118 = [-1.40865, 9.01886, 66.3605, 0.283603, -2.50754, -29.2498];
        assert_eq!(irr_secant(&a118, 0.1), Err(SecantStop::Failed));
    }

    #[test]
    fn a_stall_far_from_zero_npv_is_no_stop() {
        // gen-m9/fec39efaba9284c3 Calc!A283: no root; the run creeps to d = 1, where NPV levels off
        // at the first flow, 0.2, and stalls there. An absolute 1e-4 stall bound rejects it (a
        // relative one, 1e-7 of the 1.2E+8 flow, did not); Excel is `#NUM!`.
        let flows = [
            0.2,
            -0.0001,
            1e-15,
            -1.0,
            0.1,
            0.5,
            0.2,
            1.005,
            2.5,
            123456789.123,
            -2.5,
            2.5,
            0.2,
        ];
        assert!(compute_irr(&flows, 0.1).is_err());
    }
}
