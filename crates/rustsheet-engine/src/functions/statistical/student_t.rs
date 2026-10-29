//! The Student-t distribution behind T.DIST, T.DIST.RT, T.DIST.2T, TDIST and T.TEST (T-0969).
//!
//! Excel's own algorithm is not known; this is the model the probes-numeric t sweep fits best
//! (T-0969 attempt 3, against the exact values of the fit lab's `t_cdf_reference`):
//! - one tail L = P(T > |t|) is computed, and every other value follows from it: T.DIST of a
//!   positive t is 1 - L, T.DIST.RT(x) of a negative x is 1 - L, T.DIST.2T is 2 L. In Excel
//!   `T.DIST.RT(x,n)` = `T.DIST(-x,n,TRUE)` wherever the sweep has both, and 1 - L is bit-exact
//!   on 118 of the 146 nonzero T.DIST cells against 113 for the correctly rounded CDF;
//! - with y = n/(n+t^2) and z = t^2/(n+t^2), L = `I_y(n/2, 1/2)`/2. For t <= 1 or y > 0.99
//!   (the next item covers the rest) and away from the deep tail (y > z), L is the exact value,
//!   rounded once: for an integer n up to [`SERIES_MAX_DF`] the finite series for integer df in
//!   double-double. In the deep tail (y <= z) Excel's L carries the error of exp((n/2) ln y):
//!   Cephes' power series with that power and a correctly rounded Gamma ratio reproduces it
//!   (`T.DIST.RT(1E6,10)` = 1.230468749943611E-56, F18 of `T_DIST.xlsx`).
//! - beyond t = 1 and short of y = 0.99, deep tail included, L is `I_y(n/2, 1/2)`/2 at y as the
//!   worksheet rounds it, fl(n / fl(n + fl(t*t))), through the same kernel as T.TEST's
//!   [`student_t_tail_at`] (T-0969 attempt 7, probes-numeric t0969-t-tail-grid);
//! - at df 2, for fl(1 - y) <= 0.9 and t of any size, L is the incomplete beta's closed form on
//!   bratio's oriented arguments, halved, at that rounded y ([`closed_form_tail`], T-0969 attempt 8,
//!   T-2502);
//! - elsewhere away from the deep tail (an integer df above [`SERIES_MAX_DF`], or a series tail too
//!   small to keep its digits) L comes from Cephes' fractions with every input
//!   formed from z and t^2/n, never from the rounded y ([`centre_tail`]): Excel is within a few ulp
//!   of the exact value there up to df 1E10 (`T.DIST(2,1E10,TRUE)` = 0.977249868038323,
//!   probes-numeric m9-t-edges), and a rounded y moves L by about n eps.
//!
//! - from df 3 on, L is one call into the ported incomplete-beta kernel, K(n/2, 1/2, y, z), halved
//!   ([`kernel_tail`], T-2509), ahead of every rule below, which now covers df 1 and 2 and the
//!   points the kernel does not take;
//! - for df 13 to 16 the series tail is scaled by Excel's 1/B(n/2, 1/2) over the exact one: Excel's
//!   BETA.DIST forms B from GAMMALNs there ([`log_gamma_ratio`], T-0969 attempt 10).
//!
//! Every loop is bounded: [`SERIES_MAX_DF`] / 2 series terms, [`TAYLOR_TERMS`] Taylor terms and
//! [`MAX_POWER_SERIES_TERMS`] power-series terms a call.

use super::bratio::ported_tails;
use super::gammaln::excel_gammaln;
use super::incomplete_beta::{beta_reg_upper, closed_form_tails, incbcf, incbd};
use crate::arithmetic::flush_subnormal;
use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_mul, dd_sqrt, excel_exp};

/// A double-double: an unevaluated sum `hi + lo` with |lo| <= ulp(hi)/2.
type Dd = (f64, f64);

