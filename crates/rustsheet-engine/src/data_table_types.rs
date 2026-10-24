//! What-if data table metadata of a worksheet (`Worksheet::data_tables`); the evaluation is in
//! `data_tables.rs`.

use bitcode::{Decode, Encode};

use crate::constants::{LAST_COLUMN, LAST_ROW};

/// Result cells one data table may have (CLAUDE.md directive 5). A table re-evaluates the model
/// once per result cell and builds its results before writing them. Excel allows a table as large
/// as the sheet; rustsheet refuses one past this bound (docs/excel-semantics/recalc.md, DEVIATION).
pub const MAX_DATA_TABLE_CELLS: u64 = 1 << 20;

/// Data tables one sheet may have (directive 5): each new one is checked against the others.
pub const MAX_DATA_TABLES_PER_SHEET: usize = 1024;

/// Why a data table cannot be added to a sheet ([`DataTable::check`]).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataTableError {
    /// The results need a row above and a column left of them, all on the grid.
    #[error("a data table needs a row above and a column left of its results")]
    NoHeaders,
    /// Neither a row nor a column input cell.
    #[error("a data table needs a row or a column input cell")]
    NoInput,
    /// An input cell off the grid or among the results.
    #[error("a data table's input cell must be on the grid and outside its results")]
    BadInput,
    /// The results overlap another table's.
    #[error("a data table cannot overlap another")]
    Overlap,
    /// More than [`MAX_DATA_TABLE_CELLS`] results, or more than [`MAX_DATA_TABLES_PER_SHEET`]
    /// tables on the sheet.
    #[error("data table too large")]
    TooLarge,
    /// The sheet does not exist.
    #[error("no such sheet")]
    NoSheet,
}

/// A what-if data table (Excel's `TABLE()`, docs/excel-semantics/recalc.md). Each result cell holds
/// the value of a formula of the sheet with the input cell or cells replaced by values. For a
/// one-variable table the values come from the column left of `range` (`column_input`) or the
/// row above it (`row_input`), and the formulas are in the row above (respectively the column to
/// the left). A two-variable table has both inputs and one formula, at the corner above and left
/// of `range`.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct DataTable {
    /// The result cells: top row, left column, bottom row, right column (1-based).
    pub range: (i32, i32, i32, i32),
    /// The input cell the values of the row above `range` replace.
    pub row_input: Option<DataTableInput>,
    /// The input cell the values of the column left of `range` replace.
    pub column_input: Option<DataTableInput>,
}

impl DataTable {
    /// Checks that the table can join `others`, the tables already on its sheet: its results lie
    /// below and right of row and column 1, number at most [`MAX_DATA_TABLE_CELLS`] and overlap
    /// none of theirs, it has an input cell, every input is on the grid outside its results, and
    /// the sheet has room for it. Workbooks are untrusted, so the importer checks this too.
    ///
    /// # Errors
    /// The first rule the table breaks.
    pub fn check(&self, others: &[DataTable]) -> Result<(), DataTableError> {
        let (top, left, bottom, right) = self.range;
        if top < 2
            || left < 2
            || bottom < top
            || right < left
            || bottom > LAST_ROW
            || right > LAST_COLUMN
        {
            return Err(DataTableError::NoHeaders);
        }
        if self.cell_count() > MAX_DATA_TABLE_CELLS || others.len() >= MAX_DATA_TABLES_PER_SHEET {
            return Err(DataTableError::TooLarge);
        }
        if self.row_input.is_none() && self.column_input.is_none() {
            return Err(DataTableError::NoInput);
        }
        let inside = |r: i32, c: i32| (top..=bottom).contains(&r) && (left..=right).contains(&c);
        for input in [self.row_input, self.column_input].into_iter().flatten() {
            if let DataTableInput::Cell(r, c) = input {
                if inside(r, c) || r < 1 || c < 1 || r > LAST_ROW || c > LAST_COLUMN {
                    return Err(DataTableError::BadInput);
                }
            }
        }
        if others.iter().any(|t| {
            let (t2, l2, b2, r2) = t.range;
            t2 <= bottom && top <= b2 && l2 <= right && left <= r2
        }) {
            return Err(DataTableError::Overlap);
        }
        Ok(())
    }

    /// The number of result cells.
    #[must_use]
    pub fn cell_count(&self) -> u64 {
        let (top, left, bottom, right) = self.range;
        u64::from((bottom - top + 1).max(0).unsigned_abs())
            * u64::from((right - left + 1).max(0).unsigned_abs())
    }

    /// The text Excel shows for every result cell, e.g. `{=TABLE(,A1)}`: the row input first.
    #[must_use]
    pub fn formula_text(&self) -> String {
        let input = |i: Option<DataTableInput>| match i {
            None => String::new(),
            Some(DataTableInput::Deleted) => "#REF!".to_string(),
            Some(DataTableInput::Cell(row, column)) => {
                crate::expressions::utils::number_to_column(column)
                    .map(|c| format!("{c}{row}"))
                    .unwrap_or_default()
            }
        };
        format!(
            "{{=TABLE({},{})}}",
            input(self.row_input),
            input(self.column_input)
        )
    }
}

/// An input cell of a [`DataTable`], on the table's own sheet.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataTableInput {
    /// The cell at (row, column).
    Cell(i32, i32),
    /// The input cell was deleted (`del1`/`del2`): every result is `#REF!`.
    Deleted,
}
