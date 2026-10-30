//! The regularized incomplete beta `I_x(a, b)` at a huge first shape and a small second one
//! (T-2528): `DiDonato` and Morris' asymptotic expansion in the incomplete gamma, the series of
//! TOMS 708 `bgrat` (their section 9), in accurate double arithmetic rather than Excel's. An
//! accuracy stand-in like `incomplete_beta.rs` `half_integer_tail`: the bratio port's bgrat
//! branches (SPEC-0025, T-2510) replace it, or absorb it, when the old kernel goes.
//!
//! Cephes' continued fraction (`incomplete_beta.rs`) needs about 42,000 steps for
//! `F.DIST(2,5,1E10,TRUE)` = 1 - `I_y(5E9, 2.5)` at y = 1 - 1E-9 and is about 1E-7 relative off at
//! every cap from 300 to 300,000, where Excel's 0.9247647537543169 is the true value
//! (probes-numeric/t1530-f-df-cut A8). With nu = a + (b - 1)/2 and z = -nu ln x,
//!
//! `I_x(a, b) = Gamma(a+b) / (Gamma(a) nu^b) * (Q(b, z) + r * sum_n d_n J_n)`,
//!
//! r = z^b e^-z / Gamma(b), the `d_n` and `J_n` bgrat's recurrences from `J_0` = Q(b, z) / r; each
//! term is about (b/nu)^2 of the one before, so at b/a of [`MAX_SHAPE_RATIO`] or less the sum is
//! a correction below 1E-12 of Q.

use super::incomplete_beta::{rlog1, stirling_correction};
use super::incomplete_gamma::{gamma_density, gamma_ur};
use crate::x87_exp::{dd_mul, two_sum};
use crate::x87_log10::ln_double_double_pair;

/// The largest b/a the expansion is used at: below it the series' first term is under 1E-12 of
/// the leading Q(b, z), and Cephes' fraction is off by far more (T-2528 sweep against a 60-digit
/// mpmath `betainc`: 1E-7 relative at a = 5E9, b = 2.5).
pub(super) const MAX_SHAPE_RATIO: f64 = 1e-6;

/// The smallest first shape the expansion is used at. A tiny b at a moderate a also has b/a below
/// [`MAX_SHAPE_RATIO`], where the expansion's terms in 1/nu are not small: `BETA.DIST(0.997,3.366,
/// 3.43E-18,TRUE)` is 1.43E-17 in Excel (probes-numeric/t2464-r2-bpser-small-tiny A10) and 4E-13 by
/// it.
const MIN_SHAPE: f64 = 1e6;

/// The second shape below which the expansion is used: bratio's bgrat family (L140) is for b0 below
/// 40, and from 40 Excel's other branches decide, `#NUM!` included: `BETA.DIST(0.1,1E+200,1E+100,
/// TRUE)` is `#NUM!` (probes/t3007-beta-dist-ln-x-overflow S!A23), where the expansion's tail is 0.
const MAX_SECOND_SHAPE: f64 = 40.0;

/// bgrat's cap on the series' terms.
const MAX_SERIES_TERMS: usize = 30;

/// The deepest cut of [`legendre_fraction`]: it needs about 10 cbrt(b) partial quotients at its
/// slowest, z = b + 1 (`incomplete_gamma`'s budget), and b is at most 1E-6 of a finite a here.
const MAX_FRACTION_DEPTH: u32 = 1 << 14;

/// The relative size of the last series term at which the series stops.
const SERIES_TOLERANCE: f64 = 1e-17;

