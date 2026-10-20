//! The Student-t distribution behind T.DIST, T.DIST.RT, T.DIST.2T, TDIST and T.TEST (T-0969).
//!
//! Excel's own algorithm is not known; this is the model the probes-numeric t sweep fits best
//! (T-0969 attempt 3, against the exact values of the fit lab's `t_cdf_reference`):
//! - one tail L = P(T > |t|) is computed, and every other value follows from it: T.DIST of a
//!   positive t is 1 - L, T.DIST.RT(x) of a negative x is 1 - L, T.DIST.2T is 2 L. In Excel
//!   `T.DIST.RT(x,n)` = `T.DIST(-x,n,TRUE)` wherever the sweep has both, and 1 - L is bit-exact
//!   on 118 of the 146 nonzero T.DIST cells against 113 for the correctly rounded CDF;
//! - with y = n/(n+t^2) and z = t^2/(n+t^2), L = `I_y(n/2, 1/2)`/2. Away from the deep tail
//!   (y > z) L is the exact value, rounded once: for an integer n up to [`SERIES_MAX_DF`] the
//!   finite series for integer df in double-double. In the deep tail (y <= z) Excel's L carries
//!   the error of exp((n/2) ln y): Cephes' power series with that power and a correctly rounded
//!   Gamma ratio reproduces it (`T.DIST.RT(1E6,10)` = 1.230468749943611E-56, F18 of `T_DIST.xlsx`).
//!
//! Every loop is bounded: [`SERIES_MAX_DF`] / 2 series terms, [`TAYLOR_TERMS`] Taylor terms and
//! [`MAX_POWER_SERIES_TERMS`] power-series terms a call.

use statrs::distribution::{ContinuousCDF, StudentsT};

use super::incomplete_beta::beta_reg_upper;
use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_mul, two_sum};

/// A double-double: an unevaluated sum `hi + lo` with |lo| <= ulp(hi)/2.
type Dd = (f64, f64);

/// Largest integer df that the double-double series ([`centre_series`], [`integer_ratio`]) sum:
/// about df/2 double-double steps a call.
const SERIES_MAX_DF: f64 = 1000.0;
/// Largest df whose centre takes Cephes' fraction; above it statrs' cdf (see [`upper_tail`]).
const FRACTION_MAX_DF: f64 = 1e7;
/// From this shape on, Gamma(a + 1/2)/Gamma(a) comes from its asymptotic series.
const ASYMPTOTIC_MIN_SHAPE: f64 = 85.0;
/// Cephes' `MACHEP`, 2^-53: the power series stops below it.
const MACHEP: f64 = 1.110_223_024_625_156_5e-16;
/// Cephes' `MAXLOG`, ln of the largest double.
const MAXLOG: f64 = 709.782_712_893_384;
/// Cap on the power-series terms: in the deep tail y <= 1/2, so 60 terms reach 2^-53.
const MAX_POWER_SERIES_TERMS: u32 = 200;
/// Terms of the sin and cos Taylor series in [`dd_atan`]: theta <= pi/2 and (pi/2)^40/40! is
/// far below 2^-106.
const TAYLOR_TERMS: u32 = 40;
/// Below this the double-double tail 1/2 - A/2 keeps fewer than 40 of its 106 bits.
const MIN_SERIES_TAIL: f64 = 1e-12;
/// 1/pi as a double-double.
const FRAC_1_PI_DD: Dd = (
    f64::from_bits(0x3FD4_5F30_6DC9_C883),
    f64::from_bits(0xBC76_B01E_C541_7056),
);
/// 1/sqrt(pi), rounded.
const FRAC_1_SQRT_PI: f64 = 0.564_189_583_547_756_3;
/// Coefficients of Gamma(a + 1/2)/(Gamma(a) sqrt(a)) in powers of 1/a, 1 - 1/(8a) +
/// 1/(128a^2) + 5/(1024a^3) - 21/(32768a^4) - 399/(262144a^5) + 869/(4194304a^6): the first
/// omitted term is below 2^-53 relative from a = 85 on.
const ASYMPTOTIC_RATIO: [f64; 7] = [
    1.0,
    -1.0 / 8.0,
    1.0 / 128.0,
    5.0 / 1024.0,
    -21.0 / 32768.0,
    -399.0 / 262_144.0,
    869.0 / 4_194_304.0,
];

/// The Student-t tails `(P(T < t), P(T > t))` for `n` degrees of freedom, both from the one tail
/// L = P(T > |t|) (module doc).
pub(crate) fn student_t_tails(t: f64, n: f64) -> (f64, f64) {
    if t.is_nan() || n.is_nan() {
        return (f64::NAN, f64::NAN);
    }
    if t == 0.0 {
        return (0.5, 0.5);
    }
    let tail = upper_tail(t.abs(), n);
    if t > 0.0 {
        (1.0 - tail, tail)
    } else {
        (tail, 1.0 - tail)
    }
}

