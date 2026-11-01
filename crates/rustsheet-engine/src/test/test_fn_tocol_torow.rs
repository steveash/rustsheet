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

// T-3506: a one-cell reference to an error cell is a 1x1 array holding the error, so the other
// arguments are checked first (gen-m7 arg-pairs, excel-graph; docs/excel-semantics/arrays.md "A
// one-cell error reference is a 1x1 array in TOCOL, TOROW and FILTER").
#[test]
fn tocol_torow_one_cell_error_reference_is_a_one_by_one_array() {
    let mut model = new_empty_model();
    model._set("G9", "=1/0");
    // Data!C3:D4 of the gen-m7 cases below.
    model._set("C3", "abc");
    model._set("D3", "7");
    model._set("C4", "1");
    model._set("D4", "2.5");
    // An invalid ignore is `#VALUE!` (gen-m7/20df23f84a528470 A8724).
    model._set("A1", "=TOROW(G9,7)");
    // Ignore 3 and 2 drop the error, so nothing is left (gen-m7/7cd94f345acd55bb A8480).
    model._set("A2", "=TOCOL(G9,3,0)");
    model._set("A3", "=TOROW(G9,2)");
    // Ignore 0 and 1 keep it as the one element (T-3506 Context; A7 below, b24463215bcb1ab9 A10067).
    model._set("A4", "=TOCOL(G9,1)");
    model._set("A5", "=TOROW(G9)");
    // A range ignore lifts: "abc" and 7 are `#VALUE!`, 1 keeps the error, 2.5 (ignore 2) is
    // `#CALC!` (`TOCOL(Data!G9,Data!C3:D4)`, gen-m7/b24463215bcb1ab9 A10066:B10067).
    model._set("A6", "=TOCOL(G9,C3:D4)");
    // A range scan_by lifts too: "abc" is `#VALUE!`, each number `#CALC!`, as ignore 2 drops the
    // error (`TOCOL(Data!G9,2,Data!C3:D4)`, gen-m7/7cd94f345acd55bb A9578:B9579).
    model._set("A11", "=TOCOL(G9,2,C3:D4)");
    // A literal error is still the error (gen-m7/20df23f84a528470 A8907).
    model._set("A9", "=TOROW(#N/A,-1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#CALC!");
    assert_eq!(model._get_text("A3"), "#CALC!");
    assert_eq!(model._get_text("A4"), "#DIV/0!");
    assert_eq!(model._get_text("A5"), "#DIV/0!");
    assert_eq!(model._get_text("A6"), "#VALUE!");
    assert_eq!(model._get_text("B6"), "#VALUE!");
    assert_eq!(model._get_text("A7"), "#DIV/0!");
    assert_eq!(model._get_text("B7"), "#CALC!");
    assert_eq!(model._get_text("A11"), "#VALUE!");
    assert_eq!(model._get_text("B11"), "#CALC!");
    assert_eq!(model._get_text("A12"), "#CALC!");
    assert_eq!(model._get_text("B12"), "#CALC!");
    assert_eq!(model._get_text("A9"), "#N/A");
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
