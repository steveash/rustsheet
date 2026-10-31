//! Clears the covered cells of a sheet's merged ranges, in time proportional to the cells and
//! ranges the sheet holds, not to the area the ranges name (T-3442, directive 5).

use std::collections::BTreeMap;

use rustsheet_engine::types::{Cell, MergedCell, SheetData};

/// Empties the cells that `merged_cells` cover (all but each range's top left cell), keeping their
/// styles: covered cells must not hold content (an engine invariant), and Excel leaves them empty
/// while third-party producers sometimes leave values behind.
///
/// One sweep over the stored cells, in row order, with the ranges whose rows are open kept by first
/// column: a range can name the whole sheet in a few bytes, so walking each one's area would cost
/// 2^34 lookups. `load_merge_cells` drops intersecting ranges, so the open ones never overlap, and
/// the one with the largest first column at or before a cell is the only one that can cover it.
pub(super) fn clear_covered_cells(sheet_data: &mut SheetData, merged_cells: &[MergedCell]) {
    let mut by_row: Vec<&MergedCell> = merged_cells.iter().collect();
    by_row.sort_by_key(|merged| merged.row);
    let mut opening = by_row.into_iter().peekable();
    let mut open: BTreeMap<i32, &MergedCell> = BTreeMap::new();
    let mut covered = Vec::new();
    for (row, column, _) in sheet_data.cells() {
        while let Some(merged) = opening.next_if(|merged| merged.row <= row) {
            open.insert(merged.column, merged);
        }
        // A range whose rows have closed is dropped when first met; each is dropped once.
        while let Some((&first, merged)) = open.range(..=column).next_back() {
            if merged.last_row() >= row {
                if column <= merged.last_column() && (row, column) != (merged.row, merged.column) {
                    covered.push((row, column));
                }
                break;
            }
            open.remove(&first);
        }
    }
    for (row, column) in covered {
        if let Some(cell) = sheet_data.cell_mut(row, column) {
            *cell = Cell::EmptyCell {
                s: cell.get_style(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use rustsheet_engine::types::{Cell, MergedCell, SheetData};

    use super::clear_covered_cells;

    fn number(v: f64) -> Cell {
        Cell::NumberCell { v, s: 3 }
    }

    fn merged(row: i32, column: i32, width: i32, height: i32) -> MergedCell {
        MergedCell {
            row,
            column,
            width,
            height,
        }
    }

    /// Covered cells are emptied with their style kept; anchors and cells outside stay.
    #[test]
    fn covered_cells_are_emptied_and_the_rest_kept() {
        let mut data = SheetData::new();
        for (row, column) in [(1, 1), (1, 2), (2, 1), (2, 2), (3, 3), (5, 2), (6, 2)] {
            data.set_cell(row, column, number(1.0));
        }
        let ranges = [merged(1, 1, 2, 2), merged(5, 2, 1, 2), merged(3, 4, 1, 1)];
        clear_covered_cells(&mut data, &ranges);
        let empty = Cell::EmptyCell { s: 3 };
        assert_eq!(data.cell(1, 1), Some(&number(1.0)));
        assert_eq!(data.cell(1, 2), Some(&empty));
        assert_eq!(data.cell(2, 1), Some(&empty));
        assert_eq!(data.cell(2, 2), Some(&empty));
        assert_eq!(data.cell(3, 3), Some(&number(1.0)));
        assert_eq!(data.cell(5, 2), Some(&number(1.0)));
        assert_eq!(data.cell(6, 2), Some(&empty));
    }

    /// A closed range with a smaller first column does not hide an open one, and a range that
    /// starts later in the sheet is only open from its first row.
    #[test]
    fn closed_and_later_ranges_do_not_mix_up() {
        let mut data = SheetData::new();
        for (row, column) in [(1, 1), (1, 3), (2, 3), (4, 1), (4, 3), (5, 3)] {
            data.set_cell(row, column, number(2.0));
        }
        let ranges = [merged(1, 1, 4, 1), merged(4, 3, 1, 2)];
        clear_covered_cells(&mut data, &ranges);
        assert_eq!(data.cell(1, 3), Some(&Cell::EmptyCell { s: 3 }));
        assert_eq!(data.cell(2, 3), Some(&number(2.0)));
        assert_eq!(data.cell(4, 1), Some(&number(2.0)));
        assert_eq!(data.cell(4, 3), Some(&number(2.0)));
        assert_eq!(data.cell(5, 3), Some(&Cell::EmptyCell { s: 3 }));
    }
}
