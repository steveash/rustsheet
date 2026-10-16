use crate::{arithmetic::extended_quotient, expressions::token::Error};

// The solvers behind RATE, IRR and XIRR. IRR (a secant in v = 1/(1+r)) and XIRR (a bracket walk
// and bisection) follow models fitted to oracle sweeps in the fit lab (T-1525, T-1533; see their
// doc comments). RATE is not fitted yet (financial.md "RATE iterates on the rate itself").

// From Microsoft docs:
// https://support.microsoft.com/en-us/office/rate-function-9f665657-4a7e-4bb7-a030-83fc59e748ce
// Returns the interest rate per period of an annuity.
// RATE is calculated by iteration (using Newton-Raphson) and can have zero or more solutions.
// If the successive results of RATE do not converge to within 0.0000001 after 20 iterations,
// RATE returns the #NUM! error value.
// NOTE: We need a better algorithm here
pub(crate) fn compute_rate(
    pv: f64,
    fv: f64,
    nper: f64,
    pmt: f64,
    annuity_type: i32,
    guess: f64,
) -> Result<f64, (Error, String)> {
    let mut rate = guess;
    // Excel _claims_ to do 20 iterations, but that will have tests failing
    let max_iterations = 50;
    let eps = 0.0000001;
    let annuity_type = annuity_type as f64;
    if guess <= -1.0 {
        return Err((Error::VALUE, "Rate initial guess must be > -1".to_string()));
    }
    for _ in 1..=max_iterations {
        let t = (1.0 + rate).powf(nper - 1.0);
        let tt = t * (1.0 + rate);
        let f = pv * tt + pmt * (1.0 + rate * annuity_type) * (tt - 1.0) / rate + fv;
        let f_prime = pv * nper * t - pmt * (tt - 1.0) / (rate * rate)
            + pmt * (1.0 + rate * annuity_type) * t * nper / rate;
        let new_rate = rate - f / f_prime;
        if new_rate <= -1.0 {
            return Err((Error::NUM, "Failed to converge".to_string()));
        }
        if (new_rate - rate).abs() < eps {
            return Ok(new_rate);
        }
        rate = new_rate;
    }
    Err((Error::NUM, "Failed to converge".to_string()))
}

pub(crate) fn compute_npv(rate: f64, values: &[f64]) -> Result<f64, (Error, String)> {
    let mut npv = 0.0;
    for (i, item) in values.iter().enumerate() {
        npv += item / (1.0 + rate).powi(i as i32 + 1);
    }
    Ok(npv)
}

/// NPV at the discount factor `v` = 1/(1 + r), as IRR's search evaluates it: the sum of
/// `c_i/x^i` with `x = 1/v`, each division Excel's twice-rounded `/` and `x^i` libm's `pow`
/// (financial.md "IRR is a secant in v = 1/(1 + r)"; fit lab `irr`, `_npv_v_over_x`).
fn irr_npv(values: &[f64], v: f64) -> f64 {
    let x = extended_quotient(1.0, v);
    let mut total = 0.0;
    let mut exponent = 0.0;
    for &c in values {
        total += extended_quotient(c, x.powf(exponent));
        exponent += 1.0;
    }
    total
}

/// One run of IRR's secant from `guess`, or `None` when it fails (a flat secant, an iterate that
/// is not a positive discount factor, or no stop in 50 steps).
fn irr_secant(values: &[f64], guess: f64) -> Option<f64> {
    const MAX_STEPS: usize = 50;
    let mut v0 = extended_quotient(1.0, 1.0 + guess);
    let mut f0 = irr_npv(values, v0);
    let mut v1 = v0 + if f0 > 0.0 { 1e-3 } else { -1e-3 };
    let mut f1 = irr_npv(values, v1);
    for _ in 0..MAX_STEPS {
        if f1 == f0 {
            return None;
        }
        let v2 = v0 - extended_quotient(f0 * (v1 - v0), f1 - f0);
        if !v2.is_finite() || v2 <= 0.0 {
            return None;
        }
        let f2 = irr_npv(values, v2);
        if (v2 - v1).abs() < 1e-7 && f2.abs() < 1e-7 {
            return Some(extended_quotient(1.0, v2) - 1.0);
        }
        (v0, f0, v1, f1) = (v1, f1, v2, f2);
    }
    None
}

