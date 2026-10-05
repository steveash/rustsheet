#![allow(clippy::unwrap_used)]

//! The intersection operator (a space between two references), T-0105. Expected values are
//! Excel's, from gen-m1 (oracle excel-graph, 2026-10-05); see docs/excel-semantics/references.md
//! "Intersection operator (space)", "Disjoint intersection is #NULL!" and "Cross-sheet
//! intersection is #VALUE!".

use crate::model::Model;
use crate::test::util::new_empty_model;

/// Sheet1 (the formulas), `Data` and `My Data`, and `Q'1`, with a few values.
fn model() -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model.add_sheet("My Data").unwrap();
    model.add_sheet("Q'1").unwrap();
    model._set("Data!C7", "0.5");
    model._set("Data!C6", "TRUE");
    model._set("Data!C9", "7");
    model._set("My Data!C9", "8");
    model._set("Data!B8", "a");
    model._set("Data!A11", "1");
    model._set("Data!A12", "12");
    model._set("Data!E8", "=1/0");
    model._set("Q'1!E7", "100");
    for (cell, value) in [("Data!B2", "1"), ("Data!C2", "2"), ("Data!B3", "3")] {
        model._set(cell, value);
    }
    model
}

#[test]
fn intersection_gives_the_shared_cell() {
    let mut model = model();
    model._set("A1", "=Data!A1:C7 Data!C7:E12");
    model._set("A2", "=Data!C1:D7 Data!C1:C12 Data!A6:E6");
    model._set("A3", "=Data!C8:B3 Data!B8");
    model._set("A4", "=Data!$A$10:$D$12 Data!A12");
    model._set("A5", "=Data!A7:A7:Data!C11 Data!A11");
    model._set("A6", "='Q''1'!7:7 'Q''1'!E7");
    model._set("A7", "=Data!B:B Data!8:8");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.5");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "a");
    assert_eq!(model._get_text("A4"), "12");
    assert_eq!(model._get_text("A5"), "1");
    assert_eq!(model._get_text("A6"), "100");
    assert_eq!(model._get_text("A7"), "a");
}

#[test]
fn intersection_of_an_empty_cell_is_zero_or_empty_text() {
    let mut model = model();
    model._set("A1", "=Data!Z:Z Data!290:290");
    model._set("A2", "=(Data!D:D Data!2:2)&\"\"");
    model._set("A3", "=Data!D:D Data!2:2&\"x\"");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "0");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("A3"), "x");
}

#[test]
fn intersection_binds_tighter_than_arithmetic() {
    let mut model = model();
    model._set("A1", "=(Data!C2:C11 Data!C6:E6)+0.1");
    model._set("A2", "=Data!C2:C11 Data!C9:E9+0.1");
    model._set("A3", "=-Data!C2:C11 Data!C9:E9");
    model._set("A4", "=Data!C2:C11 Data!C9:E9^2");
    model.evaluate();

    // TRUE + 0.1 = 1.1 (gen-m1/4dc8ed1de05a61be Calc!A3).
    assert_eq!(model._get_text("A1"), "1.1");
    assert_eq!(model._get_text("A2"), "7.1");
    assert_eq!(model._get_text("A3"), "-7");
    assert_eq!(model._get_text("A4"), "49");
}

#[test]
fn disjoint_intersection_is_null_error() {
    let mut model = model();
    model._set("A1", "=Data!A3:D4 Data!Z99");
    model._set("A2", "=Data!B4:E11 Data!A1:A3");
    model._set("A3", "=(Data!B4:E11 Data!A1:A3)+1");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NULL!");
    assert_eq!(model._get_text("A2"), "#NULL!");
    assert_eq!(model._get_text("A3"), "#NULL!");
}

#[test]
fn cross_sheet_intersection_is_value_error() {
    let mut model = model();
    // The coordinates overlap (C9), but the areas are on different sheets.
    model._set("A1", "=Data!C5:C10 'My Data'!C9:D9");
    model._set("A2", "=C9 Data!C9");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
}

#[test]
fn intersection_propagates_errors() {
    let mut model = model();
    model._set("A1", "=Data!D2:E8 Data!E8:E12");
    model._set("A2", "=(1/0) Data!E8");
    model._set("A3", "=Data!E8 #REF!");
    model._set("A4", "=(1+0) Data!E8");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#DIV/0!");
    assert_eq!(model._get_text("A2"), "#DIV/0!");
    assert_eq!(model._get_text("A3"), "#REF!");
    // A value that is not a reference cannot be intersected.
    assert_eq!(model._get_text("A4"), "#VALUE!");
}

#[test]
fn intersection_is_a_reference() {
    let mut model = model();
    model._set("A1", "=SUM(Data!A1:C3 Data!B2:D4)");
    model._set("A2", "=ISREF(Data!A1:C3 Data!B2:D4)");
    model._set("A3", "=ROWS(Data!A1:C3 Data!B2:D4)");
    model._set("A4", "=SUM(B10:C11 C11:D12)");
    model._set("C11", "5");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "5");
}

#[test]
fn multi_cell_intersection_spills_like_a_range() {
    let mut model = model();
    model._set("F1", "=Data!A1:C3 Data!B2:D4");
    model.evaluate();

    assert_eq!(model._get_text("F1"), "1");
    assert_eq!(model._get_text("G1"), "2");
    assert_eq!(model._get_text("F2"), "3");
}

#[test]
fn intersection_formula_text_round_trips() {
    let mut model = model();
    model._set("A1", "=Data!A1:C7 Data!C7:E12");
    model._set("A2", "=(Data!C2:C11 Data!C6:E6)+0.1");
    model._set("A3", "=B1:C3 (C1:C2 C2)");
    model.evaluate();

    assert_eq!(model._get_formula("A1"), "=Data!A1:C7 Data!C7:E12");
    assert_eq!(model._get_formula("A2"), "=Data!C2:C11 Data!C6:E6+0.1");
    assert_eq!(model._get_formula("A3"), "=B1:C3 (C1:C2 C2)");
}

#[test]
fn whitespace_between_other_tokens_keeps_its_meaning() {
    let mut model = model();
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A1", "= 1 + 2");
    model._set("A2", "=SUM( B1 , C1 )");
    model._set("A3", "=( B1 )*( C1 )");
    model._set("A4", "=SUM( { 1 , 2 } )");
    model._set("A5", "=B1 -C1");
    model._set("A6", "=B1 & C1");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "5");
    assert_eq!(model._get_text("A3"), "6");
    assert_eq!(model._get_text("A4"), "3");
    assert_eq!(model._get_text("A5"), "-1");
    assert_eq!(model._get_text("A6"), "23");
}

#[test]
fn trailing_input_is_an_error() {
    let mut model = model();
    // Before T-0105 the parser stopped after the first complete expression and dropped the
    // rest: `=1 2` evaluated to 1 and `=SUM(B1))` to SUM(B1).
    model._set("A1", "=1 2");
    model._set("A2", "=SUM(B1))");
    model._set("A3", "=\"a\" \"b\"");
    model._set("A4", "=B1 1");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), "#ERROR!", "{cell}");
    }
}
