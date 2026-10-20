//! Excel's BESSELJ / BESSELY.
//!
//! Excel does not use fdlibm: its results are bit-identical to Numerical Recipes
//! (`bessj0/bessj1/bessy0/bessy1`, ~1e-8 accuracy): rational approximations for `|x| < 8`, the
//! asymptotic `P/Q` form for `|x| >= 8`, upward recurrence for higher orders and Miller's
//! downward recurrence for `J_n(x)` when `n >= x`. Three of Excel's large-argument coefficients
//! differ from the book (see `Asymptotic`).
//! Evidence: `eval/suites/ironcalc-calc/ENGINEERING/BESSEL.xlsx` (Excel-cached) and the
//! `probes-numeric/m9-bessel` sweep (T-1004), every cell of which these formulas reproduce exactly
//! (`docs/excel-semantics/engineering.md`, T-0908).

use crate::functions::util::poly;

/// `|x|` from which J0, J1, Y0 and Y1 switch from the rational to the asymptotic form.
const SMALL_ARG_LIMIT: f64 = 8.0;
/// Orders above this yield NaN (reported as `#NUM!`): every Bessel algorithm here, BESSELI's and
/// BESSELK's included, is O(n) per cell and the formula layer truncates `n` from untrusted input.
/// It is each kernel's own per-call cap (SPEC-0011 "Security rule for kernel work"). Excel's own cap
/// is not measured; `100_000` is far beyond any n where results are finite anyway. Deliberate
/// deviation: see docs/excel-semantics/engineering.md "BESSEL order cap" (decision 0014).
pub(crate) const MAX_ORDER: i32 = 100_000;
// Numerical Recipes truncates 2/pi; Excel's output bakes in the truncated constant.
const TWO_OVER_PI: f64 = 636_619_772.0 / 1e9; // == 0.636619772 (correctly rounded)

// The book's truncated phase shifts pi/4 and 3pi/4, also baked into Excel's output.
const QUARTER_PI: f64 = 0.785_398_164;
const THREE_QUARTER_PI: f64 = 2.356_194_491;

/// The `|x| >= 8` asymptotic coefficients `P` (without its leading 1) and `Q` in `y = (8/x)^2`.
/// Excel's tables differ from Numerical Recipes in three places, each fitted from the m9-bessel
/// sweep (least squares on 14 x values per function, then bit-exact on all 354 cells):
/// J0's `P` has `-0.1098628267e-2` (the book: `-0.1098628627e-2`), Y0's `Q` has
/// `-0.934945152e-7` (the book: `-0.934935152e-7`), and J1's `P` repeats `-0.3516396496e-4`,
/// which makes it one degree longer. Y1 uses the book's coefficients.
struct Asymptotic {
    p: &'static [f64],
    q: &'static [f64],
    shift: f64,
}

const J0_LARGE: Asymptotic = Asymptotic {
    p: &[
        -0.109_862_826_7e-2,
        0.273_451_040_7e-4,
        -0.207_337_063_9e-5,
        0.209_388_721_1e-6,
    ],
    q: &[
        -0.156_249_999_5e-1,
        0.143_048_876_5e-3,
        -0.691_114_765_1e-5,
        0.762_109_516_1e-6,
        -0.934_935_152e-7,
    ],
    shift: QUARTER_PI,
};
const Y0_LARGE: Asymptotic = Asymptotic {
    p: &[
        -0.109_862_862_7e-2,
        0.273_451_040_7e-4,
        -0.207_337_063_9e-5,
        0.209_388_721_1e-6,
    ],
    q: &[
        -0.156_249_999_5e-1,
        0.143_048_876_5e-3,
        -0.691_114_765_1e-5,
        0.762_109_516_1e-6,
        -0.934_945_152e-7,
    ],
    shift: QUARTER_PI,
};
const J1_LARGE: Asymptotic = Asymptotic {
    p: &[
        0.183_105e-2,
        -0.351_639_649_6e-4,
        -0.351_639_649_6e-4,
        0.245_752_017_4e-5,
        -0.240_337_019e-6,
    ],
    q: &[
        0.046_874_999_95,
        -0.200_269_087_3e-3,
        0.844_919_909_6e-5,
        -0.882_289_87e-6,
        0.105_787_412e-6,
    ],
    shift: THREE_QUARTER_PI,
};
const Y1_LARGE: Asymptotic = Asymptotic {
    p: &[
        0.183_105e-2,
        -0.351_639_649_6e-4,
        0.245_752_017_4e-5,
        -0.240_337_019e-6,
    ],
    q: J1_LARGE.q,
    shift: THREE_QUARTER_PI,
};

/// `sqrt(2/(pi x)) * (P cos(x - shift) - z Q sin(x - shift))` for J (`is_y` false) or
/// `sqrt(2/(pi x)) * (P sin(x - shift) + z Q cos(x - shift))` for Y, `z = 8/x`, `x >= 8`.
/// `(z * Q) * sin` is the association that is bit-exact on every sweep cell; `z * (sin * Q)` is
/// 1 ulp off at `BESSELY(8.5,1)`.
fn asymptotic(table: &Asymptotic, x: f64, is_y: bool) -> f64 {
    let z = 8.0 / x;
    let y = z * z;
    let xx = x - table.shift;
    let p = 1.0 + y * poly(table.p, y);
    let zq = z * poly(table.q, y);
    let (sin, cos) = xx.sin_cos();
    let sum = if is_y {
        sin * p + zq * cos
    } else {
        cos * p - zq * sin
    };
    (TWO_OVER_PI / x).sqrt() * sum
}

