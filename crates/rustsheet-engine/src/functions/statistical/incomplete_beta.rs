//! The regularized incomplete beta `I_x(a, b)` behind BETA.DIST, BETADIST, F.DIST, F.DIST.RT,
//! FDIST and F.TEST (T-0970), and the beta density BETA.DIST takes with `cumulative` FALSE.
//!
//! Excel's own algorithm is not known: no published candidate fits every oracle cell (T-1744,
//! docs/excel-semantics/statistics.md "Incomplete beta: Excel's closed forms and the best
//! candidate"). This is the best combination that fit found (`best_combination` in
//! `tools/src/rustsheet_tools/fitlab/kernels/beta.py`, bit-exact on 158 of the 283 BETA.DIST
//! sweep cells against 87 for statrs' `beta_reg`):
//! - Excel's closed forms `I_x(a,1) = exp(a ln x)` and `I_x(1,b) = 1 - exp(b ln(1-x))`;
//! - otherwise Cephes' two continued fractions (`incbcf`, `incbd`; no power series), reflected
//!   at the mean, x > a/(a+b), to the upper tail I_{1-x}(b,a);
//! - the prefactor x^a (1-x)^b / B(a,b) as exp(a ln x) * exp(b ln(1-x)) * Gamma(a+b) /
//!   (Gamma(a) Gamma(b)) while Gamma(a+b) is finite (a + b below Cephes' `MAXGAM`). The fit
//!   took correctly rounded Gammas; this takes musl's `tgamma` (`libm`), which can move a last
//!   digit, so the fit's 158 is not this port's count.
//!
//! Past `MAXGAM` the fit's logarithmic prefactor loses about 1,000 to 4,000 ulp (ln Gamma near
//! 3,000 cancelling to a small sum: `BETA.DIST(0.555,1000,1000,TRUE,0.1,1)` and
//! `BETA.DIST(33587,3,600,TRUE,2000,5489582)` in `BETADIST_BETAINV.xlsx`, where Excel's values are
//! within 4 ulp of the correctly rounded ones), so there the prefactor follows `DiDonato` and
//! Morris' TOMS 708 `brcomp`: the exponent in `rlog1` form with the Stirling corrections for both
//! shapes of 8 or more, and ln Gamma(a+b) - ln Gamma(b) by `algdiv`'s Stirling difference when the
//! smaller shape is below 8.
//!
//! Every loop is bounded: at most [`MAX_FRACTION_TERMS`] continued-fraction steps (Cephes' own
//! cap) and [`RLOG1_TERMS`] series terms, so one call costs a fixed amount of work.

use super::special::ln_gamma;

/// Cephes' `MAXGAM`: Gamma(z) is finite below it.
const MAXGAM: f64 = 171.624_376_956_302_7;
/// Cephes' `MACHEP`, 2^-53.
const MACHEP: f64 = 1.110_223_024_625_156_5e-16;
/// Cephes' `big` and `biginv` rescaling thresholds of the continued fractions.
const BIG: f64 = 4.503_599_627_370_496e15;
const BIGINV: f64 = 2.220_446_049_250_313e-16;
/// Cephes' own cap on the continued-fraction loop (`incbcf`, `incbd`): two convergents a step.
const MAX_FRACTION_TERMS: u32 = 300;
/// The shape from which Stirling's series (with [`stirling_correction`]'s terms) is used.
const STIRLING_MIN: f64 = 8.0;
/// 1/sqrt(2 pi).
const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_401_432_7;
/// Terms of the atanh series in [`rlog1`]: |r| <= 1/3 there, so (1/9)^20 is far below 2^-53.
const RLOG1_TERMS: u32 = 20;

/// The smallest x at which `I_x(1, b)` is Excel's cancelling closed form 1 - exp(b ln(1 - x)); below
/// it the tail is computed without the cancellation (T-0974).
const A1_CLOSED_FORM_MIN_X: f64 = 0.1;

/// `I_x(a, b)` for shapes a, b > 0; 0 at x <= 0 and 1 at x >= 1. NaN in, NaN out.
#[cfg(test)]
pub(crate) fn beta_reg(x: f64, a: f64, b: f64) -> f64 {
    beta_reg_complement(x, 1.0 - x, a, b)
}

