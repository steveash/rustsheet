//! TOMS 708 `basym`, bratio's asymptotic expansion of `I_x(a, b)` for two large shapes near the
//! mean (a0, b0 > 100 and λ within 3% of the smaller one; SPEC-0025's `basym.rs`), in doubles.
//!
//! Excel's arithmetic inside basym is not identified (T-3352 targets 15 digits, not bits). Before
//! this port such points fell through to today's kernel and then to statrs, which is grossly wrong
//! there: `BINOM.DIST(300000000,1000000000,0.3,TRUE)` came out -2.82 where Excel gives
//! 0.5000156001249112 (probes/t3352-binom-negbinom-large-n A108; statistics.md "BINOM.DIST and
//! NEGBINOM.DIST at a large n").

use super::super::incomplete_beta::rlog1;
use super::ln_beta::bcorr;
use crate::functions::util::poly;

/// The most coefficient pairs basym's series takes (TOMS's `num = 20`).
const NUM: usize = 20;

/// 2/sqrt(pi) as TOMS's `e0` writes it, 1.12837916709551 (short of `FRAC_2_SQRT_PI`'s digits).
const E0: f64 = f64::from_bits(0x3FF2_0DD7_5042_9B61);

/// 2^(-3/2), TOMS's `e1`.
const E1: f64 = 0.353_553_390_593_274;

/// TOMS 708 `basym(a, b, λ, eps)`: `I_x(a, b)` for a, b >= 15 (bratio calls it for both above
/// 100) with λ = (a + b) y - b >= 0, the Temme expansion in the scaled complementary error
/// function, summed until a pair of terms is below `eps` of the sum (bratio passes 100·eps).
/// `None` off its domain or where the result is not finite.
pub(super) fn basym(a: f64, b: f64, lambda: f64, eps: f64) -> Option<f64> {
    if !(a >= 15.0 && b >= 15.0 && lambda >= 0.0 && a.is_finite() && b.is_finite()) {
        return None;
    }
    let (h, r0, r1, w0) = if a < b {
        let h = a / b;
        (
            h,
            1.0 / (1.0 + h),
            (b - a) / b,
            1.0 / (a * (1.0 + h)).sqrt(),
        )
    } else {
        let h = b / a;
        (
            h,
            1.0 / (1.0 + h),
            (b - a) / a,
            1.0 / (b * (1.0 + h)).sqrt(),
        )
    };
    let f = a * rlog1(-lambda / a) + b * rlog1(lambda / b);
    let t = (-f).exp();
    if t == 0.0 {
        return Some(0.0);
    }
    let z0 = f.sqrt();
    let z = 0.5 * (z0 / E1);
    // z² = 2f.
    let z2 = f + f;

    // TOMS's 1-based arrays a0, b0, c, d, index i at [i - 1].
    let mut a0 = [0.0; NUM + 1];
    let mut b0 = [0.0; NUM + 1];
    let mut c = [0.0; NUM + 1];
    let mut d = [0.0; NUM + 1];
    a0[0] = (2.0 / 3.0) * r1;
    c[0] = -0.5 * a0[0];
    d[0] = -c[0];
    let mut j0 = (0.5 / E0) * erfc1_scaled(z0);
    let mut j1 = E1;
    let mut sum = j0 + d[0] * w0 * j1;

    let mut s = 1.0;
    let h2 = h * h;
    let mut hn = 1.0;
    let mut w = w0;
    let mut znm1 = z;
    let mut zn = z2;
    for n in (2..=NUM).step_by(2) {
        let n_f = n as f64;
        hn *= h2;
        a0[n - 1] = 2.0 * r0 * (1.0 + h * hn) / (n_f + 2.0);
        let np1 = n + 1;
        s += hn;
        a0[np1 - 1] = 2.0 * r1 * s / (n_f + 3.0);
        for i in n..=np1 {
            let r = -0.5 * (i as f64 + 1.0);
            b0[0] = r * a0[0];
            for m in 2..=i {
                let bsum: f64 = (1..m)
                    .map(|j| (j as f64 * r - (m - j) as f64) * a0[j - 1] * b0[m - j - 1])
                    .sum();
                b0[m - 1] = r * a0[m - 1] + bsum / m as f64;
            }
            c[i - 1] = b0[i - 1] / (i as f64 + 1.0);
            let dsum: f64 = (1..i).map(|j| d[i - j - 1] * c[j - 1]).sum();
            d[i - 1] = -(dsum + c[i - 1]);
        }
        j0 = E1 * znm1 + (n_f - 1.0) * j0;
        j1 = E1 * zn + n_f * j1;
        znm1 *= z2;
        zn *= z2;
        w *= w0;
        let t0 = d[n - 1] * w * j0;
        w *= w0;
        let t1 = d[np1 - 1] * w * j1;
        sum += t0 + t1;
        if t0.abs() + t1.abs() <= eps * sum {
            break;
        }
    }
    let u = (-bcorr(a.min(b), a.max(b))).exp();
    let result = E0 * t * u * sum;
    result.is_finite().then_some(result)
}

