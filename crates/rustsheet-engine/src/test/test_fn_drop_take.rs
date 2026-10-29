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
// D1:D2 and E1 match probes/m7-lift-take-drop-sortby W45:W46 and U2; E4 is UNVERIFIED (T-2299
// owns the rest of the grid hand-on cells).
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

// Over an array value each element is its call's top-left value (SPEC-0011 rule 2;
// probes/m7-lift-take-drop-sortby rows 5-7), errors in place; a 1x1 count lifts to one element
// too (T-2298).
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

// TAKE and DROP over an array of references keep the elements they count, still references
// (SPEC-0014 rule 7; probes/reference-lift-consumers E101, -b2 E153).
#[test]
fn take_drop_keep_an_array_of_references() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "100");
    model._set("A3", "10000");
    model._set("B1", "2");
    model._set("C1", "3");
    // By rows.
    model._set("E1", "=SUM(TAKE(OFFSET(A1,{0;1;2},0),1))");
    model._set("E2", "=SUM(DROP(OFFSET(A1,{0;1;2},0),1))");
    model._set("E3", "=SUM(TAKE(OFFSET(A1,{0;1;2},0),-2))");
    model._set("E4", "=SUM(DROP(OFFSET(A1,{0;1;2},0),-1))");
    model._set("E5", "=ROWS(DROP(OFFSET(A1,{0;1;2},0),1))");
    model._set("E6", "=INDEX(DROP(OFFSET(A1,{0;1;2},0),1),2,1)");
    // By columns.
    model._set("E7", "=SUM(TAKE(OFFSET(A1,0,{0,1,2}),1,2))");
    model._set("E8", "=SUM(DROP(OFFSET(A1,0,{0,1,2}),0,2))");
    model._set("E9", "=COLUMNS(TAKE(OFFSET(A1,0,{0,1,2}),1,-2))");
    // An error element is kept; a zero TAKE is #CALC!.
    model._set("E10", "=SUM(DROP(OFFSET(A1,{0;-1;1},0),2))");
    model._set("E11", "=SUM(TAKE(OFFSET(A1,{0;-1;1},0),2))");
    model._set("E12", "=SUM(TAKE(OFFSET(A1,{0;1;2},0),0))");
    model.evaluate();
    assert_eq!(model._get_text("E1"), *"1");
    assert_eq!(model._get_text("E2"), *"10100");
    assert_eq!(model._get_text("E3"), *"10100");
    assert_eq!(model._get_text("E4"), *"101");
    assert_eq!(model._get_text("E5"), *"2");
    assert_eq!(model._get_text("E6"), *"10000");
    assert_eq!(model._get_text("E7"), *"3");
    assert_eq!(model._get_text("E8"), *"3");
    assert_eq!(model._get_text("E9"), *"2");
    assert_eq!(model._get_text("E10"), *"100");
    assert_eq!(model._get_text("E11"), *"#REF!");
    assert_eq!(model._get_text("E12"), *"#CALC!");
}

// A 1x1 count over a range lifts to one reference element, the scalar call's A1:B2: ROWS and
// COLUMNS of the result are 1, of its element 2, and spilled directly it is `#VALUE!`, as a
// lifted OFFSET's element is (probes/m7-lift-take-drop-sortby Sheet1 F13, G13, I13:K13, W65;
// T-2298). DROP and OFFSET lift a 1x1 array alike (probes/m7-one-cell-operand-lift F6, F7, F16).
// A one-cell range count, also through an operator, is its value and keeps the scalar call (F5).
#[test]
fn take_lifts_a_one_cell_array_count_into_one_reference_element() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "10"),
        ("A2", "2"),
        ("B2", "20"),
        ("A3", "3"),
        ("B3", "30"),
        ("D1", "2"),
    ] {
        model._set(cell, value);
    }
    model._set("F1", "=ROWS(TAKE(A1:B3,{2}))");
    model._set("G1", "=COLUMNS(TAKE(A1:B3,{2}))");
    model._set("H1", "=ROWS(INDEX(TAKE(A1:B3,{2}),1,1))");
    model._set("I1", "=SUM(INDEX(TAKE(A1:B3,{2}),1,1))");
    model._set("J1", "=ROWS(TAKE(A1:B3,D1:D1))");
    model._set("F5", "=TAKE(A1:B3,{2})");
    model._set("K1", "=ROWS(DROP(A1:B3,{1}))");
    model._set("L1", "=ROWS(INDEX(DROP(A1:B3,{1}),1,1))");
    model._set("M1", "=ROWS(OFFSET(A1:B2,{1},0))");
    model._set("N1", "=ROWS(TAKE(A1:B3,--INDIRECT(\"D1\")))");
    model.evaluate();
    for (cell, want) in [
        ("F1", "1"),
        ("G1", "1"),
        ("H1", "2"),
        ("I1", "33"),
        ("J1", "2"),
        ("K1", "1"),
        ("L1", "2"),
        ("M1", "1"),
        ("N1", "2"),
        ("F5", "#VALUE!"),
        ("F6", ""),
        ("G5", ""),
    ] {
        assert_eq!(model._get_text(cell), want, "{cell}");
    }
}

