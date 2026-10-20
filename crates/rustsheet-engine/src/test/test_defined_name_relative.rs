#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A relative name is stored relative to A1 and read at the same offset from the cell that uses
// it (docs/excel-semantics/references.md "Relative references in defined names", gen-m8a case
// 051f538767ab0cfb, T-1639). The bodies are spelled as xlsx stores them, with no `=`.
#[test]
fn relative_name_is_read_from_the_using_cell() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "Jan").unwrap();
    for (name, formula) in [
        ("rel_same", "Jan!A1"),
        ("rel_right", "Jan!B1"),
        ("rel_down", "Jan!A2"),
        ("rel_col_abs", "Jan!$A1"),
        ("rel_row_abs", "Jan!A$1"),
        ("rel_box", "Jan!A1:B2"),
        ("rel_col", "Jan!A:A"),
        ("f_rel_sum", "=rel_same+rel_right"),
    ] {
        model.define_name(name, None, formula).unwrap();
    }
    for (cell, value) in [
        ("A1", "1"),
        ("A6", "2"),
        ("K1", "10"),
        ("L1", "20"),
        ("K2", "30"),
        ("K6", "40"),
        ("K7", "50"),
        ("L6", "60"),
        ("L7", "70"),
    ] {
        model._set(&format!("Jan!{cell}"), value);
    }
    model._set("K1", "=rel_same+1");
    model._set("K2", "=rel_right");
    model._set("K3", "=rel_col_abs");
    model._set("K4", "=rel_row_abs");
    model._set("K5", "=f_rel_sum");
    model._set("K6", "=rel_down");
    model._set("M6", "=SUM(rel_box)");
    model._set("M7", "=ROWS(rel_box)*10+COLUMNS(rel_box)");
    model._set("K8", "=ISREF(rel_same)");
    model._set("K9", "=SHEET(rel_same)");
    model._set("K10", "=SUM(rel_col)");
    model._set("L6", "=SUM(INDIRECT(\"rel_box\"))");
    model._set("A6", "=rel_col_abs");
    model.evaluate();
    // From K1: Jan!K1; from K2: Jan!L2 (empty); from K3: Jan!$A3 (empty); from K4: Jan!K$1.
    assert_eq!(model._get_text("K1"), "11");
    assert_eq!(model._get_text("K2"), "0");
    assert_eq!(model._get_text("K3"), "0");
    assert_eq!(model._get_text("K4"), "10");
    // From K5: Jan!K5 + Jan!L5, both empty.
    assert_eq!(model._get_text("K5"), "0");
    // From K6: Jan!K7.
    assert_eq!(model._get_text("K6"), "50");
    // From M6: Jan!M6:N7, empty; from M7: still 2x2.
    assert_eq!(model._get_text("M6"), "0");
    assert_eq!(model._get_text("M7"), "22");
    assert_eq!(model._get_text("K8"), "TRUE");
    assert_eq!(model._get_text("K9"), "2");
    // From K10: Jan!K:K.
    assert_eq!(model._get_text("K10"), "130");
    // From L6: Jan!L6:M7.
    assert_eq!(model._get_text("L6"), "130");
    // From A6: Jan!$A6.
    assert_eq!(model._get_text("A6"), "2");
}

// An absolute name is the same cell wherever it is used.
#[test]
fn absolute_name_ignores_the_using_cell() {
    let mut model = new_empty_model();
    model.define_name("abs", None, "Sheet1!$A$1").unwrap();
    model._set("A1", "7");
    model._set("C5", "=abs*2");
    model.evaluate();
    assert_eq!(model._get_text("C5"), "14");
}
