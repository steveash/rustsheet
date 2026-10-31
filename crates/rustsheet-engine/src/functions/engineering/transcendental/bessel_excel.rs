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

use crate::arithmetic::{extended_product, extended_sum};
use crate::functions::math_and_trigonometry::{excel_sin_cos, EXCEL_INNER_TRIG_LIMIT};
use crate::functions::statistical::ln_gamma;
use crate::functions::util::poly;

/// `|x|` from which J0, J1, Y0 and Y1 switch from the rational to the asymptotic form.
const SMALL_ARG_LIMIT: f64 = 8.0;
/// Orders above this do not run a recurrence: every Bessel algorithm here, BESSELI's and BESSELK's
/// included, is O(n) per cell and the formula layer truncates `n` from untrusted input. It is each
/// kernel's own per-call cap (SPEC-0011 "Security rule for kernel work"). Above it, BESSELI,
/// BESSELJ and BESSELK give Excel's 0 where an O(1) test proves the recurrence would end in 0, and
/// NaN (reported as `#NUM!`) otherwise; Excel's own cap is `EXCEL_MAX_ORDER`. Deliberate
/// deviation where Excel has a value: see docs/excel-semantics/engineering.md "BESSEL order cap"
/// (decision 0014).
pub(crate) const MAX_ORDER: i32 = 100_000;
/// Excel's own order cap for BESSELI, BESSELJ and BESSELK: `#NUM!` from 10,000,001
/// (`BESSELJ(0,1E+7)` is 0, `BESSELJ(0,10000001)`, `BESSELI(100,10000001)` and
/// `BESSELK(800,10000001)` are `#NUM!`; probes/m9-bessel-order-limits, probes/t2546-bessel-past-cap).
pub(super) const EXCEL_MAX_ORDER: i32 = 10_000_000;
/// The Miller recurrences scale their terms by `1/MILLER_RESCALE` whenever one passes it.
pub(super) const MILLER_RESCALE: f64 = 1e10;
/// The Miller recurrences' accuracy parameter (Numerical Recipes `ACC`).
pub(super) const MILLER_ACCURACY: f64 = 40.0;
/// The factor of that rescale, `1/MILLER_RESCALE`.
pub(super) const MILLER_RESCALE_FACTOR: f64 = 1e-10;
/// ln of a bound below which a Bessel value past `MAX_ORDER` is certainly computed as exactly 0 by
/// the Miller recurrence (e^-850 is about 1E-369, far enough below the subnormals that the scaled
/// term at order n is rescaled to 0 before the recurrence ends; `past_cap_rules_agree_with_the_recurrence`).
pub(super) const LN_COMPUTED_ZERO: f64 = -850.0;
/// The largest ln of the growth a Miller recurrence may compound past `MILLER_RESCALE` and still
/// provably stay finite: its terms stay below e^(23 + 50) * (m * 2/x), far from the largest double.
const MAX_MILLER_GROWTH: f64 = 50.0;
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

/// J0's asymptotic `P` and `Q`; `P` has Excel's fitted `-0.1098628267e-2` (see [`Asymptotic`]).
/// FITTED(T-0908, 14 cells, x 8..100; checked bit-exact on 354 cells, orders 0..10)
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
/// Y0's asymptotic `P` and `Q`; `Q` has Excel's fitted `-0.934945152e-7` (see [`Asymptotic`]).
/// FITTED(T-0908, 14 cells, x 8..100; checked bit-exact on 354 cells, orders 0..10)
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
/// J1's asymptotic `P` and `Q`; `P` repeats Excel's fitted `-0.3516396496e-4` (see [`Asymptotic`]).
/// FITTED(T-0908, 14 cells, x 8..100; checked bit-exact on 354 cells, orders 0..10)
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
    // Past the inner trig cut the sine and cosine do not answer and the result is `#NUM!`
    // (`BESSELJ(1E+15,0)`, `BESSELJ(134217728,0)`, whose phase is 134217727.2, while the order-1
    // phase 134217725.6 answers; probes/t2547-bessel-phase-cut, docs/excel-semantics/engineering.md).
    if xx.is_nan() || xx.abs() >= EXCEL_INNER_TRIG_LIMIT {
        return f64::NAN;
    }
    let (sin, cos) = excel_sin_cos(xx);
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

/// The even order BESSELJ's Miller recurrence starts from, as in Numerical Recipes (`ACC = 40`).
fn jn_start_order(n: i32) -> i32 {
    2 * ((n + (MILLER_ACCURACY * f64::from(n)).sqrt() as i32) / 2)
}