/// Largest integer df that the double-double series ([`centre_series`], [`integer_ratio`]) sum:
/// about df/2 double-double steps a call.
const SERIES_MAX_DF: f64 = 1000.0;
/// From this shape up to [`LOG_GAMMA_BETA_MAX`] Excel's B(a, 1/2) is a sum of GAMMALNs
/// ([`log_gamma_ratio`]). Below it (df under 13) the form is not probed and scaling by it loses
/// probes-numeric cells (statistics.md "Student t prefactor per df").
const LOG_GAMMA_BETA_MIN: f64 = 6.5;
/// Up to this shape Excel's B(a, 1/2) is a sum of GAMMALNs: past 8 GAMMALN is Stirling's series and
/// Excel's B is accurate.
const LOG_GAMMA_BETA_MAX: f64 = 8.0;
/// From this shape on, Gamma(a + 1/2)/Gamma(a) comes from its asymptotic series.
const ASYMPTOTIC_MIN_SHAPE: f64 = 85.0;
/// Cephes' `MACHEP`, 2^-53: the power series stops below it.
const MACHEP: f64 = 1.110_223_024_625_156_5e-16;
/// Cephes' `MAXLOG`, ln of the largest double.
const MAXLOG: f64 = 709.782_712_893_384;
/// Cap on the power-series terms: in the deep tail y <= 1/2, so 60 terms reach 2^-53.
const MAX_POWER_SERIES_TERMS: u32 = 200;
/// Terms of the sin and cos Taylor series in [`dd_atan`]: theta <= pi/2 and (pi/2)^40/40! is
/// far below 2^-106.
const TAYLOR_TERMS: u32 = 40;
/// Below this the double-double tail 1/2 - A/2 keeps fewer than 40 of its 106 bits.
const MIN_SERIES_TAIL: f64 = 1e-12;
/// df 1 and 2 keep today's path rather than the kernel call ([`kernel_tail`]).
const KERNEL_MIN_DF: f64 = 2.0;
/// Largest rounded y = n/(n+t^2) at which the tail of a t beyond 1 is taken at that y
/// ([`unflushed_upper_tail`]).
const ROUNDED_Y_MAX: f64 = 0.99;
/// 1/pi as a double-double.
const FRAC_1_PI_DD: Dd = (
    f64::from_bits(0x3FD4_5F30_6DC9_C883),
    f64::from_bits(0xBC76_B01E_C541_7056),
);
/// 1/sqrt(pi), rounded.
const FRAC_1_SQRT_PI: f64 = 0.564_189_583_547_756_3;
/// Coefficients of Gamma(a + 1/2)/(Gamma(a) sqrt(a)) in powers of 1/a, 1 - 1/(8a) +
/// 1/(128a^2) + 5/(1024a^3) - 21/(32768a^4) - 399/(262144a^5) + 869/(4194304a^6): the first
/// omitted term is below 2^-53 relative from a = 85 on.
const ASYMPTOTIC_RATIO: [f64; 7] = [
    1.0,
    -1.0 / 8.0,
    1.0 / 128.0,
    5.0 / 1024.0,
    -21.0 / 32768.0,
    -399.0 / 262_144.0,
    869.0 / 4_194_304.0,
];

/// The Student-t tails `(P(T < t), P(T > t))` for `n` degrees of freedom, both from the one tail
/// L = P(T > |t|) (module doc).
pub(crate) fn student_t_tails(t: f64, n: f64) -> (f64, f64) {
    if t.is_nan() || n.is_nan() {
        return (f64::NAN, f64::NAN);
    }
    if t == 0.0 {
        return (0.5, 0.5);
    }
    let tail = upper_tail(t.abs(), n);
    if t > 0.0 {
        (1.0 - tail, tail)
    } else {
        (tail, 1.0 - tail)
    }
}

/// L = P(T > t) for t > 0. A tail below the normal range is 0: main (statrs) returned 0 for a
/// subnormal tail and `T.DIST.2T(...)` in gen-m9/3ec66461400faf2f A15 is 0 in Excel; the graph
/// oracle cannot tell 0 from a kept subnormal (decision 0042), so the flush stays until a probe
/// with a companion cell decides it.
fn upper_tail(t: f64, n: f64) -> f64 {
    flush_subnormal(unflushed_upper_tail(t, n))
}

/// [`upper_tail`] before the subnormal flush.
fn unflushed_upper_tail(t: f64, n: f64) -> f64 {
    // Excel forms n + t^2 in doubles: where it rounds to n, y = 1 and L is exactly 1/2
    // (`T.DIST.RT(1E-10,3)` = `TDIST(1E-10,3,1)` = 0.5, probes-numeric t1048-tdist A27, B27).
    if n + t * t == n {
        return 0.5;
    }
    if let Some(tail) = kernel_tail(t, n) {
        return tail;
    }
    // Beyond t = 1, and short of y = 0.99, Excel evaluates the tail at y rounded as the
    // worksheet would round it, fl(n / fl(n + fl(t^2))), as T.TEST does (`student_t_tail_at`);
    // inside it the tail is the exact one at t (docs/excel-semantics/statistics.md, "T.DIST.RT
    // follows the rounded y beyond t = 1"). A t whose square overflows (y = 0) keeps the
    // path below, which still gives a far tail for a small n.
    let rounded_y = n / (n + t * t);
    if let Some(tail) = closed_form_tail(rounded_y, n) {
        return tail;
    }
    if t > 1.0 && rounded_y > 0.0 && rounded_y <= ROUNDED_Y_MAX {
        return tail_at(rounded_y, n);
    }
    let (z, y) = beta_arguments(t, n);
    let square = t * t;
    let args = TailArguments {
        z,
        y,
        square_over_n: square / n,
        n_over_square: n / square,
        inside_one: t <= 1.0,
        series: series_arguments(t, n),
    };
    tail_from(&args, n)
}

