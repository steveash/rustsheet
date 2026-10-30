//! Excel's own F density `F.DIST(x, m, n, FALSE)` for x > 0: T-2586 round 5's frozen model
//! M-final (docs/excel-semantics/statistics.md "F density is not the beta density, R's df or a
//! GAMMALN formula (T-2513)"; reference code `docs/research/t2464/r5-f-final/fdensity_excel.py`,
//! `value(x, d1, d2, fin="div")`). With a = m/2, b = n/2, h = a + b and every step an x87 one:
//! ```text
//! mx = fl(m x);  side 1 (mx <= n): r = fl(mx / n), c = a;  side 2: r = fl(n / mx), c = -b
//! P  = fl(c * LN(x))                        (stored)
//! H  = -h * L(r)                            (extended; L Excel's log1p, a double)
//! E  = K + P + H                            (extended, summed left to right)
//! F  = fl(EXP(E) / x), with EXP(E) a stored double; E < -800 and F < 2^-1022 give 0
//! ```
//! K is the exact constant -ln B(a, b) + a ln(m/n) (side 1) or -ln B(a, b) - b ln(m/n) (side 2),
//! rounded to a register, plus a measured offset in units of 2^-53 ([`constant_offset`]).
//! The argument rules (truncated df, x = 0, a subnormal x, fl(m x) overflowing) are the caller's:
//! `fisher.rs` applies them before the density.
//!
//! The model is not bit-exact and reaches 15-digit parity only where the power term |P| is small
//! (T-2586 r5-f-final RESULTS.md section 4: 99-100% of cells for |P| < 4, 87% for |P| in [4, 32),
//! 10-36% above), but it beats the correctly rounded density on every region scored there.

use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_exp::{dd_add, dd_div, dd_mul, dd_to_extended, excel_exp_extended};
use crate::x87_log10::{excel_ln, ln_double_double};
use crate::x87_log1p::excel_log1p;
use crate::x87_register::Register;

use super::gammaln::STIRLING;

/// Below this E, EXP(E) is far under 2^-1075 and Excel's density is 0 (r5-f-edges `edgemodel`).
const EXPONENT_FLOOR: f64 = -800.0;

/// Excel's log1p switches from its series to LN(fl(1 + r)) above this r (r3-tdensity `log1p_xl`).
const LOG1P_SERIES_LIMIT: f64 = 0.375;

/// From this smaller df on, the offset is the bracket error plus [`LARGE_SHAPE_OFFSET`]
/// (r5-f-bigconst `bcmodel`: `BIG_MIN`, `C_BIG`).
const LARGE_DF_MIN: f64 = 16.0;
const LARGE_SHAPE_OFFSET: f64 = -0.42;

/// r4-f-small-df `sdmodel`'s fitted constants: `C0`, and e(1/2), e(1) (units of 2^-53).
const SMALL_SHAPE_OFFSET: f64 = 5.33;
const STIRLING_ERROR_HALF: f64 = 0.22;
const STIRLING_ERROR_ONE: f64 = 0.06;

/// r3-loader-const's measured error of Excel's stirlerr at v = 1.5, 2, ..., 28 in units of 2^-53
/// (`docs/research/t2464/r3-loader-const/grid_e.tsv`; v = 1/2 and 1 are the fitted
/// [`STIRLING_ERROR_HALF`] and [`STIRLING_ERROR_ONE`]).
const STIRLING_ERROR: [f64; 54] = [
    -5.144, -5.151, -8.481, -9.05, -11.632, -14.36, -15.081, -18.88, -19.128, -22.94, -25.296,
    -27.369, -29.408, -31.661, -33.955, -35.76, -38.022, -40.557, -43.291, -45.715, -47.417,
    -49.768, -51.635, -54.797, -56.941, -59.227, -60.968, -63.958, -66.15, -68.187, -68.593,
    -73.056, -74.623, -77.785, -80.155, -82.183, -84.611, -87.165, -88.828, -90.977, -94.169,
    -95.896, -97.581, -100.701, -102.435, -105.573, -106.613, -107.937, -111.61, -112.712,
    -114.448, -118.034, -120.706, -121.421,
];

/// `sdmodel`'s linear extrapolation of [`STIRLING_ERROR`] above 28: numpy's least-squares line
/// through the table's v >= 14 points, slope and intercept bit for bit (`_ALPHA`, `_GAMMA`).
const STIRLING_ERROR_SLOPE: f64 = -4.509_156_650_246_305;
const STIRLING_ERROR_INTERCEPT: f64 = 3.785_324_137_930_950_4;

/// ln(2 pi)/2 in double-double.
const HALF_LN_TWO_PI: (f64, f64) = (0.918_938_533_204_672_8, -3.878_294_158_067_241_4e-17);

