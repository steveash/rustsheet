#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_covariance_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "9");
    model._set("A3", "2");
    model._set("A4", "7");
    model._set("A5", "4");
    model._set("A6", "12");

    model._set("B1", "5");
    model._set("B2", "15");
    model._set("B3", "6");
    model._set("B4", "17");
    model._set("B5", "8");
    model._set("B6", "20");

    model._set("C1", "=COVARIANCE.P(A1:A6, B1:B6)");
    model._set("C2", "=COVARIANCE.S(A1:A6, B1:B6)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"19.19444444");
    assert_eq!(model._get_text("C2"), *"23.03333333");
}

#[test]
fn arrays_mixed() {
    let mut model = new_empty_model();

    model._set("A2", "2");
    model._set("A3", "4");
    model._set("A4", "6");
    model._set("A5", "8");

    model._set("B2", "1");
    model._set("B3", "3");
    model._set("B4", "5");
    model._set("B5", "7");

    model._set("C1", "=COVARIANCE.P(A2:A5, {1,3,5,7})");
    model._set("C2", "=COVARIANCE.S(A2:A5, {1,3,5,7})");
    model._set("C3", "=COVARIANCE.P(A2:A5, B2:B5)");
    model._set("C4", "=COVARIANCE.S(A2:A5, B2:B5)");
    model._set("C5", "=COVARIANCE.P({2,4,6,8}, B2:B5)");
    model._set("C6", "=COVARIANCE.S({2,4,6,8}, B2:B5)");
    model._set("C7", "=COVARIANCE.P({2,4,6,8}, {1,3,5,7})");
    model._set("C8", "=COVARIANCE.S({2,4,6,8}, {1,3,5,7})");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"5");
    assert_eq!(model._get_text("C2"), *"6.666666667");
}

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=COVARIANCE.P()");
    model._set("A2", "=COVARIANCE.S()");
    model._set("A3", "=COVARIANCE.P(1)");
    model._set("A4", "=COVARIANCE.S(1)");
    model._set("A5", "=COVARIANCE.P({1}, {2})");
    model._set("A6", "=COVARIANCE.S({1}, {2})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
}

// A number or single cell is an array of one value (statistics.md "Array arguments given a
// scalar, a single cell or an error").
#[test]
fn paired_functions_take_scalars_as_one_value_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "0");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "3");
    model._set("D1", "=COVAR(0,A1)");
    model._set("D2", "=COVAR(0,B1:B2)");
    model._set("D3", "=SLOPE(1,2)");
    model._set("D4", "=FORECAST.LINEAR(1,0.2,1)");
    model._set("D5", "=FORECAST.LINEAR(1,0.2,B1:B2)");
    model._set("D6", "=T.TEST(C1:C3,5,1,1)");
    model._set("D7", "=T.TEST(A1,B1:B2,0.1,3)");
    model._set("D8", "=RSQ(A1,B1:B2)");
    model._set("E1", "x");
    model._set("E2", "TRUE");
    model._set("D9", "=CHISQ.TEST(E1,B1:B2)");
    model._set("D10", "=CORREL(E2,B1:B2)");
    model._set("D12", "=T.TEST(E1,A1:A2,0.1,3)");
    model.evaluate();
    assert_eq!(model._get_text("D9"), *"#N/A");
    assert_eq!(model._get_text("D10"), *"#N/A");
    assert_eq!(model._get_text("D12"), *"#NUM!");
    assert_eq!(model._get_text("D1"), *"0");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#DIV/0!");
    assert_eq!(model._get_text("D4"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"#N/A");
    assert_eq!(model._get_text("D6"), *"#N/A");
    assert_eq!(model._get_text("D7"), *"#NUM!");
    assert_eq!(model._get_text("D8"), *"#N/A");
}

// A size mismatch is #N/A before an error cell is read; with equal sizes the error wins
// (probes/ftest-decomposition G7:G12, T-1927).
#[test]
fn size_mismatch_beats_an_error_cell() {
    let mut model = new_empty_model();
    model._set("H1", "1");
    model._set("H2", "=1/0");
    model._set("H3", "3");
    for (cell, value) in [("I1", "2"), ("I2", "4"), ("I3", "5"), ("I4", "7")] {
        model._set(cell, value);
    }
    model._set("A1", "=COVARIANCE.S(H1:H3,I1:I4)");
    model._set("A2", "=COVARIANCE.P(I1:I4,H1:H3)");
    model._set("A3", "=COVAR(H1:H3,I1:I4)");
    model._set("A4", "=COVARIANCE.S(H1:H3,I1:I3)");
    model._set("A5", "=T.TEST(H1:H3,I1:I4,1,1)");
    model._set("A6", "=T.TEST(H1:H3,I1:I4,2,2)");
    model._set("A7", "=F.TEST(H1:H3,I1:I4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
    assert_eq!(model._get_text("A2"), "#N/A");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "#DIV/0!");
    assert_eq!(model._get_text("A5"), "#N/A");
    assert_eq!(model._get_text("A6"), "#DIV/0!");
    assert_eq!(model._get_text("A7"), "#DIV/0!");
}

/// A one-cell reference to an empty cell is `#VALUE!` in the paired functions, before the size
/// check and the other argument's error; one holding a number is one value (statistics.md,
/// T-2255, T-2426).
#[test]
fn test_paired_blank_one_cell_reference_is_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "5");
    model._set("A4", "=COVAR(A1:A3,Z1)");
    model._set("A5", "=CORREL(Z1:Z1,{1,2,3})");
    model._set("A6", "=F.TEST(Z1,A1:A3)");
    model._set("A7", "=INTERCEPT(Z1,1/0)");
    model._set("A8", "=COVAR(A1:A3,B1)");
    model._set("A9", "=T.TEST(Z1,A1:A3,2,2)");
    model._set("A10", "=CHISQ.TEST(A1:A3,Z1)");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "#VALUE!");
    assert_eq!(model._get_text("A7"), "#VALUE!");
    assert_eq!(model._get_text("A8"), "#N/A");
    assert_eq!(model._get_text("A9"), "#VALUE!");
    assert_eq!(model._get_text("A10"), "#VALUE!");
}

// T-2431: two arrays with the same cell count pair in reading order whatever their shapes.
#[test]
fn test_paired_functions_pair_by_cell_count() {
    let mut model = new_empty_model();
    model._set("A1", "=CORREL({1,2;3,4},{1;2;3;4})");
    model._set("A2", "=INTERCEPT({1;2;3;4},{1,2;3,5})");
    model._set("A3", "=CORREL({1,2;3,4},{1;2;3})");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"0.457142857");
    assert_eq!(model._get_text("A3"), *"#N/A");
}

// A typed or computed boolean is one value, so the size check and the one-point rule apply, not
// `#VALUE!` (T-2575; probes/t2436/computed-scalar A35:A37).
#[test]
fn test_covariance_boolean_scalar_is_one_value() {
    let mut model = new_empty_model();
    model._set("A1", "=COVARIANCE.P({1,2,3},TRUE)");
    model._set("A2", "=COVARIANCE.S(-2.5,FALSE)");
    model._set("A3", "=COVARIANCE.S(1,IF(1,TRUE))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
}
