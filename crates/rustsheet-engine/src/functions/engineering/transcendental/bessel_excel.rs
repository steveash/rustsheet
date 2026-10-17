//! Excel's BESSELJ / BESSELY for small arguments.
//!
//! Excel does not use fdlibm: for `|x| < 8` its results are bit-identical to the rational
//! approximations of Numerical Recipes (`bessj0/bessj1/bessy0/bessy1`, ~1e-8 accuracy), with
//! upward recurrence for higher orders and Miller's downward recurrence for `J_n(x)` when `n >= x`.
//! Evidence: `eval/suites/ironcalc-calc/ENGINEERING/BESSEL.xlsx` (Excel-cached), where these
//! formulas reproduce every cell with `|x| < 8` exactly.
//! For `|x| >= 8` the fdlibm implementation is kept (see `docs/excel-semantics/engineering.md`).

use super::{bessel_j, bessel_y};

const SMALL_ARG_LIMIT: f64 = 8.0;
/// Orders above this yield NaN (reported as `#NUM!`): every Bessel algorithm here, BESSELI's and
/// BESSELK's included, is O(n) per cell and the formula layer truncates `n` from untrusted input.
/// It is each kernel's own per-call cap (SPEC-0011 "Security rule for kernel work"). Excel's own cap
/// is not measured; `100_000` is far beyond any n where results are finite anyway. Deliberate
/// deviation: see docs/excel-semantics/engineering.md "BESSEL order cap" (decision 0014).
pub(crate) const MAX_ORDER: i32 = 100_000;
// Numerical Recipes truncates 2/pi; Excel's output bakes in the truncated constant.
const TWO_OVER_PI: f64 = 636_619_772.0 / 1e9; // == 0.636619772 (correctly rounded)

/// Evaluates `c[0] + c[1]*y + c[2]*y^2 + ...` (Horner).
fn poly(c: &[f64], y: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, &a| acc * y + a)
}

fn j0(x: f64) -> f64 {
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
    // J_0(0) is 1.00000000283141 in Excel (NR numerator/denominator ratio), not exactly 1.
    let small = ax < SMALL_ARG_LIMIT && (ax > 0.0 || n <= 1);
    if !small {
        return bessel_j(n, x);
    }
    let ans = match n {
        0 => return j0(x),
        1 => return j1(x),
        _ if ax > f64::from(n) => {
            let (mut a, mut b) = (j0(ax), j1(ax));
            for j in 1..n {
                (a, b) = (b, (2.0 * f64::from(j) / ax) * b - a);
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
    if !(x > 0.0 && x < SMALL_ARG_LIMIT) {
        return bessel_y(n, x);
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
