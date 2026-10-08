#![allow(clippy::unwrap_used)]

// `for_each_number_in_areas` is the shared reference walk behind SUMSQ, NPV,
// DEVSQ, VAR, ...: numbers count, text/booleans/empties are skipped, the first
// error cell wins, and cross-sheet ranges are #VALUE!.

use crate::test::util::new_empty_model;

#[test]
fn skips_text_and_booleans_in_references() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMSQ(B1:B5)");
    model._set("B1", "3");
    model._set("B2", "text");
    model._set("B3", "TRUE");
    model._set("B4", "4");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"25");
}

#[test]
fn first_error_cell_aborts_the_walk() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMSQ(B1:B3)");
    model._set("B1", "2");
    model._set("B2", "=1/0");
    model._set("B3", "=NA()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
}

#[test]
fn whole_column_is_walked_over_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMSQ(B:B)");
    model._set("B1", "1");
    model._set("B3", "2");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"5");
}
