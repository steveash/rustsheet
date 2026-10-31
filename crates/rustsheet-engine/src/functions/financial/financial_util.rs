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
/// ([`annuity_factor_rounded`]; FV's once-rounded `annuity_factor` misses 4 of the 782 fit-lab
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
/// r1 = r0 - 0.001·sign(f(r0)), each step `r2 = r1 - f1·(r1-r0)/(f1-f0)` with Excel's
/// twice-rounded `-`, `*` and `/`. It stops at the first iterate with |r2 - r1| < 1e-7 and
/// |f(r2)| < 1e-7. The iterate of step [`RATE_MAX_STEPS`] is returned unconverged when it passes
/// a looser test: |r2 - r1|, |f(r2)| and 1 + r2 against 1e-4, all absolute
/// (probes/rate-cap-return-rule, rate-pmt-zero-cap, rate-step-cap A30). An
/// iterate at or below -1, a flat secant (f1 = f0), a residual Excel's `^` cannot form and a run
/// that has not stopped by then are `#NUM!`. Microsoft documents Newton's method and 20 steps;
/// neither fits. The fit, its scores and the rules not yet probed are in financial.md "RATE is a
/// secant on the FV residual" (T-2005).
pub(crate) fn compute_rate(
    pv: f64,
    fv: f64,
    nper: f64,
    pmt: f64,
    annuity_type: i32,
    guess: f64,
) -> Result<f64, Error> {
    // The start offset was chosen by scoring (T-2005; fit lab `rate_candidates` scores +-1E-3 and
    // 1E-4): on the 782 cells of the fit lab's `rate` set the model is bit-exact on all, with
    // nper 1..1000, |pmt| 1E-6..1E+6, pv -100..1E+7, guess -0.5..10 and rates -0.75..2.
    /// FITTED(T-2005, 782 cells, RATE nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2)
    const OFFSET: f64 = 0.001;
    // Two uses. The step stop is not a fit: Microsoft's RATE documentation gives 0.0000001 as the
    // convergence bound. The residual stop |f| < 1e-7 is not documented and was chosen by scoring
    // (financial.md "RATE is a secant on the FV residual": |f| < 1e-6 scores 188, < 1e-8 181), as
    // IRR's is (`irr_secant`).
    /// FITTED(T-2005, 782 cells, RATE nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2; the residual stop)
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
    if guess <= -1.0 {
        return Err(Error::VALUE);
    }
    let args = (nper, pmt, pv, fv);
    let period_start = annuity_type != 0;
    let residual = |rate: f64| rate_residual(rate, args, period_start).ok_or(Error::NUM);
    let (mut r0, mut f0) = (guess, residual(guess)?);
    let mut r1 = r0 - OFFSET.copysign(f0);
    let mut f1 = residual(r1)?;
    for step in 1..=RATE_MAX_STEPS {
        if f1 == f0 {
            return Err(Error::NUM);
        }
        let r2 = extended_sum(
            r1,
            -extended_quotient(
                extended_product(f1, extended_sum(r1, -r0)),
                extended_sum(f1, -f0),
            ),
        );
        if !r2.is_finite() || r2 <= -1.0 {
            return Err(Error::NUM);
        }
        let f2 = residual(r2)?;
        let step_size = (r2 - r1).abs();
        if step_size < TOLERANCE && f2.abs() < TOLERANCE {
            return Ok(r2);
        }
        if step == RATE_MAX_STEPS
            && step_size < RATE_CAP_TOLERANCE
            && f2.abs() < RATE_CAP_TOLERANCE
            && 1.0 + r2 > RATE_CAP_TOLERANCE
        {
            return Ok(r2);
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
/// or the sum is not finite: that ends the run.
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

/// The rate 1/v - 1 at a converged discount factor `v`, or `None` past [`IRR_FAR_ROOT_BOUND`].
fn irr_from_discount(v: f64) -> Option<f64> {
    let x = extended_quotient(1.0, v);
    (x < IRR_FAR_ROOT_BOUND).then_some(x - 1.0)
}

/// One run of IRR's secant from `guess`, or `None` when it fails (a flat secant, an iterate that
/// is not a positive discount factor, or no stop in 50 steps). The state is d = 1 - v = r/(1 + r),
/// so from v = 0.5 down the iterates' v = 1 - d sit on Excel's grid of multiples of 2^-53
/// (probes/t2565-irr-guess-sweeps), and each step is anchored on the newer iterate.
/// Chosen by scoring on the fit set (financial.md "IRR is a secant in d = 1 - v"): the start offset
/// of 1E-3 toward larger |NPV| (+1E-3 on both sides scores 121 of 263) and the 1E-7 residual stop
/// (1E-6 scores 188, 1E-8 180). Not fits: the 1E-7 step stop is Microsoft's documented 0.00001
/// percent, and `MAX_STEPS` is a directive-5 cap (financial.md "The IRR and XIRR solver caps").
/// FITTED(T-1533, 263 cells, IRR of 2..41 flows, |c| 1E-6..1E+9, guess -0.99..100, roots -0.99..99)
/// The zero-step stop's bound (below):
/// FITTED(T-3120, 36 cells, IRR of {-2.5,-1,123456789.123} over a guess sweep)
fn irr_secant(values: &[f64], guess: f64) -> Option<f64> {
    const MAX_STEPS: usize = 50;
    let npv = |d: f64| irr_npv(values, 1.0 - d);
    let largest_flow = values.iter().fold(0.0_f64, |m, c| m.max(c.abs()));
    let mut d0 = extended_quotient(guess, 1.0 + guess);
    let mut f0 = npv(d0)?;
    let mut d1 = d0 - if f0 > 0.0 { 1e-3 } else { -1e-3 };
    let mut f1 = npv(d1)?;
    for _ in 0..MAX_STEPS {
        if f1 == f0 {
            return None;
        }
        let d2 = d1 - extended_quotient(f1 * (d1 - d0), f1 - f0);
        if !d2.is_finite() || d2 >= 1.0 {
            return None;
        }
        let f2 = npv(d2)?;
        // A zero step also stops when NPV is within 1e-7 of the largest flow: on the 2^-53 grid
        // `IRR({-2500000,-1000000,123456789123000})` cannot get NPV under 1e-7 (1.2E-6 at
        // Excel's 7026.083663633989), yet Excel returns it (probes/t2565-irr-guess-sweeps
        // A107-C141). The bound is fitted: anything from 1E-20 to 1E-7 of the largest flow
        // scores the same, and an absolute residual from 1E-5 to 0.1 would too.
        let stalled = d2 == d1 && f2.abs() < 1e-7 * largest_flow;
        if (d2 - d1).abs() < 1e-7 && (f2.abs() < 1e-7 || stalled) {
            return irr_from_discount(1.0 - d2);
        }
        (d0, f0, d1, f1) = (d1, f1, d2, f2);
    }
    None
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
/// [`IRR_FAR_ROOT_BOUND`] is `#NUM!` from either. The 50-step cap is a directive-5 bound
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
    let first = irr_secant(values, guess);
    // A failed run is retried once from the default guess 0.1, unless that was the guess
    let result = if guess == 0.1 {
        first
    } else {
        first.or_else(|| irr_secant(values, 0.1))
    };
    result
        .or_else(|| irr_fallback(values))
        .filter(|r| r.is_finite())
        .ok_or_else(|| (Error::NUM, "Failed to converge".to_string()))
}

/// Where both secant runs fail, Excel still finds some roots and not others: it behaves as
/// Newton's method on the NPV polynomial in v, `sum c_i v^i`, started at v = 0 (r = +inf), and is
/// `#NUM!` when the root it reaches is not a positive discount factor, when a negative root lies
/// nearer 0 (P changes sign on [-v, 0)), or when the run does not settle in
/// [`IRR_FALLBACK_STEPS`] steps. This fits every probed
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
            return if next > 0.0 && !nearer_negative_root {
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
    if guess <= -1.0 {
        return Err((Error::VALUE, "Rate initial guess must be > -1".to_string()));
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
