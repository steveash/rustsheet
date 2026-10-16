#![allow(clippy::unwrap_used, clippy::panic)]

//! PV, FV and NPER as fitted against the excel-graph sweeps (T-0901, fit lab `annuity-*`,
//! docs/excel-semantics/financial.md "Annuity evaluation order"). Each value below is the oracle's,
//! and each one the earlier `powf` formulations missed by an ulp or more.

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
fn pv_and_fv_use_the_fitted_annuity_factor() {
    // probes-numeric/m9-fin-pv-fv A1, A10, A100, A103
    let got = results(&[
        "=PV(0.2,1,-100)",
        "=PV(0.2,3,-100,1000,1)",
        "=FV(0.1,36,-100,-1000,1)",
        "=FV(0.1,60,-100)",
    ]);
    let expected = [
        83.333_333_333_333_31_f64,
        -325.925_925_925_926,
        63_816.629_119_028_57,
        303_481.639_541_419_34,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn nper_divides_plain_logarithms_with_excels_division() {
    // probes-numeric/m9-fin-nper-npv A9: the final `/` rounds twice
    let got = results(&["=NPER(0.2,-1000,-1000,20000)"]);
    assert_eq!(got, [7.827_469_119_589_407_5_f64.to_bits()]);
}