/// IRR as Excel computes it: a secant on NPV(v), v = 1/(1 + r), from v0 = 1/(1 + guess) and
/// v1 = v0 ± 0.001 (the sign of NPV(v0)). It stops at the first iterate with |v2 - v1| < 1e-7 and
/// |NPV(v2)| < 1e-7 and returns 1/v2 - 1; a run that fails is retried once from 0.1. This matches
/// 189 of the 263 cells of the fit set (probes/solver-iteration-traces sheet I,
/// probes-numeric/m9-fin-irr-rate), 207 within 4 ulp (fit lab `irr`, `irr_secant_v`, T-1533).
/// Not Excel facts, kept from the fit: the v <= 0 guard, no retry when the guess is already 0.1,
/// and `#NUM!` for flows without a sign change.
pub(crate) fn compute_irr(values: &[f64], guess: f64) -> Result<f64, (Error, String)> {
    if guess <= -1.0 {
        return Err((Error::VALUE, "Rate initial guess must be > -1".to_string()));
    }
    // The values cannot be all positive or all negative
    if values.iter().all(|&x| x >= 0.0) || values.iter().all(|&x| x <= 0.0) {
        return Err((Error::NUM, "Failed to converge".to_string()));
    }
    let first = irr_secant(values, guess);
    // A failed run is retried once from the default guess 0.1, unless that was the guess
    let result = if guess == 0.1 {
        first
    } else {
        first.or_else(|| irr_secant(values, 0.1))
    };
    match result.filter(|r| r.is_finite()) {
        Some(r) => Ok(r),
        // Not an Excel fact: Excel returns a value where the fitted secant does not stop, e.g.
        // `IRR({-400,1,1,1,1,1})` = -0.6737366751808405 (ironcalc-calc IRR.xlsx General!L17), which
        // no step cap explains. Until that path is probed (T-1686), the earlier Newton and
        // bisection solver keeps a near value there instead of `#NUM!`.
        None => irr_fallback(values, guess),
    }
}

/// The solver IRR used before T-0916 (Newton's method from the guess, then a bisection on
/// [-0.99999, 100], then Newton from 200 and from -2). It only runs where the fitted secant fails.
fn irr_fallback(values: &[f64], guess: f64) -> Result<f64, (Error, String)> {
    if let Ok(f) = irr_newton_raphson(values, guess) {
        return Ok(f);
    };
    let max_iterations = 50;
    let eps = 1e-10;
    let x1 = -0.99999;
    let x2 = 100.0;
    let f1 = compute_npv(x1, values)?;
    let f2 = compute_npv(x2, values)?;
    if f1 * f2 > 0.0 {
        // The root is not within the limits or there are two roots
        if let Ok(f) = irr_newton_raphson(values, 200.0) {
            return Ok(f);
        };
        if let Ok(f) = irr_newton_raphson(values, -2.0) {
            return Ok(f);
        };
        return Err((Error::NUM, "Failed to converge".to_string()));
    }
    let (mut rtb, mut dx) = if f1 < 0.0 {
        (x1, x2 - x1)
    } else {
        (x2, x1 - x2)
    };
    for _ in 1..max_iterations {
        dx *= 0.5;
        let x_mid = rtb + dx;
        let f_mid = compute_npv(x_mid, values)?;
        if f_mid <= 0.0 {
            rtb = x_mid;
        }
        if f_mid.abs() < eps || dx.abs() < eps {
            return Ok(x_mid);
        }
    }
    Err((Error::NUM, "Failed to converge".to_string()))
}

fn npv_prime(rate: f64, values: &[f64]) -> Result<f64, (Error, String)> {
    let mut npv = 0.0;
    for (i, item) in values.iter().enumerate() {
        npv += -item * (i as f64 + 1.0) / (1.0 + rate).powi(i as i32 + 2);
    }
    if npv.is_infinite() || npv.is_nan() {
        return Err((Error::NUM, "NaN".to_string()));
    }
    Ok(npv)
}

