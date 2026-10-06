#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Pairing rules: docs/excel-semantics/coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing".

#[test]
fn skips_pairs_with_a_non_number() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "text");
    model._set("A3", "3");
    model._set("B1", "2");
    model._set("B2", "5");
    // B3 left blank
    model._set("C1", "=SUMX2MY2(A1:A3,B1:B3)");
    model._set("C2", "=SUMX2PY2(A1:A3,B1:B3)");
    model._set("C3", "=SUMXMY2(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"-3");
    assert_eq!(model._get_text("C2"), *"5");
    assert_eq!(model._get_text("C3"), *"1");
}

#[test]
fn no_numeric_pair_is_div_zero() {
    let mut model = new_empty_model();
    model._set("A1", "text");
    model._set("B1", "1");
    model._set("C1", "=SUMX2MY2(A1:A2,B1:B2)");
    model._set("C2", "=SUMXMY2(D1:D2,E1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#DIV/0!");
    assert_eq!(model._get_text("C2"), *"#DIV/0!");
}

#[test]
fn different_counts_are_na_same_count_different_shape_pairs_row_major() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "1");
    model._set("D2", "1");
    model._set("D3", "1");
    model._set("D4", "1");
    model._set("F1", "=SUMXMY2(A1:A2,D1:D3)");
    // (1,1) (2,1) (3,1) (4,1): 0 + 1 + 4 + 9
    model._set("F2", "=SUMXMY2(A1:B2,D1:D4)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), *"#N/A");
    assert_eq!(model._get_text("F2"), *"14");
}

#[test]
fn scalar_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMXMY2(2,4)");
    model._set("A2", "=SUMX2PY2(2,3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_text("A2"), *"13");
}
