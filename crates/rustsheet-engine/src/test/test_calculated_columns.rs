#![allow(clippy::unwrap_used)]
//! A formula typed into a table column fills it as a calculated column (T-2958;
//! docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table column").

use crate::types::Cell;
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

/// The `calculatedColumnFormula` of `T`'s `Calc` column.
fn column_formula(model: &Model) -> Option<String> {
    model.workbook.tables["T"].columns[2]
        .calculated_column_formula
        .clone()
}

/// probes/t2983-identical-stored-refill: identical stored formulas are no calculated column, so a
/// formula typed into the blank row changes only that row (references.md rule 8a).
#[test]
fn identical_stored_formulas_are_not_refilled() {
    // T A1:C5, with C2:C4 stored `=A2*2` shifted and C5 blank.
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    for (row, col) in [(1, "Col"), (2, "1"), (3, "2"), (4, "4"), (5, "8")] {
        model.set_user_input(0, row, 1, col.to_string()).unwrap();
    }
    model.set_user_input(0, 1, 2, "Other".to_string()).unwrap();
    model.set_user_input(0, 1, 3, "Calc".to_string()).unwrap();
    for row in 2..=4 {
        model
            .set_user_input(0, row, 3, format!("=A{row}*2"))
            .unwrap();
    }
    model
        .add_table_with_totals(0, "T", (1, 1, 5, 3), false, false)
        .unwrap();
    typed(&mut model, 5, "=A5*3");
    assert_eq!(calc(&model, 5), ["2", "4", "8", "24"]);
    assert_eq!(column_formula(&model), None);
}