/// L = P(T > t) for t > 0.
fn upper_tail(t: f64, n: f64) -> f64 {
    let tail = unflushed_upper_tail(t, n);
    // Main (statrs) returned 0 for a subnormal tail and `T.DIST.2T(...)` in gen-m9/3ec66461400faf2f
    // A15 is 0 in Excel; the graph oracle cannot tell 0 from a kept subnormal (decision 0042), so
    // the flush stays until a probe with a companion cell decides it.
    if tail < f64::MIN_POSITIVE {
        0.0
    } else {
        tail
    }
}

/// [`upper_tail`] before the subnormal flush.
fn unflushed_upper_tail(t: f64, n: f64) -> f64 {
    // Excel forms n + t^2 in doubles: where it rounds to n, y = 1 and L is exactly 1/2
    // (`T.DIST.RT(1E-10,3)` = `TDIST(1E-10,3,1)` = 0.5, probes-numeric t1048-tdist A27, B27).
    if n + t * t == n {
        return 0.5;
    }
    let (z, y) = beta_arguments(t, n);
    if y <= z {
        if let Some(ratio) = gamma_ratio(n) {
            return 0.5 * power_series(0.5 * n, y, ratio);
        }
    } else if let Some(inside) = centre_series(t, n) {
        let tail = dd_add((0.5, 0.0), (-0.5 * inside.0, -0.5 * inside.1)).0;
        if tail > MIN_SERIES_TAIL {
            return tail;
        }
    }
    if n > FRACTION_MAX_DF {
        // Above it Cephes' prefactor (n/2) ln y, y rounded, loses more than statrs' cdf does:
        // `T.DIST(2,1E10,TRUE)` = 0.977249868038323 (probes-numeric m9-t-edges) is 7 digits off
        // through the fraction, 9 through statrs (T-2126). Its lower tail at -t, not 1 - cdf.
        return StudentsT::new(0.0, 1.0, n).map_or(f64::NAN, |dist| dist.cdf(-t));
    }
    0.5 * beta_reg_upper(z, y, 0.5, 0.5 * n)
}

/// (t^2/(n+t^2), n/(n+t^2)) for t > 0, each rounded once from double-double arithmetic: forming
/// one as 1 minus the other loses every digit at a large n, where Excel stays within an ulp of
/// the exact value (`T.DIST(-2,1E6,TRUE)`, T-0969 attempt 2).
fn beta_arguments(t: f64, n: f64) -> (f64, f64) {
    // Past 1E150 t^2 (and its error term) leave the double range; z is then 1 to the last bit.
    if t > 1e150 {
        return (1.0, n / t / t);
    }
    let square = dd_mul((t, 0.0), (t, 0.0));
    let sum = dd_add(square, (n, 0.0));
    (dd_div(square, sum).0, dd_div((n, 0.0), sum).0)
}

/// Cephes' power series for `I_y(a, 1/2)` (`pseries` with b = 1/2), its prefactor as
/// `s * ratio * exp(a ln y)` with `ratio` = Gamma(a + 1/2)/(Gamma(a) Gamma(1/2)).
fn power_series(a: f64, y: f64, ratio: f64) -> f64 {
    let b = 0.5;
    let reciprocal = 1.0 / a;
    let mut u = (1.0 - b) * y;
    let first = u / (a + 1.0);
    let mut v = first;
    let mut term = u;
    let mut k = 2.0;
    let mut sum = 0.0;
    let stop = MACHEP * reciprocal;
    for _ in 0..MAX_POWER_SERIES_TERMS {
        if v.abs() <= stop {
            break;
        }
        u = (k - b) * y / k;
        term *= u;
        v = term / (a + k);
        sum += v;
        k += 1.0;
    }
    sum += first;
    sum += reciprocal;
    let exponent = a * y.ln();
    if exponent.abs() < MAXLOG {
        sum * ratio * exponent.exp()
    } else {
        (ratio.ln() + exponent + sum.ln()).exp()
    }
}

