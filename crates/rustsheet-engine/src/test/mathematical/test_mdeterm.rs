#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn test_mdeterm_2x2() {
    let mut model = new_empty_model();
    // [1, 2; 3, 4] => det = 1*4 - 2*3 = -2
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "=MDETERM(A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "-2");
}

#[test]
fn test_mdeterm_3x3() {
    let mut model = new_empty_model();
    // identity matrix => det = 1
    model._set("A1", "1");
    model._set("B1", "0");
    model._set("C1", "0");
    model._set("A2", "0");
    model._set("B2", "1");
    model._set("C2", "0");
    model._set("A3", "0");
    model._set("B3", "0");
    model._set("C3", "1");
    model._set("D1", "=MDETERM(A1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
}

#[test]
fn test_mdeterm_singular() {
    let mut model = new_empty_model();
    // Singular matrix (row 2 = 2 * row 1) => det = 0
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "2");
    model._set("B2", "4");
    model._set("C1", "=MDETERM(A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "0");
}

#[test]
fn test_mdeterm_non_square_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    model._set("D1", "=MDETERM(A1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#VALUE!");
}

#[test]
fn test_mdeterm_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=MDETERM()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    model._set("B1", "=MDETERM(1,2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#ERROR!");
}

#[test]
fn test_mdeterm_blank_cell_is_value_error() {
    let mut model = new_empty_model();
    // A1 = 1, the rest of A1:B2 blank (poi MatrixFormulaEvalTestData H30)
    model._set("A1", "1");
    model._set("D1", "=MDETERM(A1:B2)");
    model._set("D2", "=MDETERM(A1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#VALUE!");
    assert_eq!(model._get_text("D2"), "1");
}

#[test]
fn mdeterm_keeps_the_elimination_residual() {
    // probes/stats-evidence-floor-t1623 S!Q107, S!Q120, S!Q121 (excel-graph, T-1751): a tiny last
    // pivot is not rounded to 0; only an exactly-zero pivot column is.
    let mut model = new_empty_model();
    model._set("A1", "=MDETERM({1,2,3;4,5,6;7,8,9})");
    model._set("A2", "=MDETERM({1,2,3;4,5,6;7,8,10})");
    model._set("A3", "=MDETERM({1,2;2,4})");
    model._set("A4", "=MINVERSE({1,2,3;4,5,6;7,8,9})");
    model._set("A5", "=MDETERM({1E+200,0;0,1E+200})");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 6.661_338_147_750_939e-16_f64.to_bits());
    assert_eq!(bits("A2"), (-2.999_999_999_999_998_2_f64).to_bits());
    assert_eq!(bits("A3"), 0.0_f64.to_bits());
    // MINVERSE keeps its near-singular cutoff (no Excel evidence there yet).
    assert_eq!(model._get_text("A4"), *"#NUM!");
    // An overflowing pivot product is `#NUM!`, not an infinity (no Excel evidence; T-1751).
    assert_eq!(model._get_text("A5"), *"#NUM!");
}