/// L = w/2 for w of one incomplete-beta kernel call, K(n/2, 1/2, y, z), with the worksheet's
/// y = fl(n/fl(n + fl(t·t))) and complement z = fl(fl(t·t)/fl(n + fl(t·t))) (docs/excel-semantics/
/// statistics.md "Student t tails are one kernel call, K(n/2, 1/2, y, z)"; T-2464 r2-tdist, T-2509):
/// bgrat at b = 1/2 (n > 30), bup + bgrat (n <= 30) and bpser in the centre and the deep tail.
/// `None` where t·t overflows and for df 1 and 2, which keep today's path: df 2 its closed forms
/// (which read fl(1 - y)) and today's deep tail, df 1 (bpser at a = b = 1/2) the double-double
/// series, bit-exact on 41 of cells.tsv's 67 df-1 tails against 24 through the ported bpser
/// (`T.DIST.2T(1,1)` = 0.5000000000000001 is 5 ulp off through it). At every df from 3 the
/// kernel is bit-exact on more cells than today's path in every branch.
fn kernel_tail(t: f64, n: f64) -> Option<f64> {
    let square = t * t;
    let sum = n + square;
    if !sum.is_finite() || n <= KERNEL_MIN_DF {
        return None;
    }
    let y = n / sum;
    let z = square / sum;
    let a = 0.5 * n;
    let (w, _) = ported_tails(a, 0.5, y, z)?;
    Some(0.5 * w)
}

/// L = `I_y(n/2, 1/2)`/2 at a y that is already a double: T.TEST's p-value. Excel's T.TEST
/// rounds y = df/(df + t^2) to a double first and evaluates the incomplete beta there, unlike
/// T.DIST.RT: the exact tail at T.TEST's own t is 42 ulp from Excel at
/// `T.TEST(G1:G10,H1:H10,2,2)` (probes-numeric t1048-ttest B31, df 17, t^2 = 0.039), and
/// `I_y` at the rounded y is bit-exact there. z = 1 - y is exact wherever the deep tail
/// (y <= z) does not take over.
pub(crate) fn student_t_tail_at(y: f64, n: f64) -> f64 {
    if y.is_nan() || n.is_nan() {
        return f64::NAN;
    }
    flush_subnormal(tail_at(y, n))
}

/// [`student_t_tail_at`] before the subnormal flush, for a y in [0, 1].
fn tail_at(y: f64, n: f64) -> f64 {
    if y >= 1.0 {
        return 0.5;
    }
    if y <= 0.0 {
        return 0.0;
    }
    if let Some(tail) = closed_form_tail(y, n) {
        return tail;
    }
    let z = 1.0 - y;
    let series = (y > z && is_series_df(n)).then(|| SeriesArguments {
        n,
        cos2: (y, 0.0),
        sin: dd_sqrt((z, 0.0)),
        tan: dd_sqrt(dd_div((z, 0.0), (y, 0.0))),
    });
    let args = TailArguments {
        z,
        y,
        square_over_n: z / y,
        n_over_square: y / z,
        // t <= 1, i.e. t^2 = n z / y <= 1.
        inside_one: z * n <= y,
        series,
    };
    tail_from(&args, n)
}

/// L at df 2 for y = n/(n+t^2) as the worksheet rounds it, fl(2/fl(2 + fl(t*t))), with complement
/// fl(1 - y): where the incomplete beta takes Excel's closed form on bratio's oriented arguments
/// ([`closed_form_tails`] for `I_y(1, 1/2)`), inside t = 1 too. That is every y with
/// fl(1 - y) <= 0.9: `T.DIST.RT(0.0513,2)` = 0.4818746322295301 is that form, where the exact tail
/// at t is 15 ulp off (probes-numeric t0969-t-tail-grid), and so is every `T.DIST.RT(t,2)` from
/// t = 2 to 4.24 (t0969-t-df2-cut; docs/excel-semantics/statistics.md "Incomplete beta closed forms
/// act on the oriented arguments", 508 of 508 df-2 cells, T-2502). `None` elsewhere.
fn closed_form_tail(y: f64, n: f64) -> Option<f64> {
    if n != 2.0 || !(y > 0.0 && y < 1.0) {
        return None;
    }
    closed_form_tails(y, 1.0 - y, 1.0, 0.5).map(|(lower, _)| 0.5 * lower)
}

/// The inputs one tail is computed from, z = t^2/(n+t^2) and y = n/(n+t^2) among them, so that
/// one kernel serves a t ([`unflushed_upper_tail`]) and a y already rounded
/// ([`student_t_tail_at`]).
struct TailArguments {
    z: f64,
    y: f64,
    /// t^2/n.
    square_over_n: f64,
    /// n/t^2.
    n_over_square: f64,
    /// t <= 1: the side of [`centre_tail`]'s split.
    inside_one: bool,
    /// The integer-df series' inputs; `None` where the series does not apply.
    series: Option<SeriesArguments>,
}

/// L from its [`TailArguments`] (module doc).
fn tail_from(args: &TailArguments, n: f64) -> f64 {
    let (z, y) = (args.z, args.y);
    let Some(ratio) = gamma_ratio(n) else {
        return 0.5 * beta_reg_upper(z, y, 0.5, 0.5 * n);
    };
    if y <= z {
        return 0.5 * power_series(0.5 * n, y, ratio);
    }
    if let Some(tail) = args.series.as_ref().map(exact_series_tail) {
        if tail.0 > MIN_SERIES_TAIL {
            return excel_prefactor_scale(tail, 0.5 * n);
        }
    }
    centre_tail(args, n, z, ratio)
}

