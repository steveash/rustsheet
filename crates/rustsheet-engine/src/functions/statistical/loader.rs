//! Catherine Loader's saddle point ("Fast and Accurate Computation of Binomial Probabilities",
//! 2000) as Excel's beta density computes it for min(a, b) > 1.
//!
//! Excel's beta density for min(a, b) > 1 is the same saddle point in Excel's own arithmetic
//! (statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point", T-2464, T-2513): x87
//! stored operations on Excel's LN, a 0.2 cut in its bd0 ([`excel_bd0`]), the exact Stirling error
//! ([`stirlerr_exact`]) and an exponent held in an x87 register into Excel's EXP
//! ([`excel_dbinom_exponent`], [`excel_beta_density`]). The model's shape constant carries an
//! unexplained residual of 2-4 units of 2^-53 and the series branch 3-41 units of noise (T-2586),
//! so it is not bit-exact everywhere.

use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_mul, excel_exp_extended, two_sum};
use crate::x87_log10::{excel_ln, ln_double_double_pair};
use crate::x87_register::Register;

/// A double-double `hi + lo`.
type Dd = (f64, f64);

/// ln sqrt(2 pi) = 0.918938533204672741780..., the correctly rounded double
/// (`0x3FED_67F1_C864_BEB5`). Deliberately not `gammaln.rs`'s `LN_SQRT_2PI`, which is one ulp
/// lower because it reproduces the constant Excel's GAMMALN Stirling sum adds; these pieces are
/// accurate forms, not Excel's, so they take the exact value.
const LN_SQRT_2PI_CORRECTLY_ROUNDED: f64 = 0.918_938_533_204_672_8;

/// The Stirling series coefficients 1/12, 1/360, 1/1260, 1/1680, 1/1188.
const S2: f64 = 1.0 / 1260.0;
const S3: f64 = 1.0 / 1680.0;
const S4: f64 = 1.0 / 1188.0;

/// The bound on [`bd0`]'s series terms; it converges in far fewer (|v| < 0.1, so v^2 < 0.01).
const BD0_MAX_TERMS: u32 = 1000;

/// The Stirling series terms after 1/(12 n) and -1/(360 n^3), `B_2k / (2k (2k - 1))` for k = 3..=12,
/// the first three of them [`S2`], -[`S3`] and [`S4`]. At n = 15 the next term is under 1e-26, far
/// below the 2^-64 of the exponent [`stirlerr_exact`] goes into.
const STIRLING_TAIL: [f64; 10] = [
    S2,
    -S3,
    S4,
    -691.0 / 360_360.0,
    1.0 / 156.0,
    -3617.0 / 122_400.0,
    43_867.0 / 244_188.0,
    -174_611.0 / 125_400.0,
    77_683.0 / 5_796.0,
    -236_364_091.0 / 1_506_960.0,
];

/// From this n on [`stirlerr_exact`] sums the Stirling series; below it the recurrence first
/// steps n up past it.
const STIRLING_SERIES_MIN: f64 = 15.0;

/// The exact Stirling error `ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi)` of a double
/// n > 0 as a double-double (about 1e-25 absolute), the stirlerr of Excel's beta density
/// (statistics.md "Beta density for min(a,b) > 1 ...": stirlerr is accurate, a GAMMALN or a
/// double-rounded lgamma form misses by rms 15-26 units). Not [`stirlerr`], whose rounded table
/// and lgamma difference the F density keeps. Below 15 the recurrence
/// stirlerr(m) = stirlerr(m + 1) + (m + 1/2) (ln(m + 1) - ln m) - 1 steps n up (at most 15 steps,
/// every m = n + i exact as a double-double), then the series in 1/m.
pub(crate) fn stirlerr_exact(n: f64) -> Dd {
    let mut sum = (0.0, 0.0);
    let mut step = 0.0;
    let mut ln_here = ln_double_double_pair(two_sum(n, 0.0));
    while n + step < STIRLING_SERIES_MIN {
        let ln_next = ln_double_double_pair(two_sum(n, step + 1.0));
        let factor = two_sum(n, step + 0.5);
        let term = dd_mul(factor, dd_add(ln_next, (-ln_here.0, -ln_here.1)));
        sum = dd_add(sum, dd_add(term, (-1.0, 0.0)));
        ln_here = ln_next;
        step += 1.0;
    }
    dd_add(sum, stirling_series(two_sum(n, step)))
}

