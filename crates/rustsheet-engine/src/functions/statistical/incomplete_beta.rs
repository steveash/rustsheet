//! The regularized incomplete beta `I_x(a, b)` behind BETA.DIST, BETADIST, F.DIST, F.DIST.RT,
//! FDIST and F.TEST (T-0970), and the beta density BETA.DIST takes with `cumulative` FALSE.
//!
//! Excel's own algorithm is not known: no published candidate fits every oracle cell (T-1744,
//! docs/excel-semantics/statistics.md "Incomplete beta: Excel's closed forms and the best
//! candidate"). This is the best combination that fit found (`best_combination` in
//! `tools/src/rustsheet_tools/fitlab/kernels/beta.py`, bit-exact on 158 of the 283 BETA.DIST
//! sweep cells against 87 for statrs' `beta_reg`):
//! - Excel's closed forms on bratio's oriented arguments ([`closed_form_tails`], T-2502);
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

use super::half_integer_beta::{half_integer_remainder, is_half_integer_shape};
use super::special::ln_gamma;
use crate::x87_exp::{dd_add, dd_mul, fast_two_sum, two_sum};
use crate::x87_exp::{excel_exp, excel_expm1};
use crate::x87_log10::{excel_ln, ln_double_double_pair};
use crate::x87_log1p::excel_ln_one_minus;

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
pub(crate) const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_401_432_7;
/// Terms of the atanh series in [`rlog1`]: |r| <= 1/3 there, so (1/9)^20 is far below 2^-53.
const RLOG1_TERMS: u32 = 20;

/// The smallest a + b from which half-integer shapes take [`half_integer_tail`]. Below it Cephes'
/// kernel is already within 15 digits of Excel, and the series only trades bit-exact cells: over
/// every suite (T-2219) a + b below 10 moved 35 cells off Excel's bit and 75 onto it with 6 cells
/// fixed visibly, among them the gated `FDIST(2,1,9)` and `F.DIST(1,3,8,1)` (gen-m9/c18dbf21f11093f3
/// A151, c6cfda6bd10cae93 A348) 1 ulp off, while from 10 up it fixes 122 visibly-off cells.
const SERIES_MIN_SHAPE_SUM: f64 = 10.0;

/// Below this x an a = 1 tail outside Excel's closed forms is -expm1(b ln1p(-x)), which keeps the
/// digits 1 - exp(b ln(1 - x)) cancels (T-0974): always so in [`shape_one_tails`] (x0 < 0.1 there),
/// and the cut of F.TEST's and the F quantile's unswapped form (`fisher.rs`
/// `unoriented_shape_one_tail`).
pub(crate) const A1_EXPM1_MAX_X: f64 = 0.1;

/// The largest oriented complement y0 at which a0 = 1 takes Excel's closed form
/// ([`closed_form_tails`]): Excel takes it at `T.DIST.RT(4.24,2)` (y0 = 0.899888) and not at
/// `T.DIST.RT(4.245,2)` (y0 = 0.900100), probes-numeric t0969-t-df2-cut, and takes it at y0 = 0.9
/// itself (`BETA.DIST(0.1,1,50,TRUE)`, probes-numeric m9-beta-dist-a1 A43; and swapped to a0 = 1 at
/// x0 = fl(1 - 0.9), `BETA.DIST(0.9,0.5,1,TRUE)`, m9-beta-dist-a0p5 A17). A cut at x0 >= 0.1 instead
/// scores 1,258 of the 1,286 closed-form cells against 1,286 (T-2464 w5-triangulation
/// `closed_variants.py`).
const A1_CLOSED_FORM_MAX_Y: f64 = 0.9;

/// `I_x(a, b)` for shapes a, b > 0; 0 at x <= 0 and 1 at x >= 1. NaN in, NaN out.
#[cfg(test)]
pub(crate) fn beta_reg(x: f64, a: f64, b: f64) -> f64 {
    beta_reg_complement(x, 1.0 - x, a, b)
}