/// `I_x(a, b)` with y = 1 - x given, for a of [`MIN_SHAPE`] or more, b below [`MAX_SECOND_SHAPE`]
/// and at most [`MAX_SHAPE_RATIO`] of a, and 0 < x, y < 1; `None` elsewhere or where a step is not
/// finite, and the caller keeps its kernel.
pub(super) fn large_shape_lower(a: f64, b: f64, x: f64, y: f64) -> Option<f64> {
    if !(a >= MIN_SHAPE
        && a.is_finite()
        && b > 0.0
        && b < MAX_SECOND_SHAPE
        && b <= a * MAX_SHAPE_RATIO
        && x > 0.0
        && y > 0.0)
    {
        return None;
    }
    let bm1 = b - 1.0;
    let nu = a + 0.5 * bm1;
    // ln x in double-double, from 1 - y exactly when x is near 1: the rounding of a given x near 1
    // is what nu multiplies into z (each unit of z is a factor e on the result).
    let x_pair = if x > 0.5 { two_sum(1.0, -y) } else { (x, 0.0) };
    let ln_x = ln_double_double_pair(x_pair);
    let (z, z_lo) = dd_mul((-nu, 0.0), ln_x);
    if z.is_nan() || z <= 0.0 {
        return None;
    }
    let scale = shape_ratio_factor(a, b, nu);
    if !z.is_finite() {
        return Some(0.0);
    }
    // r = z^b e^-z / Gamma(b) = z times the gamma density at z.
    let r = gamma_density(b, z, 1.0).map_or(0.0, |density| density * z);
    if !(r.is_finite() && scale.is_finite()) {
        return None;
    }
    if r == 0.0 {
        // e^-z underflows: so does the tail, Q(b, z) at most a few times r.
        let upper = gamma_ur(b, z);
        return upper.is_finite().then_some(scale * upper);
    }
    let j0 = upper_over_prefactor(b, z, r)?;
    // Q(b, z + z_lo) = Q(b, z) - z_lo r / z to first order: the low part of z.
    let series = correction_series(b, z, ln_x.0, nu, j0);
    let value = scale * r * ((j0 - z_lo / z) + series);
    (value.is_finite() && value >= 0.0).then_some(value.min(1.0))
}

/// Q(b, z) / r with r = z^b e^-z / Gamma(b): for z of b + 1 or more Legendre's continued fraction
/// 1/(z + 1 - b - 1(1 - b)/(z + 3 - b - 2(2 - b)/(z + 5 - b - ...))), evaluated from the bottom up
/// at a depth doubled until two depths agree, which keeps it within about an ulp where the forward
/// (Lentz) product of `incomplete_gamma` carries up to 7 (Q(2.5, 4.9999999825), T-2528); below,
/// that module's Q(b, z) over r. `None` past [`MAX_FRACTION_DEPTH`].
fn upper_over_prefactor(b: f64, z: f64, r: f64) -> Option<f64> {
    if z < b + 1.0 {
        let upper = gamma_ur(b, z);
        return upper.is_finite().then_some(upper / r);
    }
    let mut depth = 16_u32;
    let mut previous = legendre_fraction(b, z, depth);
    while depth < MAX_FRACTION_DEPTH {
        depth *= 2;
        let next = legendre_fraction(b, z, depth);
        if (next - previous).abs() <= f64::EPSILON * next.abs() {
            return next.is_finite().then_some(next);
        }
        previous = next;
    }
    None
}

/// Legendre's fraction for Q(b, z) / r cut at `depth` partial quotients, from the bottom up.
fn legendre_fraction(b: f64, z: f64, depth: u32) -> f64 {
    let mut tail = 0.0;
    for i in (1..=depth).rev() {
        let i = f64::from(i);
        tail = -i * (i - b) / (z + 2.0 * i + 1.0 - b + tail);
    }
    1.0 / (z + 1.0 - b + tail)
}

/// Gamma(a + b) / (Gamma(a) nu^b) for a of 8 or more, as exp of
/// -a rlog1(b/a) + (b - 1/2) ln(1 + b/a) - b ln(1 + (b - 1)/(2a)) and the Stirling corrections:
/// the logs of Gamma(a + b) / Gamma(a) and of nu^b each near b ln a, cancelled before they are
/// formed (their difference is about b(b - 1)/(2a)).
fn shape_ratio_factor(a: f64, b: f64, nu: f64) -> f64 {
    let t = b / a;
    let exponent = -a * rlog1(t) + (b - 0.5) * t.ln_1p() - b * ((nu - a) / a).ln_1p()
        + stirling_correction(a + b)
        - stirling_correction(a);
    exponent.exp()
}