/// Stirling's series `sum_k B_2k / (2k (2k - 1) m^(2k - 1))` for m >= 15, the two leading terms in
/// double-double and the [`STIRLING_TAIL`] in double (it is under 1e-9 there).
fn stirling_series(m: Dd) -> Dd {
    let z = dd_div((1.0, 0.0), m);
    let z2 = dd_mul(z, z);
    let tail = STIRLING_TAIL
        .iter()
        .rev()
        .fold(0.0, |acc, &coefficient| coefficient + z2.0 * acc);
    let second = dd_add(dd_div_small((-1.0, 0.0), 360.0), dd_mul(z2, (tail, 0.0)));
    let first = dd_add(dd_div_small((1.0, 0.0), 12.0), dd_mul(z2, second));
    dd_mul(z, first)
}

/// The ratio cut of Excel's bd0: its series runs while |k - M| < 0.2 (k + M) (statistics.md "Beta
/// density for min(a,b) > 1 ...": 0.25 drops (5.5, 12.5) from 113 to 13 of 120 exact cells; R's
/// is 0.1).
const EXCEL_BD0_CUT: f64 = 0.2;

/// Excel's bd0(k, M) = k ln(k / M) + M - k, as the double its beta density sums (model M4 of
/// docs/research/t2464/r3-loader-const, with r3-loader-bd0's knobs), in x87 stored operations:
/// d = fl(k - M) and s = fl(k + M); then
/// - the log form when |d| >= RN64(0.2 s): fl(t + fl(M - k)) with t = fl(k LN(fl(k / M))) and LN
///   Excel's (M - k stored: beyond a ratio of 2 it rounds, and the stored form fits Excel);
/// - below the cut Loader's series in v = fl(d / s) (stored), in extended precision, which carries
///   +1 times the rounding error of fl(2 k v): Excel's arithmetic there is not identified, and this
///   phenomenological term (r3-loader-bd0's `ph` knob) is what fits (T-2586 owns the rest).
///
/// For k > 0 and M > 0; the series has at most [`BD0_MAX_TERMS`] terms.
pub(crate) fn excel_bd0(k: f64, m: f64) -> f64 {
    let d = extended_sum(k, -m);
    let s = extended_sum(k, m);
    let cut = Register::load(EXCEL_BD0_CUT).mul(Register::load(s));
    if Register::load(d.abs()).add(cut.neg()).is_negative() {
        return excel_bd0_series(k, d, s);
    }
    let t = extended_product(k, excel_ln(extended_quotient(k, m)));
    extended_sum(t, extended_sum(m, -k))
}

/// [`excel_bd0`]'s series branch: R's loop `s = d v + 2k sum_j v^(2j+1) / (2j + 1)` in x87 registers
/// with v stored, less the rounding error of fl(2 k v), stored once at the end.
fn excel_bd0_series(k: f64, d: f64, s: f64) -> f64 {
    let v = Register::load(extended_quotient(d, s));
    let mut ej = Register::load(2.0 * k).mul(v);
    let rounding = Register::load(ej.store()).add(ej.neg());
    let mut sum = Register::load(d).mul(v).add(rounding.neg());
    let v2 = v.mul(v);
    for j in 1..BD0_MAX_TERMS {
        ej = ej.mul(v2);
        let next = sum.add(ej.div(Register::load(f64::from(2 * j + 1))));
        if next.same_value(sum) {
            return next.store();
        }
        sum = next;
    }
    sum.store()
}

/// ln(2 pi), the double Excel's lf starts from (R's `M_LN_2PI`, correctly rounded:
/// `0x1.d67f1c864beb5p+0`, exactly twice [`LN_SQRT_2PI_CORRECTLY_ROUNDED`]). Bit-exact counts are
/// very sensitive to it (statistics.md "Beta density for min(a,b) > 1 ...").
const LN_2PI: f64 = 2.0 * LN_SQRT_2PI_CORRECTLY_ROUNDED;

