//! A row of a sheet: its cells with their column, sorted by column, and the search for one
//! (moved out of `sheet_data.rs` by T-3628).

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
            // A dense row holds its cells at their offset from the first: tried before bisecting.
            let offset =
                |&(first, _): &(i32, Cell)| usize::try_from(column.checked_sub(first)?).ok();
            let at = |&g: &usize| self.cells.get(g).is_some_and(|(c, _)| *c == column);
            if let Some(g) = self.cells.first().and_then(offset).filter(at) {
                return Ok(g);
            }
            self.cells.binary_search_by_key(&column, |(c, _)| *c)
        }
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
