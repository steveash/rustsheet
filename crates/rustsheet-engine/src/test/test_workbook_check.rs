#![allow(clippy::unwrap_used)]

//! `Model::from_workbook` refuses a decoded workbook whose indices dangle (T-2913).

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::test::util::new_empty_model;
use crate::types::{
    ArrayKind, Cell, Col, FormulaValue, MergedCell, Row, SpillValue, Workbook, WorkbookView,
};
use crate::{Model, UserModel};

/// A one-sheet workbook with a number in A1 and a formula in A2.
fn workbook() -> Workbook {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "=A1+1");
    model.workbook
}

/// The error `Model::from_workbook` gives for `workbook` once `change` has been applied to it.
fn refusal(change: impl FnOnce(&mut Workbook)) -> String {
    let mut workbook = workbook();
    change(&mut workbook);
    match Model::from_workbook(workbook, "en") {
        Ok(_) => panic!("a workbook with a dangling index loaded"),
        Err(e) => e,
    }
}

fn set_cell(workbook: &mut Workbook, row: i32, column: i32, cell: Cell) {
    workbook.worksheets[0]
        .sheet_data
        .set_cell(row, column, cell);
}

/// A dangling shared string is read through a checked lookup, so it is not refused: an Office 2007
/// beta workbook (poi/sample-beta.xlsx) imports with no shared strings and loads today.
#[test]
fn a_dangling_shared_string_still_loads() {
    let mut workbook = workbook();
    set_cell(&mut workbook, 1, 2, Cell::SharedString { si: 40, s: 0 });
    let model = Model::from_workbook(workbook, "en").unwrap();
    assert_eq!(model._get_text("B1"), "");
}

/// The selected sheet of a view, the frozen panes and the sheet list are read through checked
/// lookups, so none of them is refused: the model loads and answers as it did before T-2913.
#[test]
fn a_view_past_the_sheets_frozen_panes_off_the_grid_and_no_sheets_still_load() {
    let mut w = workbook();
    let view = WorkbookView {
        sheet: 3,
        window_width: 800,
        window_height: 600,
    };
    w.views.insert(0, view);
    w.worksheets[0].frozen_rows = i32::MAX;
    w.worksheets[0].frozen_columns = -5;
    let mut user = UserModel::from_model(Model::from_workbook(w, "en").unwrap());
    assert_eq!(user.get_selected_view().sheet, 0);
    // Navigation on the missing selected sheet is an error, not a panic.
    assert!(user.on_arrow_down().is_err());
    assert!(user.on_page_down().is_err());

    let mut w = workbook();
    w.worksheets.clear();
    let mut model = Model::from_workbook(w, "en").unwrap();
    model.evaluate();
    assert!(model.set_user_input(0, 1, 1, "1".to_string()).is_err());
}

#[test]
fn an_intact_workbook_loads_and_round_trips() {
    let mut model = Model::from_workbook(workbook(), "en").unwrap();
    model.evaluate();
    let model = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(model._get_text("A2"), "2");
}

// T-2882's two reproductions: each panicked after loading.

#[test]
fn a_cell_style_past_the_styles_is_refused() {
    let e = refusal(|w| set_cell(w, 1, 2, Cell::NumberCell { v: 1.0, s: 9999 }));
    assert!(e.contains("style 9999"), "{e}");
}

#[test]
fn a_formula_index_with_no_shared_formula_is_refused() {
    let cell = Cell::CellFormula {
        f: 77,
        s: 0,
        v: FormulaValue::Unevaluated,
    };
    let e = refusal(|w| set_cell(w, 1, 2, cell));
    assert!(e.contains("formula 77"), "{e}");
}

#[test]
fn a_negative_style_is_refused() {
    let e = refusal(|w| set_cell(w, 1, 2, Cell::EmptyCell { s: -1 }));
    assert!(e.contains("style -1"), "{e}");
}

