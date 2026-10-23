#![allow(clippy::unwrap_used)]

//! What-if data tables (docs/excel-semantics/recalc.md "Data tables"; probes/data-tables, T-0441).

use crate::test::util::new_empty_model;
use crate::types::{DataTable, DataTableInput};

fn table(
    range: (i32, i32, i32, i32),
    row: Option<(i32, i32)>,
    column: Option<(i32, i32)>,
) -> DataTable {
    let cell = |c: Option<(i32, i32)>| c.map(|(r, c)| DataTableInput::Cell(r, c));
    DataTable {
        range,
        row_input: cell(row),
        column_input: cell(column),
    }
}

/// probes/data-tables: one-variable column and row tables, a two-variable table and a table whose
/// formula reads another formula; readers see the results and the input keeps its value.
#[test]
fn tables_substitute_their_inputs_and_readers_see_the_results() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "2"),
        ("B3", "=A1*10+B1"),
        ("A4", "1"),
        ("A5", "2"),
        ("A6", "3"),
        ("D7", "1"),
        ("E7", "2"),
        ("F7", "3"),
        ("C8", "=A1*10+B1"),
        ("A10", "=A1*10+B1"),
        ("B10", "100"),
        ("C10", "200"),
        ("D10", "300"),
        ("A11", "1"),
        ("A12", "2"),
        ("B14", "=B3*2"),
        ("A15", "5"),
        ("A16", "6"),
        ("H1", "=SUM(B4:B6)"),
        ("H6", "=SUM(B11:D12)"),
        ("H8", "=B3"),
        ("H9", "=SUM(B15:B16)"),
        ("H10", "=ISFORMULA(B5)"),
        ("H11", "=FORMULATEXT(B4)"),
        ("H12", "=FORMULATEXT(C12)"),
    ] {
        model._set(cell, value);
    }
    model
        .add_data_table(0, table((4, 2, 6, 2), None, Some((1, 1))))
        .unwrap();
    model
        .add_data_table(0, table((8, 4, 8, 6), Some((1, 1)), None))
        .unwrap();
    model
        .add_data_table(0, table((11, 2, 12, 4), Some((1, 2)), Some((1, 1))))
        .unwrap();
    model
        .add_data_table(0, table((15, 2, 16, 2), None, Some((1, 1))))
        .unwrap();
    model.evaluate();

    let texts = |cells: &[&str]| cells.iter().map(|c| model._get_text(c)).collect::<Vec<_>>();
    assert_eq!(texts(&["B4", "B5", "B6"]), ["12", "22", "32"]);
    assert_eq!(texts(&["D8", "E8", "F8"]), ["12", "22", "32"]);
    assert_eq!(
        texts(&["B11", "C11", "D11", "B12", "C12", "D12"]),
        ["110", "210", "310", "120", "220", "320"]
    );
    assert_eq!(texts(&["B15", "B16"]), ["104", "124"]);
    assert_eq!(
        texts(&["H1", "H6", "H8", "H9"]),
        ["66", "1290", "12", "228"]
    );
    assert_eq!(
        texts(&["H10", "H11", "H12"]),
        ["TRUE", "{=TABLE(,A1)}", "{=TABLE(B1,A1)}"]
    );
    assert_eq!(model._get_text("A1"), "1", "the input keeps its value");
}

/// A result recomputes when the model changes, and text and error values substitute too.
#[test]
fn results_follow_edits_and_carry_any_value_type() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("C1", "=LEN(A1)&\"/\"&ISERROR(A1)");
    model._set("B2", "abc");
    model._set("B3", "=1/0");
    model
        .add_data_table(0, table((2, 3, 3, 3), None, Some((1, 1))))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C2"), "3/FALSE");
    assert_eq!(
        model._get_text("C3"),
        "#DIV/0!",
        "an error input propagates"
    );
    model._set("C1", "=A1&\"!\"");
    model.evaluate();
    assert_eq!(model._get_text("C2"), "abc!");
    assert_eq!(model._get_text("C3"), "#DIV/0!");
}

/// A deleted input cell (`del1`) makes every result `#REF!`; a table needs an input, a row above
/// and a column left of its results, and may not overlap another.
#[test]
fn invalid_tables_are_refused_and_a_deleted_input_is_ref() {
    let mut model = new_empty_model();
    model._set("B1", "=1");
    let deleted = DataTable {
        range: (2, 2, 2, 2),
        row_input: None,
        column_input: Some(DataTableInput::Deleted),
    };
    model.add_data_table(0, deleted).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B2"), "#REF!");
    assert!(model
        .add_data_table(0, table((1, 2, 2, 2), None, Some((1, 1))))
        .is_err());
    assert!(model
        .add_data_table(0, table((3, 3, 3, 3), None, None))
        .is_err());
    assert!(model
        .add_data_table(0, table((2, 2, 3, 3), None, Some((5, 5))))
        .is_err());
    assert!(model
        .add_data_table(0, table((5, 5, 5, 5), None, Some((5, 5))))
        .is_err());
    assert!(model.remove_data_table(0, (2, 2, 2, 2)));
}