/// A typed fill makes the column a calculated column, saved as the first data row's text, and a
/// re-fill updates it; a typed constant fills nothing and keeps it.
#[test]
fn a_typed_fill_records_the_column_formula() {
    let mut model = model(false);
    assert_eq!(column_formula(&model), None);
    typed(&mut model, 3, "=A3*2");
    assert_eq!(column_formula(&model).as_deref(), Some("A2*2"));
    typed(&mut model, 4, "=A4*3");
    assert_eq!(calc(&model, 4), ["3", "6", "12"]);
    assert_eq!(column_formula(&model).as_deref(), Some("A2*3"));
    typed(&mut model, 3, "5");
    assert_eq!(column_formula(&model).as_deref(), Some("A2*3"));
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

/// A stored formula beside a table (xlsx import) does not grow it, nor does a formula whose new
/// column already holds a value (probes/t3003-occupied-*, rule 7d), so the cell is written as
/// before.
#[test]
fn no_expansion_for_stored_formulas_or_occupied_columns() {
    let mut stored = model(false);
    stored
        .set_loaded_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    stored.set_user_input(0, 3, 4, "=A3*2".to_string()).unwrap();
    assert_eq!(expansion_readout(&mut stored)[0], "3");
    assert_eq!(column_d(&stored), ["2", "4", ""]);

    let mut occupied = model(false);
    occupied.set_user_input(0, 4, 4, "7".to_string()).unwrap();
    occupied
        .set_typed_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    assert_eq!(expansion_readout(&mut occupied)[0], "3");
    assert_eq!(column_d(&occupied), ["2", "", "7"]);
}

/// probes/t3003-const-*: a constant typed just right of the table grows it and fills nothing
/// (rule 7a). Typed into the header cell, it is the header text, `5` as the text `5`.
#[test]
fn a_constant_typed_beside_the_table_grows_it() {
    for text in ["5", "hello"] {
        let mut beside = model(false);
        beside
            .set_typed_user_input(0, 2, 4, text.to_string())
            .unwrap();
        assert_eq!(
            expansion_readout(&mut beside),
            ["4", "4", "Column1", "FALSE"],
            "{text}"
        );
        assert_eq!(column_d(&beside), [text, "", ""]);

        let mut header = model(false);
        header
            .set_typed_user_input(0, 1, 4, text.to_string())
            .unwrap();
        assert_eq!(expansion_readout(&mut header), ["4", "4", text, "FALSE"]);
        assert!(matches!(
            header.workbook.worksheet(0).unwrap().cell(1, 4),
            Some(Cell::SharedString { .. })
        ));
        assert_eq!(column_d(&header), ["", "", ""]);
    }
}

/// probes/t3003-totals-*: a totals row does not stop the growth (rule 7c). Beside a data row the
/// formula fills the other data rows, not the totals cell; beside the totals row it fills nothing.
#[test]
fn a_table_with_a_totals_row_grows_too() {
    let mut data_row = model(true);
    data_row
        .set_typed_user_input(0, 2, 4, "=A2*2".to_string())
        .unwrap();
    assert_eq!(
        expansion_readout(&mut data_row),
        ["4", "5", "Column1", "FALSE"]
    );
    assert_eq!(column_d(&data_row), ["2", "4", "8"]);
    assert_eq!(data_row.get_formatted_cell_value(0, 5, 4).unwrap(), "");

    let mut totals_row = model(true);
    totals_row
        .set_typed_user_input(0, 5, 4, "=A2*2".to_string())
        .unwrap();
    assert_eq!(
        expansion_readout(&mut totals_row),
        ["4", "5", "Column1", "FALSE"]
    );
    assert_eq!(column_d(&totals_row), ["", "", ""]);
    assert_eq!(totals_row.get_formatted_cell_value(0, 5, 4).unwrap(), "2");
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

/// A typed `T[ [Col] ]` prints `T[ Col ]` (syntax.md, T-2801) and keeps working when an edit
/// rewrites its text, which the engine then re-parses; only text typed or loaded as `T[ Col ]`
/// is no column (T-3069).
#[test]
fn a_padded_column_survives_an_edit_that_rewrites_its_text() {
    let mut model = model(false);
    model.set_user_input(0, 10, 1, "100".to_string()).unwrap();
    model
        .set_typed_user_input(0, 8, 5, "=SUM(T[ [Col] ])+A10".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 8, 5).unwrap(), "107");
    model.insert_rows(0, 9, 1).unwrap();
    model.evaluate();
    assert_eq!(
        model.get_cell_formula(0, 8, 5).unwrap().as_deref(),
        Some("=SUM(T[ Col ])+A11")
    );
    assert_eq!(model.get_formatted_cell_value(0, 8, 5).unwrap(), "107");
    // Typed as it prints, Excel refuses it, and so does the engine (T-3213): the cell stays blank.
    let refused = model.set_typed_user_input(0, 9, 5, "=SUM(T[ Col ])".to_string());
    assert!(refused.is_err());
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 9, 5).unwrap(), "");
}

/// `COLUMNS(T[#All])` and `ROWS(T[#All])` after typing `text` into (`row`, `column`) of `model`.
fn grown_by(mut model: Model, row: i32, column: i32, text: &str) -> Vec<String> {
    model
        .set_typed_user_input(0, row, column, text.to_string())
        .unwrap();
    expansion_readout(&mut model)[..2].to_vec()
}

/// probes/t3003-below-last-row, -const: `=B4*2` or `7` typed into A5, just below `T` A1:C4,
/// grows `T` to A1:C5 and A5 keeps what was typed (rule 7e, T-3273).
#[test]
fn typing_just_below_the_last_data_row_grows_the_table_by_a_row() {
    for text in ["=B4*2", "7"] {
        let mut model = model(false);
        model
            .set_typed_user_input(0, 5, 1, text.to_string())
            .unwrap();
        assert_eq!(expansion_readout(&mut model)[..2], ["3", "5"], "{text}");
        let stored = model.get_cell_formula(0, 5, 1).unwrap();
        let value = model.get_formatted_cell_value(0, 5, 1).unwrap();
        if text.starts_with('=') {
            assert_eq!((stored.as_deref(), value.as_str()), (Some("=B4*2"), "80"));
        } else {
            assert_eq!((stored, value.as_str()), (None, "7"));
        }
        // The new row is the table's: its column sums see it.
        model
            .set_user_input(0, 7, 1, "=ROWS(T[Col])".to_string())
            .unwrap();
        model.evaluate();
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "4");
    }
}

