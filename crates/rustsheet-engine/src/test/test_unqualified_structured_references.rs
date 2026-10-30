#![allow(clippy::unwrap_used)]
//! A structured reference typed without its table name in a cell of the table names that table
//! (T-2979; docs/excel-semantics/references.md, "Structured references: this row, totals and
//! ranges").

use crate::Model;

/// `T` over A1:C4 (`Col` 1, 2, 4, `Other` 10, 20, 40, empty `Calc`) and `U` over E1:G4 with the
/// same column names (`Col` 100, 200, 400).
fn model() -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    for (row, col, other, u_col) in [
        (1, "Col", "Other", "Col"),
        (2, "1", "10", "100"),
        (3, "2", "20", "200"),
        (4, "4", "40", "400"),
    ] {
        model.set_user_input(0, row, 1, col.to_string()).unwrap();
        model.set_user_input(0, row, 2, other.to_string()).unwrap();
        model.set_user_input(0, row, 5, u_col.to_string()).unwrap();
        model.set_user_input(0, row, 6, other.to_string()).unwrap();
    }
    model.set_user_input(0, 1, 3, "Calc".to_string()).unwrap();
    model.set_user_input(0, 1, 7, "Calc".to_string()).unwrap();
    model
        .add_table_with_totals(0, "T", (1, 1, 4, 3), false, false)
        .unwrap();
    model
        .add_table_with_totals(0, "U", (1, 5, 4, 7), false, false)
        .unwrap();
    model
}

fn typed(model: &mut Model, row: i32, column: i32, text: &str) {
    model
        .set_typed_user_input(0, row, column, text.to_string())
        .unwrap();
    model.evaluate();
}

fn values(model: &Model, column: i32) -> Vec<String> {
    (2..=4)
        .map(|row| model.get_formatted_cell_value(0, row, column).unwrap())
        .collect()
}

/// probes/t2957-fill-one-totals: `=[@Col]*2` fills the column and prints as typed.
#[test]
fn this_row_shorthand_reads_its_own_table() {
    let mut model = model();
    typed(&mut model, 2, 3, "=[@Col]*2");
    assert_eq!(values(&model, 3), ["2", "4", "8"]);
    assert_eq!(
        model.get_cell_formula(0, 4, 3).unwrap(),
        Some("=[@Col]*2".to_string())
    );
    // The same text in `U` on the same sheet reads `U`, not `T`.
    typed(&mut model, 2, 7, "=[@Col]*2");
    assert_eq!(values(&model, 7), ["200", "400", "800"]);
    assert_eq!(values(&model, 3), ["2", "4", "8"]);
}

#[test]
fn column_and_this_row_forms() {
    let mut model = model();
    typed(&mut model, 3, 3, "=SUM([Col])");
    assert_eq!(values(&model, 3)[1], "7");
    typed(&mut model, 4, 3, "= [@Col]+1");
    assert_eq!(values(&model, 3)[2], "5");
    typed(&mut model, 3, 7, "=[[#This Row],[Col]]+1");
    assert_eq!(values(&model, 7)[1], "201");
    assert_eq!(
        model.get_cell_formula(0, 3, 7).unwrap(),
        Some("=[@Col]+1".to_string())
    );
}

/// A sheet rename re-parses every formula from its stored text, which keeps the table.
#[test]
fn survives_a_sheet_rename() {
    let mut model = model();
    typed(&mut model, 2, 3, "=[@Col]*2");
    typed(&mut model, 2, 7, "=[@Col]*2");
    model.rename_sheet("Sheet1", "Renamed").unwrap();
    model.evaluate();
    assert_eq!(values(&model, 3), ["2", "4", "8"]);
    assert_eq!(values(&model, 7), ["200", "400", "800"]);
    assert_eq!(
        model.get_cell_formula(0, 3, 7).unwrap(),
        Some("=[@Col]*2".to_string())
    );
}

#[test]
fn outside_every_table_it_is_refused() {
    let mut model = model();
    // A typed formula that does not parse is refused and the cell keeps its content (T-3213).
    let refused = model.set_typed_user_input(0, 9, 9, "=[@Col]*2".to_string());
    assert!(refused.is_err());
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 9, 9).unwrap(), "");
}
