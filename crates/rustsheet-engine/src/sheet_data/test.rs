//! The layout checks and row walks of `SheetData` (moved out of `sheet_data.rs` by T-3628).

#![allow(clippy::unwrap_used)]

use super::*;
use crate::constants::LAST_COLUMN;

fn number(v: f64) -> Cell {
    Cell::NumberCell { v, s: 0 }
}

/// A sheet with cells in rows 1 and 2000, to be broken in one place at a time.
fn two_rows() -> SheetData {
    let mut data = SheetData::new();
    data.set_cell(1, 1, number(1.0));
    data.set_cell(1, 3, number(1.0));
    data.set_cell(2000, 2, number(1.0));
    data
}

fn first_row(data: &mut SheetData) -> &mut Row {
    data.chunks[0].as_mut().unwrap().rows[1].as_mut().unwrap()
}

#[test]
fn check_layout_accepts_what_writes_build() {
    assert_eq!(two_rows().check_layout(), Ok(()));
    assert_eq!(SheetData::new().check_layout(), Ok(()));
    let mut data = SheetData::new();
    data.set_cell(LAST_ROW, LAST_COLUMN, number(1.0));
    assert_eq!(data.check_layout(), Ok(()));
}

#[test]
fn check_layout_refuses_what_bytes_can_forge() {
    let broken: [fn(&mut SheetData); 8] = [
        |d| d.chunks.resize_with(2000, || None),
        |d| d.chunks[0].as_mut().unwrap().rows.truncate(5),
        |d| d.chunks[0].as_mut().unwrap().rows[0] = Some(Row::default()),
        |d| d.chunks[0].as_mut().unwrap().used = 7,
        |d| d.row_count = 9,
        |d| first_row(d).cells.reverse(),
        |d| first_row(d).cells[1].0 = LAST_COLUMN + 1,
        |d| first_row(d).cells[0].0 = 0,
    ];
    for (case, break_it) in broken.iter().enumerate() {
        let mut data = two_rows();
        break_it(&mut data);
        assert!(data.check_layout().is_err(), "case {case}");
    }
    // A row past the last one, in the slots of the last chunk that the grid does not reach.
    let mut data = SheetData::new();
    data.set_cell(LAST_ROW, 1, number(1.0));
    let last = data.chunks.last_mut().unwrap().as_mut().unwrap();
    last.rows[1] = Some(Row::default());
    last.used += 1;
    data.row_count += 1;
    assert!(data.check_layout().is_err());
}

#[test]
fn next_row_between_skips_absent_rows_and_chunks() {
    let mut data = SheetData::new();
    data.set_cell(1, 2, number(1.0));
    data.set_cell(10_000, 3, number(1.0));
    let first = data.next_row_between(1, LAST_ROW);
    assert_eq!(
        (first.row, first.absent_chunks, first.slots),
        (Some(1), 0, 1)
    );
    // Rows 2..=1023 of chunk 0, chunks 1..=8 absent, then 785 slots of chunk 9.
    let second = data.next_row_between(2, LAST_ROW);
    assert_eq!(second.row, Some(10_000));
    assert_eq!((second.absent_chunks, second.slots), (8, 1022 + 785));
    // Past the last chunk the search stops: the chunks after it are never looked at.
    let none = data.next_row_between(10_001, LAST_ROW);
    assert_eq!((none.row, none.absent_chunks, none.slots), (None, 0, 239));
    assert_eq!(data.next_row_between(3, 2), RowSearch::default());
    assert_eq!(data.next_row_between(2, 9_999).row, None);
}

#[test]
fn columns_in_row_between_keeps_the_span() {
    let mut data = SheetData::new();
    for column in [1, 3, 5, 7, 9, 11, 13, 15, 17, 19] {
        data.set_cell(4, column, number(1.0));
    }
    let mut columns = vec![99];
    data.columns_in_row_between(4, 4, 15, &mut columns);
    assert_eq!(columns, [5, 7, 9, 11, 13, 15]);
    data.columns_in_row_between(4, 2, 2, &mut columns);
    assert!(columns.is_empty());
    data.columns_in_row_between(5, 1, 20, &mut columns);
    assert!(columns.is_empty());
}

#[test]
fn last_row_spans_chunks() {
    let mut data = SheetData::new();
    assert_eq!(data.last_row(), 0);
    data.set_cell(1, 1, number(1.0));
    assert_eq!(data.last_row(), 1);
    data.set_cell(1024, 1, number(1.0)); // first row of the second chunk
    assert_eq!(data.last_row(), 1024);
    data.set_cell(5000, 3, number(1.0));
    assert_eq!(data.last_row(), 5000);
    data.remove_cell(5000, 3);
    assert_eq!(data.last_row(), 1024);
}

