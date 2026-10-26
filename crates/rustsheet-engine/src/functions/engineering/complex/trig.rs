//! The circular and hyperbolic IM* functions (IMSIN … IMCOT, IMSINH … IMCSCH), of a parsed number.

use super::{complex_result, finite_complex_result, ComplexParts, Suffix};
use crate::calc_result::CalcResult;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::num_error;
use crate::functions::math_and_trigonometry::{excel_sin_cos, excel_tan, EXCEL_TRIG_LIMIT};
use crate::x87_exp::excel_exp;

/// IMSIN and IMCOS are `#NUM!` once `exp(|y|)` overflows (|y| above about 709.78), whatever the
/// real part, although `cosh y` is still finite there: `IMSIN("1.5+710.4i")`, `IMCOS("710.4i")`
/// (probes/eng-im-overflow-edges A25-A27, A33-A35).
fn trig_exp_overflows(y: f64) -> bool {
    y.abs().exp().is_infinite()
}

/// IMCOSH and IMSINH are `#NUM!` for a zero imaginary part once `exp(|x|)` overflows, although
/// `cosh x` is still finite: `IMCOSH("710.4+0i")` (probes/eng-im-overflow-edges A8, A16). With a
/// nonzero imaginary part, even `1E-300`, they take [`hyperbolic_log_form`].
fn hyperbolic_exp_overflows(x: f64, y: f64) -> bool {
    y == 0.0 && x.abs().exp().is_infinite()
}

/// Past |x| = 709 (709 itself takes the product form), IMCOSH and IMSINH of a number with a
/// nonzero imaginary part take both parts in log space, `exp(|x| + ln(cos y / 2))` and
/// `exp(|x| + ln(sin y / 2))`, the sign of x then going to sinh's factor. Hence `#NUM!` whenever
/// cos y or sin y is negative (`IMCOSH("709.05+2i")`, `"709.3-0.5i"`, `"-710.4-2i"`, while
/// `"709+2i"` and `"-710.4+1.5i"` answer), and digits that differ from `cosh x·cos y`:
/// `IMCOSH("710.4+1.5i")` is 1.17873946358278E+307+1.6621900184285E+308i, `IMSINH("709.5+1E-200i")`
/// has real part 6.77493159657354E+307 where `IMSINH("709.5")` is 6.77493159657316E+307
/// (probes/eng-im-overflow-edges A6, A14; eng-im-overflow-edges-2, T-2231). The exact cut between
/// 709 and 709.0000000001 is not pinned.
const HYPERBOLIC_LOG_CUT: f64 = 709.0;

/// IMCOSH's (`sinh` false) or IMSINH's value past [`HYPERBOLIC_LOG_CUT`], `None` below the cut or
/// for a zero y. `#NUM!` where cos y or sin y is not positive (Excel takes its log).
fn hyperbolic_log_form(
    (x, y, suffix): ComplexParts,
    sinh: bool,
    cell: CellReferenceIndex,
) -> Option<CalcResult> {
    if y == 0.0 || x.abs() <= HYPERBOLIC_LOG_CUT {
        return None;
    }
    let (sin_y, cos_y) = (y.sin(), y.cos());
    if sin_y <= 0.0 || cos_y <= 0.0 {
        return Some(num_error(cell));
    }
    let part = |t: f64| (x.abs() + (t / 2.0).ln()).exp();
    let (re, im) = (part(cos_y), part(sin_y));
    let (re, im) = if sinh {
        (x.signum() * re, im)
    } else {
        (re, x.signum() * im)
    };
    Some(finite_complex_result(re, im, suffix, cell))
}

/// `(sinh t, cosh t)` as IMSIN and IMCOS build them from the imaginary part: `(EXP(t) ∓ EXP(-t))/2`
/// with Excel's EXP for every t, small t too, where the worksheet SINH switches to expm1
/// (numbers.md). So the imaginary part of `IMCOS("1.5E+2-2E-3i")` is -0.0014297538124271 (libm
/// sinh gives ...242709) and `IMSIN("1E-200+1E-200i")` is "1E-200": EXP(1E-200) = EXP(-1E-200),
/// so sinh is 0 (gen-m9, T-2258 triage; engineering.md, T-2535). IMSINH does not: it keeps
/// `IMSINH("1E-200+1E-200i")`'s tiny real part, and its kernel is open (T-2778).
fn exp_sinh_cosh(t: f64) -> (f64, f64) {
    let (a, b) = (excel_exp(t), excel_exp(-t));
    ((a - b) / 2.0, (a + b) / 2.0)
}

