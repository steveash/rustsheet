//! TOCOL's ignore and `scan_by_column` and UNIQUE's `by_col` and `exactly_once` lift over a range or an
//! array, one call per element whose value is that call's top-left; the array stays whole
//! (SPEC-0011 "Array-valued functions", T-1836; probes/m7-argument-modes F9:H9, K8, F12:H12).

use crate::test::util::new_empty_model;

fn model_with_data() -> crate::model::Model {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "2"),
        ("A2", "3"),
        ("B2", "4"),
        ("A3", "5"),
        ("B3", "6"),
        ("D1", "x"),
        ("D2", "y"),
        ("D3", "x"),
    ] {
        model._set(cell, value);
    }
    model
}

fn check(cells: &[(&str, &str, &str)]) {
    let mut model = model_with_data();
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell} {formula}");
    }
}

#[test]
fn tocol_ignore_lifts_over_an_array_and_a_range() {
    check(&[
        ("F1", "=ROWS(TOCOL(A1:B3,{0;1}))", "2"),
        ("G1", "=COLUMNS(TOCOL(A1:B3,{0;1}))", "1"),
        ("H1", "=INDEX(TOCOL(A1:B3,{0;1}),1,1)", "1"),
        ("K1", "=ROWS(INDEX(TOCOL(A1:B3,{0;1}),1,1))", "1"),
        ("F2", "=ROWS(TOCOL(A1:B3,A1:A2))", "2"),
        ("F3", "=ROWS(TOCOL(A1:B3,{0,1;1,0}))", "2"),
    ]);
}

#[test]
fn unique_by_col_lifts_over_an_array() {
    check(&[
        ("F1", "=ROWS(UNIQUE(D1:D3,{FALSE;TRUE}))", "2"),
        ("G1", "=COLUMNS(UNIQUE(D1:D3,{FALSE;TRUE}))", "1"),
        ("H1", "=INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),1,1)", "x"),
    ]);
}

#[test]
fn unlifted_calls_keep_their_values() {
    check(&[
        ("F1", "=TOCOL(A1:B2,,TRUE)", "1"),
        ("F2", "", "3"),
        ("F3", "", "2"),
        ("F4", "", "4"),
        ("H1", "=UNIQUE(D1:D3)", "x"),
        ("H2", "", "y"),
        ("H3", "", ""),
        ("J1", "=UNIQUE(D1:D3,FALSE,TRUE)", "y"),
        ("L1", "=TOCOL(A1:A2,4)", "#VALUE!"),
        ("M1", "=UNIQUE(D1:D3,\"no\")", "#VALUE!"),
        ("N1", "=ROWS(TOCOL(A1:B3))", "6"),
    ]);
}

#[test]
fn an_element_error_stays_in_its_position() {
    check(&[
        ("F1", "=TOCOL(A1:B3,{0;4})", "1"),
        ("J1", "=UNIQUE(D1:D3,{FALSE;\"no\"})", "x"),
        ("J2", "", "#VALUE!"),
        ("F2", "", "#VALUE!"),
    ]);
}

/// Unless every entry is equal, only the first element that is not an error keeps its call's
/// top-left; later ones are empty: they spill as 0, ISBLANK is TRUE, and ROWS and INDEX of them
/// are `#VALUE!` (probes/m7-unique-lift-elements, m7-lift-take-drop-sortby rows 24-37; T-2300).
#[test]
fn later_elements_are_empty_unless_every_entry_is_equal() {
    check(&[
        ("F1", "=UNIQUE(D1:D3,FALSE,{FALSE;TRUE;FALSE})", "x"),
        ("F2", "", "0"),
        ("F3", "", "0"),
        ("G1", "=UNIQUE(D1:D3,FALSE,{FALSE;FALSE})", "x"),
        ("G2", "", "x"),
        ("H1", "=UNIQUE(D1:D3,FALSE,{#N/A;FALSE})", "#N/A"),
        ("H2", "", "x"),
        (
            "I1",
            "=ISBLANK(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1))",
            "TRUE",
        ),
        (
            "J1",
            "=ROWS(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1))",
            "#VALUE!",
        ),
        (
            "K1",
            "=INDEX(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1),1,1)",
            "#VALUE!",
        ),
        ("L1", "=SUM(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1))", "0"),
        // An error array is every element's result (gen-m7/8044dce50616cfe9 Calc!A1434:B1434).
        ("M1", "=UNIQUE(1/0,{0.2,1E+15})", "#DIV/0!"),
        ("N1", "", "#DIV/0!"),
    ]);
}

/// Each element walks the whole array, so a call is charged its cells per element and is `#ERROR!`
/// past `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION, docs/excel-semantics/arrays.md "Array
/// results above 1,048,576 cells").
#[test]
fn a_lifted_call_over_a_large_array_is_charged_per_element() {
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(TOCOL(SEQUENCE(131072),SEQUENCE(1100,1,0,0)))");
    model._set("B1", "=ROWS(UNIQUE(SEQUENCE(131072),SEQUENCE(1100,1,0,0)))");
    model._set("C1", "=ROWS(TOCOL(SEQUENCE(131072),SEQUENCE(2,1,0,0)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "#ERROR!");
    assert_eq!(model._get_text("C1"), "2");
}

#[test]
fn torow_lifts_scan_by_column() {
    // TOROW shares TOCOL's entry (T-2313; gen-m7/7d67c559ce7536fa A619): one element per
    // scan_by_column entry, each its call's top-left.
    check(&[
        ("F1", "=COLUMNS(TOROW(A1:B3,0,{FALSE,TRUE}))", "2"),
        ("F2", "=INDEX(TOROW(A1:B3,0,{FALSE,TRUE}),1,2)", "1"),
        ("F3", "=INDEX(TOROW(A1:B3,0,{FALSE,\"x\"}),1,2)", "#VALUE!"),
        // Unlifted calls are unchanged.
        ("F4", "=INDEX(TOROW(A1:B3,0,TRUE),1,2)", "3"),
        ("F5", "=COLUMNS(TOROW(A1:B3))", "6"),
    ]);
}

#[test]
fn an_error_array_fills_the_lifted_shape() {
    // `TOROW(<#NUM!>,3,<5x3 array>)` spills 15 `#NUM!`s (gen-m7/611ce3bbfab27cef A919:C923);
    // unlifted it is the one error.
    check(&[
        ("F1", "=ROWS(TOROW(SQRT(-1),3,{1,0;0,1}))", "2"),
        ("F2", "=INDEX(TOROW(SQRT(-1),3,{1,0;0,1}),2,2)", "#NUM!"),
        ("F3", "=TOROW(SQRT(-1),3,TRUE)", "#NUM!"),
    ]);
}

#[test]
fn an_error_argument_comes_before_an_invalid_one() {
    // An invalid ignore with an error scan_by_column is the error (gen-m7/8044dce50616cfe9 A1508).
    check(&[
        ("F1", "=TOCOL(A1:A3,7,NA())", "#N/A"),
        ("F2", "=TOROW(A1:A3,7,NA())", "#N/A"),
        ("F3", "=TOCOL(A1:A3,7,TRUE)", "#VALUE!"),
        ("F4", "=TOCOL(A1:A3,1/0,NA())", "#DIV/0!"),
    ]);
}
