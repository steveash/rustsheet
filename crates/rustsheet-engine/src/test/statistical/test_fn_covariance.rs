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
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"0");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#DIV/0!");
    assert_eq!(model._get_text("D4"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"#N/A");
    assert_eq!(model._get_text("D6"), *"#N/A");
    assert_eq!(model._get_text("D7"), *"#NUM!");
    assert_eq!(model._get_text("D8"), *"#N/A");
}