/// Whether a Miller downward recurrence from the order `m` with `tox = 2/|x|` provably never
/// overflows. A step multiplies the terms by at most `j * tox + 1`, and the rescale divides by
/// `MILLER_RESCALE` once, so only the steps with `j * tox > MILLER_RESCALE` grow the terms;
/// their compound growth is `ln(m!/start!) + (m - start) * ln(tox / MILLER_RESCALE)`.
/// A tiny x fails it, as Excel's own recurrence overflows to `#NUM!` there (`BESSELJ(1E-5,100001)`,
/// `BESSELI(2E-5,100001)`; `BESSELJ(2E-5,100001)` and `BESSELI(5E-5,100001)` are 0).
pub(super) fn miller_stays_finite(m: f64, tox: f64) -> bool {
    let start = (MILLER_RESCALE / tox).floor();
    if start >= m {
        return true;
    }
    let growth =
        ln_gamma(m + 1.0) - ln_gamma(start + 1.0) + (m - start) * (tox / MILLER_RESCALE).ln();
    growth < MAX_MILLER_GROWTH
}

/// `BESSELJ(x, n)` for `MAX_ORDER < n`, `ax = |x|`, without the recurrence: 0 where Excel's is
/// provably 0 (x = 0, or `ax <= n` with Kapteyn's bound `|J_n(nz)| <= (z e^w / (1 + w))^n`,
/// `w = sqrt(1 - z^2)`, below `LN_COMPUTED_ZERO` and a recurrence that stays finite), else NaN:
/// `BESSELJ(9E+4,100001)`, `(1E-3,100001)` and `(0,1E+7)` are 0 (probes/t2546-bessel-past-cap);
/// `BESSELJ(1E+5,100001)` is 0.0094462635180178 (probes/m9-bessel-order-limits).
fn bessel_j_past_cap(n: i32, ax: f64) -> f64 {
    if n > EXCEL_MAX_ORDER {
        return f64::NAN;
    }
    if ax == 0.0 {
        return 0.0;
    }
    let order = f64::from(n);
    if ax > order {
        return f64::NAN;
    }
    let z = ax / order;
    let w = (1.0 - z * z).sqrt();
    let ln_bound = order * (z.ln() + w - w.ln_1p());
    if ln_bound < LN_COMPUTED_ZERO && miller_stays_finite(f64::from(jn_start_order(n)), 2.0 / ax) {
        0.0
    } else {
        f64::NAN
    }
}

/// Miller's downward recurrence for `J_n(|x|)`, `n >= 2`, `0 < |x| <= n`.
pub(super) fn jn_miller(n: i32, ax: f64) -> f64 {
    let tox = 2.0 / ax;
    let m = jn_start_order(n);
    let (mut bj, mut bjp, mut ans, mut sum) = (1.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    let mut odd = false;
    for j in (1..=m).rev() {
        // Every step is x87 code stored to a double, so each product and sum rounds twice
        // (64-bit significand, then double): plain doubles drift from Excel's value by up to
        // 3E-14 near the turning point n = x, and these match all 43 Miller cells of
        // probes/t2881-besselj-turning-point bit for bit (docs/excel-semantics/engineering.md
        // "BESSELJ / BESSELY", T-2548).
        let bjm = extended_sum(
            extended_product(extended_product(f64::from(j), tox), bj),
            -bjp,
        );
        bjp = bj;
        bj = bjm;
        if bj.abs() > MILLER_RESCALE {
            bj = extended_product(bj, MILLER_RESCALE_FACTOR);
            bjp = extended_product(bjp, MILLER_RESCALE_FACTOR);
            ans = extended_product(ans, MILLER_RESCALE_FACTOR);
            sum = extended_product(sum, MILLER_RESCALE_FACTOR);
        }
        if odd {
            sum = extended_sum(sum, bj);
        }
        odd = !odd;
        if j == n {
            ans = bjp;
        }
    }
    ans / (2.0 * sum - bj)
}

/// One step of the upward recurrence `C_(j+1) = C_j * (2j * (1/x)) - C_(j-1)` of BESSELJ and BESSELY,
/// from `a = C_(j-1)` and `b = C_j`. Each product and the difference round twice, as x87 code
/// stored to doubles does (see `jn_miller`).
fn upward_step(a: f64, b: f64, j: i32, inv_x: f64) -> f64 {
    extended_sum(
        extended_product(b, extended_product(2.0 * f64::from(j), inv_x)),
        -a,
    )
}

/// `BESSELJ(x, n)` for integer `n >= 0`.
pub(crate) fn excel_bessel_j(n: i32, x: f64) -> f64 {
    let ax = x.abs();
    if n > MAX_ORDER {
        return bessel_j_past_cap(n, ax);
    }
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
                (a, b) = (b, upward_step(a, b, j, inv_x));
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
        (a, b) = (b, upward_step(a, b, j, inv_x));
    }
    b
}