fn irr_newton_raphson(values: &[f64], guess: f64) -> Result<f64, (Error, String)> {
    let mut irr = guess;
    let max_iterations = 50;
    let eps = 1e-8;
    for _ in 1..=max_iterations {
        let f = compute_npv(irr, values)?;
        let f_prime = npv_prime(irr, values)?;
        let new_irr = irr - f / f_prime;
        if (new_irr - irr).abs() < eps {
            // A rate <= -1 is a root of the NPV polynomial but not a real IRR
            if new_irr <= -1.0 {
                return Err((Error::NUM, "converged to invalid rate".to_string()));
            }
            return Ok(new_irr);
        }
        irr = new_irr;
    }
    Err((Error::NUM, "Failed to converge".to_string()))
}

/// XNPV: the sum of `value_i / (1 + rate)^((date_i - date_0)/365)`, days counted from the first date;
/// `#NUM!` when the sum is not finite.
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
        xnpv += vi / ((1.0 + rate).powf((di - d0) / 365.0));
    }
    if xnpv.is_infinite() || xnpv.is_nan() {
        return Err((Error::NUM, "NaN".to_string()));
    }
    Ok(xnpv)
}

/// XNPV at a trial rate of XIRR's search, or `None` where it has no value (`1+rate <= 0`, or the
/// sum overflows): such a point ends a bracket walk without a bracket.
fn xirr_xnpv(rate: f64, values: &[f64], dates: &[f64]) -> Option<f64> {
    if 1.0 + rate <= 0.0 {
        return None;
    }
    compute_xnpv(rate, values, dates).ok()
}

/// The bracket XIRR bisects: the first point of a trial chain whose XNPV sign differs from
/// XNPV(0)'s, with the point before it. With `s` the guess (1e-5 for a guess of 0) the chain is
/// 0, s, 2s, 4s, ... for a guess >= 0, then, when that finds nothing and `s` < 1, 0, -s and
/// p → (p - 1)/2 toward -1; a negative guess walks only 0, g, (g - 1)/2, ...
/// (docs/excel-semantics/financial.md "XIRR is a bisection"; fit lab `xirr`, T-1525).
/// Excel's cap on the walk is unprobed: 100 points per walk is enough on every oracle cell.
fn xirr_bracket(values: &[f64], dates: &[f64], guess: f64) -> Option<(f64, f64)> {
    const WALK_CAP: usize = 100;
    let f0 = xirr_xnpv(0.0, values, dates)?;
    let walk = |first: f64, next: &dyn Fn(f64) -> f64| -> Option<(f64, f64)> {
        let (mut previous, mut point) = (0.0, first);
        for _ in 0..WALK_CAP {
            let fp = xirr_xnpv(point, values, dates)?;
            if (fp > 0.0) != (f0 > 0.0) {
                return Some((previous, point));
            }
            (previous, point) = (point, next(point));
        }
        None
    };
    let toward_minus_one = |p: f64| (p - 1.0) / 2.0;
    if guess >= 0.0 {
        let step = if guess == 0.0 { 1e-5 } else { guess };
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
/// once the half-width is below 5e-9 + 1e-8·|m| and |XNPV(m)| is below
/// 2e-8·max|value|·max(1, years from the first date to the latest), with no XNPV bound when the
/// first value is 0. This matches all 227 numeric cells of the fit set (probes/solver-iteration-traces
/// sheet X, probes-numeric/m9-fin-irr-rate) and 21 of 22 out of sample (fit lab `xirr`,
/// `xirr-oos`; T-1525, T-1583). The span scaling rests on one long-span cell (T-1591 probes it).
/// Excel's bisection cap is unprobed: 200 steps is enough on every oracle cell.
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
    let d0 = dates[0];
    let latest = dates.iter().fold(d0, |m, &d| m.max(d));
    let span_years = ((latest - d0) / 365.0).max(1.0);
    // `a` (here `upper`) is the end the midpoint is measured from
    for _ in 0..BISECTION_CAP {
        let m = upper + (lower - upper) / 2.0;
        let fm = xirr_xnpv(m, values, dates).ok_or_else(no_result)?;
        let half_width = (lower - upper).abs() / 2.0;
        if half_width < 5e-9 + 1e-8 * m.abs()
            && (first_value == 0.0 || fm.abs() < 2e-8 * largest * span_years)
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
