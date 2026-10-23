#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn iserr_lifts_over_range_and_excludes_na() {
    let mut model = new_empty_model();
    model._set("A1", "=NA()");
    model._set("A2", "=1/0");
    model._set("A3", "5");
    model._set("B1", "=ISERR(A1:A3)");
    model._set("C1", "=ISNA(A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"FALSE");
    assert_eq!(model._get_text("B2"), *"TRUE");
    assert_eq!(model._get_text("B3"), *"FALSE");
    assert_eq!(model._get_text("C1"), *"TRUE");
    assert_eq!(model._get_text("C2"), *"FALSE");
}

#[test]
fn full_sheet_range_is_bounded() {
    let mut model = new_empty_model();
    // DEVIATION (decision 0014): Excel spills this; we refuse above the shared array bound.
    model._set("A1", "=ISBLANK(A:XFD)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn parity_counts_a_value_within_1e_10_below_an_integer_as_that_integer() {
    // probes/iseven-isodd-rounding-threshold and -rounding-window (excel-graph, T-2382).
    let mut model = new_empty_model();
    model._set("A1", "=ISODD(7-1E-10)");
    model._set("A2", "=ISODD(7-1.01E-10)");
    model._set("A3", "=ISODD(-(7-1E-10))");
    model._set("A4", "=ISEVEN(POWER(49,0.5-1E-15))");
    model._set("A5", "=ISODD(1E+15-0.25)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"FALSE");
    assert_eq!(model._get_text("A5"), *"TRUE");
}
