//! Accurate error function and standard-normal kernels shared by ERF*, NORM*, LOGNORM*, GAUSS, PHI
//! and Z.TEST.
//!
//! Excel's cached results for these functions equal the correctly rounded mathematical values
//! (T-0924 eval evidence: `ERFs.xlsx`, `GAUSS.xlsx`, `NORM_DIST.xlsx`); `statrs`' erf is only
//! accurate to ~1e-11. Series for |x| < 1 and a continued fraction beyond keep every term
//! positive or monotonically decreasing, so rounding error stays within a few ulp.

use std::f64::consts::{FRAC_2_SQRT_PI, PI, SQRT_2};

/// Largest |x| for which `erfc` is not zero in f64 (erfc(27.3) underflows to 0).
const ERFC_UNDERFLOW: f64 = 27.3;

/// Maclaurin series of erf, used for |x| < 1 where the alternating terms decrease monotonically.
fn erf_series(x: f64) -> f64 {
    const MAX_TERMS: usize = 40;
    let x2 = x * x;
    let mut terms = [0.0_f64; MAX_TERMS];
    let mut power = x; // (-1)^n x^(2n+1) / n!
    let mut used = 0;
    for (n, slot) in terms.iter_mut().enumerate() {
        let k = n as f64;
        if n > 0 {
            power *= -x2 / k;
        }
        *slot = power / (2.0 * k + 1.0);
        used = n + 1;
        if slot.abs() < 1e-18 * x.abs() {
            break;
        }
    }
    // Summing smallest-first keeps the rounding error of the final sum below 1 ulp.
    let sum: f64 = terms[..used].iter().rev().sum();
    FRAC_2_SQRT_PI * sum
}

/// Continued fraction for erfc(x), x >= 1, evaluated bottom-up: each step contracts earlier
/// rounding errors, unlike forward (Lentz) evaluation which accumulates them.
fn erfc_cf(x: f64) -> f64 {
    // erfc(x) = exp(-x^2)/sqrt(pi) * 1/(x + (1/2)/(x + 1/(x + (3/2)/(x + ...))))
    // Depth needed for double precision grows as ~1/x^2 (about 190 forward terms at x = 1).
    let depth = (600.0 / (x * x)).clamp(40.0, 600.0) as usize;
    let mut t = x;
    for k in (1..=depth).rev() {
        t = x + (k as f64 / 2.0) / t;
    }
    // exp(-x^2) with x^2 split exactly into hi + lo so the rounding of x*x is not amplified.
    let hi = x * x;
    let lo = x.mul_add(x, -hi);
    (-hi).exp() * (1.0 - lo) / (PI.sqrt() * t)
}

/// The error function erf(x).
pub(crate) fn erf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let a = x.abs();
    let r = if a < 1.0 {
        return erf_series(x);
    } else if a > 6.0 {
        1.0
    } else {
        1.0 - erfc_cf(a)
    };
    r.copysign(x)
}

/// The complementary error function erfc(x) = 1 - erf(x), accurate in the tails.
pub(crate) fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x < 1.0 {
        // For x <= -1 erfc is within 2 ulp of 2 - erfc(-x); for |x| < 1 erf has no cancellation.
        return if x <= -1.0 {
            2.0 - erfc(-x)
        } else {
            1.0 - erf_series(x)
        };
    }
    if x > ERFC_UNDERFLOW {
        return 0.0;
    }
    erfc_cf(x)
}

/// sqrt(2*pi), the normal density's normalising constant.
pub(crate) const SQRT_2PI: f64 = 2.506_628_274_631_000_7;

// The operation order below is the one that reproduces Excel's cached last-digit results in
// `NORM_DIST.xlsx` / `LOGNORM.xlsx` (it is the textbook form; alternatives lose exact matches).

/// Normal cumulative distribution function with the given mean and standard deviation.
pub(crate) fn normal_cdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    0.5 * erfc((mean - x) / (std_dev * SQRT_2))
}

/// Normal probability density function with the given mean and standard deviation.
pub(crate) fn normal_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let d = (x - mean) / std_dev;
    (-0.5 * d * d).exp() / (SQRT_2PI * std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 4.0 * f64::EPSILON * b.abs(), "{a} vs {b}");
    }

    // Reference values from Python's `math.erf`/`math.erfc` (correctly rounded libm).
    #[test]
    fn erf_reference_values() {
        close(erf(1.0), 0.842_700_792_949_714_9);
        close(erf(0.5), 0.520_499_877_813_046_5);
        close(erf(-2.0), -0.995_322_265_018_952_7);
        close(erf(0.0), 0.0);
        assert_eq!(erf(7.0), 1.0);
    }

    #[test]
    fn erfc_reference_values() {
        close(erfc(5.0), 1.537_459_794_428_035e-12);
        close(erfc(1.0), 0.157_299_207_050_285_13);
        close(erfc(-1.0), 1.842_700_792_949_715);
        assert_eq!(erfc(30.0), 0.0);
        assert_eq!(erfc(-30.0), 2.0);
    }

    #[test]
    fn nan_and_series_cf_boundary() {
        assert!(erf(f64::NAN).is_nan());
        assert!(erfc(f64::NAN).is_nan());
        // erf/erfc switch algorithm at |x| = 1; both sides must agree with erf + erfc = 1.
        for x in [0.999_999, 1.0, 1.000_001] {
            assert!((erf(x) + erfc(x) - 1.0).abs() < 4.0 * f64::EPSILON);
        }
    }

    #[test]
    fn normal_kernels() {
        close(normal_cdf(1.0, 0.0, 1.0), 0.841_344_746_068_542_9);
        close(normal_pdf(0.0, 0.0, 1.0), 0.398_942_280_401_432_7);
    }
}