/// [`beta_reg`] with y = 1 - x given by the caller (TOMS 708 `bratio`'s x and y), so that a
/// complement the caller has without cancellation keeps its digits through the reflection: the F
/// tails map to y = df1 x / (df1 x + df2) and its complement df2 / (df1 x + df2), and
/// `F.DIST.RT(1E-6,1,1)` is then Excel's correctly rounded 0.9993633804398389
/// (probes-numeric/m9-legacy-fdist A12), where forming 1 - w from w misses by 2E-14.
pub(crate) fn beta_reg_complement(x: f64, y: f64, a: f64, b: f64) -> f64 {
    if x.is_nan() || y.is_nan() || a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if x <= 0.0 {
        return 0.0;
    }
    if y <= 0.0 {
        return 1.0;
    }
    // Excel's closed forms (T-1744: 27 of 27 b = 1 cells, 22 of 24 a = 1 cells); exp of a
    // product, not pow.
    if b == 1.0 {
        return (a * x.ln()).exp();
    }
    // From x, not from the given y: `F.DIST.RT(0.7,4,2.4)` = I_w(1,2) is Excel's
    // 0.6597222222222223 (`F_DIST.xlsx` H3) with ln(1 - w), 2 ulp off with the exact complement.
    // Below x = 0.1 Excel keeps the digits that subtraction cancels, -expm1(b ln1p(-x)):
    // `BETA.DIST(1E-10,1,0.5,TRUE)` is 9.999999999999999E-11 (probes-numeric/t1745-beta-inv-roundtrip
    // G35), which 1 - exp misses by 8E-8 relative; the cut is T.DIST.RT's at df 2 (T-0969), and
    // `BETA.DIST(0.1,1,0.0001,TRUE)` still cancels (docs/excel-semantics/statistics.md "Incomplete
    // beta: Excel's closed forms").
    if a == 1.0 {
        if x < A1_CLOSED_FORM_MIN_X {
            return -(b * (-x).ln_1p()).exp_m1();
        }
        return 1.0 - (b * (1.0 - x).ln()).exp();
    }
    beta_tails(x, y, a, b).0
}

/// (`I_x(a, b)`, 1 - `I_x(a, b)`) from one evaluation, the unreflected side computed and the other
/// its complement, for 0 < x < 1 outside the closed forms: Excel's two F tails add up to exactly
/// 1 at the mean, `F.DIST.RT(1,3,7)+F.DIST(1,3,7,TRUE)` = 1 (probes-numeric/t1049-fdistrt A27),
/// which two separate evaluations miss by 6E-16.
fn beta_tails(x: f64, y: f64, a: f64, b: f64) -> (f64, f64) {
    let flip = x > a / (a + b);
    let (x, y, a, b) = if flip { (y, x, b, a) } else { (x, y, a, b) };
    let w = cephes_fraction(a, b, x, y);
    let v = if a + b < MAXGAM {
        (a * x.ln()).exp() * (b * y.ln()).exp() * gamma_ratio(a, b) / a * w
    } else {
        large_prefactor(a, b, x, y) * w / a
    };
    if flip {
        (1.0 - v, v)
    } else {
        (v, 1.0 - v)
    }
}

/// 1 - `I_x(a, b)`, the upper tail of [`beta_reg_complement`], with y = 1 - x given: Excel's
/// closed forms read the other way round, `I_y(b, a)` (`F.DIST.RT(0.7,4,2.4)` = 0.6597222222222223,
/// `F_DIST.xlsx` H3), else the same evaluation as the lower tail.
pub(crate) fn beta_reg_upper(x: f64, y: f64, a: f64, b: f64) -> f64 {
    if x.is_nan() || y.is_nan() || a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if x <= 0.0 {
        return 1.0;
    }
    if y <= 0.0 {
        return 0.0;
    }
    if a == 1.0 || b == 1.0 {
        return beta_reg_complement(y, x, b, a);
    }
    beta_tails(x, y, a, b).1
}

