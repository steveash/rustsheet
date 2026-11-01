//! `SheetData` writes outside the sheet's columns store nothing (T-3624), as writes outside its
//! rows always did and as `check_layout` demands of loaded bytes: a cell at column 0 or past
//! `LAST_COLUMN` would be one no reference can reach and the layout check refuses.

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::sheet_data::SheetData;
use crate::types::Cell;

fn number(v: f64) -> Cell {
    Cell::NumberCell { v, s: 0 }
}

const OFF_SHEET: [i32; 6] = [
    i32::MIN,
    -1,
    0,
    LAST_COLUMN + 1,
    LAST_COLUMN + 1000,
    i32::MAX,
];

/// `set_cell` off the sheet's columns returns nothing, stores nothing and creates no row; the
/// columns at the sheet's edges take their cells.
#[test]
fn set_cell_refuses_columns_off_the_sheet() {
    let mut data = SheetData::new();
    for column in OFF_SHEET {
        assert!(data.set_cell(5, column, number(1.0)).is_none(), "{column}");
        assert!(data.cell(5, column).is_none(), "{column}");
    }
    assert!(data.is_empty());
    assert!(data.rows().is_empty());
    for (row, column) in [
        (1, 1),
        (1, LAST_COLUMN),
        (LAST_ROW, 1),
        (LAST_ROW, LAST_COLUMN),
    ] {
        data.set_cell(row, column, number(2.0));
        assert!(data.cell(row, column).is_some(), "{row},{column}");
    }
    assert_eq!(data.check_layout(), Ok(()));
}

/// `set_row` keeps the cells on the sheet's columns and drops the rest, so the row it leaves
/// passes the layout check.
#[test]
fn set_row_drops_cells_off_the_sheet() {
    let mut data = SheetData::new();
    let mut cells: Vec<(i32, Cell)> = OFF_SHEET.iter().map(|&c| (c, number(9.0))).collect();
    cells.extend([
        (1, number(1.0)),
        (LAST_COLUMN, number(2.0)),
        (300, number(3.0)),
    ]);
    data.set_row(7, cells);
    assert_eq!(data.columns_in_row(7), vec![1, 300, LAST_COLUMN]);
    assert_eq!(data.check_layout(), Ok(()));
}
