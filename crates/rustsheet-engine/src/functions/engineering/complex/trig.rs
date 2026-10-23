//! The circular and hyperbolic IM* functions (IMSIN … IMCOT, IMSINH … IMCSCH), of a parsed number.

use super::{complex_result, finite_complex_result, ComplexParts, Suffix};
use crate::calc_result::CalcResult;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::num_error;
use crate::functions::math_and_trigonometry::{excel_sin_cos, EXCEL_TRIG_LIMIT};

/// IMSIN and IMCOS are `#NUM!` once `exp(|y|)` overflows (|y| above about 709.78), whatever the
/// real part, although `cosh y` is still finite there: `IMSIN("1.5+710.4i")`, `IMCOS("710.4i")`
/// (probes/eng-im-overflow-edges A25-A27, A33-A35).
fn trig_exp_overflows(y: f64) -> bool {
    y.abs().exp().is_infinite()
}

/// IMCOSH and IMSINH are `#NUM!` for a zero imaginary part once `exp(|x|)` overflows, although
/// `cosh x` is still finite: `IMCOSH("710.4+0i")`. With a nonzero imaginary part they answer
/// (`IMCOSH("710.4+1.5i")`); a tiny nonzero one is UNVERIFIED (probes/eng-im-overflow-edges A8,
/// A16).
fn hyperbolic_exp_overflows(x: f64, y: f64) -> bool {
    y == 0.0 && x.abs().exp().is_infinite()
}

/// The sine and cosine of `x` as SIN and COS reduce it (by the x87 64-bit π) where they answer,
/// libm's beyond 2^27, where the reciprocal IM* functions are unprobed. IMCSC, IMSEC, IMSECH and
/// IMCOT take these: the sweep's rows with a large real part (`956.615+0.00387493i`,
/// `-9447.72-0.00486262i`) need Excel's reduction (probes-numeric im-last-digit-sweep, T-2044).
fn trig_sin_cos(x: f64) -> (f64, f64) {
    if x.abs() < EXCEL_TRIG_LIMIT {
        excel_sin_cos(x)
    } else {
        (x.sin(), x.cos())
    }
}

/// The result of a reciprocal (IMSEC, IMCSC, IMSECH, IMCSCH) whose denominator `t` overflowed to
/// infinity or NaN: Excel answers `0` where `inf / inf` is NaN. A zero or finite `t` is not this
/// case and stays an error. Examples: `IMSECH("711+1.5i")`, `IMCSCH("710.4+0i")`,
/// `IMSEC("1.5+711i")` (probes/eng-im-overflow-edges A55, A56, A64, A74, A82).
fn reciprocal_result(x: f64, y: f64, t: f64, suffix: Suffix) -> CalcResult {
    if !t.is_finite() && (x.is_nan() || y.is_nan()) {
        return complex_result(0.0, 0.0, suffix);
    }
    complex_result(x, y, suffix)
}