/// Gamma((n+1)/2)/(Gamma(n/2) Gamma(1/2)), rounded once where it can be: by [`integer_ratio`] for
/// an integer n up to [`SERIES_MAX_DF`] and by its asymptotic series from n/2 =
/// [`ASYMPTOTIC_MIN_SHAPE`]; by libm's `tgamma` for a fractional n below that (T.TEST's Welch df).
fn gamma_ratio(n: f64) -> Option<f64> {
    let a = 0.5 * n;
    if a >= ASYMPTOTIC_MIN_SHAPE {
        let inverse = 1.0 / a;
        let mut sum = (0.0, 0.0);
        for &coefficient in ASYMPTOTIC_RATIO.iter().rev() {
            sum = dd_add(dd_mul(sum, (inverse, 0.0)), (coefficient, 0.0));
        }
        return Some(dd_mul(dd_mul(sum, dd_sqrt((a, 0.0))), (FRAC_1_SQRT_PI, 0.0)).0);
    }
    if let Some(ratio) = integer_ratio(n) {
        return Some(ratio.0);
    }
    let ratio = libm::tgamma(a + 0.5) / (libm::tgamma(a) * libm::tgamma(0.5));
    ratio.is_finite().then_some(ratio)
}

/// Gamma((n+1)/2)/(Gamma(n/2) sqrt(pi)) for an integer n up to [`SERIES_MAX_DF`], in
/// double-double: 1/pi times the product of (m+1)/m over m = 1, 3, ..., n-2 for an odd n, and
/// 1/2 times the product over m = 2, 4, ..., n-2 for an even n (the ratio for n is the one for
/// n - 2 times (n-1)/(n-2)).
fn integer_ratio(n: f64) -> Option<Dd> {
    if !is_series_df(n) {
        return None;
    }
    let odd = n % 2.0 == 1.0;
    let mut ratio = if odd { FRAC_1_PI_DD } else { (0.5, 0.0) };
    let mut m = if odd { 1.0 } else { 2.0 };
    while m < n {
        ratio = dd_div_small(dd_mul(ratio, (m + 1.0, 0.0)), m);
        m += 2.0;
    }
    Some(ratio)
}

/// An integer df from 1 to [`SERIES_MAX_DF`].
fn is_series_df(n: f64) -> bool {
    (1.0..=SERIES_MAX_DF).contains(&n) && n.fract() == 0.0
}

/// A = P(|T| < t) for t > 0 and an integer df n <= [`SERIES_MAX_DF`], in double-double, by the
/// finite series for integer df (Hill 1970, CACM algorithm 395), with cos^2 = n/(n+t^2) and
/// sin = t/sqrt(n+t^2):
/// - even n: A = sin (1 + 1/2 cos^2 + 1*3/(2*4) cos^4 + ...), n/2 terms;
/// - odd n: A = 2/pi (theta + sin cos (1 + 2/3 cos^2 + 2*4/(3*5) cos^4 + ...)), (n-1)/2 terms,
///   theta = atan(t/sqrt(n)).
fn centre_series(t: f64, n: f64) -> Option<Dd> {
    if !is_series_df(n) || t > 1e150 {
        return None;
    }
    let square = dd_mul((t, 0.0), (t, 0.0));
    let sum = dd_add(square, (n, 0.0));
    let cos2 = dd_div((n, 0.0), sum);
    let sin = dd_div((t, 0.0), dd_sqrt(sum));
    let odd = n % 2.0 == 1.0;
    let offset = if odd { 1.0 } else { 0.0 };
    // sum_k c_k cos^(2k), c_0 = 1 and c_k = c_(k-1) (2k - 1 + offset)/(2k + offset).
    let terms = (n - offset) / 2.0;
    let mut total = (0.0, 0.0);
    let mut term = (1.0, 0.0);
    let mut k = 0.0;
    while k < terms {
        total = dd_add(total, term);
        k += 1.0;
        term = dd_div_small(
            dd_mul(dd_mul(term, cos2), (2.0 * k - 1.0 + offset, 0.0)),
            2.0 * k + offset,
        );
    }
    if !odd {
        return Some(dd_mul(sin, total));
    }
    let theta = dd_atan(dd_div((t, 0.0), dd_sqrt((n, 0.0))));
    let inner = if terms > 0.0 {
        dd_mul(dd_mul(sin, dd_sqrt(cos2)), total)
    } else {
        (0.0, 0.0)
    };
    let half = dd_mul(dd_add(theta, inner), FRAC_1_PI_DD);
    Some((2.0 * half.0, 2.0 * half.1))
}

