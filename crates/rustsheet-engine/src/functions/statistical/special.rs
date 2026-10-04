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

/// Wichura's AS241 (PPND16) inverse of the standard normal CDF, accurate to about 1e-16.
///
/// `p` must be in (0, 1). The statrs quantile goes through a lower-accuracy `erf_inv` and differs
/// from Excel's cached NORM.S.INV/NORM.INV/LOGNORM.INV/CONFIDENCE.NORM values around 1e-11.
pub(crate) fn normal_inv_std(p: f64) -> f64 {
    fn poly(c: &[f64; 8], r: f64) -> f64 {
        c.iter().rev().fold(0.0, |acc, &k| acc * r + k)
    }
    const A: [f64; 8] = [
        3.387_132_872_796_366_5,
        1.331_416_678_917_843_8e2,
        1.971_590_950_306_551_3e3,
        1.373_169_376_550_946_2e4,
        4.592_195_393_154_987e4,
        6.726_577_092_700_87e4,
        3.343_057_558_358_813e4,
        2.509_080_928_730_122_7e3,
    ];
    const B: [f64; 8] = [
        1.0,
        4.231_333_070_160_091e1,
        6.871_870_074_920_579e2,
        5.394_196_021_424_751e3,
        2.121_379_430_158_659_7e4,
        3.930_789_580_009_271e4,
        2.872_908_573_572_194_3e4,
        5.226_495_278_852_854e3,
    ];
    const C: [f64; 8] = [
        1.423_437_110_749_683_5,
        4.630_337_846_156_546,
        5.769_497_221_460_691,
        3.647_848_324_763_204_5,
        1.270_458_252_452_368_4,
        2.417_807_251_774_506e-1,
        2.272_384_498_926_918_4e-2,
        7.745_450_142_783_414e-4,
    ];
    const D: [f64; 8] = [
        1.0,
        2.053_191_626_637_759,
        1.676_384_830_183_803_8,
        6.897_673_349_851e-1,
        1.481_039_764_274_800_8e-1,
        1.519_866_656_361_645_7e-2,
        5.475_938_084_995_345e-4,
        1.050_750_071_644_416_9e-9,
    ];
    const E: [f64; 8] = [
        6.657_904_643_501_103,
        5.463_784_911_164_114,
        1.784_826_539_917_291_3,
        2.965_605_718_285_048_7e-1,
        2.653_218_952_657_612_4e-2,
        1.242_660_947_388_078_4e-3,
        2.711_555_568_743_487_6e-5,
        2.010_334_399_292_288e-7,
    ];
    const F: [f64; 8] = [
        1.0,
        5.998_322_065_558_879e-1,
        1.369_298_809_227_358e-1,
        1.487_536_129_085_061_5e-2,
        7.868_691_311_456_133e-4,
        1.846_318_317_510_054_7e-5,
        1.421_511_758_316_446e-7,
        2.044_263_103_389_939_7e-15,
    ];

    let q = p - 0.5;
    if q.abs() <= 0.425 {
        let r = 0.180_625 - q * q;
        return q * poly(&A, r) / poly(&B, r);
    }
    let tail = if q < 0.0 { p } else { 1.0 - p };
    let r = (-tail.ln()).sqrt();
    let z = if r <= 5.0 {
        let r = r - 1.6;
        poly(&C, r) / poly(&D, r)
    } else {
        let r = r - 5.0;
        poly(&E, r) / poly(&F, r)
    };
    if q < 0.0 {
        -z
    } else {
        z
    }
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

    // Reference values from Python's `statistics.NormalDist().inv_cdf` (also AS241).
    #[test]
    fn normal_inv_std_reference_values() {
        assert_eq!(normal_inv_std(0.5), 0.0);
        close(normal_inv_std(0.975), 1.959_963_984_540_054);
        close(normal_inv_std(0.025), -1.959_963_984_540_054);
        close(normal_inv_std(0.9), 1.281_551_565_544_600_4);
        close(normal_inv_std(1e-10), -6.361_340_902_404_056);
        close(normal_inv_std(1.0 - 1e-10), 6.361_340_889_697_422);
        assert!(normal_inv_std(1e-300) < -37.0);
    }

    #[test]
    fn normal_kernels() {
        close(normal_cdf(1.0, 0.0, 1.0), 0.841_344_746_068_542_9);
        close(normal_pdf(0.0, 0.0, 1.0), 0.398_942_280_401_432_7);
    }
}