pub(super) fn imcos((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if trig_exp_overflows(y) {
        return num_error(cell);
    }
    finite_complex_result(x.cos() * y.cosh(), -x.sin() * y.sinh(), suffix, cell)
}

pub(super) fn imcosh((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if hyperbolic_exp_overflows(x, y) {
        return num_error(cell);
    }
    finite_complex_result(x.cosh() * y.cos(), x.sinh() * y.sin(), suffix, cell)
}

/// cot(x+iy) = `(sin 2x - i·sinh 2y) / (cosh 2y - (2cos²x - 1))`, with SIN and COS reduced as
/// Excel's own (`trig_sin_cos`): all 63 cells of probes-numeric im-last-digit-sweep with an
/// imaginary part (T-2044), including the digits Excel loses for tiny arguments, where the
/// denominator cancels (`IMCOT("1E-5+1E-5i")` real part 49999.9958596485, `IMCOT("0.00001i")`
/// -99999.9917326303i; exact 50000 and 100000). UNVERIFIED (T-2531): below |y| ≈ 1.05E-8 `cosh 2y` rounds to 1 and
/// the result is `#NUM!`. Past `cosh 2y`'s range the value is `∓i` by the
/// sign of y (Q27:R27, R55). A real number keeps `1/tan x` (`IMCOT("1E-20")` = 1E+20).
pub(super) fn imcot((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if y == 0.0 {
        return finite_complex_result(1.0 / x.tan(), 0.0, suffix, cell);
    }
    let cosh_2y = f64::cosh(2.0 * y);
    if cosh_2y.is_infinite() {
        return complex_result(0.0, -y.signum(), suffix);
    }
    let (_, cos_x) = trig_sin_cos(x);
    let (sin_2x, _) = trig_sin_cos(2.0 * x);
    let t = cosh_2y - (2.0 * cos_x * cos_x - 1.0);
    finite_complex_result(sin_2x / t, -f64::sinh(2.0 * y) / t, suffix, cell)
}

/// csc(x+iy) = `(sin x·(cosh y/t), -cos x·(sinh y/t))` with `t = sin²x + (cosh 2y - 1)/2` and SIN
/// and COS reduced as Excel's own (`trig_sin_cos`): all 69 finite cells of probes-numeric
/// im-last-digit-sweep (T-2044). Writing sinh²y as `(cosh 2y - 1)/2` is what loses Excel's digits
/// for tiny arguments: `IMCSC("1E-5+1E-5i")` is 49999.9979339908-49999.9979306575i (exact
/// 50000.0000016667), `IMCSC("0.00001i")` -99999.9917276303i. A real number is `1/sin x`
/// (gen-m9/2f0d384b8ed0d392 A109, `IMCSC(0.2)` 5.03348954767234; the kernel gives ...235), as
/// IMSEC's is `1/cos x` and IMSECH's `1/cosh x` (gen-m9/01810fc89c5ec2c0 A318). UNVERIFIED (T-2531): below |y| ≈ 1.05E-8
/// `cosh 2y` rounds to 1, `t` is 0 and the result is `#NUM!`.
pub(super) fn imcsc((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    let (x_sin, x_cos) = trig_sin_cos(x);
    if y == 0.0 {
        return finite_complex_result(1.0 / x_sin, 0.0, suffix, cell);
    }
    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_sin * x_sin + (f64::cosh(2.0 * y) - 1.0) / 2.0;
    if t == 0.0 {
        return num_error(cell);
    }
    reciprocal_result(x_sin * (y_cosh / t), -x_cos * (y_sinh / t), t, suffix)
}

pub(super) fn imcsch((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    let x_sinh = x.sinh();

    let y_cos = y.cos();
    let y_sin = y.sin();

    let t = x_sinh * x_sinh * y_cos * y_cos + x_cosh * x_cosh * y_sin * y_sin;
    reciprocal_result(x_sinh * y_cos / t, -x_cosh * y_sin / t, t, suffix)
}

/// sec(x+iy) = `(cos x·cosh y, sin x·sinh y) / t` with `t = cos²x + (cosh 2y - 1)/2`, as IMCSC
/// writes sinh²y: 68 of the 70 finite cells of probes-numeric im-last-digit-sweep (T-2044; the
/// misses, rows 29 and 33, sit near a pole of the secant). IMSECH is not the same formula turned
/// a quarter: it squares sinh itself, so `IMSEC("4+3i")` has real part -0.0652940278579471 and
/// `IMSECH("3+4i")` -0.065294027857947 (G23, K58).
pub(super) fn imsec((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let (x_sin, x_cos) = trig_sin_cos(x);
    if y == 0.0 {
        return complex_result(1.0 / x_cos, 0.0, suffix);
    }
    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_cos * x_cos + (f64::cosh(2.0 * y) - 1.0) / 2.0;
    reciprocal_result(x_cos * y_cosh / t, x_sin * y_sinh / t, t, suffix)
}

/// sech(x+iy) = `(cosh x·cos y, -sinh x·sin y) / (cos²y + sinh²x)`: 61 of the 62 finite cells of
/// probes-numeric im-last-digit-sweep (T-2044; the miss is K34:L34, `1.34384-77.6742i`).
pub(super) fn imsech((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    if y == 0.0 {
        return complex_result(1.0 / x_cosh, 0.0, suffix);
    }
    let x_sinh = x.sinh();
    let (y_sin, y_cos) = trig_sin_cos(y);

    let t = y_cos * y_cos + x_sinh * x_sinh;
    reciprocal_result(x_cosh * y_cos / t, -x_sinh * y_sin / t, t, suffix)
}

pub(super) fn imsin((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if trig_exp_overflows(y) {
        return num_error(cell);
    }
    finite_complex_result(x.sin() * y.cosh(), x.cos() * y.sinh(), suffix, cell)
}

pub(super) fn imsinh((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if hyperbolic_exp_overflows(x, y) {
        return num_error(cell);
    }
    finite_complex_result(x.sinh() * y.cos(), x.cosh() * y.sin(), suffix, cell)
}

/// tan(x+iy) in its double-angle form, `(sin 2x + i sinh 2y) / (cos 2x + cosh 2y)`: Excel's
/// digits, where the tan/tanh form loses the last 1-10 of them when the real part is small
/// (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx N2, N7, N15: `IMTAN("1.23+9.5121i")` has real part
/// 6.89106871662165E-09; T-0907). Past `cosh 2y`'s range the value is `±0 ± i`, as tanh gives;
/// where the denominator rounds to 0 (y = 0 and x the double nearest π/2) it is `tan x`, finite.
/// Both fallbacks are unprobed, not Excel evidence (T-1966).
fn tan_parts(x: f64, y: f64) -> (f64, f64) {
    let denominator = f64::cos(2.0 * x) + f64::cosh(2.0 * y);
    if denominator == 0.0 {
        (x.tan(), 0.0)
    } else if denominator.is_finite() {
        (
            f64::sin(2.0 * x) / denominator,
            f64::sinh(2.0 * y) / denominator,
        )
    } else {
        (0.0 * f64::sin(2.0 * x), y.signum())
    }
}

pub(super) fn imtan((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let (x, y) = tan_parts(x, y);
    complex_result(x, y, suffix)
}