/// Stored or untyped input (`set_user_input`, xlsx import) never grows a table by a row.
#[test]
fn untyped_input_below_the_table_does_not_grow_it() {
    let mut model = model(false);
    model.set_user_input(0, 5, 1, "7".to_string()).unwrap();
    model
        .set_loaded_user_input(0, 5, 2, "=B4*2".to_string())
        .unwrap();
    assert_eq!(expansion_readout(&mut model)[..2], ["3", "4"]);
}

/// Unprobed neighbours of rule 7e, pinned as the engine has them (T-3273): no growth below a
/// totals row, outside the table's columns, two rows below, into a row another table reaches, or
/// into a row whose other cells are not blank (the column rule 7d's analogue).
#[test]
fn unprobed_neighbours_below_the_table_do_not_grow_it() {
    // A totals row (row 5): typing into A6 leaves `T` A1:C5.
    assert_eq!(grown_by(model(true), 6, 1, "7"), ["3", "5"]);
    // Below the table but outside its columns: E5.
    assert_eq!(grown_by(model(false), 5, 5, "7"), ["3", "4"]);
    // Two rows below: A6.
    assert_eq!(grown_by(model(false), 6, 1, "7"), ["3", "4"]);
    // Another table over C5:E6 reaches the new row.
    let mut overlap = model(false);
    overlap.set_user_input(0, 5, 3, "U1".to_string()).unwrap();
    overlap.set_user_input(0, 5, 4, "U2".to_string()).unwrap();
    overlap.set_user_input(0, 5, 5, "U3".to_string()).unwrap();
    overlap
        .add_table_with_totals(0, "U", (5, 3, 6, 5), false, false)
        .unwrap();
    assert_eq!(grown_by(overlap, 5, 1, "7"), ["3", "4"]);
    // B5 already holds a value.
    let mut occupied = model(false);
    occupied.set_user_input(0, 5, 2, "1".to_string()).unwrap();
    assert_eq!(grown_by(occupied, 5, 1, "7"), ["3", "4"]);
}

/// A table that ends on the sheet's last row has no row below it to grow into: typing past it is
/// refused before any growth (unprobed, T-3273).
#[test]
fn a_table_on_the_last_row_does_not_grow_down() {
    let last = crate::constants::LAST_ROW;
    let mut model = Model::new_empty("book", "en", "UTC", "en").unwrap();
    model
        .set_user_input(0, last - 1, 1, "Col".to_string())
        .unwrap();
    model.set_user_input(0, last, 1, "1".to_string()).unwrap();
    model
        .add_table_with_totals(0, "T", (last - 1, 1, last, 1), false, false)
        .unwrap();
    assert_eq!(model.table_expansion(0, last + 1, 1, "7").unwrap(), None);
}

/// A formula typed just below an empty column, or below a calculated column, grows the table by
/// the row and fills nothing: the other data rows stay as they were (unprobed, pinned, T-3273).
#[test]
fn a_formula_typed_below_the_table_fills_no_calculated_column() {
    let mut empty = model(false);
    typed(&mut empty, 5, "=1");
    assert_eq!(calc(&empty, 5), ["", "", "", "1"]);
    assert_eq!(expansion_readout(&mut empty)[..2], ["3", "5"]);

    let mut filled = model(false);
    typed(&mut filled, 2, "=[@Col]*2");
    assert_eq!(calc(&filled, 4), ["2", "4", "8"]);
    typed(&mut filled, 5, "=1");
    assert_eq!(calc(&filled, 5), ["2", "4", "8", "1"]);
}
