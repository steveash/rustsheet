#![allow(clippy::unwrap_used)]
//! A formula typed into a table column fills it as a calculated column (T-2958;
//! docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table column").

use crate::Model;

/// `T` over A1:C4 (A1:C5 with a totals row): `Col` 1, 2, 4, `Other` 10, 20, 40 and an empty
/// `Calc` column, as in the probes/t2957-* cases.
fn model(totals_row: bool) -> Model {
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    for (row, col, other) in [
        (1, "Col", "Other"),
        (2, "1", "10"),
        (3, "2", "20"),
        (4, "4", "40"),
    ] {
        model.set_user_input(0, row, 1, col.to_string()).unwrap();
        model.set_user_input(0, row, 2, other.to_string()).unwrap();
    }
    model.set_user_input(0, 1, 3, "Calc".to_string()).unwrap();
    let bottom = if totals_row { 5 } else { 4 };
    model
        .add_table_with_totals(0, "T", (1, 1, bottom, 3), false, totals_row)
        .unwrap();
    model
}

fn typed(model: &mut Model, row: i32, text: &str) {
    model
        .set_typed_user_input(0, row, 3, text.to_string())
        .unwrap();
    model.evaluate();
}

/// The `Calc` cells of rows 2..=`last`, formatted.
fn calc(model: &Model, last: i32) -> Vec<String> {
    (2..=last)
        .map(|row| model.get_formatted_cell_value(0, row, 3).unwrap())
        .collect()
}

/// probes/t2957-fill-one-totals: every data row, not the totals row.
#[test]
fn a_typed_formula_fills_the_data_rows() {
    let mut model = model(true);
    typed(&mut model, 3, "=T[@Col]*2");
    assert_eq!(calc(&model, 5), ["2", "4", "8", ""]);
    // Every filled row prints the typed cell's text (references.md rule 6).
    let typed_text = model.get_cell_formula(0, 3, 3).unwrap();
    assert!(typed_text.is_some());
    assert_eq!(model.get_cell_formula(0, 2, 3).unwrap(), typed_text);
    assert_eq!(model.get_cell_formula(0, 4, 3).unwrap(), typed_text);
    assert_eq!(model.get_cell_formula(0, 5, 3).unwrap(), None);
}

#[test]
fn a_stored_formula_does_not_fill() {
    let mut model = model(false);
    model
        .set_user_input(0, 2, 3, "=T[@Col]*2".to_string())
        .unwrap();
    model
        .set_loaded_user_input(0, 3, 3, "=T[@Col]*2".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(calc(&model, 4), ["2", "4", ""]);
}

/// probes/t2957-refill-second, -constant-exception, -constant-then-refill.
#[test]
fn a_filled_column_refills_unless_it_holds_an_exception() {
    let mut model = model(false);
    typed(&mut model, 2, "=T[@Col]*2");
    typed(&mut model, 3, "=T[@Col]*3");
    assert_eq!(calc(&model, 4), ["3", "6", "12"]);
    typed(&mut model, 3, "5");
    assert_eq!(calc(&model, 4), ["3", "5", "12"]);
    typed(&mut model, 4, "=T[@Col]*4");
    assert_eq!(calc(&model, 4), ["3", "5", "16"]);
}

/// probes/t2957-existing-constant: no fill beside a value.
#[test]
fn no_fill_when_another_data_cell_holds_a_value() {
    let mut model = model(false);
    model.set_user_input(0, 2, 3, "5".to_string()).unwrap();
    typed(&mut model, 3, "=T[@Col]*2");
    assert_eq!(calc(&model, 4), ["5", "4", ""]);
}

/// A blank cell beside formulas is no calculated column either (the column's other cells must
/// be all blank or all the one formula). An engine choice, unprobed in Excel (T-2983).
#[test]
fn no_fill_over_formulas_and_blanks_mixed() {
    let mut model = model(false);
    typed(&mut model, 2, "=T[@Col]*2");
    typed(&mut model, 3, "");
    typed(&mut model, 4, "=T[@Col]*3");
    assert_eq!(calc(&model, 4), ["2", "", "12"]);
}

/// Outside a table, a typed formula stays in its cell.
#[test]
fn no_fill_outside_a_table() {
    let mut model = model(false);
    model
        .set_typed_user_input(0, 2, 5, "=1+1".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 3, 5).unwrap(), "");
}