/// The exponent of Excel's Loader binomial density `dbinom_raw(k, n, p, q)` = EXP(E) for
/// 0 < k < n, as the exact double-double sum
/// E = stirlerr(n) - stirlerr(k) - stirlerr(nk) - lf/2 + X, which Excel holds in an x87 register
/// (statistics.md "Beta density for min(a,b) > 1 ...", model M4):
/// - k, n and nk = n - k are the caller's stored doubles (Excel's own, not recomputed: single-ulp
///   neighbours of a shape jump by up to 20 units with the rounding of nk);
/// - the stirlerr terms exact ([`stirlerr_exact`]);
/// - lf = RN64(RN64(RN64(ln 2 pi + LN k) + LN nk) - LN n), Excel's LN, extended sums;
/// - X = RN64(-bd0(k, fl(n p)) - bd0(nk, fl(n q))) with [`excel_bd0`].
///
/// `None` where a piece is not finite (an n p or n q of 0, a ratio past the double range). Exposed
/// for the binomial pmf, which shares it (T-2146).
pub(crate) fn excel_dbinom_exponent(k: f64, n: f64, nk: f64, p: f64, q: f64) -> Option<Dd> {
    let np = extended_product(n, p);
    let nq = extended_product(n, q);
    if !(k > 0.0 && nk > 0.0 && np > 0.0 && nq > 0.0 && n.is_finite()) {
        return None;
    }
    let bd0_p = excel_bd0(k, np);
    let bd0_q = excel_bd0(nk, nq);
    // A register loads a NaN or an infinity as 0, so a ratio past the double range (k / M for a
    // subnormal M) must stop here.
    if !(bd0_p.is_finite() && bd0_q.is_finite()) {
        return None;
    }
    let x_part = Register::load(-bd0_p).add(Register::load(-bd0_q));
    let lf = Register::load(LN_2PI)
        .add(Register::load(excel_ln(k)))
        .add(Register::load(excel_ln(nk)))
        .add(Register::load(-excel_ln(n)));
    let (lf_hi, lf_lo) = lf.to_double_double();
    let parts = dd_add(stirlerr_exact(k), stirlerr_exact(nk));
    let stirling = dd_add(stirlerr_exact(n), (-parts.0, -parts.1));
    let exponent = dd_add(
        dd_add(stirling, (-0.5 * lf_hi, -0.5 * lf_lo)),
        x_part.to_double_double(),
    );
    (exponent.0.is_finite() && exponent.1.is_finite()).then_some(exponent)
}