/// L = `I_y(n/2, 1/2)`/2 for 0 < t^2 < n and an integer n (or T.TEST's fractional Welch df,
/// checked only on the probes-numeric t1048/t1049-ttest cells), from z = t^2/(n+t^2), t^2/n and
/// n/t^2 (each within an ulp or two) with `ratio` = Gamma((n+1)/2)/(Gamma(n/2) Gamma(1/2)) and y^(n/2) = exp(-(n/2)
/// log1p(t^2/n)). Up to t = 1 the centre A = `I_z(1/2, n/2)` by `incbcf` (6 steps at most) and
/// L = (1 - A)/2; beyond it L itself by `incbd` on the ratio y/z = n/t^2. Each fraction converges
/// in under 160 steps on its side of t = 1 at every df to 1E10, within a few ulp of the exact L,
/// and loses digits on the other side (T-0969 attempt 4).
fn centre_tail(args: &TailArguments, n: f64, z: f64, ratio: f64) -> f64 {
    let a = 0.5 * n;
    let power = (-a * args.square_over_n.ln_1p()).exp();
    if args.inside_one {
        // A = z^(1/2) y^a / ((1/2) B(1/2, a)) * w.
        let centre = 2.0 * z.sqrt() * power * ratio * incbcf(0.5, a, z);
        0.5 - 0.5 * centre
    } else {
        // I_y(a, 1/2) = y^a z^(1/2) / (a B(a, 1/2)) * w / z.
        0.5 * power * z.sqrt() * ratio / a * (incbd(a, 0.5, args.n_over_square) / z)
    }
}

/// (t^2/(n+t^2), n/(n+t^2)) for t > 0, each rounded once from double-double arithmetic: forming
/// one as 1 minus the other loses every digit at a large n, where Excel stays within an ulp of
/// the exact value (`T.DIST(-2,1E6,TRUE)`, T-0969 attempt 2).
fn beta_arguments(t: f64, n: f64) -> (f64, f64) {
    // Past 1E150 t^2 (and its error term) leave the double range; z is then 1 to the last bit.
    if t > 1e150 {
        return (1.0, n / t / t);
    }
    let square = dd_mul((t, 0.0), (t, 0.0));
    let sum = dd_add(square, (n, 0.0));
    (dd_div(square, sum).0, dd_div((n, 0.0), sum).0)
}

/// Cephes' power series for `I_y(a, 1/2)` (`pseries` with b = 1/2), its prefactor as
/// `s * ratio * exp(a ln y)` with `ratio` = Gamma(a + 1/2)/(Gamma(a) Gamma(1/2)).
fn power_series(a: f64, y: f64, ratio: f64) -> f64 {
    let b = 0.5;
    let reciprocal = 1.0 / a;
    let mut u = (1.0 - b) * y;
    let first = u / (a + 1.0);
    let mut v = first;
    let mut term = u;
    let mut k = 2.0;
    let mut sum = 0.0;
    let stop = MACHEP * reciprocal;
    for _ in 0..MAX_POWER_SERIES_TERMS {
        if v.abs() <= stop {
            break;
        }
        u = (k - b) * y / k;
        term *= u;
        v = term / (a + k);
        sum += v;
        k += 1.0;
    }
    sum += first;
    sum += reciprocal;
    let exponent = a * y.ln();
    if exponent.abs() < MAXLOG {
        sum * ratio * exponent.exp()
    } else {
        (ratio.ln() + exponent + sum.ln()).exp()
    }
}

/// Gamma((n+1)/2)/(Gamma(n/2) Gamma(1/2)), rounded once where it can be: by [`integer_ratio`] for
/// an integer n up to [`SERIES_MAX_DF`] and by its asymptotic series from n/2 =
/// [`ASYMPTOTIC_MIN_SHAPE`]; for a fractional n below that (T.TEST's Welch df), by a recurrence
/// up to the asymptotic series.
fn gamma_ratio(n: f64) -> Option<f64> {
    let a = 0.5 * n;
    if a >= ASYMPTOTIC_MIN_SHAPE {
        return Some(asymptotic_ratio((a, 0.0)).0);
    }
    if let Some(ratio) = integer_ratio(n) {
        return Some(ratio.0);
    }
    if a.is_nan() || a <= 0.0 {
        return None;
    }
    // A fractional n (T.TEST's Welch df): up from a to the asymptotic series by
    // r(a) = r(a + 1) a / (a + 1/2), in double-double. libm's tgamma ratio is several ulp off,
    // and 9 ulp off at `T.TEST(D1:D10,F1:F10,2,3)` (probes-numeric t1048-ttest B18).
    let steps = (ASYMPTOTIC_MIN_SHAPE - a).ceil();
    let mut ratio = asymptotic_ratio(dd_add((a, 0.0), (steps, 0.0)));
    let mut k = steps;
    while k > 0.0 {
        k -= 1.0;
        let shape = dd_add((a, 0.0), (k, 0.0));
        ratio = dd_div(dd_mul(ratio, shape), dd_add(shape, (0.5, 0.0)));
    }
    Some(ratio.0)
}

