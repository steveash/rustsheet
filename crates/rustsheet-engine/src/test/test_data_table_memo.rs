#![allow(clippy::unwrap_used, reason = "tests")]

//! A data table is recomputed only when something it read changed (`data_table_memo.rs`, T-0453),
//! and its results always match a recomputation.

use crate::model::Model;
use crate::test::util::new_empty_model;
use crate::types::{DataTable, DataTableInput};

/// A column table over B2:B4 with input A1 and formula B1, after `cells` are set and the
/// workbook evaluated once.
fn column_table(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    model._set("A1", "1");
    for (cell, value) in [("A2", "1"), ("A3", "2"), ("A4", "3"), ("H1", "=SUM(B2:B4)")] {
        model._set(cell, value);
    }
    for (cell, value) in cells {
        model._set(cell, value);
    }
    let table = DataTable {
        range: (2, 2, 4, 2),
        row_input: None,
        column_input: Some(DataTableInput::Cell(1, 1)),
    };
    model.add_data_table(0, table).unwrap();
    model.evaluate();
    model
}

/// Sets `cell` to `value`, evaluates, and returns whether the table was recomputed.
fn edit(model: &mut Model, cell: &str, value: &str) -> bool {
    let before = model.evaluation.data_tables_recomputed;
    model._set(cell, value);
    model.evaluate();
    model.evaluation.data_tables_recomputed != before
}

fn results(model: &Model) -> Vec<String> {
    ["B2", "B3", "B4", "H1"]
        .iter()
        .map(|c| model._get_text(c))
        .collect()
}

#[test]
fn an_edit_the_table_does_not_read_keeps_its_results() {
    let mut model = column_table(&[("C1", "5"), ("B1", "=A1*C1+D1")]);
    assert_eq!(results(&model), ["5", "10", "15", "30"]);
    assert!(!edit(&mut model, "Z9", "7"), "Z9 is read by nobody");
    assert!(
        !edit(&mut model, "H2", "=H1+1"),
        "H2 only reads the results"
    );
    assert_eq!(results(&model), ["5", "10", "15", "30"]);
    assert_eq!(model._get_text("H2"), "31");
}

#[test]
fn an_edit_to_a_cell_the_formula_reads_recomputes() {
    let mut model = column_table(&[("C1", "5"), ("C2", "=C1"), ("B1", "=A1*C2+D1")]);
    assert!(edit(&mut model, "C1", "6"), "read through C2");
    assert_eq!(results(&model), ["6", "12", "18", "36"]);
    assert!(edit(&mut model, "D1", "1"), "D1 was read as empty");
    assert_eq!(results(&model), ["7", "13", "19", "39"]);
    assert!(edit(&mut model, "C2", "=C1*2"), "a new formula");
    assert_eq!(results(&model), ["13", "25", "37", "75"]);
    assert!(edit(&mut model, "A3", "5"), "a header");
    assert_eq!(results(&model), ["13", "61", "37", "111"]);
}

#[test]
fn an_edit_in_a_range_the_formula_reads_recomputes() {
    let mut model = column_table(&[("B1", "=A1+SUM(D1:D100)+SUM(E:E)+COUNTA(F:F)")]);
    assert_eq!(results(&model), ["1", "2", "3", "6"]);
    assert!(
        edit(&mut model, "D50", "10"),
        "an empty position of D1:D100"
    );
    assert_eq!(results(&model), ["11", "12", "13", "36"]);
    assert!(edit(&mut model, "E5000", "1"), "below the used area of E:E");
    assert_eq!(results(&model), ["12", "13", "14", "39"]);
    assert!(edit(&mut model, "F1", "x"), "a text in F:F");
    assert_eq!(results(&model), ["13", "14", "15", "42"]);
    assert!(!edit(&mut model, "G7", "1"));
}

#[test]
fn indirect_reads_are_followed() {
    let mut model = column_table(&[("C1", "D7"), ("B1", "=A1+INDIRECT(C1)")]);
    assert!(edit(&mut model, "D7", "10"));
    assert_eq!(results(&model), ["11", "12", "13", "36"]);
    assert!(edit(&mut model, "C1", "D8"));
    assert_eq!(results(&model), ["1", "2", "3", "6"]);
    assert!(!edit(&mut model, "D7", "20"), "D7 is no longer read");
    assert!(edit(&mut model, "D8", "1"));
    assert_eq!(results(&model), ["2", "3", "4", "9"]);
}

#[test]
fn a_changed_name_recomputes() {
    let mut model = new_empty_model();
    model.new_defined_name("k", None, "Sheet1!$C$1").unwrap();
    let mut model = {
        model._set("C1", "2");
        model._set("B1", "=A1*k");
        model._set("A1", "1");
        for (cell, value) in [("A2", "1"), ("A3", "2"), ("A4", "3"), ("H1", "=SUM(B2:B4)")] {
            model._set(cell, value);
        }
        let table = DataTable {
            range: (2, 2, 4, 2),
            row_input: None,
            column_input: Some(DataTableInput::Cell(1, 1)),
        };
        model.add_data_table(0, table).unwrap();
        model.evaluate();
        model
    };
    assert_eq!(results(&model), ["2", "4", "6", "12"]);
    model._set("D1", "3");
    model
        .update_defined_name("k", None, "k", None, "Sheet1!$D$1")
        .unwrap();
    model.evaluate();
    assert_eq!(results(&model), ["3", "6", "9", "18"]);
}

#[test]
fn a_hidden_row_recomputes() {
    let mut model = column_table(&[("D1", "1"), ("D2", "2"), ("B1", "=A1+SUBTOTAL(109,D1:D2)")]);
    assert_eq!(results(&model), ["4", "5", "6", "15"]);
    model.set_row_hidden(0, 2, true).unwrap();
    model.evaluate();
    assert_eq!(results(&model), ["2", "3", "4", "9"]);
}

#[test]
fn a_volatile_formula_always_recomputes() {
    let mut model = column_table(&[("C1", "=RAND()"), ("B1", "=A1+(C1>2)")]);
    assert!(edit(&mut model, "Z9", "1"));
    let mut model = column_table(&[("B1", "=A1+(NOW()<0)")]);
    assert!(edit(&mut model, "Z9", "1"));
}

#[test]
fn an_overwritten_result_is_recomputed() {
    let mut model = column_table(&[("B1", "=A1*2")]);
    assert!(edit(&mut model, "B3", "99"));
    assert_eq!(results(&model), ["2", "4", "6", "12"]);
}

/// The texts of the first ten rows of columns A to H.
fn sheet_texts(model: &Model) -> Vec<String> {
    (1..=10)
        .flat_map(|row| ('A'..='H').map(move |column| format!("{column}{row}")))
        .map(|cell| model._get_text(&cell))
        .collect()
}

/// Rows inserted above a table move the cells it reads (the table stays where it is): its
/// results are those of a recomputation from scratch.
#[test]
fn after_a_structural_edit_the_results_are_a_recomputation() {
    let mut model = column_table(&[("C1", "5"), ("B1", "=A1*C1")]);
    model.insert_rows(0, 1, 1).unwrap();
    model.evaluate();
    let memoized = sheet_texts(&model);
    model.evaluation.data_table_memo = Default::default();
    model.evaluate();
    assert_eq!(memoized, sheet_texts(&model));
}