/// [`beta_reg`] with y = 1 - x given by the caller (TOMS 708 `bratio`'s x and y), so that a
/// complement the caller has without cancellation keeps its digits through the reflection: the F
/// tails pass y = df1 x / (df1 x + df2) and its complement df2 / (df1 x + df2) (`fisher.rs`), and
/// `F.DIST.RT(1E-6,1,1)` is then Excel's correctly rounded 0.9993633804398389
/// (probes-numeric/m9-legacy-fdist A12), where forming 1 - w from w misses by 2E-14. A shape of 1
/// takes Excel's closed forms ([`closed_form_tails`], T-2502).
///
/// Like [`beta_reg_upper`] and [`search_tails`] it keeps Cephes' kernel at half-integer shapes,
/// which the quantile searches (BETA.INV, F.INV) and the Student-t tails still invert.
pub(crate) fn beta_reg_complement(x: f64, y: f64, a: f64, b: f64) -> f64 {
    search_tails(x, y, a, b).0
}

/// (`I_x(a, b)`, 1 - `I_x(a, b)`) with y = 1 - x given: Excel's oriented closed forms
/// ([`closed_form_tails`]), a shape of 1 outside them ([`shape_one_tails`]), else one evaluation of
/// the kernel ([`beta_tails`]: the accurate series at half-integer shapes, T-2219, else Cephes).
/// Not for a quantile search, which crosses [`search_tails`].
pub(crate) fn beta_reg_tails(x: f64, y: f64, a: f64, b: f64) -> (f64, f64) {
    reg_tails(x, y, a, b, Kernel::Accurate)
}

/// [`beta_reg_tails`] with Cephes' kernel at every shape: the tails BETA.INV's and F.INV's searches
/// cross (`beta.rs`, `fisher.rs::f_quantile_tail`). With the accurate series under them 7 gated
/// gen-m9 quantile cells move off Excel's bit (`FINV(0.025,7,10)`, gen-m9/fec39efaba9284c3 A336;
/// `F.INV.RT(0.975,9,9)`, 40425341c5034eed A205) and BETA.INV round trips such as
/// ironcalc-calc `BETADIST_BETAINV.xlsx` H67 (3 becomes 2.9999999999999996), though more quantile
/// cells move to within 15 digits than off: where Excel's search stops is T-2511's.
pub(crate) fn search_tails(x: f64, y: f64, a: f64, b: f64) -> (f64, f64) {
    reg_tails(x, y, a, b, Kernel::Cephes)
}

/// Which kernel [`beta_tails`] takes at half-integer shapes below [`MAXGAM`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kernel {
    /// [`half_integer_tail`].
    Accurate,
    /// Cephes' prefactor and continued fraction.
    Cephes,
}

fn reg_tails(x: f64, y: f64, a: f64, b: f64, kernel: Kernel) -> (f64, f64) {
    if x.is_nan() || y.is_nan() || a.is_nan() || b.is_nan() {
        return (f64::NAN, f64::NAN);
    }
    if x <= 0.0 {
        return (0.0, 1.0);
    }
    if y <= 0.0 {
        return (1.0, 0.0);
    }
    closed_form_tails(x, y, a, b)
        .or_else(|| shape_one_tails(x, y, a, b))
        .unwrap_or_else(|| beta_tails(x, y, a, b, kernel))
}

/// Excel's closed forms of the incomplete beta, (`I_x(a, b)`, 1 - `I_x(a, b)`), for 0 < x, y < 1
/// with y the caller's complement of x; `None` where they do not apply. They act on bratio's
/// oriented arguments: a shape of 1 makes min(a, b) <= 1, so bratio swaps to (b, a, y, x) when
/// x > 0.5, and then on (a0, b0, x0, y0), with Excel's own EXP and LN:
/// - b0 = 1: w = EXP(a0 LN(x0)) is computed;
/// - a0 = 1 with y0 <= 0.9: w1 = EXP(b0 LN(1 - x0)) is computed, 1 - x0 formed here from x0, not
///   the caller's y0;
/// - the other tail is (0.5 - w) + 0.5, and a0 = 1 with y0 > 0.9 has no closed form.
///
/// Bit-exact on 1,286 of 1,286 closed-form cells of BETA.DIST, F, T.DIST.RT at df 2, BINOM.DIST and
/// NEGBINOM.DIST (T-2464 w5-triangulation `closed_final.py`; docs/excel-semantics/statistics.md
/// "Incomplete beta closed forms act on the oriented arguments"). The y0 <= 0.9 cut lies between
/// x0 = 0.0998999751 (`T.DIST.RT(4.245,2)`, general kernel) and x0 = 0.09999999999999998 (closed).
pub(crate) fn closed_form_tails(x: f64, y: f64, a: f64, b: f64) -> Option<(f64, f64)> {
    if !(a == 1.0 || b == 1.0) || !(x > 0.0 && y > 0.0) {
        return None;
    }
    let swap = x > 0.5;
    let (a0, b0, x0, y0) = if swap { (b, a, y, x) } else { (a, b, x, y) };
    let (w, w1) = if b0 == 1.0 {
        let w = excel_exp(a0 * excel_ln(x0));
        (w, (0.5 - w) + 0.5)
    } else if a0 == 1.0 && y0 <= A1_CLOSED_FORM_MAX_Y {
        let w1 = excel_exp(b0 * excel_ln(1.0 - x0));
        ((0.5 - w1) + 0.5, w1)
    } else {
        return None;
    };
    Some(if swap { (w1, w) } else { (w, w1) })
}

