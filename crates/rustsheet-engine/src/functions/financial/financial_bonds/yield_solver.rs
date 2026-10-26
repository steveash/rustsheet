//! YIELD's Newton as Excel runs it ([`yield_start`], [`excel_yield_newton`]), and the generic
//! yield solver ODDFYIELD (and YIELD where that Newton gives no value) wraps around a price
//! function: Newton with a central-difference derivative from the coupon rate, then bisection.

/// The yield YIELD's Newton starts from with more than one coupon left, `years` being the
/// settlement-to-maturity term in years:
/// `(100 rate + (100 - pr)/T) / ((100 + pr)/2 - (100 - pr)/(4T))`. Fitted (T-1592) on the starts
/// two analytic-Newton steps back from Excel's values on probes/yield-start-one-input: the
/// denominator is `pr + w (100 - pr)` with `(1/2 - w) T` = 0.25000 on every row whose values sit
/// below PRICE's root (rows 22-96 but 67: prices 90.5 to 99.5, rates 0 to 0.12, terms 2.3 to
/// 30.3 years), and it moves with neither the redemption nor the frequency (financial.md "YIELD
/// stops short of PRICE's root").
pub(super) fn yield_start(rate: f64, pr: f64, years: f64) -> f64 {
    (100.0 * rate + (100.0 - pr) / years) / ((100.0 + pr) / 2.0 - (100.0 - pr) / (4.0 * years))
}

/// Newton on `price(y) - target` with the analytic slope `dprice`, from `start`: each step is
/// `(price(y) - target) / dprice(y)`, and the first step below 1e-10 is not taken, so the value
/// is the iterate before it, short of the root (T-1592: 240 of the lab's 370 YIELD cells
/// bit-exact and 361 within 4 ulp on `^` prices; the cut lies between 9.67e-11, a step
/// yield-start-sweep C18 does not take, and 1.01e-10, one yield-start-one-input B21 takes). `None`
/// when an iterate leaves PRICE's domain (a non-finite price or slope) or after 100 steps.
pub(super) fn excel_yield_newton(
    target: f64,
    start: f64,
    price: impl Fn(f64) -> f64,
    dprice: impl Fn(f64) -> f64,
) -> Option<f64> {
    let mut y = start;
    for _ in 0..100 {
        let step = (price(y) - target) / dprice(y);
        if !step.is_finite() {
            return None;
        }
        if step.abs() < 1e-10 {
            return Some(y);
        }
        y -= step;
    }
    None
}

pub(super) fn solve_yield<F>(target_price: f64, rate: f64, price_fn: F) -> Result<f64, String>
where
    F: Fn(f64) -> Option<f64>,
{
    let obj = |y: f64| price_fn(y).map(|p| p - target_price);

    let h = 1e-5;
    let eps = 1e-8;

    let mut y = rate;
    for _ in 0..100 {
        let fv = match obj(y) {
            Some(v) if v.is_finite() => v,
            _ => break,
        };
        let fp = match (obj(y + h), obj(y - h)) {
            (Some(a), Some(b)) => (a - b) / (2.0 * h),
            _ => break,
        };
        if !fp.is_finite() || fp.abs() < 1e-15 {
            break;
        }
        let yn = y - fv / fp;
        if yn <= -1.0 {
            break;
        }
        if (yn - y).abs() < eps {
            return Ok(yn);
        }
        y = yn;
    }

    // Bisection fallback
    let (x1, x2) = (-0.9999_f64, 100.0_f64);
    let f1 = obj(x1).ok_or("bisection failed")?;
    let f2 = obj(x2).ok_or("bisection failed")?;
    if f1 * f2 > 0.0 {
        return Err("yield solver: failed to converge".to_string());
    }
    let (mut lo, mut hi) = if f1 < 0.0 { (x1, x2) } else { (x2, x1) };

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = obj(mid).ok_or("bisection failed")?;
        if fm.abs() < eps || (hi - lo).abs() < 1e-12 {
            return Ok(mid);
        }
        let f_lo = obj(lo).ok_or("bisection failed")?;
        if f_lo * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Err("yield solver: failed to converge".to_string())
}