fn j0(x: f64) -> f64 {
    if x.abs() >= SMALL_ARG_LIMIT {
        return asymptotic(&J0_LARGE, x.abs(), false);
    }
    let y = x * x;
    let num = poly(
        &[
            57_568_490_574.0,
            -13_362_590_354.0,
            651_619_640.7,
            -11_214_424.18,
            77_392.330_17,
            -184.905_245_6,
        ],
        y,
    );
    let den = poly(
        &[
            57_568_490_411.0,
            1_029_532_985.0,
            9_494_680.718,
            59_272.648_53,
            267.853_271_2,
            1.0,
        ],
        y,
    );
    num / den
}

fn j1(x: f64) -> f64 {
    if x.abs() >= SMALL_ARG_LIMIT {
        let ans = asymptotic(&J1_LARGE, x.abs(), false);
        return if x < 0.0 { -ans } else { ans };
    }
    let y = x * x;
    let num = x * poly(
        &[
            72_362_614_232.0,
            -7_895_059_235.0,
            242_396_853.1,
            -2_972_611.439,
            15_704.482_60,
            -30.160_366_06,
        ],
        y,
    );
    let den = poly(
        &[
            144_725_228_442.0,
            2_300_535_178.0,
            18_583_304.74,
            99_447.433_94,
            376.999_139_7,
            1.0,
        ],
        y,
    );
    num / den
}

fn y0(x: f64) -> f64 {
    if x >= SMALL_ARG_LIMIT {
        return asymptotic(&Y0_LARGE, x, true);
    }
    let y = x * x;
    let num = poly(
        &[
            -2_957_821_389.0,
            7_062_834_065.0,
            -512_359_803.6,
            10_879_881.29,
            -86_327.927_57,
            228.462_273_3,
        ],
        y,
    );
    let den = poly(
        &[
            40_076_544_269.0,
            745_249_964.8,
            7_189_466.438,
            47_447.264_70,
            226.103_024_4,
            1.0,
        ],
        y,
    );
    num / den + TWO_OVER_PI * (j0(x) * x.ln())
}

fn y1(x: f64) -> f64 {
    if x >= SMALL_ARG_LIMIT {
        return asymptotic(&Y1_LARGE, x, true);
    }
    let y = x * x;
    let num = x * poly(
        &[
            -0.490_060_494_3e13,
            0.127_527_439_0e13,
            -0.515_343_813_9e11,
            0.734_926_455_1e9,
            -0.423_792_272_6e7,
            0.851_193_793_5e4,
        ],
        y,
    );
    let den = poly(
        &[
            0.249_958_057_0e14,
            0.424_441_966_4e12,
            0.373_365_036_7e10,
            0.224_590_400_2e8,
            0.102_042_605_0e6,
            0.354_963_288_5e3,
            1.0,
        ],
        y,
    );
    num / den + TWO_OVER_PI * (j1(x) * x.ln() - 1.0 / x)
}

/// Miller's downward recurrence for `J_n(|x|)`, `n >= 2`, `0 < |x| <= n`.
fn jn_miller(n: i32, ax: f64) -> f64 {
    let tox = 2.0 / ax;
    // Even starting order, as in Numerical Recipes (`ACC = 40`).
    let m = 2 * ((n + (40.0 * f64::from(n)).sqrt() as i32) / 2);
    let (mut bj, mut bjp, mut ans, mut sum) = (1.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    let mut odd = false;
    for j in (1..=m).rev() {
        let bjm = f64::from(j) * tox * bj - bjp;
        bjp = bj;
        bj = bjm;
        if bj.abs() > 1e10 {
            bj *= 1e-10;
            bjp *= 1e-10;
            ans *= 1e-10;
            sum *= 1e-10;
        }
        if odd {
            sum += bj;
        }
        odd = !odd;
        if j == n {
            ans = bjp;
        }
    }
    ans / (2.0 * sum - bj)
}

/// `BESSELJ(x, n)` for integer `n >= 0`.
pub(crate) fn excel_bessel_j(n: i32, x: f64) -> f64 {
    if n > MAX_ORDER {
        return f64::NAN;
    }
    let ax = x.abs();
    let ans = match n {
        // J_0(0) is 1.00000000283141 in Excel (NR numerator/denominator ratio), not exactly 1.
        0 => return j0(x),
        1 => return j1(x),
        _ if ax == 0.0 => return 0.0,
        _ if ax > f64::from(n) => {
            // `b * (2j * (1/x))`, as for Y below: `(2j / x) * b` is 1 ulp off on 11 m9-bessel
            // cells (e.g. `BESSELJ(8.5,4)`).
            let inv_x = 1.0 / ax;
            let (mut a, mut b) = (j0(ax), j1(ax));
            for j in 1..n {
                (a, b) = (b, b * (2.0 * f64::from(j) * inv_x) - a);
            }
            b
        }
        _ => jn_miller(n, ax),
    };
    if x < 0.0 && n % 2 == 1 {
        -ans
    } else {
        ans
    }
}

/// `BESSELY(x, n)` for integer `n >= 0`.
pub(crate) fn excel_bessel_y(n: i32, x: f64) -> f64 {
    if n > MAX_ORDER {
        return f64::NAN;
    }
    // Y is undefined for x <= 0 (`#NUM!` at the formula layer).
    if x <= 0.0 || x.is_nan() {
        return f64::NAN;
    }
    if n == 0 {
        return y0(x);
    }
    // `b * (2j * (1/x))` is the association that reproduces Excel bit-for-bit (BESSEL.xlsx
    // BESSELY F12/E13/F13); `(2j / x) * b` is 1 ulp off.
    let inv_x = 1.0 / x;
    let (mut a, mut b) = (y0(x), y1(x));
    for j in 1..n {
        (a, b) = (b, b * (2.0 * f64::from(j) * inv_x) - a);
    }
    b
}