/// (`I_x(a, b)`, 1 - `I_x(a, b)`) at a shape of 1 where [`closed_form_tails`] does not apply, for
/// 0 < x, y < 1: bratio's oriented a0 = 1 with y0 > 0.9 (x0 below about 0.1). Like bratio, Excel
/// computes w and forms w1 as (0.5 - w) + 0.5 (statistics.md "Incomplete beta at a shape of 1
/// outside the closed forms", T-3007):
/// - w = -expm1(b0 ln1p(-x0)) with Excel's own expm1 (Kahan's form) and log1p (the stored atanh
///   series): `F.DIST.RT(10,6,2)` = 0.09368601255412709 (gen-m9/146b46e74f2dac3b A329) and
///   `FDIST(40,8,2)` (gen-m9/abc2939ebd1e2365 A318) are then bit-exact; libm's expm1 and log1p miss
///   71 eval cells this form gets, and get 6 it misses (T-3170). `BETA.DIST(1.9999999998999996E-10,
///   1,0.5,TRUE)` is 9.999999999999999E-11 (probes-numeric/t1745-beta-inv-roundtrip G35), which
///   1 - exp misses by 8E-8 relative (T-0974); bpser(1, b0, x0) is further off Excel's w;
/// - w1 from w: `F.DIST(5,20,2,TRUE)` = 0.8203482998751552 (probes-numeric/t2220-f-dist-sweep-cdf
///   A718), which y0^b0 puts 3 ulp low.
///
/// `None` for other shapes and points. A w1 below 2^-53, which the complement rounds to 0, has no
/// evidence (TODO(T-3150)).
fn shape_one_tails(x: f64, y: f64, a: f64, b: f64) -> Option<(f64, f64)> {
    let swap = x > 0.5;
    let (a0, b0, x0, y0) = if swap { (b, a, y, x) } else { (a, b, x, y) };
    if a0 != 1.0 || b0 == 1.0 || y0 <= A1_CLOSED_FORM_MAX_Y {
        return None;
    }
    let w = -excel_expm1(b0 * excel_ln_one_minus(x0));
    let w1 = (0.5 - w) + 0.5;
    Some(if swap { (w1, w) } else { (w, w1) })
}

/// (`I_x(a, b)`, 1 - `I_x(a, b)`) from one evaluation, the unreflected side computed and the other
/// its complement, for 0 < x < 1 outside the closed forms: Excel's two F tails add up to exactly
/// 1 at the mean, `F.DIST.RT(1,3,7)+F.DIST(1,3,7,TRUE)` = 1 (probes-numeric/t1049-fdistrt A27),
/// which two separate evaluations miss by 6E-16.
fn beta_tails(x: f64, y: f64, a: f64, b: f64, kernel: Kernel) -> (f64, f64) {
    let flip = x > a / (a + b);
    let (x, y, a, b) = if flip { (y, x, b, a) } else { (x, y, a, b) };
    let accurate = match kernel {
        Kernel::Accurate => half_integer_tail(x, y, a, b),
        Kernel::Cephes => None,
    };
    let v = if let Some(v) = accurate {
        v
    } else if a + b < MAXGAM {
        let w = cephes_fraction(a, b, x, y);
        (a * x.ln()).exp() * (b * y.ln()).exp() * gamma_ratio(a, b) / a * w
    } else {
        large_prefactor(a, b, x, y) * cephes_fraction(a, b, x, y) / a
    };
    if flip {
        (1.0 - v, v)
    } else {
        (v, 1.0 - v)
    }
}

