//! A cell keeps the whitespace its formula was typed with: FORMULATEXT and the formula text print
//! it, structural edits keep it, and no value changes (docs/excel-semantics/syntax.md "Formula
//! text keeps typed whitespace…", SPEC-0023).

#![allow(clippy::unwrap_used)]

use crate::expressions::types::{Area, CellReferenceIndex};
use crate::test::util::new_empty_model;

#[test]
fn whitespace_prints_back_and_leaves_values_alone() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    let cases = [
        ("=1 + 2", "=1+2"),
        ("=SUM( B1:B2 )", "=SUM(B1:B2)"),
        ("=- 5 %", "=-5%"),
        ("=LET(x, 1, y, B2, x + y)", "=LET(x,1,y,B2,x+y)"),
        ("=LAMBDA(x, ROW(x))(B2)", "=LAMBDA(x,ROW(x))(B2)"),
        ("=ISREF( B1 )", "=ISREF(B1)"),
        ("=@ B1:B2", "=@B1:B2"),
        ("=ROWS( (B1, B2) )", "=ROWS((B1,B2))"),
    ];
    for (row, (spaced, bare)) in (1..).zip(cases) {
        model._set(&format!("C{row}"), spaced);
        model._set(&format!("D{row}"), bare);
        model._set(&format!("E{row}"), &format!("=FORMULATEXT(C{row})"));
    }
    model.evaluate();
    for (row, (spaced, _)) in (1..).zip(cases) {
        let (c, d) = (format!("C{row}"), format!("D{row}"));
        assert_eq!(model._get_text(&c), model._get_text(&d), "{spaced}");
        assert_eq!(model._get_formula(&c), spaced);
        assert_eq!(model._get_text(&format!("E{row}")), spaced);
    }
}

#[test]
fn edits_keep_the_whitespace() {
    let mut model = new_empty_model();
    model._set("B1", "4");
    model._set("A1", "=Sheet1!B1 + 1");
    model.evaluate();
    assert_eq!(model.insert_rows(0, 1, 1), Ok(()));
    assert_eq!(model.rename_sheet("Sheet1", "Data"), Ok(()));
    model.evaluate();
    assert_eq!(model._get_formula("A2"), "=Data!B2 + 1");
    assert_eq!(model._get_text("A2"), "5");

    let source = &CellReferenceIndex {
        sheet: 0,
        row: 2,
        column: 1,
    };
    let target = &CellReferenceIndex {
        sheet: 0,
        row: 3,
        column: 3,
    };
    let area = &Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 2,
        height: 2,
    };
    let moved = model.move_cell_value_to_area("=SUM( B2 ) + 1", source, target, area);
    assert_eq!(moved.unwrap(), "=SUM( D3 ) + 1");
}