#[test]
fn last_column_survives_clearing_a_wide_sheet() {
    let mut data = SheetData::new();
    for row in 1..=20_000 {
        data.set_cell(row, 3, number(1.0));
        data.set_cell(row, 8, number(1.0));
    }
    data.set_cell(1, 8, number(2.0)); // a replace does not count twice
    for row in 1..=20_000 {
        data.remove_cell(row, 8);
    }
    assert_eq!(data.last_column(), 3);
    for row in 1..=20_000 {
        data.remove_row(row);
    }
    assert_eq!(data.last_column(), 0);
}

#[test]
fn last_column_is_rebuilt_after_decoding() {
    let mut data = SheetData::new();
    data.set_cell(2, 6, number(1.0));
    data.set_cell(9, 3, number(1.0));
    let mut decoded: SheetData = bitcode::decode(&bitcode::encode(&data)).unwrap();
    // the counts are not part of the bytes
    assert_eq!(decoded.last_column(), 0);
    decoded.rebuild_column_counts();
    assert_eq!(decoded.last_column(), 6);
    decoded.remove_cell(2, 6);
    assert_eq!(decoded.last_column(), 3);
}

/// The kept last row (T-0465) is right before and after a decode, before the rebuild too
/// (it scans then), and after every kind of write.
#[test]
fn the_last_row_is_kept_by_every_write() {
    let mut data = SheetData::new();
    data.set_cell(9, 3, number(1.0));
    data.set_cell(3000, 1, number(1.0));
    let mut decoded: SheetData = bitcode::decode(&bitcode::encode(&data)).unwrap();
    assert_eq!(decoded.last_row(), 3000);
    decoded.set_cell(2000, 1, number(1.0));
    assert_eq!(decoded.last_row(), 3000);
    decoded.rebuild_column_counts();
    assert_eq!(decoded.last_row(), 3000);
    decoded.set_row(4000, Vec::new());
    assert_eq!(decoded.last_row(), 4000);
    decoded.remove_row(4000);
    assert_eq!(decoded.last_row(), 3000);
    decoded.remove_cell(3000, 1);
    assert_eq!(decoded.last_row(), 2000);
    decoded.remove_cell(9, 3);
    decoded.remove_cell(2000, 1);
    assert_eq!(decoded.last_row(), 0);
    let mut fresh = SheetData::default();
    fresh.set_cell(5, 1, number(1.0));
    assert_eq!(fresh.last_row(), 5);
}

#[test]
fn the_column_index_follows_every_write() {
    let mut data = SheetData::new();
    let mut rows = Vec::new();
    let mut columns = Vec::new();
    for row in 1..=9_000 {
        data.set_cell(row, 7, number(1.0));
    }
    // not built yet: no columns, and the caller walks by rows
    assert!(!data.columns_between(1, 16_384, 8, &mut columns));
    assert!(columns.is_empty());
    data.build_column_index();
    data.set_cell(3, 2, number(1.0));
    data.set_cell(9_500, 2, number(1.0));
    data.set_cell(2, 3, number(1.0));
    data.rows_in_column_between(2, (1, LAST_ROW), 64, &mut rows);
    assert_eq!(rows, [3, 9_500]);
    data.rows_in_column_between(2, (4, 9_499), 64, &mut rows);
    assert!(rows.is_empty());
    data.rows_in_column_between(7, (10, LAST_ROW), 3, &mut rows);
    assert_eq!(rows, [10, 11, 12]);
    // an empty span is no rows, not a panic
    data.rows_in_column_between(7, (10, 9), 3, &mut rows);
    assert!(rows.is_empty());
    assert!(data.columns_between(1, 16_384, 8, &mut columns));
    assert_eq!(columns, [2, 3, 7]);
    assert!(!data.columns_between(1, 16_384, 2, &mut columns));
    assert_eq!(columns, [2, 3]);
    assert!(data.columns_between(4, 6, 8, &mut columns));
    assert!(columns.is_empty());
    // removing a cell, a row, or replacing a row keeps it in step, and each such write
    // changes the layout version
    let version = data.layout_version();
    data.remove_cell(3, 2);
    assert_ne!(data.layout_version(), version);
    data.set_row(2, vec![(4, number(1.0))]);
    data.remove_row(9_500);
    data.rows_in_column_between(2, (1, LAST_ROW), 64, &mut rows);
    assert!(rows.is_empty());
    assert!(data.columns_between(1, 16_384, 8, &mut columns));
    assert_eq!(columns, [4, 7]);
    // replacing a cell's value does not
    let version = data.layout_version();
    data.set_cell(2, 4, number(2.0));
    assert_eq!(data.layout_version(), version);
}

