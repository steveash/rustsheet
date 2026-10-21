#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=ARABIC()");
    model._set("A2", "=ARABIC(\"V\")");
    model._set("A3", "=ARABIC(\"V\", 2)");

    model._set("A4", "=ROMAN()");
    model._set("A5", "=ROMAN(5)");
    model._set("A6", "=ROMAN(5, 0)");
    model._set("A7", "=ROMAN(5, 0, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"5");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"V");
    assert_eq!(model._get_text("A6"), *"V");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
}

// gen-m5 excel-graph oracle, 2026-10-20 (T-2154; numbers.md "ROMAN truncates before its sign check;
// BASE checks the sign first"). A1-A3, A5-A6 are oracle cells; A4 (`ROMAN(-1)`), A7 (`BASE(-1,5)`) and A8
// (`BASE(0.9,5,2)`) are hand-written expectations from the documented rules, not probed.
#[test]
fn roman_truncates_before_the_sign_check_and_reads_its_form_for_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=ROMAN(-0.5025)");
    model._set("A2", "=ROMAN(0,NA())");
    model._set("A3", "=ROMAN(0.2,8.09)");
    model._set("A4", "=ROMAN(-1)");
    model._set("A5", "=ROMAN(0)");
    model._set("A6", "=BASE(-0.0001,5,4)");
    model._set("A7", "=BASE(-1,5)");
    model._set("A8", "=BASE(0.9,5,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"00");
}
