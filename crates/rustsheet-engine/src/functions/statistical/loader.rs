//! Catherine Loader's saddle-point pieces for binomial-type densities ("Fast and Accurate
//! Computation of Binomial Probabilities", 2000): the Stirling-formula error [`stirlerr`], the
//! deviance term [`bd0`] and the binomial density [`dbinom_raw_dev`] built from them. They keep full
//! relative accuracy where a direct power or a difference of `ln Gamma` values overflows or
//! cancels (a large shape), which is why R's nmath `dbinom`, `df`, `dbeta` and `dt` use them.
//! The structure follows R's nmath (`stirlerr.c`, `bd0.c`, `dbinom.c`, written here from the
//! published formulas); the half-integer table is the exact values (mpmath, 40 digits) rounded to
//! doubles, equal to R's `sferr_halves`. Two departures from R keep digits R loses: bd0 and the
//! density take the deviation x - np from the caller ([`bd0_dev`]), and ln(2 pi x (1 - x/n)) is one
//! logarithm with 1 - x/n from the exact n - x past n/2 (R's `log1p(-x/n)` was 4e-8 relative off
//! at `F.DIST(2,1E10,5,FALSE)`, and its `x ln(x/np) + np - x` up to 9,000 ulp at
//! `F.DIST(57400,50,1,FALSE)`).
//!
//! These are accurate forms, not Excel's algorithms: Excel's own densities carry their own
//! rounding (docs/excel-semantics/statistics.md, "F density is not the beta density, R's df or a
//! GAMMALN formula"). Shared by the F density (T-1864) and meant for the beta (T-2507) and
//! Student-t (T-2585) densities.

use std::f64::consts::PI;

use super::special::ln_gamma;

/// ln sqrt(2 pi) = 0.918938533204672741780..., the correctly rounded double
/// (`0x3FED_67F1_C864_BEB5`). Deliberately not `gammaln.rs`'s `LN_SQRT_2PI`, which is one ulp
/// lower because it reproduces the constant Excel's GAMMALN Stirling sum adds; these pieces are
/// accurate forms, not Excel's, so they take the exact value.
const LN_SQRT_2PI_CORRECTLY_ROUNDED: f64 = 0.918_938_533_204_672_8;

/// `stirlerr(k / 2)` for k = 0..=30; k = 0 is a placeholder (the function is infinite at 0).
const STIRLERR_HALVES: [f64; 31] = [
    0.0,
    0.153_426_409_720_027_36,
    0.081_061_466_795_327_26,
    0.054_814_121_051_917_65,
    0.041_340_695_955_409_3,
    0.033_162_873_519_936_29,
    0.027_677_925_684_998_34,
    0.023_746_163_656_297_496,
    0.020_790_672_103_765_093,
    0.018_488_450_532_673_187,
    0.016_644_691_189_821_193,
    0.015_134_973_221_917_378,
    0.013_876_128_823_070_748,
    0.012_810_465_242_920_227,
    0.011_896_709_945_891_77,
    0.011_104_559_758_206_917,
    0.010_411_265_261_972_096,
    0.009_799_416_126_158_804,
    0.009_255_462_182_712_733,
    0.008_768_700_134_139_386,
    0.008_330_563_433_362_87,
    0.007_934_114_564_314_02,
    0.007_573_675_487_951_841,
    0.007_244_554_301_320_383,
    0.006_942_840_107_209_53,
    0.006_665_247_032_707_682,
    0.006_408_994_188_004_207,
    0.006_171_712_263_039_458,
    0.005_951_370_112_758_847_5,
    0.005_746_216_513_010_115_5,
    0.005_554_733_551_962_801,
];

/// The Stirling series coefficients 1/12, 1/360, 1/1260, 1/1680, 1/1188.
const S0: f64 = 1.0 / 12.0;
const S1: f64 = 1.0 / 360.0;
const S2: f64 = 1.0 / 1260.0;
const S3: f64 = 1.0 / 1680.0;
const S4: f64 = 1.0 / 1188.0;