/// Binet's function mu(v) = ln Gamma(v) - (v - 1/2) ln v + v - ln(2 pi)/2 at v = 1/2, 1, ..., 15
/// in double-double (mpmath, 200 bits); above 15 [`binet`] sums Stirling's series.
const BINET_TABLE: [(f64, f64); 30] = [
    (0.153_426_409_720_027_36, -1.159_523_406_923_149_8e-17),
    (0.081_061_466_795_327_26, -2.850_421_842_770_954_6e-18),
    (0.054_814_121_051_917_65, 2.789_905_337_804_290_4e-18),
    (0.041_340_695_955_409_3, -2.941_654_530_929_307e-18),
    (0.033_162_873_519_936_29, -3.415_190_180_768_476_8e-18),
    (0.027_677_925_684_998_34, 7.917_435_820_236_853e-19),
    (0.023_746_163_656_297_496, -2.314_769_165_964_602e-19),
    (0.020_790_672_103_765_093, -2.532_572_426_720_811_6e-19),
    (0.018_488_450_532_673_187, -1.601_653_674_640_336_3e-18),
    (0.016_644_691_189_821_193, -9.759_029_781_538_585e-19),
    (0.015_134_973_221_917_378, 8.206_551_210_066_371e-19),
    (0.013_876_128_823_070_748, -4.371_626_011_095_74e-19),
    (0.012_810_465_242_920_227, -4.585_328_603_983_469e-19),
    (0.011_896_709_945_891_77, 5.674_518_257_356_195e-19),
    (0.011_104_559_758_206_917, -2.038_131_079_125_446e-20),
    (0.010_411_265_261_972_096, 2.953_086_875_573_401e-19),
    (0.009_799_416_126_158_804, -6.207_313_358_118_634e-19),
    (0.009_255_462_182_712_733, 6.290_071_715_112_313e-20),
    (0.008_768_700_134_139_386, -6.995_800_208_116_076e-19),
    (0.008_330_563_433_362_87, 4.637_604_645_200_097e-19),
    (0.007_934_114_564_314_02, 6.316_244_311_725_502e-19),
    (0.007_573_675_487_951_841, 2.046_770_746_049_083_7e-19),
    (0.007_244_554_301_320_383, 2.007_446_035_469_048_8e-19),
    (0.006_942_840_107_209_53, -5.224_462_195_760_719e-20),
    (0.006_665_247_032_707_682, 3.815_414_230_769_613_5e-19),
    (0.006_408_994_188_004_207, -7.471_039_504_395_375e-20),
    (0.006_171_712_263_039_458, 3.981_841_757_148_414e-20),
    (0.005_951_370_112_758_847_5, 2.399_535_274_410_237e-19),
    (0.005_746_216_513_010_115_5, 2.200_470_475_921_111_6e-19),
    (0.005_554_733_551_962_801, 3.185_347_414_196_856e-19),
];

/// Excel's F density at x > 0 for truncated degrees of freedom `m`, `n` (1 <= m, n <= 1E10) with
/// fl(m x) finite (module docs).
pub(super) fn excel_f_density(x: f64, m: f64, n: f64) -> f64 {
    let (a, b, h) = (m / 2.0, n / 2.0, (m + n) / 2.0);
    let mx = extended_product(m, x);
    let (side_one, r) = if mx <= n {
        (true, extended_quotient(mx, n))
    } else {
        (false, extended_quotient(n, mx))
    };
    let c = if side_one { a } else { -b };
    let power = extended_product(c, excel_ln(x));
    let log_term = Register::load(-h).mul(Register::load(excel_log1p_positive(r)));
    let exponent = constant(a, b, side_one)
        .add(Register::load(power))
        .add(log_term);
    if exponent.store() < EXPONENT_FLOOR {
        return 0.0;
    }
    let density = extended_quotient(excel_exp_extended(exponent), x);
    if density < f64::MIN_POSITIVE {
        0.0
    } else {
        density
    }
}

/// Excel's ln(1 + r) for r >= 0 as the F density takes it: the atanh series up to 0.375, then
/// LN(fl(1 + r)) (r3-tdensity `log1p_xl`).
fn excel_log1p_positive(r: f64) -> f64 {
    if r <= LOG1P_SERIES_LIMIT {
        excel_log1p(r)
    } else {
        excel_ln(extended_sum(1.0, r))
    }
}

/// K: the exact constant rounded to a register, plus [`constant_offset`] units of 2^-53
/// (r5-f-bigconst `bcmodel.K_of`).
fn constant(a: f64, b: f64, side_one: bool) -> Register {
    let (w, o) = if side_one { (a, b) } else { (b, a) };
    let h = a + b;
    let bracket = exact_bracket(h, o);
    // -ln B(a, b) + a ln(a/b) = ln(w)/2 + (h - 1/2) ln(h/o) - ln(2 pi)/2 - mu(a) - mu(b) + mu(h),
    // with (w, o) = (a, b) on side 1 and (b, a) on side 2 (Stirling's formula for each ln Gamma).
    let ln_w = ln_double_double(w);
    let binet_sum = dd_add(dd_add(binet(h), neg(binet(a))), neg(binet(b)));
    let exact = dd_add(
        dd_add(
            dd_add((ln_w.0 / 2.0, ln_w.1 / 2.0), bracket),
            neg(HALF_LN_TWO_PI),
        ),
        binet_sum,
    );
    let offset = constant_offset(a, b, h, w, o, bracket);
    rounded_to_register(exact).add(Register::load(offset * f64::EPSILON / 2.0))
}

