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

/// The `@` form is decided by the printing cell (T-3241): `T[[#This Row],[Col]]` and `T[@Col]`
/// print `[@Col]` in a data row of `T` (probes/t2800-this-row-whitespace B3:E3, padding kept),
/// and the qualified form in a cell of `U` or outside every table.
#[test]
fn a_qualified_this_row_prints_in_its_at_form_inside_its_table() {
    let formula = |model: &Model, row, column| model.get_cell_formula(0, row, column).unwrap();
    for (typed_text, printed) in [
        ("=T[[#This Row],[Col]]*2", "=[@Col]*2"),
        ("=T[@Col]*2", "=[@Col]*2"),
        ("=T[ [#This Row] , [Col] ]*2", "=[ @Col ]*2"),
    ] {
        let mut model = model();
        // Typed into the empty column, it fills the column (a calculated column).
        typed(&mut model, 2, 3, typed_text);
        assert_eq!(values(&model, 3), ["2", "4", "8"], "{typed_text}");
        for row in 2..=4 {
            assert_eq!(
                formula(&model, row, 3),
                Some(printed.to_string()),
                "{typed_text}"
            );
        }
        typed(&mut model, 2, 9, "=FORMULATEXT(C3)");
        assert_eq!(model.get_formatted_cell_value(0, 2, 9).unwrap(), printed);
        // In `U` the reference to `T` keeps its table name.
        typed(&mut model, 2, 7, "=T[@Col]+1");
        assert_eq!(
            formula(&model, 2, 7),
            Some("=T[[#This Row],[Col]]+1".to_string())
        );
    }
}

/// A this-row formula of `T`, with or without the table name, copied or filled to a cell outside
/// `T` prints with the name there, so the text parses at the target (T-3241).
#[test]
fn copied_or_filled_out_of_the_table_it_keeps_the_table_name() {
    let mut model = model();
    typed(&mut model, 2, 3, "=[@Col]*2");
    typed(&mut model, 3, 3, "=T[@Col]*2");
    let qualified = "=T[[#This Row],[Col]]*2";
    for row in [2, 3] {
        // Filled right, past the table, on the same row.
        assert_eq!(model.extend_to(0, row, 3, row, 9).unwrap(), qualified);
        // Filled down, below the table.
        assert_eq!(model.extend_to(0, row, 3, row + 6, 3).unwrap(), qualified);
        // Filled down inside the table keeps the `@` form.
        assert_eq!(model.extend_to(0, row, 3, 4, 3).unwrap(), "=[@Col]*2");
    }
    let source = crate::expressions::types::CellReferenceIndex {
        sheet: 0,
        row: 2,
        column: 3,
    };
    let target = crate::expressions::types::CellReferenceIndex {
        sheet: 0,
        row: 3,
        column: 9,
    };
    let copied = model
        .extend_copied_value("=[@Col]*2", &source, &target)
        .unwrap();
    assert_eq!(copied, qualified);
    // The copied text works where it lands: row 3 of `T` reads `Col` 2.
    typed(&mut model, 3, 9, &copied);
    assert_eq!(model.get_formatted_cell_value(0, 3, 9).unwrap(), "4");
    assert_eq!(
        model.get_cell_formula(0, 3, 9).unwrap(),
        Some(qualified.to_string())
    );
    // A column reference written without the name gets it outside the table too.
    let copied = model
        .extend_copied_value("=SUM([Col])", &source, &target)
        .unwrap();
    assert_eq!(copied, "=SUM(T[Col])");
}

/// A table shrunk so a this-row formula's cell falls outside it prints the formula with the
/// table name (T-3241); the formula keeps reading the table.
#[test]
fn a_resize_that_leaves_the_cell_outside_prints_the_table_name() {
    let mut model = model();
    typed(&mut model, 4, 2, "=[@Col]*2");
    typed(&mut model, 3, 2, "=T[@Col]*2");
    model.remove_table("T").unwrap();
    model
        .add_table_with_totals(0, "T", (1, 1, 4, 1), false, false)
        .unwrap();
    model.evaluate();
    let formula = |row| model.get_cell_formula(0, row, 2).unwrap();
    assert_eq!(formula(4), Some("=T[[#This Row],[Col]]*2".to_string()));
    assert_eq!(formula(3), Some("=T[[#This Row],[Col]]*2".to_string()));
    assert_eq!(model.get_formatted_cell_value(0, 3, 2).unwrap(), "4");
}
