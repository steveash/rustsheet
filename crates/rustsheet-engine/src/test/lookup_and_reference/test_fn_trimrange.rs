#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── trim both edges (default) ─────────────────────────────────────────────────

#[test]
fn test_trimrange_default_trims_leading_and_trailing_rows() {
    let mut model = new_empty_model();
    // Row 1 blank, rows 2-3 have data, row 4 blank
    model._set("A2", "10");
    model._set("A3", "20");
    // Default trim_rows=3, trim_cols=3
    model._set("C1", "=TRIMRANGE(A1:A4)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "10");
    assert_eq!(model._get_text("C2"), "20");
    // C3 should be empty (result is only 2 rows)
    assert_eq!(model._get_text("C3"), "");
}

#[test]
fn test_trimrange_default_trims_leading_and_trailing_cols() {
    let mut model = new_empty_model();
    // Column A blank, column B has data, column C blank
    model._set("B1", "hello");
    model._set("D1", "=TRIMRANGE(A1:C1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "hello");
    // E1 should be empty (result is just 1 column)
    assert_eq!(model._get_text("E1"), "");
}

// ── trim_rows mode ────────────────────────────────────────────────────────────

#[test]
fn test_trimrange_trim_rows_none() {
    let mut model = new_empty_model();
    model._set("A2", "42");
    // trim_rows=0: no row trimming → still 3 rows in result.
    // Blank cells in spilled arrays render as "0" in IronCalc.
    model._set("C1", "=TRIMRANGE(A1:A3,0,0)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "0"); // blank row kept (renders as 0)
    assert_eq!(model._get_text("C2"), "42");
    assert_eq!(model._get_text("C3"), "0"); // blank row kept (renders as 0)
}

#[test]
fn test_trimrange_trim_leading_rows_only() {
    let mut model = new_empty_model();
    model._set("A2", "5");
    model._set("A3", "6");
    // trim_rows=1: only leading blank row removed
    model._set("C1", "=TRIMRANGE(A1:A3,1,0)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "5");
    assert_eq!(model._get_text("C2"), "6");
}

#[test]
fn test_trimrange_trim_trailing_rows_only() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    model._set("A2", "8");
    // row 3 is blank — trim_rows=2 trims it
    model._set("C1", "=TRIMRANGE(A1:A3,2,0)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "7");
    assert_eq!(model._get_text("C2"), "8");
    assert_eq!(model._get_text("C3"), "");
}

// ── interior blanks are preserved ─────────────────────────────────────────────

#[test]
fn test_trimrange_preserves_interior_blank_rows() {
    let mut model = new_empty_model();
    model._set("A1", "start");
    // A2 intentionally blank (interior)
    model._set("A3", "end");
    model._set("C1", "=TRIMRANGE(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "start");
    assert_eq!(model._get_text("C2"), "0"); // interior blank preserved (renders as 0)
    assert_eq!(model._get_text("C3"), "end");
}

// ── fully blank range → #REF! ─────────────────────────────────────────────────

#[test]
fn test_trimrange_all_blank_returns_ref_error() {
    let mut model = new_empty_model();
    // A1:A3 all blank
    model._set("C1", "=TRIMRANGE(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#REF!");
}

// ── 2-D trimming ──────────────────────────────────────────────────────────────

#[test]
fn test_trimrange_2d_trims_rows_and_cols() {
    let mut model = new_empty_model();
    // Row 1 and column A are blank; data starts at B2
    model._set("B2", "1");
    model._set("C2", "2");
    model._set("B3", "3");
    model._set("C3", "4");
    model._set("E1", "=TRIMRANGE(A1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "1");
    assert_eq!(model._get_text("F1"), "2");
    assert_eq!(model._get_text("E2"), "3");
    assert_eq!(model._get_text("F2"), "4");
}

// An array trim_rows lifts TRIMRANGE into an array of references (SPEC-0014, T-1957;
// probes/reference-lift-elements).
#[test]
fn lifted_trim_rows_is_an_array_of_references() {
    let mut model = new_empty_model();
    model._set("A2", "7");
    model._set("D1", "=INDEX(TRIMRANGE(A2,{0;1}),2,1)");
    model._set("D2", "=ROWS(TRIMRANGE(A2,{0;1}))");
    model._set("D3", "=ISREF(TRIMRANGE(A2,{0;1}))");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"7");
    assert_eq!(model._get_text("D2"), *"2");
    assert_eq!(model._get_text("D3"), *"FALSE");
}

// The trim of an array source is a value: a lifted call gives each element's top-left value
// (probes/trimrange-lift-sources D1-D10, F40:F41; T-2089).
#[test]
fn lifted_trim_of_an_array_is_its_top_left_values() {
    let mut model = new_empty_model();
    model._set("D1", "=INDEX(TRIMRANGE({1,2},{0;1}),2,1)");
    model._set("D2", "=SUM(TRIMRANGE({1,2},{0;1}))");
    model._set("D3", "=ISREF(INDEX(TRIMRANGE({1,2},{0;1}),2,1))");
    model._set("D4", "=INDEX(TRIMRANGE({1;2},0,{0,1}),1,2)");
    model._set("F1", "=TRIMRANGE({1,2},{0;1})");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"1");
    assert_eq!(model._get_text("D2"), *"2");
    assert_eq!(model._get_text("D3"), *"FALSE");
    assert_eq!(model._get_text("D4"), *"1");
    assert_eq!(model._get_text("F1"), *"1");
    assert_eq!(model._get_text("F2"), *"1");
}

// A one-cell source is a reference whether written, named or computed, and SUM reads each
// element's top-left cell (probes/trimrange-lift-sources D11-D27; T-2089).
#[test]
fn lifted_trim_of_a_computed_reference_is_a_reference() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("A2", "7");
    model
        .new_defined_name("srcOne", None, "Sheet1!$A$1")
        .unwrap();
    model._set("D1", "=INDEX(TRIMRANGE(srcOne,{0;1}),2,1)");
    model._set("D2", "=ISREF(INDEX(TRIMRANGE(srcOne,{0;1}),2,1))");
    model._set("D3", "=INDEX(TRIMRANGE(IF(TRUE,A1),{0;1}),2,1)");
    model._set("D4", "=SUM(TRIMRANGE(CHOOSE(1,A1),{0;1}))");
    model._set("D5", "=SUM(TRIMRANGE(A1:B5,{0;1}))");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"5");
    assert_eq!(model._get_text("D2"), *"TRUE");
    assert_eq!(model._get_text("D3"), *"5");
    assert_eq!(model._get_text("D4"), *"10");
    assert_eq!(model._get_text("D5"), *"10");
}

// An error source is that error in every element (probes/trimrange-lift-sources D28-D34, H40:H41;
// T-2089).
#[test]
fn lifted_trim_of_an_error_is_an_array_of_it() {
    let mut model = new_empty_model();
    model._set("D1", "=ROWS(TRIMRANGE(NA(),{0;1}))");
    model._set("D2", "=COUNTA(TRIMRANGE(NA(),{0;1}))");
    model._set("D3", "=COLUMNS(TRIMRANGE(1/0,0,{0,1}))");
    model._set("F1", "=TRIMRANGE(NA(),{0;1})");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"2");
    assert_eq!(model._get_text("D2"), *"2");
    assert_eq!(model._get_text("D3"), *"2");
    assert_eq!(model._get_text("F1"), *"#N/A");
    assert_eq!(model._get_text("F2"), *"#N/A");
}
