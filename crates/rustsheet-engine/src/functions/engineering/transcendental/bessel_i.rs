use super::bessel_excel::{
    miller_stays_finite, EXCEL_MAX_ORDER, LN_COMPUTED_ZERO, MAX_ORDER, MILLER_ACCURACY,
    MILLER_RESCALE, MILLER_RESCALE_FACTOR,
};
use crate::functions::statistical::ln_gamma;
use crate::x87_exp::excel_exp;

/// The even order BESSELI's Miller recurrence starts from, `2 (n + sqrt(MILLER_ACCURACY n))`.
fn in_start_order(n: i32) -> f64 {
    2.0 * ((MILLER_ACCURACY * f64::from(n)).sqrt().trunc() + f64::from(n))
}

/// `BESSELI(x, n)` for `MAX_ORDER < n` without the recurrence: 0 where Excel's is provably 0, else
/// NaN. I0(x) overflows from x = 709.79, which makes Excel's result `#NUM!` at every order
/// (`BESSELI(713,100001)`, `(1E+4,100001)`); otherwise the bound
/// `I_n(x) <= (x/2)^n / n! * e^(x^2 / (4 (n + 1)))` (the series with `(n + k)! >= n! (n + 1)^k`)
/// below `LN_COMPUTED_ZERO` and a recurrence that stays finite give 0: `BESSELI(100,100001)`,
/// `(700,100001)`, `(8,1E+6)` and `(100,1E+7)` are 0 (probes/t2546-bessel-past-cap).
fn bessel_i_past_cap(n: i32, x: f64) -> f64 {
    if n > EXCEL_MAX_ORDER {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    let ax = x.abs();
    if !bessel_i0(ax).is_finite() {
        return f64::NAN;
    }
    let order = f64::from(n);
    let ln_bound =
        order * (ax / 2.0).ln() - ln_gamma(order + 1.0) + ax * ax / (4.0 * (order + 1.0));
    if ln_bound < LN_COMPUTED_ZERO && miller_stays_finite(in_start_order(n), 2.0 / ax) {
        0.0
    } else {
        f64::NAN
    }
}

// This are somewhat lower precision than the BesselJ and BesselY

// needed for BesselK
pub(crate) fn bessel_i0(x: f64) -> f64 {
    // Parameters of the polynomial approximation
    let p1 = 1.0;
    let p2 = 3.5156229;
    let p3 = 3.0899424;
    let p4 = 1.2067492;
    let p5 = 0.2659732;
    let p6 = 3.60768e-2;
    let p7 = 4.5813e-3;

    let q1 = 0.39894228;
    let q2 = 1.328592e-2;
    let q3 = 2.25319e-3;
    let q4 = -1.57565e-3;
    let q5 = 9.16281e-3;
    let q6 = -2.057706e-2;
    let q7 = 2.635537e-2;
    let q8 = -1.647633e-2;
    let q9 = 3.92377e-3;

    let k1 = 3.75;
    let ax = x.abs();

    if x.is_infinite() {
        return 0.0;
    }

    if ax < k1 {
        // let xx = x / k1;
        // let y = xx * xx;
        // let mut result = 1.0;
        // let max_iter = 50;
        // let mut term = 1.0;
        // for i in 1..max_iter {
        //     term = term * k1*k1*y /(2.0*i as f64).powi(2);
        //     result += term;
        // };
        // result

        let xx = x / k1;
        let y = xx * xx;
        p1 + y * (p2 + y * (p3 + y * (p4 + y * (p5 + y * (p6 + y * p7)))))
    } else {
        let y = k1 / ax;
        // Excel's own EXP: the platform exp is 2 ulps off `BESSELI(100.21..,5)` (engineering.md, T-2548).
        (excel_exp(ax) / (ax).sqrt())
            * (q1
                + y * (q2
                    + y * (q3 + y * (q4 + y * (q5 + y * (q6 + y * (q7 + y * (q8 + y * q9))))))))
    }
}

// needed for BesselK
pub(crate) fn bessel_i1(x: f64) -> f64 {
    let p1 = 0.5;
    let p2 = 0.87890594;
    let p3 = 0.51498869;
    let p4 = 0.15084934;
    let p5 = 2.658733e-2;
    let p6 = 3.01532e-3;
    let p7 = 3.2411e-4;

    let q1 = 0.39894228;
    let q2 = -3.988024e-2;
    let q3 = -3.62018e-3;
    let q4 = 1.63801e-3;
    let q5 = -1.031555e-2;
    let q6 = 2.282967e-2;
    let q7 = -2.895312e-2;
    let q8 = 1.787654e-2;
    let q9 = -4.20059e-3;

    let k1 = 3.75;
    let ax = x.abs();

    if ax < k1 {
        let xx = x / k1;
        let y = xx * xx;
        x * (p1 + y * (p2 + y * (p3 + y * (p4 + y * (p5 + y * (p6 + y * p7))))))
    } else {
        let y = k1 / ax;
        let result = (excel_exp(ax) / (ax).sqrt())
            * (q1
                + y * (q2
                    + y * (q3 + y * (q4 + y * (q5 + y * (q6 + y * (q7 + y * (q8 + y * q9))))))));
        if x < 0.0 {
            return -result;
        }
        result
    }
}

pub(crate) fn bessel_i(n: i32, x: f64) -> f64 {
    // The order cap bounds the downward recurrence below (about 2n steps).
    if n < 0 {
        return f64::NAN;
    }
    if n > MAX_ORDER {
        return bessel_i_past_cap(n, x);
    }

    if n == 0 {
        return bessel_i0(x);
    }
    if x == 0.0 {
        // I_n(0) = 0 for all n!= 0
        return 0.0;
    }
    if n == 1 {
        return bessel_i1(x);
    }

    // I_n(x) overflows long before 1e10: Excel is `#NUM!` for every order from x = 710 on, such as
    // `BESSELI(1E+15,2)` (probes/eng-besselk-underflow-bit-shift), so the overflow is returned.
    if x.abs() > MILLER_RESCALE {
        return f64::INFINITY;
    };
    in_miller(n, x)
}

/// BESSELI's Miller downward recurrence for `n >= 2`, `0 < |x| <= MILLER_RESCALE`, at any order
/// (the callers cap it).
pub(super) fn in_miller(n: i32, x: f64) -> f64 {
    let tox = 2.0 / x.abs();
    let mut bip = 0.0;
    let mut bi = 1.0;
    let mut result = 0.0;
    let m = in_start_order(n) as i32;

    for j in (1..=m).rev() {
        (bip, bi) = (bi, bip + (j as f64) * tox * bi);
        // Prevent overflow
        if bi.abs() > MILLER_RESCALE {
            result *= MILLER_RESCALE_FACTOR;
            bi *= MILLER_RESCALE_FACTOR;
            bip *= MILLER_RESCALE_FACTOR;
        }
        if j == n {
            result = bip;
        }
    }

    result *= bessel_i0(x) / bi;
    if (x < 0.0) && (n % 2 == 1) {
        result = -result;
    }

    result
}