/// `I_x(a, b)` for half-integer shapes (the F tails) with a + b from [`SERIES_MIN_SHAPE_SUM`] to
/// below [`MAXGAM`] and x at most the mean, close to correctly rounded: x^a y^b with a ln x + b ln y
/// in double-double, times the remainder `I_x(a,b)/(x^a y^b)`, a double-double series of positive
/// terms (`half_integer_beta::half_integer_remainder`). Cephes' prefactor and fraction there are up
/// to about 40 ulp off a 50-digit reference (`F.DIST.RT(1,50,100)`, `F.DIST(0.5,50,100,TRUE)`,
/// probes-numeric/m9-f-dist-rt A18, m9-f-dist A25), where Excel is within half an ulp (T-2219).
/// `None` for other shapes. Not Excel's arithmetic: an accuracy stand-in for the bfrac, basym and
/// bgrat branches of the bratio port (SPEC-0025), which replace it, libm `exp` included, as they
/// land (T-2508).
fn half_integer_tail(x: f64, y: f64, a: f64, b: f64) -> Option<f64> {
    if !(is_half_integer_shape(a)
        && is_half_integer_shape(b)
        && (SERIES_MIN_SHAPE_SUM..MAXGAM).contains(&(a + b)))
    {
        return None;
    }
    // The argument above 1/2 is taken as 1 minus the other, exactly in double-double: the caller
    // forms x and y each rounded, and the rounding of the one near 1 is what a large shape
    // multiplies (`F.DIST(1,50,7,TRUE)`, probes-numeric/t2220-f-dist-sweep-cdf A854, is 13 ulp
    // off with ln x and within 1 with ln(1 - y)).
    let (x, y) = (complement_above_half(x, y), complement_above_half(y, x));
    let exponent = dd_add(
        dd_mul((a, 0.0), ln_double_double_pair(x)),
        dd_mul((b, 0.0), ln_double_double_pair(y)),
    );
    let (hi, lo) = fast_two_sum(exponent.0, exponent.1);
    Some(hi.exp() * (1.0 + lo) * half_integer_remainder(x, a, b))
}

/// `v` as a double-double, as 1 - `other` (exact by `TwoSum`) when `v` is above 1/2.
fn complement_above_half(v: f64, other: f64) -> (f64, f64) {
    if v > 0.5 {
        two_sum(1.0, -other)
    } else {
        (v, 0.0)
    }
}

/// 1 - `I_x(a, b)`, the upper tail of [`beta_reg_complement`], with y = 1 - x given: the other tail
/// of the same evaluation, Excel's closed forms ([`closed_form_tails`]) included.
pub(crate) fn beta_reg_upper(x: f64, y: f64, a: f64, b: f64) -> f64 {
    search_tails(x, y, a, b).1
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
    (a * ln_near_one(x, y) + b * ln_near_one(y, x) - ln_beta).exp()
}

/// ln x for x in (0, 1) with its complement y = 1 - x: ln(1 - y) from y above x = 1/2, as TOMS 708
/// `brcmp1` (`alnrel(-y)`), because a rounded x near 1 keeps only the digits of y that fit next to
/// the 1. A huge shape multiplies that error: `F.INV.RT(0.05,99999,9)` (`I_x(49999.5, 4.5)` with
/// y = 3.3E-5) was 48 units of the 15th digit off Excel's 2.7067247920355353 with ln x, 1 with ln(1 - y)
/// (gen-m9/27601819397dd56e A81, T-2465).
pub(crate) fn ln_near_one(x: f64, y: f64) -> f64 {
    if x > 0.5 {
        (-y).ln_1p()
    } else {
        x.ln()
    }
}

/// ln Gamma(b + a) - ln Gamma(b) for b of [`STIRLING_MIN`] or more (TOMS 708 `algdiv`):
/// a ln b + (b + a - 1/2) ln(1 + a/b) - a plus the Stirling corrections.
fn gamma_ln_shift(a: f64, b: f64) -> f64 {
    a * b.ln() + (b + a - 0.5) * (a / b).ln_1p() - a + stirling_correction(b + a)
        - stirling_correction(b)
}

