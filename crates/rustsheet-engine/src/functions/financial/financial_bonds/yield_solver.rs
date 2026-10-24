//! The generic yield solver YIELD and ODDFYIELD wrap around their price functions: Newton with a
//! central-difference derivative from the coupon rate, then bisection.

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
