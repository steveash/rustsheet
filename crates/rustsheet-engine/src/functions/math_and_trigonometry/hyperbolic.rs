//! Excel's SINH/COSH/TANH formulation.
//!
//! libm's `sinh`/`cosh`/`tanh` differ from Excel in the last ulp. Fitting against the Excel-cached
//! values in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` (sheet `SINH_COSH_TANH`)
//! shows Excel uses `(e^x ∓ e^-x)/2` (COSH matches 34/34 cells vs 33/34 for libm), with a Taylor
//! series for small |x| in SINH (cells B10, B42; libm misses B42). See
//! `docs/excel-semantics/numbers.md`.

/// Below this |x| the exp formulation loses precision and Excel's values match the Taylor series
/// (evidence: `SINH_COSH_TANH!B10`, B18, B42; the exp form matches all cells with |x| >= 0.5).
const SINH_SERIES_LIMIT: f64 = 0.5;
/// `exp(x)` overflows near here; defer to libm so the finite/infinite boundary is unchanged.
const EXP_SAFE_LIMIT: f64 = 700.0;
/// |tanh(x)| rounds to 1.0 in f64 beyond this.
const TANH_SATURATION: f64 = 20.0;

/// From here up to 1 TANH uses the `e^2x` form; below it, sinh/cosh fits (cells with |x| <= 0.604).
const TANH_E2X_FROM: f64 = 0.9;

fn sinh_series(x: f64) -> f64 {
    let x2 = x * x;
    let mut term = x;
    let mut sum = x;
    for k in 1..16_u32 {
        term = term * x2 / f64::from((2 * k) * (2 * k + 1));
        sum += term;
    }
    sum
}

pub(super) fn sinh(x: f64) -> f64 {
    if x.abs() < SINH_SERIES_LIMIT {
        sinh_series(x)
    } else if x.abs() > EXP_SAFE_LIMIT {
        x.sinh()
    } else {
        (x.exp() - (-x).exp()) / 2.0
    }
}

pub(super) fn cosh(x: f64) -> f64 {
    if x.abs() > EXP_SAFE_LIMIT {
        x.cosh()
    } else {
        (x.exp() + (-x).exp()) / 2.0
    }
}

pub(super) fn tanh(x: f64) -> f64 {
    if x.abs() > TANH_SATURATION {
        // exact in f64 (as the existing COTH shortcut)
        x.signum()
    } else if x.abs() >= TANH_E2X_FROM && x.abs() < 1.0 {
        // Excel's values for 0.9999999999 (SINH_COSH_TANH!D15/D17) and -0.9950546550253772
        // (poi FormulaEvalTestData_Copy.xlsx EverythingTests!G1360) both match this form; neither
        // sinh/cosh nor the exp quotient fits the latter.
        let e2x = (2.0 * x).exp();
        (e2x - 1.0) / (e2x + 1.0)
    } else if x.abs() < 1.0 {
        // the exp quotient loses digits here; Excel's values (TANH 1e-12, 0.5, 0.604) match
        // sinh/cosh with the series SINH
        sinh(x) / cosh(x)
    } else {
        let (a, b) = (x.exp(), (-x).exp());
        (a - b) / (a + b)
    }
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

    #[test]
    fn overflow_defers_to_libm() {
        assert!(sinh(1000.0).is_infinite());
        assert!(cosh(-1000.0).is_infinite());
    }
}