/// The Student-t density for an integer df n <= [`SERIES_MAX_DF`], rounded once from
/// double-double: ratio cos^(n+1) / sqrt(n) with cos^2 = n/(n+t^2) and the ratio of
/// [`integer_ratio`]. `None` elsewhere.
pub(crate) fn student_t_density(t: f64, n: f64) -> Option<f64> {
    if t.is_nan() || t.abs() >= 1e150 {
        return None;
    }
    let ratio = integer_ratio(n)?;
    let square = dd_mul((t, 0.0), (t, 0.0));
    let cos2 = dd_div((n, 0.0), dd_add(square, (n, 0.0)));
    // cos^(n+1) = (cos^2)^((n+1)/2), with one sqrt when n + 1 is odd.
    let mut power = if n % 2.0 == 1.0 {
        (1.0, 0.0)
    } else {
        dd_sqrt(cos2)
    };
    let mut base = cos2;
    // n <= SERIES_MAX_DF: the cast is exact.
    let mut exponent = ((n + 1.0) / 2.0).floor() as u32;
    while exponent > 0 {
        if exponent & 1 == 1 {
            power = dd_mul(power, base);
        }
        base = dd_mul(base, base);
        exponent >>= 1;
    }
    Some(dd_div(dd_mul(ratio, power), dd_sqrt((n, 0.0))).0)
}

/// sqrt of a positive double-double: the double root and one Newton correction.
fn dd_sqrt(a: Dd) -> Dd {
    let root = a.0.sqrt();
    let square = dd_mul((root, 0.0), (root, 0.0));
    let residual = dd_add(a, (-square.0, -square.1));
    two_sum(root, residual.0 / (2.0 * root))
}

/// atan of a positive double-double u: the double atan `theta_0`, then one Newton step on
/// tan(theta) = u, d = (u cos - sin)/(cos + u sin), with sin and cos of `theta_0` by their
/// double-double Taylor series.
fn dd_atan(u: Dd) -> Dd {
    let theta = u.0.atan();
    let (mut sin, mut cos) = ((0.0, 0.0), (0.0, 0.0));
    let mut power = (1.0, 0.0);
    for k in 0..TAYLOR_TERMS {
        let sign = if (k / 2) % 2 == 0 { 1.0 } else { -1.0 };
        let signed = (sign * power.0, sign * power.1);
        if k % 2 == 0 {
            cos = dd_add(cos, signed);
        } else {
            sin = dd_add(sin, signed);
        }
        power = dd_div_small(dd_mul(power, (theta, 0.0)), f64::from(k + 1));
    }
    let numerator = dd_add(dd_mul(u, cos), (-sin.0, -sin.1));
    let denominator = dd_add(cos, dd_mul(u, sin));
    dd_add((theta, 0.0), dd_div(numerator, denominator))
}

#[cfg(test)]
mod tests {
    use super::super::incomplete_beta::tests::ulps;
    use super::*;

    #[test]
    fn excel_cells() {
        // T_DIST.xlsx `T.DIST T.INV T.DIST.RT`: D15 and D16 are bit-exact.
        assert_eq!(student_t_tails(1.0, 1.0).0, 0.75);
        assert_eq!(student_t_tails(1.0, 2.0).0, 0.788_675_134_594_812_9);
        // F18: the deep tail keeps its digits, one ulp from Excel's 1.230468749943611E-56 (the
        // exact value is 33 ulp away).
        let deep = student_t_tails(1e6, 10.0);
        assert_eq!(deep.1, 1.230_468_749_943_610_7e-56);
        assert_eq!(student_t_tails(-1e6, 10.0).0, deep.1);
        assert_eq!(ulps(deep.1, 1.230_468_749_943_611e-56), 1);
    }

    #[test]
    fn density_cells() {
        // T_DIST.xlsx `T.DIST T.INV T.DIST.RT`: D10 (t = 0, n = 10) is the correctly rounded
        // 945/768/sqrt(10), 3 ulp below Excel's 0.38910838396603115; D3 (t = 8, n = 3).
        assert_eq!(student_t_density(0.0, 10.0), Some(0.389_108_383_966_031_04));
        let d3 = student_t_density(8.0, 3.0).unwrap_or(f64::NAN);
        assert!(ulps(d3, 0.000_736_906_520_946_926_6) <= 4, "{d3}");
        assert_eq!(student_t_density(1.0, 1001.0), None);
    }

    #[test]
    fn ratio_paths_agree() {
        // The asymptotic series and the integer product meet at n = 170.
        let product = integer_ratio(170.0).map(|r| r.0).unwrap_or(f64::NAN);
        let asymptotic = gamma_ratio(170.0).unwrap_or(f64::NAN);
        assert!(ulps(product, asymptotic) <= 1, "{product} {asymptotic}");
    }

    #[test]
    fn extremes_stay_finite() {
        for &n in &[1.0, 2.0, 3.0, 999.0, 1000.0, 1001.0, 1e6, 1e10, 2.5] {
            for &t in &[1e-300, 1e-10, 0.5, 1.0, 30.0, 1e6, 1e150, 1e200, f64::MAX] {
                let (lower, upper) = student_t_tails(t, n);
                assert!((0.0..=1.0).contains(&lower), "{t} {n} {lower}");
                assert!((0.0..=1.0).contains(&upper), "{t} {n} {upper}");
            }
        }
    }
}
