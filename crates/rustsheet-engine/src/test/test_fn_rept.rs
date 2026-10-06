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

#[test]
fn broadcast_under_the_cap_returns_the_array() {
    let mut model = new_empty_model();
    model._set("A1", "=REPT(\"x\", SEQUENCE(1000)*0+32767)");

    model.evaluate();

    assert_eq!(model._get_text("A1").len(), 32767);
    assert_eq!(model._get_text("A1000").len(), 32767);
}

#[test]
fn length_limit_counts_utf16_units() {
    // probes/concatenate-limit-spill E11/E12 (excel-graph): REPT's 32,767 limit is in UTF-16
    // code units, not UTF-8 bytes.
    let mut model = new_empty_model();
    model._set("A1", "=REPT(\"é\",32767)");
    model._set("A2", "=REPT(\"😀\",16383)");
    model._set("A3", "=REPT(\"😀\",16384)");
    model._set("A4", "=REPT(\"é\",32768)");

    model.evaluate();

    assert_eq!(model._get_text("A1").chars().count(), 32767);
    assert_eq!(model._get_text("A2").chars().count(), 16383);
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
}
