#![allow(clippy::unwrap_used)]

//! Written parentheses are transparent to evaluation (SPEC-0007, T-1340): a group around an
//! argument or operand gives the value the bare form gives, reference-only shape tests included
//! (probes/written-parens-reference-context, probes/written-parens-reference-only), and the
//! formula text keeps them through edits.

use crate::expressions::types::{Area, CellReferenceIndex};
use crate::test::util::new_empty_model;

#[test]
fn a_group_gives_the_value_of_the_bare_form() {
    let mut model = new_empty_model();
    model._set("A1", "text");
    model._set("A2", "2");
    model._set("A3", "4");
    model._set("B1", "1");
    let pairs = [
        ("COUNT((A1))", "COUNT(A1)"),
        ("COUNTA(((A1)))", "COUNTA(A1)"),
        ("AVERAGE((A1),(A2))", "AVERAGE(A1,A2)"),
        ("MAX((A1),3)", "MAX(A1,3)"),
        ("AND((B1),TRUE)", "AND(B1,TRUE)"),
        ("ISREF((A1))", "ISREF(A1)"),
        ("ROWS(((A1:A3)))", "ROWS(A1:A3)"),
        ("SUM(((A2,A3)))", "SUM((A2,A3))"),
        ("LET(x,(A1:A3),ROWS(x))", "LET(x,A1:A3,ROWS(x))"),
        ("PRODUCT((A2))", "PRODUCT(A2)"),
        ("CONCAT((A1),(A2))", "CONCAT(A1,A2)"),
        ("(A1:A3) (A2)", "A1:A3 A2"),
        ("(A2)+((A3))*2", "A2+A3*2"),
        ("-(A2)", "-A2"),
    ];
    for (row, (grouped, bare)) in (1..).zip(pairs) {
        model._set(&format!("D{row}"), &format!("={grouped}"));
        model._set(&format!("E{row}"), &format!("={bare}"));
    }
    model.evaluate();
    for (row, (grouped, _)) in (1..).zip(pairs) {
        let value = model._get_text(&format!("D{row}"));
        assert_eq!(value, model._get_text(&format!("E{row}")), "{grouped}");
        assert_eq!(
            model._get_formula(&format!("D{row}")),
            format!("={grouped}")
        );
    }
}

#[test]
fn edits_keep_the_written_groups() {
    let mut model = new_empty_model();
    model._set("A1", "=((B1+C1)+D1)*(E1)");
    model.evaluate();
    model.insert_rows(0, 1, 1).unwrap();
    model.rename_sheet("Sheet1", "Renamed").unwrap();
    assert_eq!(model._get_formula("A2"), "=((B2+C2)+D2)*(E2)");
    let cell = |row, column| CellReferenceIndex {
        sheet: 0,
        row,
        column,
    };
    let area = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 5,
        height: 5,
    };
    let moved = model
        .move_cell_value_to_area("=(B1)^2^(C1)", &cell(1, 1), &cell(10, 10), &area)
        .unwrap();
    assert_eq!(moved, "=(K10)^2^(L10)");
}
