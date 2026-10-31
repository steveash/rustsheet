//! Excel's SINH/COSH/TANH formulation.
//!
//! libm's `sinh`/`cosh`/`tanh` differ from Excel in the last ulp. Fitting against the Excel-cached
//! values in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` (sheet `SINH_COSH_TANH`)
//! shows Excel uses `(e^x ∓ e^-x)/2` (COSH matches 34/34 cells vs 33/34 for libm). Below |x| = 1,
//! SINH and TANH come from `a = expm1(x)` and `b = expm1(-x)` instead (T-2134, probes-numeric
//! t2178-hyperbolic-unit-sweep). See `docs/excel-semantics/numbers.md`.
//!
//! ATANH and ACOTH are x87 code (T-0917, numbers.md "ATANH and ACOTH"): a halved FYL2X of a
//! quotient (`0.5 * excel_ln`, exact: halving commutes with both roundings) or the atanh power series with its running term in an 80-bit
//! register ([`atanh_series`]), each branch on its own range.

use crate::arithmetic::{
    extended_div, extended_mul, extended_of, extended_quotient, extended_sum, store_extended,
    Extended,
};
use crate::x87_exp::excel_expm1;
use crate::x87_log10::excel_ln;

/// Below this |x| SINH and TANH are built from `expm1` ([`expm1_pair`]); from it on, from `exp`.
const EXPM1_FORM_LIMIT: f64 = 1.0;
/// |tanh(x)| rounds to 1.0 in f64 beyond this.
const TANH_SATURATION: f64 = 20.0;

/// `(expm1(x), expm1(-x))`, the two values Excel's SINH and TANH share below
/// [`EXPM1_FORM_LIMIT`] (T-2134, docs/excel-semantics/numbers.md "SINH and TANH below 1 are built
/// from expm1").
fn expm1_pair(x: f64) -> (f64, f64) {
    (excel_expm1(x), excel_expm1(-x))
}

/// Overflows to infinity where `e^|x|` does (just above 709.78; Excel's exact cut is unprobed,
/// T-1944): SINH/COSH report that as `#NUM!` and SECH/CSCH
/// as 0 (numbers.md "SINH and COSH overflow is `#NUM!` at once").
pub(super) fn sinh(x: f64) -> f64 {
    if x.abs() < EXPM1_FORM_LIMIT {
        // (a - b)/2 is exact in its halving; a - b is the one rounding.
        let (a, b) = expm1_pair(x);
        (a - b) / 2.0
    } else {
        (x.exp() - (-x).exp()) / 2.0
    }
}

pub(super) fn cosh(x: f64) -> f64 {
    (x.exp() + (-x).exp()) / 2.0
}

pub(super) fn tanh(x: f64) -> f64 {
    if x.abs() > TANH_SATURATION {
        // exact in f64 (as the existing COTH shortcut)
        x.signum()
    } else if x.abs() < EXPM1_FORM_LIMIT {
        // sinh/cosh as (a - b)/((a + b) + 2), summed in that order: (a - b)/(2 + a + b) fits 644
        // of the sweep's 1015 cells below 1, this order 781.
        let (a, b) = expm1_pair(x);
        (a - b) / (a + b + 2.0)
    } else {
        let (a, b) = (x.exp(), (-x).exp());
        (a - b) / (a + b)
    }
}

/// ATANH takes the series at or below this |x| and the log above it (probes-numeric
/// t0917-hyperbolic-switchover A58-A78: the series through 1E-4, the log from 1.1E-4 on).
/// FITTED(T-0917, 21 cells, |x| 6E-5..4.5E-4; cut bracketed 1E-4..1.1E-4)
const ATANH_SERIES_TO: f64 = 1E-4;

/// ACOTH takes the log below this |x| and the series in 1/x from it on: probes-numeric
/// t0917-acoth-switch-edge has the log at 3.69662 and the series at 3.69668.
/// FITTED(T-0917, 303 cells, |x| 3.69..300; cut bracketed 3.69662..3.69668)
const ACOTH_SERIES_FROM: f64 = 3.69665;