/// The beta density x^(a-1) (1-x)^(b-1) / B(a, b) for 0 < x < 1 while Gamma(a+b) is finite, by
/// `powf` times the Gamma ratio: bit-exact on `BETA.DIST(2,8,10,FALSE,1,3)` = 1.4837646484375
/// (`BETA_GAMMA.xlsx` G4) and `BETA.DIST(3,2,3,FALSE,1,10)` (`BETADIST_BETAINV.xlsx` G3), where
/// exp((a-1) ln x + (b-1) ln(1-x) - ln B) misses by up to 4 ulp. `None` elsewhere, where the
/// caller keeps statrs' density.
pub(crate) fn beta_density(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(x > 0.0 && x < 1.0 && a + b < MAXGAM) {
        return None;
    }
    // Gamma(a) Gamma(b) can overflow below MAXGAM (a shape near 1E-300): no ratio then.
    let ratio = gamma_ratio(a, b);
    if !(ratio.is_finite() && ratio > 0.0) {
        return None;
    }
    let density = x.powf(a - 1.0) * (1.0 - x).powf(b - 1.0) * ratio;
    density.is_finite().then_some(density)
}

/// Gamma(a+b) / (Gamma(a) Gamma(b)), Cephes' order, for a + b below [`MAXGAM`].
fn gamma_ratio(a: f64, b: f64) -> f64 {
    libm::tgamma(a + b) / (libm::tgamma(a) * libm::tgamma(b))
}

/// Cephes' `incbcf` below x (a+b-2) = a-1 and `incbd` / y above it, y = 1 - x: the continued
/// fraction w with `I_x(a,b) = x^a y^b / (a B(a,b)) * w`, at most [`MAX_FRACTION_TERMS`] steps.
fn cephes_fraction(a: f64, b: f64, x: f64, y: f64) -> f64 {
    if x * (a + b - 2.0) - (a - 1.0) < 0.0 {
        incbcf(a, b, x)
    } else {
        incbd(a, b, x / y) / y
    }
}

/// Cephes' `incbcf`: the continued fraction w with `I_x(a,b) = x^a (1-x)^b / (a B(a,b)) * w`.
pub(super) fn incbcf(a: f64, b: f64, x: f64) -> f64 {
    fraction(
        x,
        [a, a + b, a, a + 1.0, 1.0, b - 1.0, a + 1.0, a + 2.0],
        false,
    )
}

/// Cephes' `incbd` before its final division by 1 - x, taking `ratio` = x/(1-x) from the caller:
/// `I_x(a,b) = x^a (1-x)^b / (a B(a,b)) * incbd(a, b, ratio) / (1-x)`. A caller that knows 1 - x
/// better than x forms the ratio without rounding x first (the Student-t tails, `student_t.rs`).
pub(super) fn incbd(a: f64, b: f64, ratio: f64) -> f64 {
    fraction(
        ratio,
        [a, b - 1.0, a, a + 1.0, 1.0, a + b, a + 1.0, a + 2.0],
        true,
    )
}

/// The convergent loop shared by `incbcf` (`second` false) and `incbd`.
fn fraction(x: f64, k: [f64; 8], second: bool) -> f64 {
    let [mut k1, mut k2, mut k3, mut k4, mut k5, mut k6, mut k7, mut k8] = k;
    let (mut pkm2, mut qkm2, mut pkm1, mut qkm1) = (0.0, 1.0, 1.0, 1.0);
    let mut ans = 1.0;
    let mut r = 1.0;
    let thresh = 3.0 * MACHEP;
    let step = if second { -1.0 } else { 1.0 };
    for _ in 0..MAX_FRACTION_TERMS {
        let xk = -(x * k1 * k2) / (k3 * k4);
        let pk = pkm1 + pkm2 * xk;
        let qk = qkm1 + qkm2 * xk;
        (pkm2, pkm1, qkm2, qkm1) = (pkm1, pk, qkm1, qk);
        let xk = (x * k5 * k6) / (k7 * k8);
        let pk = pkm1 + pkm2 * xk;
        let qk = qkm1 + qkm2 * xk;
        (pkm2, pkm1, qkm2, qkm1) = (pkm1, pk, qkm1, qk);
        if qk != 0.0 {
            r = pk / qk;
        }
        let t = if r == 0.0 {
            1.0
        } else {
            let t = ((ans - r) / r).abs();
            ans = r;
            t
        };
        if t < thresh {
            break;
        }
        k1 += 1.0;
        k2 += step;
        k3 += 2.0;
        k4 += 2.0;
        k5 += 1.0;
        k6 -= step;
        k7 += 2.0;
        k8 += 2.0;
        if qk.abs() + pk.abs() > BIG {
            pkm2 *= BIGINV;
            pkm1 *= BIGINV;
            qkm2 *= BIGINV;
            qkm1 *= BIGINV;
        }
        if qk.abs() < BIGINV || pk.abs() < BIGINV {
            pkm2 *= BIG;
            pkm1 *= BIG;
            qkm2 *= BIG;
            qkm1 *= BIG;
        }
    }
    ans
}

