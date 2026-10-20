#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── TAKE ──────────────────────────────────────────────────────────────────────

#[test]
fn take_first_rows() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=TAKE(A1:A3,2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "");
}

#[test]
fn take_last_rows_negative() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=TAKE(A1:A3,-2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "3");
    assert_eq!(model._get_text("B3"), "");
}

#[test]
fn take_rows_exceeds_array_size() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("B1", "=TAKE(A1:A2,5)");
    model.evaluate();
    // returns all 2 rows
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("B2"), "20");
}

#[test]
fn take_first_cols() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("D1", "=TAKE(A1:C1,1,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("F1"), "");
}

#[test]
fn take_last_cols_negative() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("D1", "=TAKE(A1:C1,1,-2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("F1"), "");
}

#[test]
fn take_zero_rows_is_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=TAKE(A1:A3,0)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#CALC!");
}

// ── DROP ──────────────────────────────────────────────────────────────────────

#[test]
fn drop_first_rows() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=DROP(A1:A3,1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "3");
    assert_eq!(model._get_text("B3"), "");
}

#[test]
fn drop_last_rows_negative() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=DROP(A1:A3,-1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "");
}

#[test]
fn drop_all_rows_returns_calc_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "=DROP(A1:A2,2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#CALC!");
}

#[test]
fn drop_zero_rows_returns_full_array() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=DROP(A1:A3,0)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "3");
}

#[test]
fn drop_first_cols() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("D1", "=DROP(A1:C1,0,1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("F1"), "");
}

// ── Lifted rows and columns (SPEC-0011 "Elements that are references", T-1838) ──────────────

/// A1:B3 = 1, 10; 2, 20; 3, 30, as in probes/m7-argument-modes.
fn lift_model() -> crate::model::Model {
    let mut model = new_empty_model();
    for (row, (a, b)) in [("1", "10"), ("2", "20"), ("3", "30")].iter().enumerate() {
        model._set(&format!("A{}", row + 1), a);
        model._set(&format!("B{}", row + 1), b);
    }
    model
}

// probes/m7-argument-modes F4:I6, K1-K4: each element is the reference its call returns.
#[test]
fn take_drop_lift_over_a_range_into_references() {
    let mut model = lift_model();
    model._set("F4", "=ROWS(TAKE(A1:B3,A1:A2))");
    model._set("G4", "=COLUMNS(TAKE(A1:B3,A1:A2))");
    model._set("H4", "=INDEX(TAKE(A1:B3,A1:A2),1,1)");
    model._set("F5", "=ROWS(TAKE(A1:B3,{1;2}))");
    model._set("G5", "=COLUMNS(TAKE(A1:B3,{1;2}))");
    model._set("H5", "=INDEX(TAKE(A1:B3,{1;2}),1,1)");
    model._set("F6", "=ROWS(DROP(A1:B3,{1;2}))");
    model._set("G6", "=COLUMNS(DROP(A1:B3,{1;2}))");
    model._set("H6", "=INDEX(DROP(A1:B3,{1;2}),1,1)");
    // Row 7 of the probe is not empty: the 2x2 element collides.
    model._set("I7", "x");
    model._set("H9", "=INDEX(DROP(A1:B3,{1;2}),1,1)");
    model._set("K1", "=ROWS(INDEX(TAKE(A1:B3,{1;2}),1,1))");
    model._set("K2", "=COLUMNS(INDEX(TAKE(A1:B3,{1;2}),1,1))");
    model._set("K3", "=ROWS(INDEX(DROP(A1:B3,{1;2}),1,1))");
    model._set("K4", "=COLUMNS(INDEX(DROP(A1:B3,{1;2}),1,1))");
    model.evaluate();
    for (cell, want) in [
        ("F4", "2"),
        ("G4", "1"),
        ("H4", "1"),
        ("I4", "10"),
        ("F5", "2"),
        ("G5", "1"),
        ("H5", "1"),
        ("I5", "10"),
        ("F6", "2"),
        ("G6", "1"),
        ("H6", "#SPILL!"),
        ("H9", "2"),
        ("I9", "20"),
        ("H10", "3"),
        ("I10", "30"),
        ("K1", "1"),
        ("K2", "2"),
        ("K3", "2"),
        ("K4", "2"),
    ] {
        assert_eq!(model._get_text(cell), *want, "{cell}");
    }
}

// The consumers of an array of references read a lifted TAKE or DROP as they read a lifted
// OFFSET (SPEC-0014 rules 2-7): spilled, each element is `#VALUE!`; ISREF of the whole is FALSE;
// LET hands it on; an error element stays itself.
#[test]
fn take_drop_reference_elements_reach_the_grid_consumers() {
    let mut model = lift_model();
    model._set("D1", "=TAKE(A1:B3,{1;2})");
    model._set("E1", "=ISREF(TAKE(A1:B3,{1;2}))");
    model._set("E2", "=ISREF(INDEX(TAKE(A1:B3,{1;2}),2,1))");
    model._set("E3", "=LET(x,DROP(A1:B3,{1;2},1),SUM(INDEX(x,1,1)))");
    model._set("E4", "=INDEX(TAKE(A1:B3,{1;0}),2,1)");
    model._set("E5", "=SUM(INDEX(TAKE(A1:B3,{1;2},-1),2,1))");
    model._set("E6", "=ROWS(TAKE(A1:B3,{1,2;3,1}))");
    model.evaluate();
    for (cell, want) in [
        ("D1", "#VALUE!"),
        ("D2", "#VALUE!"),
        ("E1", "FALSE"),
        ("E2", "TRUE"),
        ("E3", "50"),
        ("E4", "#CALC!"),
        ("E5", "30"),
        ("E6", "2"),
    ] {
        assert_eq!(model._get_text(cell), *want, "{cell}");
    }
}

// Over an array value each element is its call's top-left value (SPEC-0011 rule 2, pending
// T-1960), errors in place; an unlifted call and a 1x1 count keep the whole array.
#[test]
fn take_drop_lift_over_an_array_value_gives_top_left_values() {
    let mut model = new_empty_model();
    model._set("A1", "=TAKE({1,10;2,20;3,30},{1;-1})");
    model._set("B1", "=DROP({1,10;2,20;3,30},{1;2;3})");
    model._set("C1", "=TAKE({1,10;2,20;3,30},{1},-1)");
    model._set("D1", "=TAKE({1,10;2,20},2,{-1,1})");
    model.evaluate();
    for (cell, want) in [
        ("A1", "1"),
        ("A2", "3"),
        ("B1", "2"),
        ("B2", "3"),
        ("B3", "#CALC!"),
        ("C1", "10"),
        ("C2", ""),
        ("D1", "10"),
        ("E1", "1"),
        ("D2", ""),
    ] {
        assert_eq!(model._get_text(cell), *want, "{cell}");
    }
}

// Each element may copy the whole array, so the work is checked before any element runs
// (CLAUDE.md directive 5): 1000 elements of a 10^6-cell array are refused.
#[test]
fn take_lift_past_the_work_bound_is_an_error() {
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(TAKE(SEQUENCE(1000,1000),SEQUENCE(1000)))");
    model._set("A2", "=ROWS(TAKE(SEQUENCE(10,10),SEQUENCE(1000)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"1000");
}
