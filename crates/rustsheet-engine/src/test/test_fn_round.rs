#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_round_approximation() {
    let mut model = new_empty_model();
    model._set("A1", "=ROUND(1.05*(0.0284+0.0046)-0.0284,4)");
    model._set("A2", "=ROUNDDOWN(1.05*(0.0284+0.0046)-0.0284,5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.0063");
    assert_eq!(model._get_text("A2"), *"0.00625");
}

#[test]
fn round_family_lifts_over_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "5.55");
    model._set("B1", "7.77");
    model._set("C1", "=ROUNDDOWN(A1:B1,1)");
    model._set("C2", "=ROUNDUP(2.51,A1:B1/5)");
    model._set("C3", "=ROUND(A1:B1,1/0)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "5.5");
    assert_eq!(model._get_text("D1"), "7.7");
    assert_eq!(model._get_text("C2"), "2.6");
    // An error argument spills as an error over the array's shape (gen-m2/893302d6c5b99e2e).
    assert_eq!(model._get_text("C3"), "#DIV/0!");
    assert_eq!(model._get_text("D3"), "#DIV/0!");
}

#[test]
fn concat_joins_array_elements_row_by_row() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCAT(MID({\"abc\",\"def\"},2,1))");
    model._set("A2", "=CONCAT(MID({\"abc\",\"def\"},7,0))");
    model._set("A3", "=CONCAT({1,2;3,TRUE})");
    model._set("A4", "=CONCAT({1,#DIV/0!})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "be");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("A3"), "123TRUE");
    assert_eq!(model._get_text("A4"), "#DIV/0!");
}
