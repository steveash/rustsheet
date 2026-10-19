//! SEQUENCE and MAKEARRAY lift their scalar arguments over a range or an array, one call per
//! element whose value is that call's top-left (SPEC-0011 "Array-valued functions", T-1834;
//! probes/m7-argument-modes, excel-graph, 2026-10-09).

use crate::test::util::new_empty_model;

fn model_with_a1_a3() -> crate::model::Model {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model._set(cell, value);
    }
    model
}

#[test]
fn sequence_lifts_over_a_range_and_an_array() {
    let mut model = model_with_a1_a3();
    // F1:H1, F2:H2, F3:H3, K9 of probes/m7-argument-modes.
    let cells = [
        ("F1", "=ROWS(SEQUENCE(A1:A3))", "3"),
        ("G1", "=COLUMNS(SEQUENCE(A1:A3))", "1"),
        ("H1", "=INDEX(SEQUENCE(A1:A3),1,1)", "1"),
        ("F2", "=ROWS(SEQUENCE({1;2;3}))", "3"),
        ("G2", "=COLUMNS(SEQUENCE({1;2;3}))", "1"),
        ("H2", "=INDEX(SEQUENCE({1;2;3}),1,1)", "1"),
        ("F3", "=ROWS(SEQUENCE(2,2,{1;10}))", "2"),
        ("G3", "=COLUMNS(SEQUENCE(2,2,{1;10}))", "1"),
        ("H3", "=INDEX(SEQUENCE(2,2,{1;10}),1,1)", "1"),
        ("K9", "=ROWS(INDEX(SEQUENCE({1;2;3}),3,1))", "1"),
        ("K11", "=ROWS(SEQUENCE({3;4}))", "2"),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), expected, "{cell} {formula}");
    }
}

#[test]
fn makearray_lifts_over_a_range() {
    let mut model = model_with_a1_a3();
    // F23:H23 and K10 of probes/m7-argument-modes; F48 and H48 keep passing.
    let cells = [
        ("F23", "=ROWS(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)))", "2"),
        ("G23", "=COLUMNS(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)))", "1"),
        ("H23", "=INDEX(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)),1,1)", "2"),
        (
            "K10",
            "=ROWS(INDEX(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)),2,1))",
            "1",
        ),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), expected, "{cell} {formula}");
    }
}

#[test]
fn unlifted_calls_keep_their_values() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(2,2,10,5)");
    model._set("D1", "=MAKEARRAY(2,2,LAMBDA(r,c,r*10+c))");
    model._set("G1", "=SEQUENCE(0)");
    model._set("H1", "=SEQUENCE(-1)");
    model.evaluate();
    for (cell, expected) in [("A1", "10"), ("B1", "15"), ("A2", "20"), ("B2", "25")] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
    for (cell, expected) in [("D1", "11"), ("E1", "12"), ("D2", "21"), ("E2", "22")] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
    assert_eq!(model._get_text("G1"), "#CALC!");
    assert_eq!(model._get_text("H1"), "#VALUE!");
}

#[test]
fn an_element_error_stays_in_its_position() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE({1;0})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "#CALC!");
}

/// Each lifted element is charged the cells it builds (`KernelWork::Output`), so a lifted call
/// past `limits::MAX_LIFTED_CELL_VISITS` is `#ERROR!` (DEVIATION, docs/excel-semantics/arrays.md
/// "Array results above 1,048,576 cells").
#[test]
fn lifted_output_is_charged_against_the_visit_cap() {
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(SEQUENCE(SEQUENCE(1024),1024))");
    model._set("B1", "=ROWS(MAKEARRAY(SEQUENCE(2048),1024,LAMBDA(r,c,r)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("B1"), "#ERROR!");
}
