#![allow(clippy::unwrap_used, clippy::panic)]

//! `UserModel::apply_external_diffs` refuses a crafted diff list instead of panicking, and a
//! refused list leaves the model unchanged (T-2896, SPEC-0026 decision 4).

use std::collections::HashMap;

use crate::cf_types::CfRule;
use crate::expressions::types::Area;
use crate::types::{Cell, Col, Color, FormulaValue, MergedCell, Row, Style, Workbook};
use crate::user_model::history::{ColumnData, Diff, DiffType, QueueDiffs, RowData};
use crate::UserModel;

const BIG: i32 = i32::MAX;

/// A model with a number, a formula and a string on its one sheet.
fn model() -> UserModel {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "1").unwrap();
    model.set_user_input(0, 2, 1, "=A1+1").unwrap();
    model.set_user_input(0, 3, 1, "hello").unwrap();
    model
}

fn encode(entries: Vec<QueueDiffs>) -> Vec<u8> {
    bitcode::encode(&entries)
}

fn decode(bytes: &[u8]) -> Vec<QueueDiffs> {
    bitcode::decode(bytes).unwrap()
}

/// Applies `bytes` to `model`, asserting the list is refused and the workbook left as it was.
fn assert_refused(model: &mut UserModel, bytes: &[u8]) -> String {
    let before: Workbook = model.model.workbook.clone();
    let error = match model.apply_external_diffs(bytes) {
        Ok(()) => panic!("a crafted diff list applied"),
        Err(error) => error,
    };
    assert!(
        model.model.workbook == before,
        "a refused list changed the model: {error}"
    );
    error
}

fn number(s: i32) -> Cell {
    Cell::NumberCell { v: 1.0, s }
}

// T-2882's way: flush a real send queue, change one index in it, apply.

/// The undo of a typed value restores the cell it replaced; that cell's style is an index.
#[test]
fn a_real_queue_with_a_dangling_style_index_is_refused() {
    let mut peer = model();
    peer.set_user_input(0, 1, 2, "5").unwrap();
    peer.set_user_input(0, 1, 2, "6").unwrap();
    peer.undo().unwrap();
    let mut entries = decode(&peer.flush_send_queue());
    let Some(Diff::SetCellValue { old_value, .. }) = entries.last_mut().unwrap().list.last_mut()
    else {
        panic!("the undo is not of a cell value");
    };
    **old_value = Some(number(9999));

    let mut model = model();
    let error = assert_refused(&mut model, &encode(entries));
    assert!(error.contains("style 9999"), "{error}");
}

#[test]
fn a_real_queue_with_a_dangling_sheet_index_is_refused() {
    let mut peer = model();
    peer.set_user_input(0, 1, 2, "5").unwrap();
    peer.set_user_input(0, 1, 3, "6").unwrap();
    let mut entries = decode(&peer.flush_send_queue());
    let Some(Diff::SetCellValue { sheet, .. }) = entries.last_mut().unwrap().list.last_mut() else {
        panic!("the edit is not of a cell value");
    };
    *sheet = 7;

    // The first edit of the list applied before the second was refused, and is undone too.
    let mut model = model();
    let error = assert_refused(&mut model, &encode(entries));
    assert!(error.contains("sheet"), "{error}");
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "");
}

#[test]
fn a_real_queue_with_a_dangling_formula_index_is_refused() {
    let mut peer = model();
    peer.set_user_input(0, 1, 2, "=A1*3").unwrap();
    peer.set_user_input(0, 1, 2, "6").unwrap();
    peer.undo().unwrap();
    let mut entries = decode(&peer.flush_send_queue());
    let Some(Diff::SetCellValue { old_value, .. }) = entries.last_mut().unwrap().list.last_mut()
    else {
        panic!("the undo is not of a cell value");
    };
    **old_value = Some(Cell::CellFormula {
        f: 77,
        s: 0,
        v: FormulaValue::Unevaluated,
    });

    let mut model = model();
    let error = assert_refused(&mut model, &encode(entries));
    assert!(error.contains("formula 77"), "{error}");
}

