//! Excel's own ln(1 + a) for -0.375 <= a < 0, the log1p behind BINOM.DIST(0,n,p) and the beta
//! density (T-2464 r2-log1p; docs/excel-semantics/statistics.md "Excel's log1p is a 14-term atanh
//! series in alnrel's variable"; reference code `docs/research/t2464/r2-log1p/excel_log1p.py`).
//!
//! It is the atanh series in TOMS 708 alnrel's variable r = a/(a + 2), summed forward, every
//! operation an x87 operation stored to a double (rounded to 64 bits, then to 53):
//! ```text
//! r  = a / (a + 2)          t2 = r * r          p = r
//! k = 1..14:  p = p * t2;   s = s + p * c_k     c_k = RN(1/(2k+1)), a double
//! l  = 2 * (r + s)
//! ```
//! Above x = 0.375 (a < -0.375) Excel's ln(1 - x) is LN(fl(1 - x)), TOMS brcomp's switch.
//! The model gives Excel's bits on 15,764 of the 15,791 rows r2-log1p pinned from Excel's
//! BINOM.DIST(0,n,p) cells (probes-numeric t2464-r2-log1p-pilot, -runs, -grid, -hold); every
//! miss is exactly 1 ulp off, a near-tie of r + s that Excel rounds the other way. A correctly
//! rounded log1p gets 10,821.

use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_log10::excel_ln;

/// The series' coefficients, `c_k = RN(1/(2k+1))` for k = 1..=14 as doubles (Excel's first one is
/// `RN(1/3)` exactly, r2-log1p `coef_scan.py`; 12 terms and more give the same doubles on every
/// pinned row).
const COEFFICIENTS: [f64; 14] = {
    let mut coefficients = [0.0; 14];
    let (mut k, mut odd) = (0, 3.0);
    while k < 14 {
        coefficients[k] = 1.0 / odd;
        k += 1;
        odd += 2.0;
    }
    coefficients
};

/// The largest x for which Excel's ln(1 - x) takes the series (at 0.375 both forms give the
/// same double; the next double up already takes LN).
const SERIES_LIMIT: f64 = 0.375;

/// Excel's ln(1 + a) by the stored atanh series (module docs). It is Excel's for -0.375 <= a < 0
/// (a = 0 gives 0); outside that range the series is still evaluated, but Excel's positive-argument
/// alnrel is a different function (r2-log1p `algdiv_try.py`, T-3468) and below -0.375 Excel takes
/// [`excel_ln_one_minus`]'s LN. A NaN gives NaN.
pub(crate) fn excel_log1p(a: f64) -> f64 {
    let r = extended_quotient(a, a + 2.0);
    let t2 = extended_product(r, r);
    let mut power = r;
    let mut sum = 0.0;
    for (k, &coefficient) in COEFFICIENTS.iter().enumerate() {
        power = extended_product(power, t2);
        let term = extended_product(power, coefficient);
        sum = if k == 0 {
            term
        } else {
            extended_sum(sum, term)
        };
    }
    2.0 * extended_sum(r, sum)
}

/// Excel's ln(1 - x) for 0 < x < 1 as BINOM.DIST(0,n,p) and the beta density use it: the series
/// [`excel_log1p`] of -x for x <= 0.375, and Excel's LN of the double 1 - x above.
pub(crate) fn excel_ln_one_minus(x: f64) -> f64 {
    if x <= SERIES_LIMIT {
        excel_log1p(-x)
    } else {
        excel_ln(1.0 - x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::kernel_rows::fixture_rows;

    /// The committed sample (`test/fixtures/log1p_kernel.fixture`): 5,211 pinned rows with the
    /// research model's value, every one of the 27 rows where Excel differs from the model among
    /// them. The engine is the model (`excel_log1p.ln1m`) on every row, and Excel on all but those
    /// 27, each 1 ulp away.
    #[test]
    fn equals_the_research_model_on_the_fixture() {
        let rows = fixture_rows(include_str!("test/fixtures/log1p_kernel.fixture"));
        assert_eq!(rows.len(), 5211);
        let mut series_rows = 0;
        let mut excel_misses = 0;
        for row in &rows {
            let ours = excel_ln_one_minus(row.x);
            assert_eq!(
                ours.to_bits(),
                row.model.to_bits(),
                "ln(1 - {:e}) vs the model",
                row.x
            );
            if row.x <= SERIES_LIMIT {
                series_rows += 1;
            }
            if ours.to_bits() != row.excel.to_bits() {
                excel_misses += 1;
                assert_eq!(
                    ours.to_bits().abs_diff(row.excel.to_bits()),
                    1,
                    "{:e}",
                    row.x
                );
            }
        }
        assert!(series_rows >= 5000, "{series_rows}");
        assert_eq!(excel_misses, 27);
    }

    /// Excel's own l for a few p (r2-log1p pilot rows; p = 0.3 is w5's readout, one ulp below the
    /// correctly rounded -0.35667494393873245), both sides of the 0.375 switch, and the ends.
    #[test]
    fn pinned_cells() {
        assert_eq!(excel_ln_one_minus(0.1), -0.105_360_515_657_826_31);
        assert_eq!(excel_ln_one_minus(0.2), -0.223_143_551_314_209_76);
        assert_eq!(excel_ln_one_minus(0.3), -0.356_674_943_938_732_4);
        assert_eq!(
            excel_ln_one_minus(0.375),
            excel_ln(1.0 - 0.375),
            "the two forms meet at 0.375"
        );
        let above = f64::from_bits(0.375_f64.to_bits() + 1);
        assert_eq!(excel_ln_one_minus(above), excel_ln(1.0 - above));
        assert_eq!(excel_log1p(0.0), 0.0);
        assert_eq!(excel_log1p(-1e-300), -1e-300);
        assert!(excel_log1p(f64::NAN).is_nan());
        assert_eq!(COEFFICIENTS[0], 1.0 / 3.0);
        assert_eq!(COEFFICIENTS[13], 1.0 / 29.0);
    }

    /// Every finite pinned row of r2-log1p's pilot, runs, grid and hold cases, when
    /// `RUSTSHEET_T2464_LOG1P` names the directory holding their `*.pins.tsv` (p, n, offset, l):
    /// Excel's bits on 15,764 of the 15,791, every miss 1 ulp off (verify-r2-log1p/vscore.out).
    #[test]
    #[ignore = "needs the T-2464 research data (RUSTSHEET_T2464_LOG1P)"]
    fn matches_every_pinned_row() {
        let Ok(directory) = std::env::var("RUSTSHEET_T2464_LOG1P") else {
            return;
        };
        let (mut rows, mut exact) = (0, 0);
        for case in ["pilot", "runs", "grid", "hold"] {
            let text = std::fs::read_to_string(format!("{directory}/{case}.pins.tsv")).unwrap();
            for line in text.lines() {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields[2] == "none" {
                    continue;
                }
                let (p, l): (f64, f64) = (fields[0].parse().unwrap(), fields[3].parse().unwrap());
                if !l.is_finite() {
                    continue;
                }
                rows += 1;
                let ours = excel_ln_one_minus(p);
                if ours.to_bits() == l.to_bits() {
                    exact += 1;
                } else {
                    assert_eq!(ours.to_bits().abs_diff(l.to_bits()), 1, "p = {p:e}");
                }
            }
        }
        assert_eq!((exact, rows), (15_764, 15_791));
    }
}