/// 1/B(a, 1/2) the way Excel's BETA.DIST forms B for a shape up to [`LOG_GAMMA_BETA_MAX`]:
/// exp(GAMMALN(a) + (GAMMALN(1/2) - GAMMALN(a + 1/2))) with Excel's own GAMMALN and EXP.
fn log_gamma_ratio(a: f64) -> Option<f64> {
    if !(LOG_GAMMA_BETA_MIN..=LOG_GAMMA_BETA_MAX).contains(&a) {
        return None;
    }
    let beta = excel_exp(excel_gammaln(0.5) + (excel_gammaln(a) - excel_gammaln(a + 0.5)));
    Some(1.0 / beta)
}

/// The exact tail L = 1/2 - A/2 of the integer-df series, in double-double.
fn exact_series_tail(args: &SeriesArguments) -> Dd {
    let inside = centre_series(args);
    dd_add((0.5, 0.0), (-0.5 * inside.0, -0.5 * inside.1))
}

/// The exact tail L (a double-double) times Excel's 1/B(a, 1/2) over the exact one, rounded once:
/// where [`log_gamma_ratio`] applies, Excel's tail carries its Beta function's error at every t.
fn excel_prefactor_scale(tail: Dd, a: f64) -> f64 {
    // The series tail never used the exact 1/B; it is formed here only as the divisor.
    match (log_gamma_ratio(a), integer_ratio(2.0 * a)) {
        (Some(excel), Some(exact)) => dd_mul(tail, dd_div((excel, 0.0), exact)).0,
        _ => tail.0,
    }
}

/// Gamma(a + 1/2)/(Gamma(a) sqrt(pi)) by its asymptotic series, for a >= [`ASYMPTOTIC_MIN_SHAPE`].
fn asymptotic_ratio(a: Dd) -> Dd {
    let inverse = 1.0 / a.0;
    let mut sum = (0.0, 0.0);
    for &coefficient in ASYMPTOTIC_RATIO.iter().rev() {
        sum = dd_add(dd_mul(sum, (inverse, 0.0)), (coefficient, 0.0));
    }
    dd_mul(dd_mul(sum, dd_sqrt(a)), (FRAC_1_SQRT_PI, 0.0))
}

/// Gamma((n+1)/2)/(Gamma(n/2) sqrt(pi)) for an integer n up to [`SERIES_MAX_DF`], in
/// double-double: 1/pi times the product of (m+1)/m over m = 1, 3, ..., n-2 for an odd n, and
/// 1/2 times the product over m = 2, 4, ..., n-2 for an even n (the ratio for n is the one for
/// n - 2 times (n-1)/(n-2)).
fn integer_ratio(n: f64) -> Option<Dd> {
    if !is_series_df(n) {
        return None;
    }
    let odd = n % 2.0 == 1.0;
    let mut ratio = if odd { FRAC_1_PI_DD } else { (0.5, 0.0) };
    let mut m = if odd { 1.0 } else { 2.0 };
    while m < n {
        ratio = dd_div_small(dd_mul(ratio, (m + 1.0, 0.0)), m);
        m += 2.0;
    }
    Some(ratio)
}

/// An integer df from 1 to [`SERIES_MAX_DF`].
fn is_series_df(n: f64) -> bool {
    (1.0..=SERIES_MAX_DF).contains(&n) && n.fract() == 0.0
}

/// A = P(|T| < t) for t > 0 and an integer df n <= [`SERIES_MAX_DF`], in double-double, by the
/// finite series for integer df (Hill 1970, CACM algorithm 395), with cos^2 = n/(n+t^2) and
/// sin = t/sqrt(n+t^2):
/// - even n: A = sin (1 + 1/2 cos^2 + 1*3/(2*4) cos^4 + ...), n/2 terms;
/// - odd n: A = 2/pi (theta + sin cos (1 + 2/3 cos^2 + 2*4/(3*5) cos^4 + ...)), (n-1)/2 terms,
///   theta = atan(t/sqrt(n)).
fn centre_series(args: &SeriesArguments) -> Dd {
    let SeriesArguments { n, cos2, sin, tan } = *args;
    let odd = n % 2.0 == 1.0;
    let offset = if odd { 1.0 } else { 0.0 };
    // sum_k c_k cos^(2k), c_0 = 1 and c_k = c_(k-1) (2k - 1 + offset)/(2k + offset).
    let terms = (n - offset) / 2.0;
    let mut total = (0.0, 0.0);
    let mut term = (1.0, 0.0);
    let mut k = 0.0;
    while k < terms {
        total = dd_add(total, term);
        k += 1.0;
        term = dd_div_small(
            dd_mul(dd_mul(term, cos2), (2.0 * k - 1.0 + offset, 0.0)),
            2.0 * k + offset,
        );
    }
    if !odd {
        return dd_mul(sin, total);
    }
    let theta = dd_atan(tan);
    let inner = if terms > 0.0 {
        dd_mul(dd_mul(sin, dd_sqrt(cos2)), total)
    } else {
        (0.0, 0.0)
    };
    let half = dd_mul(dd_add(theta, inner), FRAC_1_PI_DD);
    (2.0 * half.0, 2.0 * half.1)
}

