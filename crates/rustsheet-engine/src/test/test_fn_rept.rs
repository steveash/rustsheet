#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "Well");

    model._set("B1", "=REPT(A1, 3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"WellWellWell");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "Well");

    model._set("B1", "=REPT(A1)");
    model._set("B2", "=REPT(A1,3,1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn broadcast_over_huge_range_is_bounded() {
    let mut model = new_empty_model();
    // 1M cells x 32767 chars would be ~32 GB; the broadcast cap turns it into #VALUE!.
    model._set("A1", "=REPT(\"x\", SEQUENCE(1000000)*0+32767)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
}