/// The series stops once a term no longer changes the sum; for |y| <= 0.2706 each term is under
/// 0.074 of the one before, so it stops after about 15 terms. This only bounds the loop.
const SERIES_TERMS: u32 = 64;

/// Excel's atanh series `y + y^3/3 + y^5/5 + ...` for a positive register value `y`, as x87 code
/// runs it: the power `y^(2k+1)` and `y^2` stay in 80-bit registers, each term `power/(2k+1)` and
/// each partial sum is stored to a double (every sum rounded twice), and the loop ends when a term
/// leaves the sum unchanged. Keeping `y` and the power in a register is what fits ACOTH(4.7),
/// ACOTH(4.92) and ACOTH(3.69734) (probes-numeric t0917-acoth-switch-edge); an all-double series
/// misses them. `None` when `y` is not a normal double.
fn atanh_series(y: Extended) -> Option<f64> {
    let mut sum = store_extended(y)?;
    let square = extended_mul(y, y);
    let mut power = y;
    for k in 1..=SERIES_TERMS {
        power = extended_mul(power, square);
        let term = extended_div(power, extended_of(f64::from(2 * k + 1)));
        // A term under the normal range cannot change a normal sum.
        let next = extended_sum(sum, store_extended(term).unwrap_or(0.0));
        if next == sum {
            break;
        }
        sum = next;
    }
    Some(sum)
}

/// Excel's ATANH(x) for |x| < 1: the series at or below 1E-4, else `0.5*ln((1+x)/(1-x))` with the
/// sums and the quotient rounded twice and the x87 half log. A negative `x` is evaluated as it is,
/// not as `-ATANH(-x)`: ATANH(-0.0002) and ATANH(-0.0003) need that (probes-numeric
/// t0917-hyperbolic-switchover A77, A78). The log branch keeps the cancellation of `1+x` for small
/// |x|: ATANH(0.0005) is 327 ulp below the true value (probes-numeric m9-hyperbolic A96).
pub(super) fn atanh(x: f64) -> f64 {
    if x.abs() <= ATANH_SERIES_TO {
        // The series is odd and every rounding is symmetric, so the sign comes back exactly.
        atanh_series(extended_of(x)).map_or(x, |value| value.copysign(x))
    } else {
        0.5 * excel_ln(extended_quotient(
            extended_sum(1.0, x),
            extended_sum(1.0, -x),
        ))
    }
}