/// The offset of Excel's K from the exact one in units of 2^-53, as `bcmodel.delta` (above
/// [`LARGE_DF_MIN`]) and r4-f-small-df's `sdmodel.delta` (below) form it in doubles: the stirlerr
/// error terms e(h) - e(a) - e(b), a constant, and the rounding of Excel's (h - 1/2) ln(h/o)
/// taken as (h - 1/2) L(fl(w/o)) with Excel's log1p, product kept extended.
fn constant_offset(a: f64, b: f64, h: f64, w: f64, o: f64, bracket: (f64, f64)) -> f64 {
    let excel_bracket = Register::load(h - 0.5).mul(Register::load(excel_log1p_positive(
        extended_quotient(w, o),
    )));
    let bracket_error =
        dd_add(register_to_double_double(excel_bracket), neg(bracket)).0 / (f64::EPSILON / 2.0);
    let stirling = stirling_error(h) - stirling_error(a) - stirling_error(b) + SMALL_SHAPE_OFFSET;
    let small_shape = stirling + bracket_error;
    if a.min(b) * 2.0 >= LARGE_DF_MIN {
        // bcmodel's `side_err` is sdmodel's delta less its stirlerr part, both doubles.
        (small_shape - stirling) + LARGE_SHAPE_OFFSET
    } else {
        small_shape
    }
}

/// sdmodel's `e_of`: Excel's stirlerr error at a half-integer v (units of 2^-53).
fn stirling_error(v: f64) -> f64 {
    if v == 0.5 {
        return STIRLING_ERROR_HALF;
    }
    if v == 1.0 {
        return STIRLING_ERROR_ONE;
    }
    let index = 2.0 * v - 3.0;
    if (0.0..=53.0).contains(&index) && index.fract() == 0.0 {
        // In 0..=53 and integral: the cast is exact.
        return STIRLING_ERROR[index as usize];
    }
    STIRLING_ERROR_SLOPE * v + STIRLING_ERROR_INTERCEPT
}

/// (h - 1/2) ln(h/o) in double-double.
fn exact_bracket(h: f64, o: f64) -> (f64, f64) {
    let ln_ratio = dd_add(ln_double_double(h), neg(ln_double_double(o)));
    dd_mul((h - 0.5, 0.0), ln_ratio)
}

/// Binet's mu(v) in double-double for a half-integer v >= 1/2: the table to 15, Stirling's
/// series above.
fn binet(v: f64) -> (f64, f64) {
    let index = 2.0 * v - 1.0;
    if index < 30.0 {
        // A half-integer v in [1/2, 15]: index is integral in 0..30.
        return BINET_TABLE[index.max(0.0) as usize];
    }
    // mu(v) = (1/v) (c_1 + c_2 z + ... + c_10 z^9), z = 1/v^2, with c_k gammaln.rs's Stirling
    // terms; c_1 = 1/12 in double-double. At v > 15 the first omitted term is below 2E-24.
    let z = dd_div((1.0, 0.0), dd_mul((v, 0.0), (v, 0.0)));
    let mut sum = (0.0, 0.0);
    for &coefficient in STIRLING[1..].iter().rev() {
        sum = dd_add((coefficient, 0.0), dd_mul(sum, z));
    }
    sum = dd_add(dd_div((1.0, 0.0), (12.0, 0.0)), dd_mul(sum, z));
    dd_div(sum, (v, 0.0))
}

fn neg((hi, lo): (f64, f64)) -> (f64, f64) {
    (-hi, -lo)
}

/// A double-double rounded to a register (RN64).
fn rounded_to_register((hi, lo): (f64, f64)) -> Register {
    if hi == 0.0 {
        return Register::load(0.0);
    }
    let negative = hi < 0.0;
    let (significand, exponent) = dd_to_extended(if negative { (-hi, -lo) } else { (hi, lo) });
    Register::from_parts(negative, significand, exponent)
}

/// A register's value exactly, as a double-double: its top 53 significand bits and the other 11.
fn register_to_double_double(value: Register) -> (f64, f64) {
    let (negative, significand, exponent) = value.parts();
    let sign = if negative { -1.0 } else { 1.0 };
    let scale = |shift: i32| 2f64.powi(exponent + shift);
    // Both parts have at most 53 bits, so the casts are exact.
    let hi = ((significand >> 11) as f64) * scale(11);
    let lo = ((significand & 0x7FF) as f64) * scale(0);
    (sign * hi, sign * lo)
}

#[cfg(test)]
mod tests;
