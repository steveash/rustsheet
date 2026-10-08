#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// docs/excel-semantics/lookup.md "XLOOKUP: return_array may be wider than the lookup vector".
fn model() -> crate::Model {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model._set("C1", "x");
    model._set("C2", "y");
    model._set("C3", "z");
    model
}

#[test]
fn wide_return_range_gives_the_matched_row() {
    let mut model = model();
    model._set("E1", "=XLOOKUP(2,A1:A3,B1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), *"b");
    assert_eq!(model._get_text("F1"), *"y");
}

#[test]
fn different_row_count_is_value_error() {
    let mut model = model();
    model._set("E1", "=XLOOKUP(2,A1:A3,B1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), *"#VALUE!");
}

#[test]
fn single_column_return_range_is_scalar() {
    let mut model = model();
    model._set("E1", "=XLOOKUP(3,A1:A3,C1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), *"z");
    assert_eq!(model._get_text("F1"), *"");
}

// Task T-1126: probes/isref-reference-lookups H10 (an array lookup_array, a range return_array).
#[test]
fn array_lookup_array_picks_from_the_return_range() {
    let mut model = model();
    model._set("E1", "=XLOOKUP(2,{1;2;3},B1:B3)");
    model._set("E2", "=ISREF(XLOOKUP(2,{1;2;3},B1:B3))");
    model._set("E3", "=XLOOKUP(9,{1;2;3},B1:B3,\"none\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), *"b");
    assert_eq!(model._get_text("E2"), *"TRUE");
    assert_eq!(model._get_text("E3"), *"none");
}