/// Excel's ACOTH(x) for |x| > 1, odd: `0.5*ln((|x|+1)/(|x|-1))` (sums and quotient rounded twice,
/// x87 half log) below 3.69665, else the atanh series in `1/|x|` with the reciprocal kept in a
/// register. Unlike ATANH the sign comes last: ACOTH(-3.5) is -ACOTH(3.5), not the log of
/// `(-2.5)/(-4.5)` (probes-numeric t0917-hyperbolic-switchover A44, A45).
pub(super) fn acoth(x: f64) -> f64 {
    let magnitude = x.abs();
    let value = if magnitude < ACOTH_SERIES_FROM {
        0.5 * excel_ln(extended_quotient(
            extended_sum(magnitude, 1.0),
            extended_sum(magnitude, -1.0),
        ))
    } else {
        // Past about 4.5E307 the reciprocal is subnormal; no Excel value is known there.
        atanh_series(extended_div(extended_of(1.0), extended_of(magnitude)))
            .unwrap_or_else(|| extended_quotient(1.0, magnitude))
    };
    value.copysign(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values are Excel-cached (TRIGONOMETRIC.xlsx, sheet SINH_COSH_TANH).
    #[test]
    fn matches_excel_cached_values() {
        assert_eq!(sinh(0.006_249_999_999_999_999), 0.006_250_040_690_183_638); // B42
        assert_eq!(sinh(1e-12), 1e-12); // B18
        assert_eq!(tanh(1e-12), 1e-12); // D18
        assert_eq!(tanh(0.5), 0.462_117_157_260_009_8_f64); // D11 sign-flipped
                                                            // poi FormulaEvalTestData_Copy.xlsx EverythingTests!G1360; SINH_COSH_TANH!D15
        assert_eq!(tanh(-0.995_054_655_025_377_2), -0.759_509_403_050_760_8);
        assert_eq!(tanh(0.999_999_999_9), 0.761_594_155_913_767_4);
        assert_eq!(tanh(100.0), 1.0);
        assert_eq!(tanh(-100.0), -1.0);
    }

    // Below 1, (a - b)/2 and (a - b)/((a + b) + 2) with a = expm1(x), b = expm1(-x)
    // (probes-numeric t2178-hyperbolic-unit-sweep, gen-m5, power-ulp-sources; excel-graph): the
    // series SINH and sinh/cosh TANH missed each of these by an ulp.
    #[test]
    fn below_one_is_the_expm1_form() {
        assert_eq!(sinh(0.3), 0.304_520_293_447_142_6);
        assert_eq!(tanh(0.2), 0.197_375_320_224_904_03);
        assert_eq!(tanh(0.4), 0.379_948_962_255_224_9);
        assert_eq!(sinh(-0.3), -0.304_520_293_447_142_6);
    }

    // Kahan's expm1 over Excel's EXP and LN (T-2240): libm's expm1 put each of these an ulp off.
    // TRIGONOMETRIC.xlsx SINH_COSH_TANH!B51/D51 (PI()/4) and B15 (0.9999999999), Excel-cached.
    #[test]
    fn expm1_is_kahans_formula() {
        let quarter_pi = std::f64::consts::FRAC_PI_4;
        assert_eq!(sinh(quarter_pi), 0.868_670_961_486_009_6);
        assert_eq!(tanh(quarter_pi), 0.655_794_202_632_672_4);
        assert_eq!(sinh(0.999_999_999_9), 1.175_201_193_489_493_5);
        assert_eq!(excel_expm1(1e-20), 1e-20);
    }

    #[test]
    fn overflow_defers_to_libm() {
        assert!(sinh(1000.0).is_infinite());
        assert!(cosh(-1000.0).is_infinite());
    }

    // probes-numeric m9-hyperbolic and t0917-* (excel-graph): one cell per branch and quirk.
    #[test]
    fn atanh_matches_excel() {
        assert_eq!(atanh(5e-5), 5.000_000_004_166_667e-5); // series (m9-hyperbolic A95)
        assert_eq!(atanh(1e-4), 0.000_100_000_000_333_333_34); // series edge (switchover A62)
        assert_eq!(atanh(0.0005), 0.000_500_000_041_666_637_5); // log, 327 ulp low (A96)
        assert_eq!(atanh(0.005), 0.005_000_041_667_291_589); // A97
        assert_eq!(atanh(-0.5), -0.549_306_144_334_054_9); // TRIGONOMETRIC.xlsx D11
        assert_eq!(atanh(0.0), 0.0);
    }

    #[test]
    fn acoth_matches_excel() {
        assert_eq!(acoth(1.0001), 4.951_768_775_643_139); // x87 FYL2X rounding (m9-hyperbolic A54)
        assert_eq!(acoth(1.9), 0.585_035_626_325_127_3); // stored, not RN64, quotient
        assert_eq!(acoth(3.5), 0.293_893_332_451_059_53); // switchover A44
        assert_eq!(acoth(-3.5), -0.293_893_332_451_059_53); // odd (A45)
        assert_eq!(acoth(4.74), 0.214_186_799_452_517_17); // series, 2 ulp low (switchover A38)
        assert_eq!(acoth(4.7), 0.216_066_677_595_162_85); // series with y in a register
        assert_eq!(acoth(3.69734), 0.277_365_144_421_817_53);
        assert_eq!(acoth(1.5e10), 6.666_666_666_666_667e-11); // m9-hyperbolic A110
    }
}