/// The bound on [`bd0`]'s series terms; it converges in far fewer (|v| < 0.1, so v^2 < 0.01).
const BD0_MAX_TERMS: u32 = 1000;

/// The error of Stirling's formula, `ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi)`, for
/// n > 0: exact table values at the half-integers up to 15, the Stirling series above 15 with
/// as many terms as the size of n needs (R's cut points 35, 80 and 500), and the defining
/// `ln Gamma` difference for other n up to 15 (less accurate, it cancels).
pub(crate) fn stirlerr(n: f64) -> f64 {
    if n <= 15.0 {
        let twice = n + n;
        if let Some((&value, _)) = STIRLERR_HALVES
            .iter()
            .zip(0_u8..)
            .skip(1)
            .find(|&(_, k)| f64::from(k) == twice)
        {
            return value;
        }
        return ln_gamma(n + 1.0) - (n + 0.5) * n.ln() + n - LN_SQRT_2PI_CORRECTLY_ROUNDED;
    }
    let nn = n * n;
    if n > 500.0 {
        (S0 - S1 / nn) / n
    } else if n > 80.0 {
        (S0 - (S1 - S2 / nn) / nn) / n
    } else if n > 35.0 {
        (S0 - (S1 - (S2 - S3 / nn) / nn) / nn) / n
    } else {
        (S0 - (S1 - (S2 - (S3 - S4 / nn) / nn) / nn) / nn) / n
    }
}

/// The deviance term `x ln(x / np) + np - x` for x >= 0 and np > 0. See [`bd0_dev`].
pub(crate) fn bd0(x: f64, np: f64) -> f64 {
    bd0_dev(x, np, x - np)
}

/// [`bd0`] with the deviation `diff = x - np` supplied by a caller that has it more accurately
/// than the rounded subtraction (the term is about diff^2 / (2 np) near np, so its relative
/// error is twice diff's). Within 10% of x + np it is Loader's series in v = diff / (x + np),
/// `diff v + 2 x sum_{j >= 1} v^(2j+1) / (2j + 1)`, at most [`BD0_MAX_TERMS`] terms. Further out
/// it is `x ln(x/np) - diff`, the logarithm taken as `ln_1p(diff/np)` for x >= np/2 so that it
/// keeps diff's digits too.
pub(crate) fn bd0_dev(x: f64, np: f64, diff: f64) -> f64 {
    if diff.abs() < 0.1 * (x + np) {
        let v = diff / (x + np);
        let mut s = diff * v;
        if s.abs() < f64::MIN_POSITIVE {
            return s;
        }
        let mut ej = 2.0 * x * v;
        let v2 = v * v;
        for j in 1..BD0_MAX_TERMS {
            ej *= v2;
            let s1 = s + ej / f64::from(2 * j + 1);
            if s1 == s {
                return s1;
            }
            s = s1;
        }
        return s;
    }
    if x == 0.0 {
        // The limit x ln x -> 0, where the product would be 0 * -inf.
        return np;
    }
    // np - x is -diff, so the sum's one cancelling term keeps diff's digits.
    let log_ratio = if x >= 0.5 * np {
        (diff / np).ln_1p()
    } else {
        (x / np).ln()
    };
    x * log_ratio - diff
}

