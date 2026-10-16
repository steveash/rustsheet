#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! IRR and XIRR as fitted against the excel-graph solver traces (T-0916; fit lab `irr`, `xirr`;
//! docs/excel-semantics/financial.md "IRR is a secant in v" and "XIRR is a bracketing bisection").
//! Each value is the oracle's; the guess changes Excel's last digits, so each one pins the path.

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn results(formulas: &[&str]) -> Vec<u64> {
    let mut model = new_empty_model();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(
            |row| match model.get_cell_value_by_ref(&format!("Sheet1!A{row}")) {
                Ok(CellValue::Number(n)) => n.to_bits(),
                other => panic!("A{row}: expected a number, got {other:?}"),
            },
        )
        .collect()
}

#[test]
fn irr_is_a_secant_in_the_discount_factor() {
    // probes/solver-iteration-traces I!A1, I!A10
    let got = results(&["=IRR({-20,5,5,5,5,5},0.05)", "=IRR({-20,5,5,5,5,5},0.0725)"]);
    let expected = [0.079_308_261_158_238_75_f64, 0.079_308_261_160_528_69];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_bisects_the_walked_bracket() {
    // probes/solver-iteration-traces X!A1, X!A10
    let got = results(&[
        "=XIRR({-1,1.013},{45000,45365},1)",
        "=XIRR({-1,1.3613},{45000,45365},1)",
    ]);
    let expected = [0.013_000_000_268_220_901_f64, 0.361_299_999_058_246_6];
    assert_eq!(got, expected.map(f64::to_bits));
}