/// ln Gamma(z) - ((z - 1/2) ln z - z + ln(2 pi)/2) for z of [`STIRLING_MIN`] or more: Stirling's
/// series to the 1/z^15 term (the next is below 2^-53 times the sum at z = 8).
pub(super) fn stirling_correction(z: f64) -> f64 {
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
pub(super) fn rlog1(t: f64) -> f64 {
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
mod tests {
    use super::*;
    use crate::test::util::ulps;

    // T-2465: with one shape below 8 and a huge other, ln x of a rounded x near 1 lost most of its
    // digits; ln(1 - y) from the given y keeps them. References: 60-digit mpmath I_y(b, a) at the
    // same doubles, the F.DIST.RT tails at Excel's F.INV.RT(0.05,99999,9) and
    // F.INV.RT(0.001,99999,1) (gen-m9/27601819397dd56e A81, c18dbf21f11093f3 A190), which ln x
    // missed by about 1E-12 relative.
    #[test]
    fn a_huge_shape_keeps_the_digits_of_a_small_complement() {
        for (df1, df2, x, expected) in [
            (
                99_999.0,
                9.0,
                2.706_724_792_035_535_3,
                0.049_999_999_999_999_93,
            ),
            (
                99_999.0,
                1.0,
                636_616.255_908_127_8,
                0.000_999_999_999_999_999_4,
            ),
        ] {
            let total = df1 * x + df2;
            let tail = beta_reg_upper(df1 * x / total, df2 / total, df1 / 2.0, df2 / 2.0);
            // Within 1E-14 relative (about 2E-15 now; ln x was about 1E-12 off).
            assert!((tail / expected - 1.0).abs() < 1e-14, "{df1} {df2}: {tail}");
        }
        assert_eq!(ln_near_one(1.0 - 3.3e-5, 3.3e-5), (-3.3e-5f64).ln_1p());
    }

    /// Excel cells of T-2464 w5-triangulation's closed-form set (`closed_final.py`, 1,286 of 1,286
    /// bit-exact): (x, y, a, b, lower tail, Excel's tail), x and y as each function passes them
    /// (BETA.DIST, BINOM, NEGBINOM: y = fl(1 - x); F: `fisher.rs` `f_beta_arguments`; T.DIST.RT at
    /// df 2: x = fl(2/fl(2 + t^2)), y = fl(1 - x), the tail twice Excel's value). All from
    /// probes-numeric.
    const CLOSED_FORM_CELLS: [(f64, f64, f64, f64, bool, f64); 24] = [
        // BETA.DIST(0.1,1,50,TRUE) m9-beta-dist-a1 A43: a0 = 1 at y0 = 0.9.
        (0.1, 0.9, 1.0, 50.0, true, 0.994_846_224_792_679_9),
        // BETA.DIST(0.5,1,2.5,TRUE) m9-beta-dist-a1 A27.
        (0.5, 0.5, 1.0, 2.5, true, 0.823_223_304_703_363_1),
        // BETA.DIST(0.9,1,2.5,TRUE) and (0.9,1,5,TRUE) m9-beta-dist-a1 A29, A35: swapped to b0 = 1
        // at x0 = fl(1 - 0.9) (T-1744's two a = 1 misses).
        (
            0.9,
            0.099_999_999_999_999_98,
            1.0,
            2.5,
            true,
            0.996_837_722_339_831_7,
        ),
        (
            0.9,
            0.099_999_999_999_999_98,
            1.0,
            5.0,
            true,
            0.999_989_999_999_999_9,
        ),
        // BETA.DIST(0.9,0.5,1,TRUE) m9-beta-dist-a0p5 A17: swapped to a0 = 1.
        (
            0.9,
            0.099_999_999_999_999_98,
            0.5,
            1.0,
            true,
            0.948_683_298_050_513_8,
        ),
        // BETA.DIST(0.1,1,1,TRUE) m9-beta-dist-a1 A13.
        (0.1, 0.9, 1.0, 1.0, true, 0.100_000_000_000_000_02),
        // BETA.DIST(0.5011872336272724,10,1,TRUE) t1745-beta-inv-roundtrip H110.
        (
            0.501_187_233_627_272_4,
            0.498_812_766_372_727_65,
            10.0,
            1.0,
            true,
            0.001_000_000_000_000_001,
        ),
        // BINOM.DIST(4,5,0.9,TRUE) and (14,15,0.125,TRUE) t2145-binomial-sweep A374, A1894: the
        // upper tail of I_p(k+1, n-k).
        (
            0.9,
            0.099_999_999_999_999_98,
            5.0,
            1.0,
            false,
            0.409_509_999_999_999_93,
        ),
        (0.125, 0.875, 15.0, 1.0, false, 0.999_999_999_999_971_6),
        // BINOM.DIST(4,5,0.2,TRUE) t2145-binomial-sweep A338.
        (0.2, 0.8, 5.0, 1.0, false, 0.999_68),
        // F.DIST(0.5,2,1,TRUE) t2220-f-dist-sweep-cdf A103.
        (0.5, 0.5, 1.0, 0.5, true, 0.292_893_218_813_452_4),
        // F.DIST(1,2,4,TRUE) t2220-f-dist-sweep-cdf A134.
        (
            0.333_333_333_333_333_3,
            0.666_666_666_666_666_6,
            1.0,
            2.0,
            true,
            0.555_555_555_555_555_5,
        ),
        // F.DIST.RT(3,4,2) t2220-f-dist-sweep-rt A317: swapped to a0 = 1, 1 - x0 recomputed.
        (
            0.857_142_857_142_857_1,
            0.142_857_142_857_142_85,
            2.0,
            1.0,
            false,
            0.265_306_122_448_979_44,
        ),
        // F.DIST(0.0356,1,2,TRUE) t2220-f-dist-sweep-d1-1 A124.
        (
            0.017_488_701_120_062_88,
            0.982_511_298_879_937,
            0.5,
            1.0,
            true,
            0.132_244_852_905_747_8,
        ),
        // F.DIST(5,2,2,TRUE) t2220-f-dist-sweep-cdf A118.
        (
            0.833_333_333_333_333_4,
            0.166_666_666_666_666_66,
            1.0,
            1.0,
            true,
            0.833_333_333_333_333_3,
        ),
        // F.DIST.RT(1270,2,1) t2220-f-dist-sweep-d2-1 A69.
        (
            0.999_606_454_151_908_7,
            0.000_393_545_848_091_302_6,
            1.0,
            0.5,
            false,
            0.019_837_990_021_453_856,
        ),
        // NEGBINOM.DIST(5,1,0.25,TRUE) t2145-negbinom-sweep A34: I_p(r, f+1).
        (0.25, 0.75, 1.0, 6.0, true, 0.822_021_484_375),
        // NEGBINOM.DIST(0,10,0.9,TRUE) t2145-negbinom-sweep A1036.
        (
            0.9,
            0.099_999_999_999_999_98,
            10.0,
            1.0,
            true,
            0.348_678_440_100_000_04,
        ),
        // NEGBINOM.DIST(0,5,0.1,TRUE) t2145-negbinom-sweep A728.
        (0.1, 0.9, 5.0, 1.0, true, 1.000_000_000_000_001_6e-5),
        // NEGBINOM.DIST(3,1,0.7,TRUE) and (3,1,0.999,TRUE) t2145-negbinom-sweep A140, A206.
        (0.7, 0.300_000_000_000_000_04, 1.0, 4.0, true, 0.9919),
        (
            0.999,
            0.001_000_000_000_000_000_9,
            1.0,
            4.0,
            true,
            0.999_999_999_999,
        ),
        // T.DIST.RT(3.79,2) t0969-t-df2-cut B359.
        (
            0.122_218_759_357_373_77,
            0.877_781_240_642_626_2,
            1.0,
            0.5,
            true,
            0.063_100_197_116_774_84,
        ),
        // T.DIST.RT(0.5,2) t0969-t-beta-link C9: swapped to b0 = 1.
        (
            0.888_888_888_888_888_8,
            0.111_111_111_111_111_16,
            1.0,
            0.5,
            true,
            0.666_666_666_666_666_6,
        ),
        // T.DIST.RT(0.8513,2) t0969-t-tail-grid B18.
        (
            0.734_022_615_067_945_1,
            0.265_977_384_932_054_9,
            1.0,
            0.5,
            true,
            0.484_270_046_504_903,
        ),
    ];

    #[test]
    fn closed_forms_on_the_oriented_arguments_match_excel() {
        for (x, y, a, b, lower, expected) in CLOSED_FORM_CELLS {
            let tails = closed_form_tails(x, y, a, b);
            let got = if lower {
                beta_reg_complement(x, y, a, b)
            } else {
                beta_reg_upper(x, y, a, b)
            };
            assert_eq!(got.to_bits(), expected.to_bits(), "{x} {y} {a} {b}: {got}");
            let (l, u) = tails.unwrap_or((f64::NAN, f64::NAN));
            assert_eq!(if lower { l } else { u }.to_bits(), expected.to_bits());
            // The other tail is (0.5 - w) + 0.5 of the computed one.
            assert!(l == (0.5 - u) + 0.5 || u == (0.5 - l) + 0.5, "{x} {a} {b}");
        }
    }

    /// The a0 = 1 cut, y0 <= 0.9: x0 = fl(1 - 0.9) takes the closed form, and x0 = 0.0998999751
    /// (`T.DIST.RT(4.245,2)`, probes-numeric t0969-t-df2-cut B450) is the general kernel. The
    /// unswapped a0 = 1 call at x0 = fl(1 - 0.9) with b = 2.5 is synthetic, not an Excel cell; the
    /// swapped one is `BETA.DIST(0.9,b,1,TRUE)`'s orientation (m9-beta-dist-a0p5 A17 at b = 0.5).
    #[test]
    fn a0_one_cut_is_at_y0_one_tenth_below_one() {
        let closed = 0.099_999_999_999_999_98;
        assert_eq!(1.0 - closed, 0.9);
        assert!(closed_form_tails(closed, 1.0 - closed, 1.0, 2.5).is_some());
        let open: f64 = 2.0 / (2.0 + 4.245 * 4.245);
        assert!((open - 0.099_899_975_1).abs() < 1e-10);
        assert_eq!(closed_form_tails(open, 1.0 - open, 1.0, 0.5), None);
        // Swapped, the same cut on the oriented y0 (the caller's x).
        assert!(closed_form_tails(1.0 - closed, closed, 2.5, 1.0).is_some());
        assert_eq!(closed_form_tails(1.0 - open, open, 0.5, 1.0), None);
        // b0 = 1 has no cut.
        assert!(closed_form_tails(1e-300, 1.0, 3.0, 1.0).is_some());
        // Other shapes are not closed forms.
        assert_eq!(closed_form_tails(0.3, 0.7, 2.0, 3.0), None);
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

    // T-2219: F tails at half-integer shapes from a + b = 10 against a 50-digit mpmath `betainc` at
    // the exact df1 x/(df1 x + df2). Cephes' kernel (`search_tails`) was 17 to 40 ulp off on these
    // (F.DIST.RT(1,50,100), probes-numeric/m9-f-dist-rt A18: 0.48912050780793204).
    #[test]
    fn half_integer_tails_are_close_to_correctly_rounded() {
        for (x, df1, df2, upper, reference) in [
            (1.0, 50.0, 100.0, false, 0.510_879_492_192_069_9),
            (1.0, 50.0, 100.0, true, 0.489_120_507_807_930_15),
            (1.0, 100.0, 3.0, true, 0.603_813_750_381_993_4),
            (1.0, 50.0, 7.0, false, 0.442_463_015_782_617_2),
            (10.0, 1.0, 50.0, true, 0.002_660_554_190_491_072),
        ] {
            let scaled: f64 = df1 * x;
            let total = scaled + df2;
            let (lower, upper_tail) =
                beta_reg_tails(scaled / total, df2 / total, df1 / 2.0, df2 / 2.0);
            let got = if upper { upper_tail } else { lower };
            assert!(
                ulps(got, reference) <= 4,
                "F({x},{df1},{df2}) upper={upper}: {got} against {reference}"
            );
        }
        // Below a + b = 10 and in the quantile searches' tails Cephes' kernel stays.
        let (u, v) = (0.25, 0.75);
        assert_eq!(beta_reg_tails(u, v, 1.5, 4.0), search_tails(u, v, 1.5, 4.0));
    }
}
