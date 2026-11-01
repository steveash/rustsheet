#![allow(clippy::unwrap_used)]
//! A row or column insert that would push a table's cells off the sheet is refused, even when
//! the cells pushed off are empty (excel-graph 2026-10-28, T-2845: 409 `InsertDeleteConflict`).

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::Model;

/// An empty workbook whose cells at `headers` hold distinct header texts.
fn model(headers: &[(i32, i32)]) -> Model {
    let mut m = Model::new_empty("book", "en", "UTC", "en").unwrap();
    for (i, (row, column)) in headers.iter().enumerate() {
        m.set_user_input(0, *row, *column, format!("h{i}")).unwrap();
    }
    m
}

#[test]
fn insert_rows_refused_when_table_would_leave_sheet() {
    let mut m = model(&[(LAST_ROW - 3, 2), (LAST_ROW - 3, 3)]);
    m.add_table(0, "Sales", (LAST_ROW - 3, 2, LAST_ROW, 3), false)
        .unwrap();
    assert!(m.insert_rows(0, 10, 1).is_err());
    // The table did not move.
    assert_eq!(
        m.workbook.tables["Sales"].reference,
        format!("B{}:C{}", LAST_ROW - 3, LAST_ROW)
    );
}

#[test]
fn insert_rows_allowed_when_table_stays_on_sheet() {
    let mut m = model(&[(LAST_ROW - 4, 2), (LAST_ROW - 4, 3)]);
    m.add_table(0, "Sales", (LAST_ROW - 4, 2, LAST_ROW - 1, 3), false)
        .unwrap();
    m.insert_rows(0, 10, 1).unwrap();
    // Ending exactly at the last row is allowed (unprobed): no table cell left the sheet.
    assert_eq!(
        m.workbook.tables["Sales"].reference,
        format!("B{}:C{}", LAST_ROW - 3, LAST_ROW)
    );
}

#[test]
fn insert_columns_refused_when_table_would_leave_sheet() {
    let mut m = model(&[(3, LAST_COLUMN - 1), (3, LAST_COLUMN)]);
    m.add_table(0, "Sales", (3, LAST_COLUMN - 1, 6, LAST_COLUMN), false)
        .unwrap();
    assert!(m.insert_columns(0, 3, 1).is_err());
    assert_eq!(m.workbook.tables["Sales"].reference, "XFC3:XFD6");
}