#[test]
fn last_column_is_the_rightmost_cell() {
    let mut data = SheetData::new();
    assert_eq!(data.last_column(), 0);
    data.set_cell(1, 2, number(1.0));
    data.set_cell(5000, 7, number(1.0));
    data.set_cell(3, 4, number(1.0));
    assert_eq!(data.last_column(), 7);
    data.remove_cell(5000, 7);
    assert_eq!(data.last_column(), 4);
    // removing a whole row, or replacing it, lowers it too
    data.set_cell(9, 12, number(1.0));
    assert_eq!(data.last_column(), 12);
    data.set_row(9, vec![(5, number(1.0))]);
    assert_eq!(data.last_column(), 5);
    data.set_row(9, Vec::new());
    assert_eq!(data.last_column(), 4);
    data.set_cell(9, 12, number(1.0));
    data.remove_row(9);
    assert_eq!(data.last_column(), 4);
    data.remove_row(1);
    data.remove_row(3);
    assert_eq!(data.last_column(), 0);
}

#[test]
fn set_get_replace_remove() {
    let mut data = SheetData::new();
    assert!(data.is_empty());
    assert_eq!(data.set_cell(3, 2, number(1.0)), None);
    assert_eq!(data.set_cell(3, 2, number(2.0)), Some(number(1.0)));
    assert_eq!(data.cell(3, 2), Some(&number(2.0)));
    assert_eq!(data.cell(3, 1), None);
    assert_eq!(data.cell(4, 2), None);
    *data.cell_mut(3, 2).unwrap() = number(5.0);
    assert_eq!(data.remove_cell(3, 2), Some(number(5.0)));
    assert_eq!(data.remove_cell(3, 2), None);
    // the last cell of the row took the row with it
    assert!(data.is_empty());
    assert!(data.rows().is_empty());
}

#[test]
fn natural_order_whatever_the_order_of_insertion() {
    let mut data = SheetData::new();
    // two chunks, and a row longer than the linear search limit, filled
    // from the right
    for column in (1..=20).rev() {
        data.set_cell(2000, column, number(column as f64));
    }
    data.set_cell(5, 7, number(0.0));
    data.set_cell(5, 3, number(0.0));
    data.set_cell(1, 9, number(0.0));
    let positions: Vec<(i32, i32)> = data.cells().map(|(r, c, _)| (r, c)).collect();
    let mut sorted = positions.clone();
    sorted.sort_unstable();
    assert_eq!(positions, sorted);
    assert_eq!(positions.len(), 23);
    assert_eq!(data.rows(), vec![1, 5, 2000]);
    assert_eq!(data.columns_in_row(5), vec![3, 7]);
    assert_eq!(data.cell(2000, 13), Some(&number(13.0)));
    assert_eq!(data.cells_in_row(2000).len(), 20);
    assert!(data.columns_in_row(6).is_empty());
}

#[test]
fn a_row_can_exist_without_cells() {
    let mut data = SheetData::new();
    data.set_row(10, Vec::new());
    assert!(!data.is_empty());
    assert_eq!(data.rows(), vec![10]);
    assert_eq!(data.cells().count(), 0);
    data.remove_row(10);
    assert!(data.is_empty());
}

#[test]
fn set_row_replaces_and_the_last_duplicate_stays() {
    let mut data = SheetData::new();
    data.set_cell(4, 1, number(1.0));
    data.set_row(
        4,
        vec![(5, number(5.0)), (2, number(2.0)), (5, number(6.0))],
    );
    assert_eq!(data.columns_in_row(4), vec![2, 5]);
    assert_eq!(data.cell(4, 5), Some(&number(6.0)));
    assert_eq!(data.cell(4, 1), None);
}

#[test]
fn rows_outside_the_sheet_hold_nothing() {
    let mut data = SheetData::new();
    for row in [0, -3, LAST_ROW + 1, i32::MAX, i32::MIN] {
        assert_eq!(data.set_cell(row, 1, number(1.0)), None);
        data.set_row(row, vec![(1, number(1.0))]);
        assert_eq!(data.cell(row, 1), None);
        data.remove_row(row);
    }
    assert!(data.is_empty());
    // the first and the last row of the sheet are fine
    data.set_cell(1, 1, number(1.0));
    data.set_cell(LAST_ROW, 1, number(2.0));
    assert_eq!(data.rows(), vec![1, LAST_ROW]);
}

#[test]
fn equality_is_about_cells_not_about_chunks() {
    let mut a = SheetData::new();
    let mut b = SheetData::new();
    a.set_cell(1, 1, number(1.0));
    b.set_cell(1, 1, number(1.0));
    // `a` has had a cell far away, and no longer has
    a.set_cell(900_000, 4, number(9.0));
    a.remove_cell(900_000, 4);
    assert_eq!(a, b);
    b.set_cell(1, 2, number(1.0));
    assert_ne!(a, b);
}