/// x^a y^b / B(a, b), y = 1 - x, for a + b of [`MAXGAM`] or more (so the larger shape is above
/// 163), after TOMS 708 `brcomp`.
fn large_prefactor(a: f64, b: f64, x: f64, y: f64) -> f64 {
    if a >= STIRLING_MIN && b >= STIRLING_MIN {
        // x/x0 = 1 - lambda/a and y/y0 = 1 + lambda/b around the mean x0 = a/(a+b), so
        // a ln(x/x0) + b ln(y/y0) = -(a rlog1(-lambda/a) + b rlog1(lambda/b)) without the
        // cancellation of a ln x + b ln y against ln B.
        let lambda = if a > b {
            (a + b) * y - b
        } else {
            a - (a + b) * x
        };
        let exponent = a * rlog1(-lambda / a) + b * rlog1(lambda / b);
        let correction =
            stirling_correction(a) + stirling_correction(b) - stirling_correction(a + b);
        return FRAC_1_SQRT_2PI * (a * b / (a + b)).sqrt() * (-(exponent + correction)).exp();
    }
    // One shape below 8: ln B(a,b) = ln Gamma(small) - (ln Gamma(small + large) - ln Gamma(large)).
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    let ln_beta = ln_gamma(small) - gamma_ln_shift(small, large);
    (a * x.ln() + b * y.ln() - ln_beta).exp()
}

/// ln Gamma(b + a) - ln Gamma(b) for b of [`STIRLING_MIN`] or more (TOMS 708 `algdiv`):
/// a ln b + (b + a - 1/2) ln(1 + a/b) - a plus the Stirling corrections.
fn gamma_ln_shift(a: f64, b: f64) -> f64 {
    a * b.ln() + (b + a - 0.5) * (a / b).ln_1p() - a + stirling_correction(b + a)
        - stirling_correction(b)
}

/// ln Gamma(z) - ((z - 1/2) ln z - z + ln(2 pi)/2) for z of [`STIRLING_MIN`] or more: Stirling's
/// series to the 1/z^15 term (the next is below 2^-53 times the sum at z = 8).
fn stirling_correction(z: f64) -> f64 {
    // B_2k / (2k (2k-1)) for k = 1..8.
    const C: [f64; 8] = [
        1.0 / 12.0,
        -1.0 / 360.0,
        1.0 / 1260.0,
        -1.0 / 1680.0,
        1.0 / 1188.0,
        -691.0 / 360_360.0,
        1.0 / 156.0,
        -3617.0 / 122_400.0,
    ];
    let w = 1.0 / (z * z);
    let mut sum = 0.0;
    for c in C.iter().rev() {
        sum = sum * w + c;
    }
    sum / z
}

