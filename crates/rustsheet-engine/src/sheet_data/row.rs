//! A row of a sheet: its cells with their column, sorted by column, and the search for one.

use bitcode::{Decode, Encode};

use crate::constants::LAST_COLUMN;
use crate::types::Cell;

/// Rows this short are searched from the left; longer ones by bisection.
const LINEAR_SEARCH_LIMIT: usize = 8;

/// Whether `column` is one of the sheet's: a row holds cells at these columns only (T-3624).
#[inline]
pub(super) fn on_sheet_column(column: i32) -> bool {
    (1..=LAST_COLUMN).contains(&column)
}

/// The cells of a row with their column, sorted by column.
#[derive(Encode, Decode, Debug, Clone, Default, PartialEq)]
pub(super) struct Row {
    pub(super) cells: Vec<(i32, Cell)>,
}

impl Row {
    /// Where the column is, or where it would go.
    #[inline]
    pub(super) fn position(&self, column: i32) -> Result<usize, usize> {
        if self.cells.len() <= LINEAR_SEARCH_LIMIT {
            for (index, (c, _)) in self.cells.iter().enumerate() {
                if *c >= column {
                    return if *c == column { Ok(index) } else { Err(index) };
                }
            }
            Err(self.cells.len())
        } else {
            let window = self.window(column);
            self.cells[window.clone()]
                .binary_search_by_key(&column, |(c, _)| *c)
                .map(|i| window.start + i)
                .map_err(|i| window.start + i)
        }
    }

    /// The indices a column can sit at, or be inserted at, in a row of more than one cell: its
    /// columns are distinct and ascending, so the cell at index `i` is at least `i` columns past
    /// the first (`i <= column - first`) and at least `len - 1 - i` columns before the last
    /// (`i >= len - 1 - (last - column)`). A dense row, or one with a few gaps,
    /// leaves a window of a few cells, bisected in a probe or two on neighbouring cache lines
    /// instead of ten across the row (T-3687; a dense row's window is its one cell, T-3608).
    fn window(&self, column: i32) -> std::ops::Range<usize> {
        let (Some(&(first, _)), Some(&(last, _))) = (self.cells.first(), self.cells.last()) else {
            return 0..0;
        };
        let len = self.cells.len();
        let count = |d: i64| usize::try_from(d).unwrap_or(0).min(len);
        let (column, first, last) = (i64::from(column), i64::from(first), i64::from(last));
        // Below the first, the window is `0..0` (an insertion before it); past the last, `len..len`.
        let low = count(i64::try_from(len).unwrap_or(i64::MAX) - 1 - (last - column));
        let high = count(column - first + 1);
        low.min(high)..high
    }

    pub(super) fn insert(&mut self, column: i32, cell: Cell) -> Option<Cell> {
        // Rows are mostly filled from left to right.
        if self.cells.last().is_none_or(|(last, _)| *last < column) {
            self.cells.push((column, cell));
            return None;
        }
        match self.position(column) {
            Ok(index) => Some(std::mem::replace(&mut self.cells[index].1, cell)),
            Err(index) => {
                self.cells.insert(index, (column, cell));
                None
            }
        }
    }
}
