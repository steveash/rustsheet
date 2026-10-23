//! EXPAND, WRAPROWS and WRAPCOLS lift their scalar arguments over a range or an array, one call
//! per element whose value is that call's top-left (SPEC-0011 "Array-valued functions", T-1835;
//! probes/m7-argument-modes, probes/m7-lift-take-drop-sortby, excel-graph, 2026-10-09/21).

use crate::test::util::new_empty_model;

fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model._set(cell, value);
    }
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell} {formula}");
    }
}

#[test]
fn expand_lifts_rows_and_columns_but_not_pad_with() {
    check(&[
        ("F7", "=ROWS(EXPAND(A1:A2,{3;4}))", "2"),
        ("G7", "=COLUMNS(EXPAND(A1:A2,{3;4}))", "1"),
        ("H7", "=INDEX(EXPAND(A1:A2,{3;4}),1,1)", "1"),
        ("K5", "=ROWS(INDEX(EXPAND(A1:A2,{3;4}),1,1))", "1"),
        ("F20", "=ROWS(EXPAND(A1:A2,3,{1;2}))", "2"),
        ("F21", "=ROWS(EXPAND(A1:A2,3,1,{0;1}))", "#VALUE!"),
        ("G21", "=COLUMNS(EXPAND(A1:A2,3,1,{0;1}))", "#VALUE!"),
    ]);
}

/// A multi-cell range `pad_with` is one `#VALUE!` with nothing spilled, as an array one is; a
/// one-cell range pads with its value (gen-m7/2d9b48c6363e7a16 A366, A830, A2686; T-2415).
#[test]
fn expand_range_pad_with() {
    check(&[
        ("C1", "=INDEX(EXPAND(A1,2,1,A3:A3),2,1)", "3"),
        ("E1", "=EXPAND(A1,2,1,A2:A3)", "#VALUE!"),
        ("F1", "=EXPAND(A1,1,1,A1:B3)", "#VALUE!"),
        ("G1", "=ROWS(EXPAND(A1:A2,3,2,A1:B1))", "#VALUE!"),
        ("I2", "=INDEX(EXPAND(A1,2,2,A3:A3),2,2)", "3"),
    ]);
}

#[test]
fn wrap_functions_lift_wrap_count_and_pad_with() {
    check(&[
        ("F8", "=ROWS(WRAPROWS(A1:A3,{1;2}))", "2"),
        ("G8", "=COLUMNS(WRAPROWS(A1:A3,{1;2}))", "1"),
        ("K7", "=COLUMNS(INDEX(WRAPROWS(A1:A3,{1;2}),2,1))", "1"),
        ("G22", "=COLUMNS(WRAPROWS(A1:A3,2,{0;1}))", "1"),
        ("G42", "=COLUMNS(WRAPCOLS(A1:A3,2,{0;1}))", "1"),
        ("F23", "=ROWS(WRAPCOLS(A1:A3,{1;2}))", "2"),
    ]);
}

#[test]
fn unlifted_calls_are_unchanged() {
    check(&[
        ("B1", "=ROWS(EXPAND(A1:A2,3,2,0))", "3"),
        ("B2", "=COLUMNS(WRAPROWS(A1:A3,2,0))", "2"),
        ("B3", "=INDEX(WRAPCOLS(A1:A3,2,9),2,2)", "9"),
        ("B4", "=WRAPROWS(A1:A3,0)", "#NUM!"),
        ("B5", "=EXPAND(A1:A2,1)", "#VALUE!"),
    ]);
}

#[test]
fn a_lifted_element_error_stays_in_its_position() {
    // The second element is wrap_count 0, `#NUM!`; the first is its call's top-left.
    check(&[
        ("B1", "=INDEX(WRAPROWS(A1:A3,{2;0}),1,1)", "1"),
        ("B2", "=INDEX(WRAPROWS(A1:A3,{2;0}),2,1)", "#NUM!"),
    ]);
}

#[test]
fn a_lifted_call_is_bounded() {
    // SPEC-0011 rule 3: 1024 elements of 1024 x 1 cells is past the cap.
    check(&[("A9", "=ROWS(EXPAND(1,SEQUENCE(1024),1024))", "#ERROR!")]);
}

#[test]
fn a_vector_error_fills_the_lifted_shape() {
    // The vector's `#VALUE!` comes before wrap_count's `#NUM!` and is every element's result
    // (gen-m7/90de8755a2e565e6 A889:D894); unlifted, it is the one error. Hand-written: Excel's
    // cell there has an all-`#VALUE!` pad_with, so this clean pad_with is UNVERIFIED (T-2365).
    check(&[
        ("B1", "=ROWS(WRAPROWS(SIGN(\"x\"),-1,{1,2;3,4}))", "2"),
        ("B2", "=COLUMNS(WRAPROWS(SIGN(\"x\"),-1,{1,2;3,4}))", "2"),
        (
            "B3",
            "=INDEX(WRAPROWS(SIGN(\"x\"),-1,{1,2;3,4}),2,2)",
            "#VALUE!",
        ),
        ("B4", "=WRAPCOLS(SIGN(\"x\"),-1)", "#VALUE!"),
    ]);
}
