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
/// be all blank or all the one formula).
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
