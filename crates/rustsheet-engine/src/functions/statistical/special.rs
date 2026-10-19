//! Standard-normal kernels, bisection helpers and ln-gamma shared by NORM*, LOGNORM*, GAUSS, PHI,
//! Z.TEST and GAMMALN. The error functions are Excel's own kernels in [`super::erf`].

use std::f64::consts::FRAC_1_SQRT_2;

use super::erf::erfc;
use crate::arithmetic::{extended_product, extended_quotient, extended_sum};

/// sqrt(2*pi), the normal density's normalising constant.
pub(crate) const SQRT_2PI: f64 = 2.506_628_274_631_000_7;

/// Normal cumulative distribution function with the given mean and standard deviation, through
/// the identities Excel's own functions obey (docs/excel-semantics/statistics.md, T-1778):
/// NORM.DIST(x,m,s,TRUE) = NORM.S.DIST((x-m)/s,TRUE) and NORM.S.DIST(z,TRUE) =
/// ERFC.PRECISE(d)/2 with d = fl53(fl64(-z*fl(sqrt(0.5)))), the product rounded twice
/// (`kmyst-erfc-r9side4`, 600 of 600 against 9 for a single rounding; `NORM_DIST.xlsx`,
/// `LOGNORM.xlsx`). The quotient and difference round like the worksheet operators.
pub(crate) fn normal_cdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let z = extended_quotient(extended_sum(x, -mean), std_dev);
    0.5 * erfc(extended_product(-z, FRAC_1_SQRT_2))
}

/// Normal probability density function with the given mean and standard deviation.
pub(crate) fn normal_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let d = (x - mean) / std_dev;
    (-0.5 * d * d).exp() / (SQRT_2PI * std_dev)
}

/// Standard normal density, shared by PHI and NORM.S.DIST. This operation order reproduces Excel's
/// cached last digits in PHI.xlsx and `NORM.S.DIST(-37.6,FALSE)` (4.0441448093482273E-308,
/// probes/subnormal-text-and-tail-companions); `normal_pdf`'s statrs-style order does not.
pub(crate) fn standard_normal_pdf(x: f64) -> f64 {
    (-(x * x) / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Smallest `x >= 0` with `f(x) >= target` for a non-decreasing `f`, found by doubling a bracket
/// (at most `max_doublings` times, else `INFINITY`) and then halving it (at most 200 times, the
/// loop stops early once the midpoint no longer moves). The time is bounded, unlike `statrs`'
/// `inverse_cdf`, which hangs or panics for extreme arguments (T-1530, T-1531).
pub(crate) fn bisect_increasing(f: impl Fn(f64) -> f64, target: f64, max_doublings: u32) -> f64 {
    let mut hi = 1.0_f64;
    let mut doublings = 0;
    while f(hi) < target {
        hi *= 2.0;
        doublings += 1;
        if doublings > max_doublings {
            return f64::INFINITY;
        }
    }
    halve_increasing(&f, target, 0.0, hi, 200)
}

/// Smallest `x` in [0, 1] with `f(x) >= target` for a non-decreasing `f` with `f(1) >= target`,
/// halving down to the smallest subnormal (at most 1100 steps), so a quantile as small as 1E-300
/// keeps its digits where [`bisect_increasing`]'s 200 steps stop near 1E-60. The time is bounded,
/// unlike `statrs`' `inverse_cdf` (T-1543).
pub(crate) fn bisect_unit_interval(f: impl Fn(f64) -> f64, target: f64) -> f64 {
    halve_increasing(&f, target, 0.0, 1.0, 1100)
}

/// Halves `[lo, hi]` (at most `steps` times) around the point where `f` reaches `target`.
fn halve_increasing(
    f: &impl Fn(f64) -> f64,
    target: f64,
    mut lo: f64,
    mut hi: f64,
    steps: u32,
) -> f64 {
    for _ in 0..steps {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        if f(mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// ln|Gamma(x)| from the fdlibm/musl `lgamma` (< 1 ulp, exactly 0 at x = 1 and x = 2).
///
/// Excel's cached GAMMALN values are the correctly rounded ones (T-0926 eval evidence:
/// `GAMMALN_GAMMALNPRECISE.xlsx`, e.g. GAMMALN(1) = 0, GAMMALN(4) = ln 6), which the Lanczos
/// approximation in `statrs` misses by a few ulp.
pub(crate) fn ln_gamma(x: f64) -> f64 {
    libm::lgamma(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 4.0 * f64::EPSILON * b.abs(), "{a} vs {b}");
    }

    #[test]
    fn normal_kernels() {
        close(normal_cdf(1.0, 0.0, 1.0), 0.841_344_746_068_542_9);
        close(normal_pdf(0.0, 0.0, 1.0), 0.398_942_280_401_432_7);
    }

    #[test]
    fn ln_gamma_exact_points() {
        assert_eq!(ln_gamma(1.0), 0.0);
        assert_eq!(ln_gamma(2.0), 0.0);
        assert_eq!(ln_gamma(4.0).to_bits(), 6.0_f64.ln().to_bits());
        // Gamma(1/2) = sqrt(pi)
        close(ln_gamma(0.5), 0.572_364_942_924_700_1);
        // ln(170!) = ln Gamma(171), a large argument
        close(ln_gamma(171.0), 706.573_062_245_787_5);
    }
}