#[test]
fn take_cols_error_beats_zero_rows() {
    // gen-m7/7d67c559ce7536fa A1296 (T-2316): both counts are read before a zero is `#CALC!`.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("D1", "=TAKE(A1:B1,0,1/0)");
    model._set("D2", "=TAKE(A1:B1,1/0,0)");
    model._set("D3", "=TAKE(A1:B1,0,1)");
    model._set("D4", "=TAKE(A1:B1,1,0)");
    model._set("D5", "=TAKE(A1:B1,1,1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#DIV/0!");
    assert_eq!(model._get_text("D2"), "#DIV/0!");
    assert_eq!(model._get_text("D3"), "#CALC!");
    assert_eq!(model._get_text("D4"), "#CALC!");
    assert_eq!(model._get_text("D5"), "1");
}

// A one-cell TAKE/DROP of a range is a reference, so it is a scalar under `+0` or `N()`, while an
// array constant still lifts (probes/t2454-offset-one-cell-array-args A5:D5, A9:D9; T-2519).
#[test]
fn one_cell_take_drop_of_a_range_is_a_scalar_height() {
    let mut model = new_empty_model();
    model._set("H2", "3");
    model._set("H3", "4");
    model._set("A1", "=ROWS(OFFSET(B9:C12,5,1,TAKE(H2:H3,1)+0))");
    model._set("A2", "=ROWS(OFFSET(B9:C12,5,1,N(DROP(H2:H3,-1,0))))");
    model._set("A3", "=ROWS(OFFSET(B9:C12,5,1,{3}+0))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "1");
}

// A TAKE/DROP of a one-cell reference keeps the reference, as one of `E7:E7` does: under
// arithmetic it is a scalar, so a text cell makes the operator `#VALUE!` (not a 1x1 array), and
// ISREF sees a reference (probes/t3072-take-drop-one-cell-reference P1-P4, P42; T-3072).
#[test]
fn take_drop_of_a_one_cell_reference_is_a_reference() {
    let mut model = new_empty_model();
    model._set("E7", "a*");
    model._set("C8", "5");
    model._set("A1", "=ROWS(E9-TAKE(E7,2,42))");
    model._set("A2", "=ROWS(E9-DROP(E7,0))");
    model._set("A3", "=ROWS(TAKE(E7,1)+0)");
    model._set("A4", "=ISREF(TAKE(E7,1))");
    model._set("A5", "=CELL(\"address\",TAKE(E7,1))");
    model._set("A6", "=SUMXMY2(TAKE(C11,-1,1),B6:C10)");
    model._set("A7", "=SUMXMY2(TAKE(C8,1),B6:C10)");
    model._set("A8", "=TYPE(1+TAKE(C8,{1}))");
    model.add_sheet("Data").unwrap();
    model._set("Data!A1", "x");
    model._set("A9", "=ROWS(E9-TAKE(Data!A1,1))");
    model._set("A10", "=ISREF(DROP(Data!A1,0))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "TRUE");
    assert_eq!(model._get_text("A5"), "$E$7");
    assert_eq!(model._get_text("A6"), "#VALUE!");
    assert_eq!(model._get_text("A7"), "#N/A");
    assert_eq!(model._get_text("A8"), "64");
    assert_eq!(model._get_text("A9"), "#VALUE!");
    assert_eq!(model._get_text("A10"), "TRUE");
}
