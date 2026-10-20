#![allow(clippy::unwrap_used)]

//! Edits that re-enter a cell's formula through its display text (row and column inserts and
//! deletes, moves, fill and paste) keep a legacy N/T marker (SPEC-0012, T-1852). Excel keeps a
//! legacy cell legacy after a row insert above it (probes/sheet-ops/sheet-ops-insert-row-legacy-
//! cell, excel-graph, T-1933); the other edits are assumed to do the same (SPEC-0012 rule 6).

use crate::expressions::types::Area;
use crate::test::user_model::util::new_empty_user_model;
use crate::UserModel;

/// The legacy formula of every test: `#VALUE!` in a legacy cell, a two-cell spill when dynamic.
const FORMULA: &str = "=N(IF(1,{2;3}))+$A$9";

/// A model with A9 = 1, a legacy `FORMULA` in C2 and its dynamic twin in E2 (spilling 3, 4).
fn legacy_and_dynamic() -> UserModel {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 9, 1, "1").unwrap();
    model
        .model
        .set_cell_with_legacy_formula(0, 2, 3, FORMULA)
        .unwrap();
    model.set_user_input(0, 2, 5, FORMULA).unwrap();
    model.evaluate();
    assert_legacy(&model, (2, 3), "=N(IF(1,{2;3}))+$A$9");
    assert_dynamic(&model, (2, 5));
    model
}

fn text(model: &UserModel, (row, column): (i32, i32)) -> String {
    model.get_formatted_cell_value(0, row, column).unwrap()
}

/// The cell holds the marked formula `formula` and is a scalar `#VALUE!`.
fn assert_legacy(model: &UserModel, (row, column): (i32, i32), formula: &str) {
    assert!(model.model.cell_has_legacy_marker(0, row, column).unwrap());
    assert_eq!(model.get_cell_content(0, row, column).unwrap(), formula);
    assert_eq!(text(model, (row, column)), "#VALUE!");
    assert_eq!(text(model, (row + 1, column)), "");
}

/// The cell holds the unmarked twin and spills 3, 4.
fn assert_dynamic(model: &UserModel, (row, column): (i32, i32)) {
    assert!(!model.model.cell_has_legacy_marker(0, row, column).unwrap());
    assert_eq!(text(model, (row, column)), "3");
    assert_eq!(text(model, (row + 1, column)), "4");
}

#[test]
fn a_row_insert_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    model.insert_rows(0, 1, 1).unwrap();
    assert_legacy(&model, (3, 3), "=N(IF(1,{2;3}))+$A$10");
    assert_dynamic(&model, (3, 5));
}

#[test]
fn a_column_delete_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    model.delete_columns(0, 2, 1).unwrap();
    assert_legacy(&model, (2, 2), FORMULA);
    assert_dynamic(&model, (2, 4));
}

// Only the reference changes: the legacy cell stays where it is and is rewritten in place
// (`shift_cell_formula`), keeping its style as a dynamic cell does: a percent A9 does not turn
// the cell's General format into a percent one.
#[test]
fn a_shift_of_a_referenced_cell_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    model.set_user_input(0, 9, 1, "100%").unwrap();
    model.insert_rows(0, 5, 1).unwrap();
    assert_legacy(&model, (2, 3), "=N(IF(1,{2;3}))+$A$10");
    assert_dynamic(&model, (2, 5));
    for column in [3, 5] {
        let style = model.get_cell_style(0, 2, column).unwrap();
        assert_eq!(style.num_fmt, "general", "column {column}");
    }
}

#[test]
fn a_row_move_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    model.move_rows_action(0, 2, 1, 3).unwrap();
    assert_legacy(&model, (5, 3), FORMULA);
    assert_dynamic(&model, (5, 5));
}

#[test]
fn a_column_move_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    model.move_columns_action(0, 3, 1, 3).unwrap();
    assert_legacy(&model, (2, 6), FORMULA);
    assert_dynamic(&model, (2, 4));
}

#[test]
fn a_fill_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    let source = Area {
        sheet: 0,
        row: 2,
        column: 3,
        width: 1,
        height: 1,
    };
    model.auto_fill_rows(&source, 4).unwrap();
    assert_legacy(&model, (4, 3), FORMULA);
    model.auto_fill_columns(&source, 4).unwrap();
    assert_legacy(&model, (2, 4), FORMULA);
}

#[test]
fn a_paste_keeps_the_marker() {
    for is_cut in [false, true] {
        let mut model = legacy_and_dynamic();
        model.set_selected_cell(2, 3).unwrap();
        model.set_selected_range(2, 3, 2, 5).unwrap();
        let clipboard = model.copy_to_clipboard().unwrap();
        model.set_selected_cell(12, 3).unwrap();
        model
            .paste_from_clipboard(0, (2, 3, 2, 5), &clipboard.data, is_cut)
            .unwrap();
        assert_legacy(&model, (12, 3), FORMULA);
        assert_dynamic(&model, (12, 5));
    }
}

// A cut and paste of the cell a legacy formula reads rewrites that formula where it is
// (`get_external_formula_updates_for_cut`).
#[test]
fn a_cut_of_a_referenced_cell_keeps_the_marker() {
    let mut model = legacy_and_dynamic();
    model.set_selected_cell(9, 1).unwrap();
    model.set_selected_range(9, 1, 9, 1).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(20, 1).unwrap();
    model
        .paste_from_clipboard(0, (9, 1, 9, 1), &clipboard.data, true)
        .unwrap();
    assert_legacy(&model, (2, 3), "=N(IF(1,{2;3}))+$A$20");
    assert_dynamic(&model, (2, 5));
}