/// `COLUMNS(T[#All])`, `ROWS(T[#All])`, D1's text and `ISFORMULA(D1)` after the typing, as the
/// probes/t2957-expand-right-* cases read them in H1:H4.
fn expansion_readout(model: &mut Model) -> Vec<String> {
    for (row, formula) in [
        (1, "=COLUMNS(T[#All])"),
        (2, "=ROWS(T[#All])"),
        (3, "=D1&\"\""),
        (4, "=ISFORMULA(D1)"),
    ] {
        model
            .set_user_input(0, row, 8, formula.to_string())
            .unwrap();
    }
    model.evaluate();
    (1..=4)
        .map(|row| model.get_formatted_cell_value(0, row, 8).unwrap())
        .collect()
}

/// The new column D's cells of rows 2..=4, formatted.
fn column_d(model: &Model) -> Vec<String> {
    (2..=4)
        .map(|row| model.get_formatted_cell_value(0, row, 4).unwrap())
        .collect()
}

/// probes/t2957-expand-right-data: `=A2*2` typed into D2, just right of `T` A1:C4, grows `T` to
/// A1:D4, names the new column `Column1` and fills it shifted, like a fill-down.
#[test]
fn a_formula_typed_beside_a_data_row_grows_the_table() {
    let mut model = model(false);
    model
        .set_typed_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(column_d(&model), ["2", "4", "8"]);
    assert_eq!(
        model.get_cell_formula(0, 4, 4).unwrap().as_deref(),
        Some("=A4*2")
    );
    assert_eq!(
        expansion_readout(&mut model),
        ["4", "4", "Column1", "FALSE"]
    );
    // The new column is a column of the table by its name.
    model
        .set_user_input(0, 6, 1, "=SUM(T[Column1])".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "14");
}

/// probes/t2957-expand-right-header: `=1+1` typed into D1, just right of the header row, grows
/// `T` too, but D1 holds the text `0` (the column's name) and the data rows stay blank.
#[test]
fn a_formula_typed_beside_the_header_row_names_the_new_column_0() {
    let mut model = model(false);
    model
        .set_typed_user_input(0, 1, 4, "=1+1".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_cell_formula(0, 1, 4).unwrap(), None);
    assert_eq!(column_d(&model), ["", "", ""]);
    assert_eq!(expansion_readout(&mut model), ["4", "4", "0", "FALSE"]);
}

/// A stored formula beside a table (xlsx import) does not grow it, nor does a typed constant, a
/// formula beside a table with a totals row, or one whose new column already holds a value
/// (each unprobed in Excel, so the cell is written as before).
#[test]
fn no_expansion_for_stored_formulas_constants_totals_or_occupied_columns() {
    let mut stored = model(false);
    stored
        .set_loaded_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    stored.set_user_input(0, 3, 4, "=A3*2".to_string()).unwrap();
    assert_eq!(expansion_readout(&mut stored)[0], "3");
    assert_eq!(column_d(&stored), ["2", "4", ""]);

    let mut constant = model(false);
    constant
        .set_typed_user_input(0, 2, 4, "5".to_string())
        .unwrap();
    assert_eq!(expansion_readout(&mut constant)[0], "3");

    let mut totals = model(true);
    totals
        .set_typed_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    totals.evaluate();
    assert_eq!(column_d(&totals), ["2", "", ""]);

    let mut occupied = model(false);
    occupied.set_user_input(0, 4, 4, "7".to_string()).unwrap();
    occupied
        .set_typed_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    assert_eq!(expansion_readout(&mut occupied)[0], "3");
    assert_eq!(column_d(&occupied), ["2", "", "7"]);
}

/// A formula that does not parse as typed grows no table, beside the header row or a data row:
/// Excel refuses the entry and changes nothing (a parse error, and a LET name spelled like a
/// cell, which typed text refuses; decision 0103).
#[test]
fn no_expansion_for_a_formula_that_does_not_parse_as_typed() {
    for row in [1, 2] {
        for text in ["=1+", "=LET(f1,1,f1+1)"] {
            let mut model = model(false);
            // The refused LET is an error; the parse error is stored as before.
            let _ = model.set_typed_user_input(0, row, 4, text.to_string());
            assert_eq!(expansion_readout(&mut model)[0], "3", "{text} in row {row}");
            let header = model.get_formatted_cell_value(0, 1, 4).unwrap();
            assert_ne!(header, "Column1", "{text} in row {row}");
            assert_ne!(header, "0", "{text} in row {row}");
        }
    }
}