/// The binomial density `C(n, x) p^x q^(n-x)` for real 0 <= x <= n (R's `dbinom_raw`), with
/// q = 1 - p and the deviation `diff = x - n p` passed separately so the caller keeps their
/// digits (the F density forms diff without cancellation; a caller without a better one passes
/// `x - n * p`). Loader's
/// `exp(lc - lf / 2)` with lc = stirlerr(n) - stirlerr(x) - stirlerr(n - x) - bd0(x, np) -
/// bd0(n - x, nq), each bd0 given its deviation (diff and -diff; np and nq are n p and n q, whose
/// relative accuracy is all bd0 needs once it has the deviation), and
/// lf = ln(2 pi x (1 - x/n)), taken as one logarithm; the end cases x = 0 and x = n in closed
/// form.
pub(crate) fn dbinom_raw_dev(x: f64, n: f64, p: f64, q: f64, diff: f64) -> f64 {
    if p == 0.0 {
        return if x == 0.0 { 1.0 } else { 0.0 };
    }
    if q == 0.0 {
        return if x == n { 1.0 } else { 0.0 };
    }
    if x == 0.0 {
        if n == 0.0 {
            return 1.0;
        }
        let lc = if p < 0.1 {
            -bd0(n, n * q) - n * p
        } else {
            n * q.ln()
        };
        return lc.exp();
    }
    if x == n {
        let lc = if q < 0.1 {
            -bd0(n, n * p) - n * q
        } else {
            n * p.ln()
        };
        return lc.exp();
    }
    if x < 0.0 || x > n {
        return 0.0;
    }
    let rest = n - x;
    let lc = stirlerr(n)
        - stirlerr(x)
        - stirlerr(rest)
        - bd0_dev(x, n * p, diff)
        - bd0_dev(rest, n * q, -diff);
    // 1 - x/n, from the exact (Sterbenz) n - x once x >= n/2, where R's log1p(-x/n) loses x/n's
    // rounding; then one logarithm of the product, which is small where ln x and ln(1 - x/n) are
    // large and of opposite sign (`F.DIST(2,1E10,5,FALSE)`: R's form was 4e-8 relative off).
    let rest_share = if x < 0.5 * n { 1.0 - x / n } else { rest / n };
    let product = 2.0 * PI * x * rest_share;
    let lf = if product.is_finite() && product > 0.0 {
        product.ln()
    } else {
        (2.0 * PI).ln() + x.ln() + rest_share.ln()
    };
    (lc - 0.5 * lf).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::util::ulps;

    #[test]
    fn stirlerr_matches_its_definition() {
        // The table and the series against the defining lgamma difference where that is still
        // accurate to about 1e-13 relative (n near 1), and the series' continuity at 15.
        for k in 1..=6 {
            let n = f64::from(k) / 2.0;
            let direct = ln_gamma(n + 1.0) - (n + 0.5) * n.ln() + n - LN_SQRT_2PI_CORRECTLY_ROUNDED;
            assert!((stirlerr(n) - direct).abs() < 1e-14, "{n}");
        }
        // Exact stirlerr(16) and stirlerr(100) (mpmath); the series' truncation: about 1e-16 absolute at 16 (the next term, 691/360360/n^11),
        // far below the rounding of the terms stirlerr is added to.
        assert!((stirlerr(16.0) - 0.005_207_655_919_609_640_4).abs() < 2e-16);
        assert!((stirlerr(100.0) - 0.000_833_330_555_634_914_7).abs() < 1e-17);
        assert_eq!(stirlerr(7.5).to_bits(), STIRLERR_HALVES[15].to_bits());
    }

    #[test]
    fn bd0_series_and_closed_form_agree() {
        // Near x = np the series; elsewhere the closed form. Both equal x ln(x/np) + np - x.
        assert_eq!(bd0(5.0, 5.0), 0.0);
        assert_eq!(bd0(0.0, 2.5), 2.5);
        let x = 10.0_f64;
        for np in [9.5, 10.5, 30.0, 1.0] {
            let closed = x * (x / np).ln() + np - x;
            assert!((bd0(x, np) - closed).abs() <= 1e-12 * closed.abs(), "{np}");
        }
    }

    #[test]
    fn ln_sqrt_2pi_is_correctly_rounded_and_one_ulp_above_gammaln_s() {
        // mpmath: ln sqrt(2 pi) = 0.91893853320467274178..., nearest double 0x1.d67f1c864beb5p-1.
        assert_eq!(
            LN_SQRT_2PI_CORRECTLY_ROUNDED.to_bits(),
            0x3FED_67F1_C864_BEB5
        );
        assert_eq!(
            LN_SQRT_2PI_CORRECTLY_ROUNDED.to_bits() - super::super::gammaln::LN_SQRT_2PI.to_bits(),
            1
        );
    }

    #[test]
    fn bd0_closed_form_below_half_np() {
        // x < np/2 takes x ln(x/np) - diff; references are mpmath (600 digits) at the same doubles,
        // rounded. Within 2 ulp: ln(x/np) rounds twice (the quotient, the logarithm) and the sum
        // once, and here the sum does not cancel (np - x dominates).
        for (x, np, expected) in [
            (1.0, 10.0, 6.697_414_907_005_954),
            (0.3, 1000.0, 997.266_481_575_007_6),
            (2.0, 4.5, 0.878_139_567_567_342_5),
        ] {
            let got = bd0(x, np);
            assert!(
                ulps(got, expected) <= 2,
                "bd0({x}, {np}) = {got}, want {expected}"
            );
        }
    }

    #[test]
    fn dbinom_raw_end_cases() {
        // x = 0 is q^n and x = n is p^n: through bd0 for a small p (resp. q), else n ln q. p =
        // 2^-10 keeps q = 1 - p exact. References: mpmath (1023/1024)^1000 and 0.75^10, rounded.
        // Within 4 ulp: the result is exp of a value near -1 or -2.9 carrying a few roundings
        // (an ulp of the exponent is about 0.25 to 1 ulp of the result).
        let small = 2.0_f64.powi(-10);
        let big = 1.0 - small;
        let cases = [
            (0.0, 1000.0, small, big, 0.376_423_798_056_724_05),
            (0.0, 10.0, 0.25, 0.75, 0.056_313_514_709_472_656),
            (1000.0, 1000.0, big, small, 0.376_423_798_056_724_05),
            (10.0, 10.0, 0.75, 0.25, 0.056_313_514_709_472_656),
        ];
        for (x, n, p, q, expected) in cases {
            let got = dbinom_raw_dev(x, n, p, q, x - n * p);
            assert!(
                ulps(got, expected) <= 4,
                "x={x} n={n} p={p}: {got}, want {expected}"
            );
        }
    }

    #[test]
    fn dbinom_raw_huge_n_takes_the_logarithm_apart() {
        // 2 pi x (1 - x/n) overflows for x = n/2 = 0.75 * 2^1023, so lf is the sum of three
        // logarithms. Reference: mpmath (600 digits) exp(lgamma(n+1) - 2 lgamma(x+1) - n ln 2)
        // at the same doubles. The density is exp(-0.5 lf) with lf about 710, whose ulp (1.1e-13)
        // is about 256 ulp of the result, so allow 512.
        let n = 1.5 * 2.0_f64.powi(1023);
        let x = 0.5 * n;
        assert!(!(2.0 * PI * x * 0.5).is_finite());
        let got = dbinom_raw_dev(x, n, 0.5, 0.5, 0.0);
        let expected = 6.871_501_566_382_49e-155;
        assert!(ulps(got, expected) <= 512, "{got}, want {expected}");
    }

    #[test]
    fn dbinom_raw_small_cases() {
        // C(4, 2) / 16 = 0.375; the end cases are q^n and p^n.
        assert!((dbinom_raw_dev(2.0, 4.0, 0.5, 0.5, 0.0) - 0.375).abs() < 1e-15);
        assert!((dbinom_raw_dev(0.0, 3.0, 0.5, 0.5, -1.5) - 0.125).abs() < 1e-16);
        assert!((dbinom_raw_dev(3.0, 3.0, 0.5, 0.5, 1.5) - 0.125).abs() < 1e-16);
        assert_eq!(dbinom_raw_dev(5.0, 3.0, 0.5, 0.5, 3.5), 0.0);
    }
}
