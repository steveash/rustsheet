//! `SheetData::cell` tries the index its last read found (T-3606), and a row's search tries a dense
//! row's offset (T-3608), before bisecting: guesses, so every read must still find exactly the cell
//! at its position, or none, whatever the rows around it hold and however writes moved cells since.

use std::collections::BTreeMap;

use crate::sheet_data::SheetData;
use crate::types::Cell;

fn number(v: f64) -> Cell {
    Cell::NumberCell { v, s: 0 }
}

/// Every position of `rows` x `columns` read against `model`, the cells as a map.
fn assert_reads(data: &SheetData, model: &BTreeMap<(i32, i32), f64>, rows: i32, columns: i32) {
    for row in 1..=rows {
        for column in 1..=columns {
            let found = data.cell(row, column).map(|cell| match cell {
                Cell::NumberCell { v, .. } => *v,
                _ => f64::NAN,
            });
            assert_eq!(found, model.get(&(row, column)).copied(), "{row},{column}");
        }
    }
    // Down a column too, where the hint serves the next row.
    for column in [1, columns / 2, columns] {
        for row in 1..=rows {
            let found = data.cell(row, column).is_some();
            assert_eq!(found, model.contains_key(&(row, column)), "{row},{column}");
        }
    }
}

/// Rows of different widths and gaps, read across and down, then after writes that shift the
/// cells of a row (an insert before them, a removal), so a hint left by one row points at another
/// column, or past the end, of the next.
#[test]
fn reads_find_their_cell_whatever_the_hint() {
    let mut data = SheetData::new();
    let mut model = BTreeMap::new();
    for row in 1..=30 {
        for column in 1..=40 {
            // Every third row is dense, the others skip columns by a pattern of their own.
            if row % 3 == 0 || (column * row) % 7 < 3 {
                let v = f64::from(row * 100 + column);
                data.set_cell(row, column, number(v));
                model.insert((row, column), v);
            }
        }
    }
    assert_reads(&data, &model, 30, 40);
    for row in (1..=30).step_by(4) {
        data.remove_cell(row, 20);
        model.remove(&(row, 20));
        data.set_cell(row, 1, number(-1.0));
        model.insert((row, 1), -1.0);
        data.remove_cell(row, 2);
        model.remove(&(row, 2));
    }
    assert_reads(&data, &model, 30, 40);
}

/// Columns from anywhere in `i32`, as an unchecked caller may pass them, to reads and removals of a
/// dense row: none overflows the offset guess (directive 5), and the row is unchanged.
#[test]
fn reads_at_extreme_columns_find_nothing() {
    let mut data = SheetData::new();
    for column in 1..=20 {
        data.set_cell(3, column, number(f64::from(column)));
    }
    // 21 is on the sheet but holds nothing; the others are off it.
    assert!(data.cell(3, 21).is_none());
    for column in [i32::MIN, i32::MIN + 1, -1, 0, i32::MAX] {
        assert!(data.cell(3, column).is_none(), "{column}");
        assert!(data.cell_mut(3, column).is_none(), "{column}");
        assert!(data.remove_cell(3, column).is_none(), "{column}");
    }
    assert_eq!(data.columns_in_row(3), (1..=20).collect::<Vec<_>>());
}