#[test]
fn a_dangling_font_fill_or_border_is_refused() {
    assert!(refusal(|w| w.styles.cell_xfs[0].font_id = 50).contains("font 50"));
    assert!(refusal(|w| w.styles.cell_xfs[0].fill_id = 50).contains("fill 50"));
    assert!(refusal(|w| w.styles.cell_xfs[0].border_id = 50).contains("border 50"));
    assert!(refusal(|w| w.styles.cell_style_xfs[0].font_id = 50).contains("font 50"));
}

#[test]
fn a_negative_number_format_id_is_refused() {
    let e = refusal(|w| w.styles.cell_xfs[0].num_fmt_id = -3);
    assert!(e.contains("number format id -3"), "{e}");
}

#[test]
fn a_workbook_without_cell_styles_is_refused() {
    let e = refusal(|w| w.styles.cell_xfs.clear());
    assert!(e.contains("no cell styles"), "{e}");
}

#[test]
fn an_array_formula_spilling_off_the_grid_is_refused() {
    let anchor = |f, r| Cell::ArrayFormula {
        f,
        s: 0,
        r,
        kind: ArrayKind::Dynamic,
        v: FormulaValue::Unevaluated,
    };
    assert!(refusal(|w| set_cell(w, 1, 2, anchor(9, (1, 1)))).contains("formula 9"));
    let e = refusal(|w| set_cell(w, 2, 2, anchor(0, (1, LAST_ROW))));
    assert!(e.contains("spills outside"), "{e}");
    let e = refusal(|w| set_cell(w, 1, 2, anchor(0, (LAST_COLUMN, 1))));
    assert!(e.contains("spills outside"), "{e}");
    let e = refusal(|w| set_cell(w, 1, 2, anchor(0, (-1, 1))));
    assert!(e.contains("spills outside"), "{e}");
}

#[test]
fn a_spill_cell_anchored_off_the_grid_is_refused() {
    let cell = Cell::SpillCell {
        s: 0,
        a: (0, 1),
        v: SpillValue::Number(1.0),
    };
    let e = refusal(|w| set_cell(w, 1, 2, cell));
    assert!(e.contains("spills from outside"), "{e}");
}

#[test]
fn row_and_column_records_off_the_grid_or_without_a_style_are_refused() {
    let row = |r, s| Row {
        r,
        height: 15.0,
        custom_format: true,
        custom_height: false,
        s,
        hidden: false,
    };
    assert!(refusal(|w| w.worksheets[0].rows.push(row(0, 0))).contains("row 0"));
    assert!(refusal(|w| w.worksheets[0].rows.push(row(LAST_ROW + 1, 0))).contains("record of row"));
    assert!(refusal(|w| w.worksheets[0].rows.push(row(3, 70))).contains("record of row 3"));
    let col = |min, max, style| Col {
        min,
        max,
        width: 10.0,
        custom_width: true,
        hidden: false,
        style,
    };
    assert!(refusal(|w| w.worksheets[0].cols.push(col(3, 2, None))).contains("columns 3:2"));
    assert!(
        refusal(|w| w.worksheets[0].cols.push(col(1, LAST_COLUMN + 1, None))).contains("columns")
    );
    assert!(refusal(|w| w.worksheets[0].cols.push(col(1, 1, Some(70)))).contains("columns 1:1"));
}

#[test]
fn a_merged_range_off_the_grid_is_refused() {
    let merged = |row, column, width, height| MergedCell {
        row,
        column,
        width,
        height,
    };
    for range in [
        merged(1, 1, 0, 1),
        merged(1, 1, 1, LAST_ROW + 1),
        merged(1, LAST_COLUMN, 2, 1),
        merged(0, 1, 1, 1),
        merged(i32::MAX, 1, 1, 1),
    ] {
        let e = refusal(|w| w.worksheets[0].merged_cells.push(range));
        assert!(e.contains("merged range"), "{range:?}: {e}");
    }
}

#[test]
fn the_refusal_names_the_sheet() {
    let e = refusal(|w| set_cell(w, 1, 2, Cell::NumberCell { v: 1.0, s: 9999 }));
    assert!(e.starts_with("Sheet 'Sheet1'"), "{e}");
}
