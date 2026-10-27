#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_sumproduct_basic() {
    let mut model = new_empty_model();
    // SUMPRODUCT({1,2,3}, {4,5,6}) = 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "4");
    model._set("B2", "5");
    model._set("B3", "6");
    model._set("C1", "=SUMPRODUCT(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "32");
}

#[test]
fn test_sumproduct_single_array() {
    let mut model = new_empty_model();
    // SUMPRODUCT({1,2,3}) = 1+2+3 = 6
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=SUMPRODUCT(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "6");
}

#[test]
fn test_sumproduct_three_arrays() {
    let mut model = new_empty_model();
    // SUMPRODUCT({1,2}, {3,4}, {5,6}) = 1*3*5 + 2*4*6 = 15 + 48 = 63
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "3");
    model._set("B2", "4");
    model._set("C1", "5");
    model._set("C2", "6");
    model._set("D1", "=SUMPRODUCT(A1:A2,B1:B2,C1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "63");
}

#[test]
fn test_sumproduct_dimension_mismatch() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "4");
    model._set("B2", "5");
    model._set("C1", "=SUMPRODUCT(A1:A3,B1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#VALUE!");
}

#[test]
fn test_sumproduct_ignores_text() {
    let mut model = new_empty_model();
    // Text in one array is treated as 0
    model._set("A1", "2");
    model._set("A2", "hello");
    model._set("B1", "3");
    model._set("B2", "4");
    model._set("C1", "=SUMPRODUCT(A1:A2,B1:B2)");
    model.evaluate();
    // 2*3 + 0*4 = 6
    assert_eq!(model._get_text("C1"), "6");
}

#[test]
fn test_sumproduct_2d() {
    let mut model = new_empty_model();
    // 2x2 arrays
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "5");
    model._set("D1", "6");
    model._set("C2", "7");
    model._set("D2", "8");
    model._set("E1", "=SUMPRODUCT(A1:B2,C1:D2)");
    model.evaluate();
    // 1*5 + 2*6 + 3*7 + 4*8 = 5+12+21+32 = 70
    assert_eq!(model._get_text("E1"), "70");
}

#[test]
fn test_sumproduct_blank_one_cell_argument() {
    // A blank one-cell reference is #VALUE!; a one-cell text or boolean is a 0 factor
    // (probes/t2200-blank-and-padded-m5-args A4-A16; beside a typed value, gen-m5
    // 1b30e4d075b252ea A365).
    let mut model = new_empty_model();
    model._set("A2", "x");
    model._set("A3", "TRUE");
    model._set("A4", "7");
    model._set("C1", "=SUMPRODUCT(A1,A4)");
    model._set("C2", "=SUMPRODUCT(A4:A4,A1:A1)");
    model._set("C3", "=SUMPRODUCT(A1)");
    model._set("C4", "=SUMPRODUCT(A2,A4)");
    model._set("C5", "=SUMPRODUCT(A3)");
    model._set("C6", "=SUMPRODUCT(A1,7)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#VALUE!");
    assert_eq!(model._get_text("C2"), "#VALUE!");
    assert_eq!(model._get_text("C3"), "#VALUE!");
    assert_eq!(model._get_text("C4"), "0");
    assert_eq!(model._get_text("C5"), "0");
    assert_eq!(model._get_text("C6"), "#VALUE!");
}

#[test]
fn test_sumproduct_first_error_beats_later_shape_mismatch() {
    // Every argument is read before shapes are compared: the first error or blank wins
    // (probes/t2200-blank-and-padded-m5-args A20-A23).
    let mut model = new_empty_model();
    model._set("D1", "1");
    model._set("E1", "2");
    model._set("D2", "3");
    model._set("E2", "4");
    model._set("C1", "=SUMPRODUCT(1,D1:E2,NA())");
    model._set("C2", "=SUMPRODUCT(NA(),D1:E2,A1)");
    model._set("C3", "=SUMPRODUCT(A1,D1:E2,NA())");
    model._set("C4", "=SUMPRODUCT(D1:D2,D1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#N/A");
    assert_eq!(model._get_text("C2"), "#N/A");
    assert_eq!(model._get_text("C3"), "#VALUE!");
    assert_eq!(model._get_text("C4"), "#VALUE!");
}

#[test]
fn test_sumproduct_error_element_after_shape_mismatch() {
    // An error element beats a shape mismatch in either argument order (probes/
    // t2296-sumproduct-error-element, T-2450).
    let mut model = new_empty_model();
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("E1", "=1/0");
    model._set("C1", "=SUMPRODUCT(D1:D2,D1:E2)");
    model._set("C2", "=SUMPRODUCT(D1:E2,D1:D2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#DIV/0!");
    assert_eq!(model._get_text("C2"), "#DIV/0!");
}

#[test]
fn test_sumproduct_one_cell_numeric_text_is_a_zero_factor() {
    // Excel: a one-cell reference to numeric text is a 0 factor beside a one-cell number, in
    // either order, like non-numeric text (probes/t2921-sumproduct-one-cell-numeric-text A1-A3,
    // A45, excel-graph). Excel 2013 cached #VALUE! for poi FormulaEvalTestData_Copy J1328
    // (CONFLICTING, docs/excel-semantics/coercion.md).
    let mut model = new_empty_model();
    model._set("A1", "'4");
    model._set("B1", "2");
    model._set("C1", "=SUMPRODUCT(A1,B1)");
    model._set("C2", "=SUMPRODUCT(B1,A1)");
    model._set("C3", "=SUMPRODUCT(A1:A1,B1:B1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "0");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_text("C3"), "0");
}
