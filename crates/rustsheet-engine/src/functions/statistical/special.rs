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

/// Whether Excel's normal density at the standardized `z` is `#NUM!` instead of its limit 0: it is
/// wherever the squared value leaves the double range, so from |z| = 2^512 on. `NORM.S.DIST(2^512,
/// FALSE)`, `PHI(1E+200)` and `NORM.DIST(3,1,2^-512,FALSE)` are `#NUM!`, while `SQRT(f64::MAX)` and
/// every double below 2^512 are 0 (probes/t2435-dist-extremes A42:A47, A50:A57, A82:A84, A119:A125;
/// `NORMDIST(1E+300,0.3,2.5,0)`, probes/t2475-stat-subexpressions P!A572).
/// Excel: see docs/excel-semantics/statistics.md, "Normal densities are #NUM! once the square
/// overflows (T-2435)"
pub(crate) fn normal_density_overflows(z: f64) -> bool {
    !(z * z).is_finite()
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

/// The most bracket expansions [`refine_increasing`] makes around an estimate; each multiplies the
/// step by 4, so the last one reaches 2^98 times the first step of 2^-30 relative.
const REFINE_MAX_EXPANSIONS: u32 = 64;

/// The crossing of a non-decreasing `f` with `target` within `[min, max]` (`0 <= min < max`): a
/// bracket `f(lo) < target <= f(hi)`, grown around `estimate` when it is finite and otherwise the
/// whole interval, is halved over the doubles' bit patterns (ordered like the values for
/// non-negative doubles) down to adjacent doubles, whose arithmetic mean (the even-significand end)
/// it returns, as T-1745 found Excel's quantiles to be crossings of the cdf (docs/excel-semantics/
/// statistics.md "BETA.INV solves Excel's own CDF to the crossing"). At most
/// [`REFINE_MAX_EXPANSIONS`] + 65 evaluations of `f`, whatever the interval.
pub(crate) fn refine_increasing(
    f: impl Fn(f64) -> f64,
    target: f64,
    estimate: f64,
    min: f64,
    max: f64,
) -> f64 {
    let (lo, hi) = crossing_bracket(&f, target, estimate, min, max);
    0.5 * (lo + hi)
}

/// [`refine_increasing`] without an estimate, returning whichever end of the final bracket has
/// `f` nearer `target` (the lower end on a tie): the correctly rounded crossing where `f` is
/// accurate, as Excel's converged BETA.INV search returns it (docs/excel-semantics/statistics.md
/// "BETA.INV at a huge shape is AS 109's Newton search (T-1538)"). At most 65 + 2 evaluations.
pub(crate) fn nearest_crossing(f: impl Fn(f64) -> f64, target: f64, min: f64, max: f64) -> f64 {
    nearest_crossing_from(f, target, f64::NAN, min, max)
}

/// [`nearest_crossing`] with the bracket grown around `estimate` when it is finite (as
/// [`refine_increasing`] does): the same final bracket for a monotone `f`, in fewer evaluations
/// when the estimate is close (about 25 within 2^-30 relative). At most
/// [`REFINE_MAX_EXPANSIONS`] + 65 + 2 evaluations.
pub(crate) fn nearest_crossing_from(
    f: impl Fn(f64) -> f64,
    target: f64,
    estimate: f64,
    min: f64,
    max: f64,
) -> f64 {
    let (lo, hi) = crossing_bracket(&f, target, estimate, min, max);
    if (f(hi) - target).abs() < (target - f(lo)).abs() {
        hi
    } else {
        lo
    }
}

/// The final bracket of [`refine_increasing`]: adjacent doubles `lo < hi` (or the interval's ends)
/// with `f(lo) < target <= f(hi)`.
fn crossing_bracket(
    f: &impl Fn(f64) -> f64,
    target: f64,
    estimate: f64,
    min: f64,
    max: f64,
) -> (f64, f64) {
    let (mut lo, mut hi) = (min, max);
    if estimate.is_finite() {
        // + 0.0 turns a -0.0 into +0.0, whose bit pattern orders with the non-negative doubles.
        let x0 = estimate.clamp(min, max) + 0.0;
        let mut step = (x0 * 2f64.powi(-30)).max(f64::MIN_POSITIVE);
        (lo, hi) = (x0, x0);
        // An expansion that never brackets the crossing leaves the rest of the interval on its
        // side to the halving.
        if f(x0) < target {
            hi = max;
            for _ in 0..REFINE_MAX_EXPANSIONS {
                let next = (lo + step).min(max);
                if next >= max || f(next) >= target {
                    hi = next;
                    break;
                }
                lo = next;
                step *= 4.0;
            }
        } else {
            lo = min;
            for _ in 0..REFINE_MAX_EXPANSIONS {
                let next = (hi - step).max(min);
                if next <= min || f(next) < target {
                    lo = next;
                    break;
                }
                hi = next;
                step *= 4.0;
            }
        }
    }
    // Each halving at least halves the bit-pattern gap of at most 2^63, so this ends in 64 steps.
    let (mut lo_bits, mut hi_bits) = (lo.to_bits(), hi.to_bits());
    while hi_bits - lo_bits > 1 {
        let mid_bits = lo_bits + (hi_bits - lo_bits) / 2;
        if f(f64::from_bits(mid_bits)) < target {
            lo_bits = mid_bits;
        } else {
            hi_bits = mid_bits;
        }
    }
    (f64::from_bits(lo_bits), f64::from_bits(hi_bits))
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

    #[test]
    fn refine_increasing_ends_at_adjacent_doubles_of_the_crossing() {
        use std::cell::Cell;
        // x^3 crosses 2 at the cube root of 2; the result is one of the two doubles around it,
        // from a far estimate, a close one, or none (the whole range), in at most 129 evaluations.
        let root = 2f64.cbrt();
        for estimate in [1e-300, 1.25, root, 1e300, f64::NAN] {
            let calls = Cell::new(0);
            let f = |x: f64| {
                calls.set(calls.get() + 1);
                x * x * x
            };
            let x = refine_increasing(f, 2.0, estimate, 0.0, f64::MAX);
            assert!(x.to_bits().abs_diff(root.to_bits()) <= 1, "{estimate}: {x}");
            assert!(calls.get() <= 129, "{estimate}: {}", calls.get());
        }
        // A target the function never reaches ends at the upper bound.
        assert_eq!(refine_increasing(|x| x, 2.0, 0.5, 0.0, 1.0), 1.0);
    }

    #[test]
    fn nearest_crossing_is_the_end_nearer_the_target() {
        // 3 * (1/3 rounded) is exactly 1, so the upper end of the final bracket hits the target.
        assert_eq!(nearest_crossing(|x| 3.0 * x, 1.0, 0.0, 1.0), 1.0 / 3.0);
        // A jump at 1/2 from 1/2 to 3/2: 0.75 is nearer f just below the jump, so the lower end.
        let below = f64::from_bits(0.5f64.to_bits() - 1);
        let jump = |x: f64| if x < 0.5 { x } else { x + 1.0 };
        assert_eq!(nearest_crossing(jump, 0.75, 0.0, 1.0), below);
        assert_eq!(nearest_crossing(jump, 1.4, 0.0, 1.0), 0.5);
    }
}