/// The inputs of [`centre_series`] in double-double, for an integer df n <= [`SERIES_MAX_DF`].
#[derive(Clone, Copy)]
struct SeriesArguments {
    n: f64,
    /// cos^2 = n/(n+t^2).
    cos2: Dd,
    /// sin = t/sqrt(n+t^2).
    sin: Dd,
    /// tan = t/sqrt(n), the odd-n series' atan argument.
    tan: Dd,
}

/// [`SeriesArguments`] of a t > 0; `None` where the series does not apply.
fn series_arguments(t: f64, n: f64) -> Option<SeriesArguments> {
    if !is_series_df(n) || t > 1e150 {
        return None;
    }
    let square = dd_mul((t, 0.0), (t, 0.0));
    let sum = dd_add(square, (n, 0.0));
    Some(SeriesArguments {
        n,
        cos2: dd_div((n, 0.0), sum),
        sin: dd_div((t, 0.0), dd_sqrt(sum)),
        tan: dd_div((t, 0.0), dd_sqrt((n, 0.0))),
    })
}

/// The Student-t density for an integer df n: below [`SERIES_MAX_DF`] rounded once from
/// double-double, ratio cos^(n+1) / sqrt(n) with cos^2 = n/(n+t^2) and the ratio of
/// [`integer_ratio`]; from it on ratio / sqrt(n) * exp(-((n+1)/2) log1p(t^2/n)), within 2 ulp
/// of Excel at t = 2 from df 1E10 to 1E11 (`T.DIST(2,1E11,FALSE)` = 0.05399096651413289,
/// probes-numeric m9-t-edges) and closer to Excel than the exact density at df 1000
/// (`T.DIST(30,1000,FALSE)`: 2 ulp against 94, m9-t-dist-pdf A129), though not at
/// `T.DIST(30,1E6,FALSE)` (348 against 200, A144). `None` elsewhere.
pub(crate) fn student_t_density(t: f64, n: f64) -> Option<f64> {
    if t.is_nan() || t.abs() >= 1e150 {
        return None;
    }
    if n >= SERIES_MAX_DF && n.fract() == 0.0 {
        let ratio = gamma_ratio(n)?;
        let density = ratio / n.sqrt() * (-0.5 * (n + 1.0) * (t * t / n).ln_1p()).exp();
        return density.is_finite().then_some(density);
    }
    let ratio = integer_ratio(n)?;
    let square = dd_mul((t, 0.0), (t, 0.0));
    let cos2 = dd_div((n, 0.0), dd_add(square, (n, 0.0)));
    // cos^(n+1) = (cos^2)^((n+1)/2), with one sqrt when n + 1 is odd.
    let mut power = if n % 2.0 == 1.0 {
        (1.0, 0.0)
    } else {
        dd_sqrt(cos2)
    };
    let mut base = cos2;
    // n <= SERIES_MAX_DF: the cast is exact.
    let mut exponent = ((n + 1.0) / 2.0).floor() as u32;
    while exponent > 0 {
        if exponent & 1 == 1 {
            power = dd_mul(power, base);
        }
        base = dd_mul(base, base);
        exponent >>= 1;
    }
    Some(dd_div(dd_mul(ratio, power), dd_sqrt((n, 0.0))).0)
}

/// atan of a positive double-double u: the double atan `theta_0`, then one Newton step on
/// tan(theta) = u, d = (u cos - sin)/(cos + u sin), with sin and cos of `theta_0` by their
/// double-double Taylor series.
fn dd_atan(u: Dd) -> Dd {
    let theta = u.0.atan();
    let (mut sin, mut cos) = ((0.0, 0.0), (0.0, 0.0));
    let mut power = (1.0, 0.0);
    for k in 0..TAYLOR_TERMS {
        let sign = if (k / 2) % 2 == 0 { 1.0 } else { -1.0 };
        let signed = (sign * power.0, sign * power.1);
        if k % 2 == 0 {
            cos = dd_add(cos, signed);
        } else {
            sin = dd_add(sin, signed);
        }
        power = dd_div_small(dd_mul(power, (theta, 0.0)), f64::from(k + 1));
    }
    let numerator = dd_add(dd_mul(u, cos), (-sin.0, -sin.1));
    let denominator = dd_add(cos, dd_mul(u, sin));
    dd_add((theta, 0.0), dd_div(numerator, denominator))
}

#[cfg(test)]
mod tests {
    use super::super::incomplete_beta::{beta_reg_complement, A1_EXPM1_MAX_X};
    use super::*;
    use crate::test::util::ulps;

    #[test]
    fn bgrat_tails_match_excel() {
        // probes-numeric t2464-r2-tdist-p1 part C (excel-graph): T.DIST.RT(t, n) for n > 30 in
        // bgrat's region, one kernel call K(n/2, 1/2, y, z) with bgrat at b = 1/2 (T-2509).
        for (t, n, excel) in [
            (6.876_766_968_460_287, 1184.0, 4.944_295_635_855_830_5e-12),
            (6.688_772_703_856_486, 1462.0, 1.597_408_460_492_365_4e-11),
            (1.194_984_828_755_362_2, 818.0, 0.116_219_651_401_460_62),
            (5.651_628_247_834_669, 450.0, 1.412_472_689_666_297_8e-8),
            (2.907_418_672_926_900_7, 869.0, 0.001_868_483_605_870_204_5),
        ] {
            assert_eq!(student_t_tails(t, n).1, excel, "T.DIST.RT({t},{n})");
            assert_eq!(student_t_tails(-t, n).0, excel, "T.DIST({t},{n},TRUE)");
        }
    }