/// bgrat's sum of `d_n J_n` for n = 1, 2, ... ([`MAX_SERIES_TERMS`] at most), from
/// `J_0` = Q(b, z) / r: `J_n` = ((b + 2n - 2)(b + 2n - 1) `J_{n-1}` + (z + b + 2n - 1) t^(n-1)) v
/// with v = 1/(4 nu^2), t = (ln x)^2 / 4, and `d_n` from the coefficients `c_n` = 1/(2n+1)!.
fn correction_series(b: f64, z: f64, ln_x: f64, nu: f64, j0: f64) -> f64 {
    let bm1 = b - 1.0;
    let v = 0.25 / (nu * nu);
    let t2 = 0.25 * ln_x * ln_x;
    let mut c = [0.0; MAX_SERIES_TERMS];
    let mut d = [0.0; MAX_SERIES_TERMS];
    let (mut j, mut sum, mut t, mut cn, mut n2) = (j0, 0.0, 1.0, 1.0, 0.0);
    for n in 1..=MAX_SERIES_TERMS {
        let bp2n = b + n2;
        j = (bp2n * (bp2n + 1.0) * j + (z + bp2n + 1.0) * t) * v;
        n2 += 2.0;
        t *= t2;
        cn /= n2 * (n2 + 1.0);
        let nm1 = n - 1;
        c[nm1] = cn;
        let mut s = 0.0;
        let mut coef = b - n as f64;
        for i in 1..n {
            s += coef * c[i - 1] * d[nm1 - i];
            coef += b;
        }
        d[nm1] = bm1 * cn + s / n as f64;
        let dj = d[nm1] * j;
        sum += dj;
        if dj.abs() <= SERIES_TOLERANCE * (j0 + sum).abs() {
            break;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::large_shape_lower;
    use crate::functions::statistical::incomplete_beta::beta_reg_tails;

    /// Above the mean of Beta(2.5, 5E9) (5E-10) the reflected side takes the expansion: both tails
    /// stay finite, in [0, 1], add up to 1 and fall monotonically in the upper tail.
    #[test]
    fn a_sweep_above_the_mean_stays_in_range() {
        let mut previous_upper = 1.0;
        for step in 0..=2_000 {
            let x = 5e-10 * 1.01_f64.powi(step) * 1.000_001;
            let (lower, upper) = beta_reg_tails(x, 1.0 - x, 2.5, 5e9);
            assert!(lower.is_finite() && upper.is_finite(), "{x}");
            assert!(
                (0.0..=1.0).contains(&lower) && (0.0..=1.0).contains(&upper),
                "{x}"
            );
            assert!((lower + upper - 1.0).abs() <= f64::EPSILON, "{x}");
            assert!(
                upper <= previous_upper,
                "{x}: {upper} after {previous_upper}"
            );
            previous_upper = upper;
        }
        assert_eq!(previous_upper, 0.0);
    }

    /// 1 - `I_u(5/2, df2/2)` = `I_v(df2/2, 5/2)`, the F tail above x, against a 60-digit mpmath
    /// `betainc(5/2, df2/2, u, 1)` (120 digits: at 60 it is 4E-14 off at x = 50) at the double
    /// u = 5x / (5x + df2) (T-2528).
    #[test]
    fn f_upper_tails_at_a_huge_df2_match_the_reference() {
        for (df2, x, reference) in [
            (1e10, 2.0, 0.07523524624568315),
            (1e10, 10.0, 1.3857974134229093e-09),
            (1e10, 20.0, 5.285149604903511e-20),
            (1e10, 50.0, 5.496930852316323e-52),
            (1e9, 2.0, 0.07523524713822163),
            (1e8, 5.0, 0.0001393339615389781),
            (1e8, 20.0, 5.285272758334189e-20),
        ] {
            let scaled: f64 = 5.0 * x;
            let total = scaled + df2;
            let got = large_shape_lower(df2 / 2.0, 2.5, df2 / total, scaled / total);
            let got = got.unwrap_or(f64::NAN);
            assert!(
                (got / reference - 1.0).abs() < 2e-15,
                "{df2} {x}: {got} against {reference}"
            );
        }
    }

    #[test]
    fn only_a_huge_first_shape_takes_the_expansion() {
        assert_eq!(large_shape_lower(100.0, 2.5, 0.99, 0.01), None);
        assert_eq!(large_shape_lower(3.366, 3.43e-18, 0.997, 0.003), None);
        assert_eq!(large_shape_lower(1e200, 1e100, 0.1, 0.9), None);
        assert_eq!(large_shape_lower(5e9, 0.0, 0.99, 0.01), None);
        assert_eq!(large_shape_lower(5e9, 2.5, 0.0, 1.0), None);
        // Far below the mean the tail is 0, not NaN.
        assert_eq!(large_shape_lower(5e9, 2.5, 1e-300, 1.0), Some(0.0));
    }
}