/// TOMS 708 `erfc1(1, x)` = exp(x²)·erfc(x) for x >= 0, its three rational approximations (basym
/// calls it at z0 = sqrt(f) >= 0 only). Coefficients lowest degree first, TOMS's arrays reversed.
fn erfc1_scaled(x: f64) -> f64 {
    const C: f64 = 0.564_189_583_547_756;
    const A: [f64; 5] = [
        0.128_379_167_095_513,
        0.047_913_714_560_768_1,
        0.032_307_657_922_583_4,
        -0.001_337_337_729_973_39,
        7.710_584_950_013_2e-5,
    ];
    const B: [f64; 3] = [
        0.375_795_757_275_549,
        0.053_897_168_774_028_6,
        0.003_010_486_317_038_95,
    ];
    const P: [f64; 8] = [
        300.459_261_020_162,
        451.918_953_711_873,
        339.320_816_734_344,
        152.989_285_046_94,
        43.162_227_222_056_7,
        7.211_758_250_883_09,
        0.564_195_517_478_974,
        -1.368_648_573_827_17e-7,
    ];
    const Q: [f64; 8] = [
        300.459_260_956_983,
        790.950_925_327_898,
        931.354_094_850_61,
        638.980_264_465_631,
        277.585_444_743_988,
        77.000_152_935_229_5,
        12.782_727_319_629_4,
        1.0,
    ];
    const R: [f64; 5] = [
        0.282_094_791_773_523,
        4.658_078_287_184_7,
        21.368_820_055_508_7,
        26.237_014_167_516_9,
        2.101_441_264_790_64,
    ];
    const S: [f64; 4] = [
        18.012_457_594_874_7,
        99.019_181_462_391_4,
        187.114_811_799_59,
        94.153_775_055_546,
    ];
    if x <= 0.5 {
        let t = x * x;
        let top = poly(&A, t) + 1.0;
        let bot = poly(&B, t) * t + 1.0;
        return t.exp() * (0.5 - x * (top / bot) + 0.5);
    }
    if x <= 4.0 {
        return poly(&P, x) / poly(&Q, x);
    }
    let t = 1.0 / (x * x);
    let top = poly(&R, t);
    let bot = poly(&S, t) * t + 1.0;
    (C - t * top / bot) / x
}

#[cfg(test)]
mod tests {
    use super::*;

    // exp(x^2) erfc(x) from 50-digit mpmath (`mpmath.erfc(x) * mpmath.exp(x**2)`), across the
    // three ranges; TOMS's rationals are good to about 1E-14 relative.
    #[test]
    fn erfc1_scaled_matches_mpmath() {
        for (x, reference) in [
            (0.0, 1.0),
            (0.25, 0.770_346_547_730_996_7),
            (0.5, 0.615_690_344_192_925_9),
            (1.0, 0.427_583_576_155_807),
            (3.0, 0.179_001_151_181_389_95),
            (4.0, 0.136_999_457_625_061_4),
            (10.0, 0.056_140_992_743_822_59),
            (26.0, 0.021_683_584_850_562_91),
        ] {
            let ours = erfc1_scaled(x);
            assert!(
                ((ours - reference) / reference).abs() < 1e-13,
                "x {x}: {ours} vs {reference}"
            );
        }
    }

    // BINOM.DIST(300000000,1000000000,0.3,TRUE) = 1 - I_0.3(300000001, 700000000), basym with λ as
    // bratio forms it: Excel's 0.5000156001249112 (probes/t3352-binom-negbinom-large-n A108; the
    // math at the double 0.3 is 0.50001560012554; the gap fits the rounding of (a + b) x in λ).
    #[test]
    fn basym_matches_excel_near_the_mean() {
        let (a, b, x) = (300_000_001.0, 700_000_000.0, 0.3);
        let lambda = a - (a + b) * x;
        let w = basym(a, b, lambda, 1e-13).unwrap_or(f64::NAN);
        let reference = 1.0 - 0.500_015_600_124_911_2;
        assert!(
            ((w - reference) / reference).abs() < 1e-13,
            "{w} vs {reference}"
        );
    }
}