    #[test]
    fn kernel_tail_covers_every_df_from_three() {
        // bup + bgrat (n <= 30), bgrat (n > 30), the centre and the deep tail (bpser) go through
        // the kernel; df 1, df 2 and an overflowing t·t keep today's path.
        for (t, n) in [
            (2.5, 30.0),
            (2.5, 100.0),
            (0.5, 10.0),
            (50.0, 10.0),
            (2.0, 3.0),
        ] {
            assert!(kernel_tail(t, n).is_some(), "{t}, {n}");
        }
        assert!(kernel_tail(1.0, 1.0).is_none());
        assert!(kernel_tail(3.0, 2.0).is_none());
        assert!(kernel_tail(1e300, 10.0).is_none());
    }

    #[test]
    fn tail_beyond_one_is_taken_at_the_rounded_y() {
        // probes-numeric t0969-t-tail-grid B1117 and B1078: `T.DIST.RT(3.2013,100)` and
        // `T.DIST.RT(1.2513,100)` are I_y/2 at y = fl(100/fl(100 + fl(t*t))); the exact tails at t
        // are 33 and 25 ulp away.
        assert_eq!(
            student_t_tails(3.2013, 100.0).1,
            0.000_917_214_524_131_372_6
        );
        assert_eq!(student_t_tails(1.2513, 100.0).1, 0.106_872_219_526_027_31);
        let y = 100.0 / (100.0 + 3.2013 * 3.2013);
        assert_eq!(
            student_t_tails(3.2013, 100.0).1,
            student_t_tail_at(y, 100.0)
        );
    }

    #[test]
    fn df_fifteen_tail_carries_excels_beta_function() {
        // probes-numeric t0969-t-beta-link: Excel's `T.DIST.RT(2,15)` is 0.03197250364236003, about
        // 20 x 2^-53 under the exact tail, the error of its B(7.5, 1/2) from GAMMALNs
        // (t0969-beta-switch); df 17 is past the GAMMALN form and stays exact. The exact tail is
        // further off than the one ulp left here.
        let excel = 0.031_972_503_642_360_03;
        assert!(ulps(student_t_tails(2.0, 15.0).1, excel) <= 1);
        let exact = series_arguments(2.0, 15.0).map_or(f64::NAN, |args| exact_series_tail(&args).0);
        assert!(ulps(exact, excel) > 1, "{exact}");
        assert_eq!(log_gamma_ratio(8.5), None);
        assert_eq!(log_gamma_ratio(6.0), None);
    }

    #[test]
    fn df_two_tail_is_the_closed_form_down_to_y_one_tenth() {
        // probes-numeric t0969-t-tail-grid B2 and t0969-t-df2-cut B1, B201, B449: 1 - exp(ln(1-y)/2)
        // at the rounded y, inside t = 1 and out to t = 4.24 (y = 0.100112).
        assert_eq!(student_t_tails(0.0513, 2.0).1, 0.481_874_632_229_530_1);
        assert_eq!(student_t_tails(2.0, 2.0).1, 0.091_751_709_536_136_98);
        assert_eq!(student_t_tails(3.0, 2.0).1, 0.047_732_983_133_354_61);
        assert_eq!(student_t_tails(4.24, 2.0).1, 0.025_687_899_579_994_455);
        assert_eq!(student_t_tails(-4.24, 2.0).0, 0.025_687_899_579_994_455);
        // T.TEST's y path takes it too.
        let y = 2.0 / (2.0 + 3.0 * 3.0);
        assert_eq!(student_t_tail_at(y, 2.0), 0.047_732_983_133_354_61);
        // Below y = 0.1 (t = 4.245, B450) Excel leaves the closed form, which is 1 ulp off there;
        // the incomplete beta's a = 1 tail is the uncancelled form there (T-0974) and gives
        // Excel's value.
        let y = 2.0 / (2.0 + 4.245 * 4.245);
        assert!(y < A1_EXPM1_MAX_X);
        assert_eq!(closed_form_tail(y, 2.0), None);
        let closed = 0.5 * (1.0 - (0.5 * (1.0 - y).ln()).exp());
        assert_eq!(ulps(closed, 0.025_631_992_844_658_943), 1);
        let tail = 0.5 * beta_reg_complement(y, 1.0 - y, 1.0, 0.5);
        assert_eq!(ulps(tail, 0.025_631_992_844_658_943), 0);
        assert_eq!(closed_form_tail(0.5, 3.0), None);
    }

