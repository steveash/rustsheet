#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── TOCOL ─────────────────────────────────────────────────────────────────────

#[test]
fn tocol_2d_to_column() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "=TOCOL(A1:B2)");
    model.evaluate();
    // row-by-row: 1, 2, 3, 4
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "3");
    assert_eq!(model._get_text("C4"), "4");
    assert_eq!(model._get_text("C5"), "");
}

#[test]
fn tocol_scan_by_col() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "=TOCOL(A1:B2,0,TRUE)");
    model.evaluate();
    // col-by-col: 1, 3, 2, 4
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("C2"), "3");
    assert_eq!(model._get_text("C3"), "2");
    assert_eq!(model._get_text("C4"), "4");
}

#[test]
fn tocol_single_column_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "=TOCOL(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("B2"), "20");
    assert_eq!(model._get_text("B3"), "30");
}

// ── TOROW ─────────────────────────────────────────────────────────────────────

#[test]
fn torow_2d_to_row() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "=TOROW(A1:B2)");
    model.evaluate();
    // row-by-row: 1, 2, 3, 4
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("F1"), "4");
    assert_eq!(model._get_text("G1"), "");
}

#[test]
fn torow_scan_by_col() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("C1", "=TOROW(A1:B2,0,TRUE)");
    model.evaluate();
    // col-by-col: 1, 3, 2, 4
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("F1"), "4");
}

#[test]
fn torow_single_row_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A2", "=TOROW(A1:C1)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("B2"), "20");
    assert_eq!(model._get_text("C2"), "30");
}

// T-2409: a lone blank cell stays blank (docs/excel-semantics/arrays.md "TOROW and UNIQUE of a lone blank cell").
#[test]
fn torow_lone_blank_cell_is_ignored() {
    let mut model = new_empty_model();
    model._set("B1", "=TOROW(Z1,3)");
    model._set("B2", "=UNIQUE(Z1)&\"x\"");
    model._set("A1", "5");
    model._set("B3", "=TOROW(A1,3)");
    model._set("B4", "=TOCOL(Z1,1)");
    model.evaluate();
    assert_eq!(model._get_text("B4"), "#CALC!");
    assert_eq!(model._get_text("B1"), "#CALC!");
    assert_eq!(model._get_text("B2"), "x");
    assert_eq!(model._get_text("B3"), "5");
}

// ── Which bad argument wins (T-3507) ──────────────────────────────────────────

/// An ignore that is not a number is `#VALUE!` before a `scan_by_column` error; an out-of-range
/// number loses to it; a range ignore lifts, each element on its own (gen-m7/52ccea5df17b29a8
/// A4454, A5613, A10676:B10676; 7cd94f345acd55bb A11530; docs/excel-semantics/arrays.md "TOCOL,
/// TOROW and SORT: which bad argument wins").
#[test]
fn text_ignore_beats_a_scan_by_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("D1", "x");
    model._set("D2", "7");
    model._set("E1", "=1/0");
    model._set("B1", "=TOCOL(A1,\"abc\",#NULL!)");
    model._set("B2", "=TOROW(A1,\"\",#NAME?)");
    model._set("B3", "=TOCOL(A1,D1,E1)");
    model._set("B4", "=TOCOL(A1,7,E1)");
    model._set("B5", "=TOROW(A1,-2.5,#NULL!)");
    model._set("B6", "=TOCOL(A1,#N/A,#NULL!)");
    model._set("B7", "=TOCOL(A1,\"abc\")");
    model._set("C10", "=TOROW(A1,D1:D2,E1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#VALUE!");
    assert_eq!(model._get_text("B2"), "#VALUE!");
    assert_eq!(model._get_text("B3"), "#VALUE!");
    assert_eq!(model._get_text("B4"), "#DIV/0!");
    assert_eq!(model._get_text("B5"), "#NULL!");
    assert_eq!(model._get_text("B6"), "#N/A");
    assert_eq!(model._get_text("B7"), "#VALUE!");
    assert_eq!(model._get_text("C10"), "#VALUE!");
    assert_eq!(model._get_text("C11"), "#DIV/0!");
}