/// Excel's BETA.DIST(x, a, b, FALSE) for min(a, b) > 1 and 0 < x < 1: R dbeta's
/// `(a + b - 1) dbinom_raw(a - 1, a + b - 2, x, fl(1 - x))` (Loader 2000) in Excel's arithmetic,
/// EXP(LN(n + 1) + E) with E [`excel_dbinom_exponent`]'s, the sum held in an x87 register into
/// Excel's EXP (statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point", T-2464,
/// T-2513). k = fl(a - 1), n = fl(fl(a + b) - 2) and nk = fl(n - k) are x87 stored operations. No
/// switch at large shapes (no a0 = 8 or a + b = 171.6 branch). `None` outside that domain or where
/// a piece is not finite, where the caller keeps its other forms.
pub(crate) fn excel_beta_density(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(a > 1.0 && b > 1.0 && x > 0.0 && x < 1.0 && a.is_finite() && b.is_finite()) {
        return None;
    }
    let k = extended_sum(a, -1.0);
    let n = extended_sum(extended_sum(a, b), -2.0);
    let nk = extended_sum(n, -k);
    let q = extended_sum(1.0, -x);
    let exponent = excel_dbinom_exponent(k, n, nk, x, q)?;
    let ln_scale = excel_ln(extended_sum(n, 1.0));
    let total = dd_add((ln_scale, 0.0), exponent);
    if !total.0.is_finite() {
        return None;
    }
    let density = excel_exp_extended(Register::from_double_double(total));
    density.is_finite().then_some(density)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stirlerr_exact_matches_mpmath() {
        // mpmath (50 digits) ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi) at the same
        // doubles, as hi + lo: the recurrence below 15 and the series from 15 on.
        for (n, hi, lo) in [
            (1e-10, 10.593_986_934_110_418, 7.124_858_491_460_29e-16),
            (0.03, 0.952_950_298_497_347_8, -9.226_922_120_730_322e-19),
            (0.5, 0.153_426_409_720_027_36, -1.159_523_406_923_149_8e-17),
            (1.0, 0.081_061_466_795_327_26, -2.850_421_842_770_954_6e-18),
            (4.3, 0.019_345_426_904_332_788, 2.960_181_844_418_298e-19),
            (7.5, 0.011_104_559_758_206_917, -2.038_131_079_125_446e-20),
            (14.99, 0.005_558_438_081_636_329, 2.937_702_605_005_666e-19),
            (15.0, 0.005_554_733_551_962_801, 3.185_347_414_196_856e-19),
            (
                16.0,
                0.005_207_655_919_609_640_4,
                -2.883_608_994_661_908_6e-21,
            ),
            (
                100.0,
                0.000_833_330_555_634_914_7,
                1.246_488_391_106_547_6e-20,
            ),
        ] {
            let (got_hi, got_lo) = stirlerr_exact(n);
            let error = (got_hi - hi) + (got_lo - lo);
            assert!(
                error.abs() < 1e-24 * hi.max(1.0),
                "stirlerr({n}): {error:e}"
            );
        }
    }

    #[test]
    fn excel_beta_density_matches_excel() {
        // Excel's BETA.DIST(x, a, b, FALSE) bit for bit (the arguments are the doubles of each
        // cell's formula; the model is bit-exact on about 20-30% of cells, T-2586): both bd0 sides
        // in the log form (probes-numeric t2513-r3-loader-const-hold1 A342, A369, A3251), a series
        // side (t2513-r3-loader-bd0-a A4814, A5192, A5787; t2513-verify-r3-loader-bd0-hold A649,
        // A1214), integer and half-integer shapes (t2513-r3-loader-bd0-b A2760, A3666), a shape near 1
        // and min >= 8 (t2464-r2-density-large-p1 A9967, A7281) and diverse shapes
        // (t2513-r3-loader-const-diverse1 A252, A1307).
        let cases: [(f64, f64, f64, f64); 14] = [
            (
                0.376_922_732_080_715_4,
                8.3,
                5.899_999_999_999_998,
                0.886_766_852_474_694_8,
            ),
            (
                0.365_288_592_308_167_5,
                8.3,
                5.899_999_999_999_999_5,
                0.772_288_531_152_508_3,
            ),
            (
                0.444_288_270_094_662_3,
                33.976,
                14.85,
                0.009_542_680_147_732_819,
            ),
            (0.523_647_654_910_737_2, 5.5, 12.5, 0.534_367_175_242_143_3),
            (0.344_717_003_125_003_9, 5.5, 12.5, 3.187_884_049_445_971_7),
            (0.192_030_067_200_634_6, 5.5, 12.5, 2.547_970_230_648_507),
            (0.128_688_030_000_000_23, 9.7, 21.3, 0.258_876_334_448_779_7),
            (
                0.707_297_924_512_993_8,
                4.75,
                11.25,
                0.011_149_186_257_390_1,
            ),
            (0.247_103_647_826_088_96, 3.0, 5.0, 2.060_100_389_491_967),
            (0.679_114_979_411_773_7, 4.5, 4.5, 1.439_649_122_004_031),
            (0.099_840_000_000_003_4, 1.1, 1.9, 1.579_203_777_920_108_2),
            (0.165_435_806_451_616, 8.5, 40.0, 7.447_826_760_045_769),
            (
                0.781_164_318_528_283_5,
                7.3,
                7.501,
                0.232_804_337_859_794_24,
            ),
            (0.616_917_685_110_905_7, 8.67, 1.45, 0.428_699_138_085_424_5),
        ];
        for (x, a, b, excel) in cases {
            let got = excel_beta_density(x, a, b);
            assert_eq!(
                got.map(f64::to_bits),
                Some(excel.to_bits()),
                "BETA.DIST({x},{a},{b},FALSE) = {got:?}, Excel {excel}"
            );
        }
    }

    #[test]
    fn excel_beta_density_domain() {
        // Outside min(a, b) > 1 and 0 < x < 1, and where n x underflows to 0, the caller keeps
        // its other forms.
        assert_eq!(excel_beta_density(0.5, 1.0, 3.0), None);
        assert_eq!(excel_beta_density(0.5, 3.0, 0.5), None);
        assert_eq!(excel_beta_density(0.0, 3.0, 3.0), None);
        assert_eq!(excel_beta_density(1.0, 3.0, 3.0), None);
        assert_eq!(excel_beta_density(5e-324, 1.5, 3.0), None);
        assert_eq!(excel_beta_density(f64::NAN, 3.0, 3.0), None);
    }
}