/// A list whose later diffs need what its earlier ones made still applies: the checks see the
/// model as it is when each diff applies.
#[test]
fn a_list_building_on_itself_still_applies() {
    let mut peer = model();
    peer.new_sheet().unwrap();
    peer.set_user_input(1, 1, 1, "on the new sheet").unwrap();
    let range = Area {
        sheet: 0,
        row: 5,
        column: 5,
        width: 1,
        height: 1,
    };
    peer.update_range_style(&range, "font.b", "true").unwrap();
    peer.set_user_input(0, 5, 5, "=A1*3").unwrap();
    peer.set_user_input(0, 5, 5, "bold").unwrap();
    peer.undo().unwrap();
    let queue = peer.flush_send_queue();

    let mut model = model();
    model.apply_external_diffs(&queue).unwrap();
    assert_eq!(
        model.get_formatted_cell_value(1, 1, 1).unwrap(),
        "on the new sheet"
    );
    assert_eq!(model.get_formatted_cell_value(0, 5, 5).unwrap(), "3");
    assert!(model.get_cell_style(0, 5, 5).unwrap().font.b);
}

/// Each diff of one entry, crafted to reach an unchecked index, an `i32` overflow or a write off
/// the grid, is refused with the model unchanged.
#[test]
fn crafted_diffs_are_refused() {
    let worksheet = |change: fn(&mut crate::types::Worksheet)| {
        let mut worksheet = model().model.workbook.worksheets[0].clone();
        worksheet.name = "Other".to_string();
        worksheet.sheet_id = 9;
        change(&mut worksheet);
        Box::new(worksheet)
    };
    let row = |r, s| Row {
        r,
        height: 15.0,
        custom_format: true,
        custom_height: false,
        s,
        hidden: false,
    };
    let merged = |row, column, width, height| MergedCell {
        row,
        column,
        width,
        height,
    };
    let cell_value = |sheet, row, column, old: Option<Cell>| Diff::SetCellValue {
        sheet,
        row,
        column,
        new_value: "1".to_string(),
        legacy: false,
        old_value: Box::new(old),
    };
    let array = |height| Diff::SetArrayValue {
        sheet: 0,
        row: 1,
        column: 1,
        width: 3,
        height,
        new_value: "=1".to_string(),
        old_values: vec![],
    };
    use DiffType::{Redo, Undo};
    let cases = vec![
        (Undo, cell_value(0, 1, 2, Some(number(-1)))),
        (Undo, cell_value(7, 1, 2, None)),
        (Undo, cell_value(0, 0, 2, Some(number(0)))),
        (Undo, cell_value(0, BIG, BIG, Some(number(0)))),
        (Redo, cell_value(0, BIG, 0, None)),
        (Undo, array(BIG)),
        (Redo, array(BIG)),
        (
            Undo,
            Diff::RangeClearAll {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 1,
                old_value: vec![vec![Some(number(0))]],
                old_style: vec![],
            },
        ),
        (
            Redo,
            Diff::RangeClearContents {
                sheet: 0,
                row: 1,
                column: 1,
                width: BIG,
                height: BIG,
                old_value: vec![],
            },
        ),
        (
            Undo,
            Diff::CellClearFormatting {
                sheet: 0,
                row: BIG,
                column: 0,
                old_style: Box::new(None),
            },
        ),
        (
            Undo,
            Diff::SetCellStyle {
                sheet: 0,
                row: -5,
                column: 1,
                old_value: Box::new(None),
                new_value: Box::new(Style::default()),
            },
        ),
        (
            Redo,
            Diff::SetRowStyle {
                sheet: 0,
                row: BIG,
                old_value: Box::new(None),
                new_value: Box::new(Style::default()),
            },
        ),
        (
            Redo,
            Diff::InsertRows {
                sheet: 0,
                row: 1,
                count: BIG,
            },
        ),
        (
            Redo,
            Diff::InsertColumns {
                sheet: 0,
                column: 1,
                count: BIG,
            },
        ),
        (
            Redo,
            Diff::DeleteRows {
                sheet: 0,
                row: 2,
                count: BIG,
                old_data: vec![],
            },
        ),
        (
            Redo,
            Diff::DeleteColumns {
                sheet: 0,
                column: 1,
                count: BIG,
                old_data: vec![],
            },
        ),
        (
            Undo,
            Diff::DeleteRows {
                sheet: 0,
                row: 2,
                count: 1,
                old_data: vec![RowData {
                    row: Some(row(2, 9999)),
                    data: HashMap::new(),
                }],
            },
        ),
        (
            Undo,
            Diff::DeleteRows {
                sheet: 0,
                row: 2,
                count: 1,
                old_data: vec![RowData {
                    row: None,
                    data: HashMap::from([(BIG, number(0))]),
                }],
            },
        ),
        (
            Undo,
            Diff::DeleteRows {
                sheet: 0,
                row: BIG,
                count: 1,
                old_data: vec![RowData {
                    row: None,
                    data: HashMap::new(),
                }],
            },
        ),
        (
            Undo,
            Diff::DeleteColumns {
                sheet: 0,
                column: 2,
                count: 1,
                old_data: vec![ColumnData {
                    column: Some(Col {
                        min: 2,
                        max: 2,
                        width: 10.0,
                        custom_width: true,
                        hidden: false,
                        style: Some(9999),
                    }),
                    data: HashMap::new(),
                }],
            },
        ),
        (
            Redo,
            Diff::MoveColumns {
                sheet: 0,
                column: BIG,
                column_count: BIG,
                delta: BIG,
            },
        ),
        (
            Undo,
            Diff::MoveColumns {
                sheet: 0,
                column: BIG,
                column_count: 1,
                delta: BIG,
            },
        ),
        (
            Undo,
            Diff::MoveRows {
                sheet: 0,
                row: BIG,
                row_count: 1,
                delta: BIG,
            },
        ),
        (
            Redo,
            Diff::MoveRows {
                sheet: 0,
                row: 1,
                row_count: BIG,
                delta: 5,
            },
        ),
        (
            Redo,
            Diff::DuplicateSheet {
                source_index: 0,
                new_index: 100,
            },
        ),
        (
            Undo,
            Diff::DeleteSheet {
                sheet: 1,
                old_data: worksheet(|w| {
                    w.sheet_data.set_cell(1, 5, number(9999));
                }),
                tables: vec![],
                rewritten: vec![],
            },
        ),
        (
            Undo,
            Diff::DeleteSheet {
                sheet: 1,
                old_data: worksheet(|w| {
                    w.merged_cells.push(MergedCell {
                        row: 1,
                        column: 1,
                        width: BIG,
                        height: BIG,
                    });
                }),
                tables: vec![],
                rewritten: vec![],
            },
        ),
        (
            Redo,
            Diff::SetMergedCells {
                sheet: 0,
                old_value: vec![],
                new_value: vec![merged(BIG, 1, BIG, 5)],
            },
        ),
        (
            Undo,
            Diff::SetMergedCells {
                sheet: 0,
                old_value: vec![merged(1, 1, -5, 0)],
                new_value: vec![],
            },
        ),
        (
            Undo,
            Diff::DeleteConditionalFormatting {
                sheet: 0,
                index: 99,
                old_range: "A1".to_string(),
                old_rule: Box::new(CfRule::Blanks {
                    dxf_id: 0,
                    stop_if_true: false,
                }),
                old_priority: 1,
            },
        ),
        (
            Redo,
            Diff::SetSheetColor {
                index: 99,
                old_value: Color::None,
                new_value: Color::None,
            },
        ),
    ];
    for (direction, diff) in cases {
        let mut model = model();
        let bytes = encode(vec![QueueDiffs {
            r#type: direction,
            list: vec![diff],
        }]);
        assert_refused(&mut model, &bytes);
        // What was refused left nothing behind that a later edit or a reload trips on.
        model.set_user_input(0, 1, 2, "5").unwrap();
        model.insert_rows(0, 1, 1).unwrap();
        model.undo().unwrap();
        crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    }
}
