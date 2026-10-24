#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_minverse_2x2() {
    let mut model = new_empty_model();
    // [1,2;3,4]^-1 = [-2, 1; 1.5, -0.5]
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "=MINVERSE(A1:B2)");
    model.evaluate();
    // Top-left: -2
    assert_eq!(model._get_text("C1"), "-2");
    // Top-right: 1
    assert_eq!(model._get_text("D1"), "1");
    // Bottom-left: 1.5
    assert_eq!(model._get_text("C2"), "1.5");
    // Bottom-right: -0.5
    assert_eq!(model._get_text("D2"), "-0.5");
}

#[test]
fn test_minverse_identity() {
    let mut model = new_empty_model();
    // Inverse of identity is identity
    model._set("A1", "1");
    model._set("B1", "0");
    model._set("A2", "0");
    model._set("B2", "1");
    model._set("C1", "=MINVERSE(A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("D1"), "0");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_text("D2"), "1");
}

#[test]
fn test_minverse_singular_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "2");
    model._set("B2", "4");
    model._set("C1", "=MINVERSE(A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#NUM!");
}

#[test]
fn test_minverse_non_square_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    model._set("D1", "=MINVERSE(A1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#VALUE!");
}

#[test]
fn test_minverse_blank_cell_is_value_error() {
    let mut model = new_empty_model();
    // A1 = 1, the rest of A1:B2 blank (poi MatrixFormulaEvalTestData H34:J36)
    model._set("A1", "1");
    model._set("D1", "=MINVERSE(A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#VALUE!");
}

#[test]
fn test_minverse_tiny_pivot_is_inverted() {
    let mut model = new_empty_model();
    // Only an exactly-zero pivot is singular (probes/minverse-singular-near-singular, T-2362).
    model._set("A1", "=INDEX(MINVERSE({1,2,3;4,5,6;7,8,9}),3,3)");
    model._set("A2", "=MINVERSE(1E-300)");
    model._set("A3", "=MINVERSE(1E-300*1E-10)");
    model.evaluate();
    assert_eq!(
        model.get_cell_value_by_index(0, 1, 1),
        Ok(crate::cell::CellValue::Number(-4_503_599_627_370_495.5))
    );
    assert_eq!(
        model.get_cell_value_by_index(0, 2, 1),
        Ok(crate::cell::CellValue::Number(1.0 / 1E-300))
    );
    // A reciprocal past the double range is #NUM! (unprobed).
    assert_eq!(model._get_text("A3"), "#NUM!");
}