    #[test]
    fn t_test_tail_at_a_rounded_y() {
        // probes-numeric t1048-ttest B31, `T.TEST(G1:G10,H1:H10,2,2)`: df 17, t =
        // 0.19827206571599848 and y = fl(17/(17 + t^2)). Excel's 0.8451838964108318 is I_y at
        // that y; the exact two tails at t are 42 ulp away.
        let y = 0.997_692_875_599_852_2;
        assert_eq!(2.0 * student_t_tail_at(y, 17.0), 0.845_183_896_410_831_8);
        let at_t = 2.0 * student_t_tails(0.198_272_065_715_998_48, 17.0).1;
        assert!(ulps(at_t, 0.845_183_896_410_831_8) > 30);
        // Ends: y = 1 (t tiny) is 1/2 and y = 0 (t infinite) is 0.
        assert_eq!(student_t_tail_at(1.0, 5.0), 0.5);
        assert_eq!(student_t_tail_at(0.0, 5.0), 0.0);
        assert!(student_t_tail_at(f64::NAN, 5.0).is_nan());
        // A fractional (Welch) df, t1048-ttest B27 `T.TEST(D1:D10,F1:F11,2,3)`: within 5 ulp of
        // Excel's 0.32079800240540124, where libm's Gamma ratio and Cephes' plain fraction are
        // 9 and 111 ulp off on t1048-ttest cells.
        let welch = 2.0 * student_t_tail_at(0.946_466_489_851_057_3, 18.405_961_938_104_15);
        assert!(ulps(welch, 0.320_798_002_405_401_24) <= 5);
    }

    #[test]
    fn excel_cells() {
        // T_DIST.xlsx `T.DIST T.INV T.DIST.RT`: D15 and D16 are bit-exact.
        assert_eq!(student_t_tails(1.0, 1.0).0, 0.75);
        assert_eq!(student_t_tails(1.0, 2.0).0, 0.788_675_134_594_812_9);
        // F18: the deep tail keeps its digits, one ulp from Excel's 1.230468749943611E-56 (the
        // exact value is 33 ulp away).
        let deep = student_t_tails(1e6, 10.0);
        assert_eq!(deep.1, 1.230_468_749_943_610_7e-56);
        assert_eq!(student_t_tails(-1e6, 10.0).0, deep.1);
        assert_eq!(ulps(deep.1, 1.230_468_749_943_611e-56), 1);
    }

    #[test]
    fn density_cells() {
        // T_DIST.xlsx `T.DIST T.INV T.DIST.RT`: D10 (t = 0, n = 10) is the correctly rounded
        // 945/768/sqrt(10), 3 ulp below Excel's 0.38910838396603115; D3 (t = 8, n = 3).
        assert_eq!(student_t_density(0.0, 10.0), Some(0.389_108_383_966_031_04));
        let d3 = student_t_density(8.0, 3.0).unwrap_or(f64::NAN);
        assert!(ulps(d3, 0.000_736_906_520_946_926_6) <= 4, "{d3}");
        assert_eq!(student_t_density(1.0, 2.5), None);
    }

    #[test]
    fn large_df_cells() {
        // probes-numeric m9-t-edges A11 and m9-t-dist-cdf A139 (excel-graph): a rounded y missed
        // these in the 10th digit; m9-t-dist-rt A140 and m9-t-edges A20 (the density at 1E11).
        let close = |actual: f64, excel: f64| ulps(actual, excel) <= 4;
        let (cdf, _) = student_t_tails(2.0, 1e10);
        assert!(close(cdf, 0.977_249_868_038_323), "{cdf}");
        let (cdf, _) = student_t_tails(1.0, 1e6);
        assert!(close(cdf, 0.841_344_625_083_211_1), "{cdf}");
        let (_, rt) = student_t_tails(2.0, 1e6);
        assert!(close(rt, 0.022_750_266_925_659_606), "{rt}");
        let (_, rt) = student_t_tails(0.1, 1e6);
        assert!(close(rt, 0.460_172_172_746_021_57), "{rt}");
        let density = student_t_density(2.0, 1e11).unwrap_or(f64::NAN);
        assert!(close(density, 0.053_990_966_514_132_89), "{density}");
        let density = student_t_density(0.0, 1e6).unwrap_or(f64::NAN);
        assert!(close(density, 0.398_942_180_665_875_06), "{density}");
    }

    #[test]
    fn ratio_paths_agree() {
        // The asymptotic series and the integer product meet at n = 170.
        let product = integer_ratio(170.0).map(|r| r.0).unwrap_or(f64::NAN);
        let asymptotic = gamma_ratio(170.0).unwrap_or(f64::NAN);
        assert!(ulps(product, asymptotic) <= 1, "{product} {asymptotic}");
    }

    #[test]
    fn extremes_stay_finite() {
        for &n in &[1.0, 2.0, 3.0, 999.0, 1000.0, 1001.0, 1e6, 1e10, 2.5] {
            for &t in &[1e-300, 1e-10, 0.5, 1.0, 30.0, 1e6, 1e150, 1e200, f64::MAX] {
                let (lower, upper) = student_t_tails(t, n);
                assert!((0.0..=1.0).contains(&lower), "{t} {n} {lower}");
                assert!((0.0..=1.0).contains(&upper), "{t} {n} {upper}");
            }
        }
    }
}