/// t - ln(1 + t) for t > -1, without the cancellation of the plain difference near 0: with
/// r = t/(2+t), ln(1+t) = 2 atanh r and t - 2r = r t, so the result is
/// r t - 2 (r^3/3 + r^5/5 + ...), used for |t| <= 1/2 (|r| <= 1/3).
fn rlog1(t: f64) -> f64 {
    if t.abs() > 0.5 {
        return t - t.ln_1p();
    }
    let r = t / (2.0 + t);
    let r2 = r * r;
    let mut power = r * r2;
    let mut series = 0.0;
    for k in 0..RLOG1_TERMS {
        let term = power / f64::from(2 * k + 3);
        series += term;
        if term.abs() <= f64::EPSILON * series.abs() {
            break;
        }
        power *= r2;
    }
    r * t - 2.0 * series
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// The ulps between two doubles of the same sign.
    pub(in crate::functions::statistical) fn ulps(a: f64, b: f64) -> u64 {
        let (ia, ib) = (a.to_bits() as i64, b.to_bits() as i64);
        ia.abs_diff(ib)
    }

    #[test]
    fn closed_forms() {
        for &x in &[0.1, 0.5, 0.9] {
            for &s in &[0.5, 2.0, 7.5] {
                assert_eq!(beta_reg(x, s, 1.0).to_bits(), (s * x.ln()).exp().to_bits());
                assert_eq!(
                    beta_reg(x, 1.0, s).to_bits(),
                    (1.0 - (s * (1.0 - x).ln()).exp()).to_bits()
                );
            }
        }
    }

    #[test]
    fn symmetry_and_known_values() {
        for &(x, a, b) in &[(0.3, 2.0, 3.0), (0.7, 2.5, 0.5), (0.2, 10.0, 30.0)] {
            let lower = beta_reg(x, a, b);
            let upper = beta_reg(1.0 - x, b, a);
            assert!((lower + upper - 1.0).abs() < 1e-15, "{x} {a} {b}");
        }
        // I_0.3(2,3) = 0.3483 exactly.
        assert!(ulps(beta_reg(0.3, 2.0, 3.0), 0.3483) <= 4);
        // Excel's cached value (BETADIST_BETAINV.xlsx G29): large shapes.
        assert!(
            ulps(
                beta_reg(0.505_555_555_555_555_6, 1000.0, 1000.0),
                0.690_355_130_198_537_6
            ) <= 4
        );
    }

    #[test]
    fn tiny_and_large_shapes_stay_in_range() {
        for &(x, a, b) in &[
            (0.5, 1e-4, 1e-4),
            (0.08, 1e-4, 2.0),
            (0.08, 2.0, 1e-4),
            (0.5, 1e3, 1e3),
            (0.4, 1e3, 1e-4),
            (0.5, 1e6, 1e6),
            (1e-300, 0.5, 0.5),
        ] {
            let v = beta_reg(x, a, b);
            assert!((0.0..=1.0).contains(&v), "{x} {a} {b}: {v}");
        }
        assert!(beta_reg(f64::NAN, 2.0, 3.0).is_nan());
        assert_eq!(beta_reg(0.0, 2.0, 3.0), 0.0);
        assert_eq!(beta_reg(1.0, 2.0, 3.0), 1.0);
    }

    #[test]
    fn rlog1_matches_the_plain_difference_away_from_zero() {
        for &t in &[-0.9, -0.5, -0.1, 0.1, 0.5, 3.0] {
            let plain = t - f64::ln_1p(t);
            assert!(
                (rlog1(t) - plain).abs() <= 1e-15 * plain.abs().max(1e-3),
                "{t}"
            );
        }
        // 1e-8 - ln(1 + 1e-8) = 5e-17 - 3.3e-25 + ...
        assert!((rlog1(1e-8) / 5e-17 - 1.0).abs() < 1e-7);
    }

    #[test]
    fn upper_tail_cells() {
        // F.DIST.RT(0.7,4,2.4) (F_DIST.xlsx H3): I_w(1, 2) by the closed form, read from w.
        let (scaled, df2) = (2.8, 2.0);
        let total = scaled + df2;
        assert_eq!(
            beta_reg_upper(scaled / total, df2 / total, 2.0, 1.0),
            0.659_722_222_222_222_3
        );
        // F.DIST.RT(1E-6,1,1) (probes-numeric/m9-legacy-fdist A12): the exact complement keeps
        // the small tail's digits.
        let (scaled, df2) = (1e-6, 1.0);
        let total = scaled + df2;
        assert_eq!(
            beta_reg_upper(scaled / total, df2 / total, 0.5, 0.5),
            0.999_363_380_439_838_9
        );
        // The two tails of one evaluation add up to 1 (t1049-fdistrt A27: F at x = 1, df 3, 7).
        let (x, y) = (3.0 / 10.0, 7.0 / 10.0);
        assert_eq!(
            beta_reg_complement(x, y, 1.5, 3.5) + beta_reg_upper(x, y, 1.5, 3.5),
            1.0
        );
    }

    #[test]
    fn density_matches_excel_cells() {
        // BETA.DIST(2,8,10,FALSE,1,3) = 1.4837646484375 (BETA_GAMMA.xlsx G4): density 2.967...
        let d = beta_density(0.5, 8.0, 10.0).map(|d| d / 2.0);
        assert_eq!(d, Some(1.483_764_648_437_5));
        assert_eq!(beta_density(0.0, 2.0, 3.0), None);
        assert_eq!(beta_density(0.5, 100.0, 100.0), None);
    }
}