/// IMSIN, IMCOS, IMSEC, IMCSC and IMCOT of a real number `x` (imaginary part 0): `value` of SIN,
/// COS and TAN's own kernels (the x87 reduction, [`excel_sin_cos`] and [`excel_tan`]), and
/// `#NUM!` from |x| = 2^27 as SIN is, except IMCOT, which is "0" there (`past_limit`).
/// probes/t2534-im-trig-real-limit (excel-graph): `IMSIN("710")` is 0.0000602887066915739 and
/// `IMCOT("123456789.123")` 0.0177255441505541, where libm gave other last digits; at 134217728,
/// 1E+10, 1E+15 and ±1E+200 IMSIN, IMCOS, IMTAN, IMSEC and IMCSC are `#NUM!` and IMCOT "0"
/// (docs/excel-semantics/engineering.md, T-2534). IMTAN takes the limit only (see [`imtan`]).
fn real_trig(
    x: f64,
    suffix: Suffix,
    cell: CellReferenceIndex,
    past_limit: Option<f64>,
    value: impl Fn(f64) -> f64,
) -> CalcResult {
    if x.abs() >= EXCEL_TRIG_LIMIT {
        return past_limit.map_or_else(|| num_error(cell), |v| complex_result(v, 0.0, suffix));
    }
    finite_complex_result(value(x), 0.0, suffix, cell)
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

/// IMSEC, IMCSC and IMSECH where the denominator `t` overflows (the hyperbolic argument `v` past
/// about 355): cosh v and |sinh v| are then both `M = e^|v|/2` and `t` is M², so the result is the
/// trigonometric parts `(re, im)` over M, computed as `2e^-|v|` so that it stays finite. Excel
/// keeps these tiny parts: `IMCSC("-724.227-562.884i")` is
/// -6.94796703439417E-245-6.26263832046024E-246i and `IMSECH("-524.561+5.3113i")`
/// 1.73046857535442E-228-2.53533899849465E-228i (probes-numeric im-last-digit-sweep E55:F55,
/// K42:L42). Past |v| ≈ 708 the parts are subnormal and print as 0 (`IMSEC("1.5+711i")`).
fn overflowed_reciprocal(re: f64, im: f64, v: f64, suffix: Suffix) -> CalcResult {
    let scale = 2.0 * (-v.abs()).exp();
    complex_result(re * scale, im * scale, suffix)
}

/// The result of a reciprocal (IMCSCH) whose denominator `t` overflowed to
/// infinity or NaN: Excel answers `0` where `inf / inf` is NaN. A zero or finite `t` is not this
/// case and stays an error. Example: `IMCSCH("710.4+0i")` (probes/eng-im-overflow-edges A64).
/// Whether IMCSCH keeps normal tiny parts as IMSECH does (`overflowed_reciprocal`) is unprobed.
fn reciprocal_result(x: f64, y: f64, t: f64, suffix: Suffix) -> CalcResult {
    if !t.is_finite() && (x.is_nan() || y.is_nan()) {
        return complex_result(0.0, 0.0, suffix);
    }
    complex_result(x, y, suffix)
}

pub(super) fn imcos((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if y == 0.0 {
        return real_trig(x, suffix, cell, None, |x| excel_sin_cos(x).1);
    }
    if trig_exp_overflows(y) {
        return num_error(cell);
    }
    let (y_sinh, y_cosh) = exp_sinh_cosh(y);
    finite_complex_result(x.cos() * y_cosh, -x.sin() * y_sinh, suffix, cell)
}

pub(super) fn imcosh((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if hyperbolic_exp_overflows(x, y) {
        return num_error(cell);
    }
    if let Some(result) = hyperbolic_log_form((x, y, suffix), false, cell) {
        return result;
    }
    finite_complex_result(x.cosh() * y.cos(), x.sinh() * y.sin(), suffix, cell)
}

/// cot(x+iy) = `(sin 2x - i·sinh 2y) / ((cosh 2y + 1) - 2cos²x)`, with SIN and COS reduced as
/// Excel's own (`trig_sin_cos`): all 63 cells of probes-numeric im-last-digit-sweep with an
/// imaginary part (T-2044) and `IMCOT("-1.5+0.5i")` -0.0557109884865415-0.463942860183828i (45
/// gen-m9 cells, T-2537), where `cosh 2y - (2cos²x - 1)` prints ...5416. That includes the digits
/// Excel loses for tiny arguments, where the denominator cancels (`IMCOT("1E-5+1E-5i")` real part 49999.9958596485, `IMCOT("0.00001i")`
/// -99999.9917326303i; exact 50000 and 100000). UNVERIFIED (T-2531): below |y| ≈ 1.05E-8 `cosh 2y` rounds to 1 and
/// the result is `#NUM!`. Past `cosh 2y`'s range the value is `∓i` by the
/// sign of y (Q27:R27, R55). A real number is 1 over TAN's kernel ([`real_trig`]; `IMCOT("1E-20")` = 1E+20).
pub(super) fn imcot((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if y == 0.0 {
        return real_trig(x, suffix, cell, Some(0.0), |x| 1.0 / excel_tan(x));
    }
    let cosh_2y = f64::cosh(2.0 * y);
    if cosh_2y.is_infinite() {
        return complex_result(0.0, -y.signum(), suffix);
    }
    let (_, cos_x) = trig_sin_cos(x);
    let (sin_2x, _) = trig_sin_cos(2.0 * x);
    let t = cosh_2y + 1.0 - 2.0 * cos_x * cos_x;
    finite_complex_result(sin_2x / t, -f64::sinh(2.0 * y) / t, suffix, cell)
}

/// csc(x+iy) = `(sin x·(cosh y/t), -cos x·(sinh y/t))` with `t = sin²x + (cosh 2y - 1)/2` and SIN
/// and COS reduced as Excel's own (`trig_sin_cos`): all 69 finite cells of probes-numeric
/// im-last-digit-sweep (T-2044). Writing sinh²y as `(cosh 2y - 1)/2` is what loses Excel's digits
/// for tiny arguments: `IMCSC("1E-5+1E-5i")` is 49999.9979339908-49999.9979306575i (exact
/// 50000.0000016667), `IMCSC("0.00001i")` -99999.9917276303i. A real number is 1 over SIN's kernel ([`real_trig`])
/// (gen-m9/2f0d384b8ed0d392 A109, `IMCSC(0.2)` 5.03348954767234; the complex formula gives ...235), as
/// IMSEC's is `1/cos x` and IMSECH's `1/cosh x` (gen-m9/01810fc89c5ec2c0 A318). UNVERIFIED (T-2531): below |y| ≈ 1.05E-8
/// `cosh 2y` rounds to 1, `t` is 0 and the result is `#NUM!`.
pub(super) fn imcsc((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if y == 0.0 {
        return real_trig(x, suffix, cell, None, |x| 1.0 / excel_sin_cos(x).0);
    }
    let (x_sin, x_cos) = trig_sin_cos(x);
    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_sin * x_sin + (f64::cosh(2.0 * y) - 1.0) / 2.0;
    if t == 0.0 {
        return num_error(cell);
    }
    if !t.is_finite() {
        return overflowed_reciprocal(x_sin, -x_cos * y.signum(), y, suffix);
    }
    complex_result(x_sin * (y_cosh / t), -x_cos * (y_sinh / t), suffix)
}

/// csch(x+iy) is `#NUM!` when the squared modulus of sinh underflows to 0: `IMCSCH("0")` and
/// `IMCSCH("1E-200+1E-200i")` (gen-m9/2f0d384b8ed0d392 A315, 59a16b5ab566ee5c A274), as IMCSC's `t`.
pub(super) fn imcsch((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    let x_sinh = x.sinh();

    let y_cos = y.cos();
    let y_sin = y.sin();

    let t = x_sinh * x_sinh * y_cos * y_cos + x_cosh * x_cosh * y_sin * y_sin;
    if t == 0.0 {
        return num_error(cell);
    }
    reciprocal_result(x_sinh * y_cos / t, -x_cosh * y_sin / t, t, suffix)
}

/// sec(x+iy) = `(cos x·cosh y, sin x·sinh y) / t` with `t = cos²x + ((cosh 2y + 1)/2 - 1)`, sinh²y
/// written as cosh²y - 1 by the half-angle identity: all 70 finite cells of probes-numeric
/// im-last-digit-sweep (T-2743). IMCSC's `(cosh 2y - 1)/2` misses rows 29 and 33 near a pole of the
/// secant (`IMSEC("956.615+0.00387493i")` is -2.46276584105671+258.045027895372i, off from the exact
/// value in the 10th digit). IMSECH is not the same formula turned a quarter: it squares sinh
/// itself, so `IMSEC("4+3i")` has real part -0.0652940278579471 and `IMSECH("3+4i")`
/// -0.065294027857947 (G23, K58).
pub(super) fn imsec((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if y == 0.0 {
        return real_trig(x, suffix, cell, None, |x| 1.0 / excel_sin_cos(x).1);
    }
    let (x_sin, x_cos) = trig_sin_cos(x);
    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_cos * x_cos + ((f64::cosh(2.0 * y) + 1.0) / 2.0 - 1.0);
    if !t.is_finite() {
        return overflowed_reciprocal(x_cos, x_sin * y.signum(), y, suffix);
    }
    complex_result(x_cos * y_cosh / t, x_sin * y_sinh / t, suffix)
}

/// sech(x+iy) = `(cosh x·cos y, -sinh x·sin y) / t` with `t = cos²y + sinh²x`, sinh²x fused into the
/// sum (one rounding, `mul_add`): all 62 finite cells of probes-numeric im-last-digit-sweep (T-2750).
/// The unfused sum (T-2044) misses only `IMSECH("1.34384-77.6742i")`, whose imaginary part Excel
/// prints as ...306336 (J34; exact 0.37667538230633553); fusing cos²y instead also misses it.
pub(super) fn imsech((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    if y == 0.0 {
        return complex_result(1.0 / x_cosh, 0.0, suffix);
    }
    let x_sinh = x.sinh();
    let (y_sin, y_cos) = trig_sin_cos(y);

    let t = x_sinh.mul_add(x_sinh, y_cos * y_cos);
    if !t.is_finite() {
        return overflowed_reciprocal(y_cos, -y_sin * x.signum(), x, suffix);
    }
    complex_result(x_cosh * y_cos / t, -x_sinh * y_sin / t, suffix)
}

pub(super) fn imsin((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if y == 0.0 {
        return real_trig(x, suffix, cell, None, |x| excel_sin_cos(x).0);
    }
    if trig_exp_overflows(y) {
        return num_error(cell);
    }
    let (y_sinh, y_cosh) = exp_sinh_cosh(y);
    finite_complex_result(x.sin() * y_cosh, x.cos() * y_sinh, suffix, cell)
}

pub(super) fn imsinh((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if hyperbolic_exp_overflows(x, y) {
        return num_error(cell);
    }
    if let Some(result) = hyperbolic_log_form((x, y, suffix), true, cell) {
        return result;
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

pub(super) fn imtan((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    // Below the limit a real number keeps the double-angle form, which gives the Mac desktop cache
    // of ironcalc-calc COMPLEXs.xlsx N6 (`IMTAN(5)` -3.38051500624658) where the web gives TAN's
    // kernel (-3.38051500624659): a web-vs-desktop conflict left to T-1075 item 17 (T-2534).
    if y == 0.0 && x.abs() >= EXCEL_TRIG_LIMIT {
        return num_error(cell);
    }
    let (x, y) = tan_parts(x, y);
    complex_result(x, y, suffix)
}

