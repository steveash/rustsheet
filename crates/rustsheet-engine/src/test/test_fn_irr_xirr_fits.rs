#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! IRR and XIRR as fitted against the excel-graph solver traces (T-0916; fit lab `irr`, `xirr`;
//! docs/excel-semantics/financial.md "IRR is a secant in v" and "XIRR is a bracketing bisection").
//! Each value is the oracle's; the guess changes Excel's last digits, so each one pins the path.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `formulas` in A1, A2, ... of Sheet1, evaluated.
fn evaluated(formulas: &[&str]) -> Model {
    let mut model = new_empty_model();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    model
}

fn results(formulas: &[&str]) -> Vec<u64> {
    let model = evaluated(formulas);
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

#[test]
fn xirr_with_a_zero_first_value_needs_no_later_date() {
    // probes/xirr-span-zero-first A28 (first date after the others); T-2329
    let got = results(&["=XIRR({0,-1000,1100},{46000,45000,45365})"]);
    let expected = [2.980_232_238_769_531_4e-9_f64];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_with_a_zero_first_value_brackets_at_the_first_chain_point() {
    // probes/xirr-zero-first-bracket A1, A3, A8, A9, A10: the flows do not matter, the guess does
    let got = results(&[
        "=XIRR({0,-1000,2000},{45000,45000,45365})",
        "=XIRR({0,-1000,2000},{45000,45000,45365},0.3)",
        "=XIRR({0,-100,110},{45000,45000,45000})",
        "=XIRR({0,-100,110},{45000,45000,45000},0)",
        "=XIRR({0,-100,110},{45000,45000,45000},2)",
    ]);
    let expected = [
        2.980_232_238_769_531_4e-9_f64,
        4.470_348_358_154_297e-9,
        2.980_232_238_769_531_4e-9,
        4.882_812_5e-9,
        3.725_290_298_461_914e-9,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

fn assert_num_errors(formulas: &[&str]) {
    let model = evaluated(formulas);
    for row in 1..=formulas.len() {
        assert_eq!(
            model.get_cell_value_by_ref(&format!("Sheet1!A{row}")),
            Ok(CellValue::String("#NUM!".to_string())),
            "A{row}"
        );
    }
}

#[test]
fn xirr_with_a_zero_first_value_and_a_negative_guess_is_num() {
    // probes/xirr-zero-first-bracket A6, A7, A11 (A6 has a root on the downward chain)
    assert_num_errors(&[
        "=XIRR({0,-1000,900},{45000,45000,45365},-0.2)",
        "=XIRR({0,-1000,1100},{45000,45000,45365},-0.5)",
        "=XIRR({0,-100,110},{45000,45000,45000},-0.5)",
    ]);
}

#[test]
fn xirr_with_a_nonzero_first_value_still_needs_a_later_date() {
    // probes/xirr-mismatched-empties X9
    assert_num_errors(&["=XIRR({-1000,1100},{45365,45000})"]);
}

#[test]
fn irr_is_num_at_a_far_root() {
    // probes/irr-fallback-roots A29, A39 and irr-guess-edges-2 A15, A31: 1 + r at or past 1E+7
    assert_num_errors(&["=IRR({-1,1E+7})", "=IRR({-1,1E+13})", "=IRR({-1E-15,1})"]);
    // A28, A39: 3E+6 and 1 + r = 9999998.994 still return
    let model = evaluated(&["=IRR({-1,3E+6})", "=IRR({-1,0,1E+14})"]);
    for row in 1..=2 {
        let value = model.get_cell_value_by_ref(&format!("Sheet1!A{row}"));
        assert!(
            matches!(value, Ok(CellValue::Number(n)) if n > 2e6),
            "A{row}: {value:?}"
        );
    }
}

#[test]
fn irr_where_the_secant_fails_is_num_at_a_root_near_minus_one() {
    // probes/irr-fallback-roots A1, A5, A8, A10: the NPV polynomial's root nearest v = 0 is negative
    assert_num_errors(&[
        "=IRR({1,7,-1})",
        "=IRR({1,7,-1},1)",
        "=IRR({1,20,-1})",
        "=IRR({1,3,-1})",
    ]);
}

#[test]
fn irr_where_the_secant_fails_finds_the_root_nearest_v_zero() {
    // probes/irr-fallback-roots A12, A15, A16, A22: within a few ulp of Excel's
    let model = evaluated(&[
        "=IRR({42,1,1,-1},0.5)",
        "=IRR({42,1,1,-1},0)",
        "=IRR({0.3,-2.5,1.005},0.01)",
        "=IRR({-400,1,1,1,1,1},0.5)",
    ]);
    let expected = [
        -0.746_704_476_653_737_1,
        -0.746_704_476_653_737_1,
        6.909_808_543_634_611,
        -0.673_736_675_180_840_5,
    ];
    for (row, want) in (1..).zip(expected) {
        let value = model.get_cell_value_by_ref(&format!("Sheet1!A{row}"));
        assert!(
            matches!(value, Ok(CellValue::Number(n)) if (n - want).abs() < 1e-14),
            "A{row}: {value:?}"
        );
    }
}
