//! The harness-only structural edits of a [`Workbook`] (decisions 0076, 0098): not [`crate::Txn`]
//! edits, so no undo and no change set, and none recalculates.

use rustsheet_engine::{LAST_COLUMN, LAST_ROW};

use super::Workbook;
use crate::error::RefError;
use crate::refs::SheetId;

impl Workbook {
    /// Inserts `count` blank rows above the 0-based `row` of `sheet`, shifting the cells below
    /// down and rewriting the formulas that refer to them, as Excel's row insert does. Harness
    /// only (decision 0076): it is not a [`crate::Txn`] edit, so it has no undo and reports no
    /// change set; the eval replays a case's `insert_row` sheet op with it. It does not
    /// recalculate.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, a row past the sheet or a `count` of 0 or more
    /// rows than lie below `row`; [`EditError::Engine`] when
    /// the engine refuses the insert (it would push cells off the sheet).
    #[cfg(feature = "harness_row_insert")]
    pub fn insert_rows(
        &mut self,
        sheet: SheetId,
        row: u32,
        count: u32,
    ) -> Result<(), crate::error::EditError> {
        let index = self.sheet_index(sheet)?;
        let (row, count) = harness_span(row, count, LAST_ROW)?;
        self.model
            .insert_rows(index, row, count)
            .map_err(crate::error::EditError::Engine)
    }

    /// Deletes `count` rows from the 0-based `row` of `sheet`, shifting the cells below up; the engine
    /// rewrites the formulas that refer to them (Excel: docs/excel-semantics/references.md "Structured
    /// references under row deletes, column edits, convert, table and sheet delete"; T-2338). Harness only (decisions 0076, 0098):
    /// as [`Self::insert_rows`], it has no undo and does not recalculate.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, a row past the sheet or a `count` of 0 or more
    /// rows than lie from `row`; [`EditError::Engine`] when the engine refuses the delete.
    #[cfg(feature = "harness_row_insert")]
    pub fn delete_rows(
        &mut self,
        sheet: SheetId,
        row: u32,
        count: u32,
    ) -> Result<(), crate::error::EditError> {
        let index = self.sheet_index(sheet)?;
        let (row, count) = harness_span(row, count, LAST_ROW)?;
        self.model
            .delete_rows(index, row, count)
            .map_err(crate::error::EditError::Engine)
    }

    /// Inserts `count` blank columns left of the 0-based `column` of `sheet`, shifting the cells
    /// right. Harness only (decisions 0076, 0098); errors as [`Self::insert_rows`] with columns.
    ///
    /// # Errors
    /// As [`Self::insert_rows`], for columns.
    #[cfg(feature = "harness_row_insert")]
    pub fn insert_columns(
        &mut self,
        sheet: SheetId,
        column: u32,
        count: u32,
    ) -> Result<(), crate::error::EditError> {
        let index = self.sheet_index(sheet)?;
        let (column, count) = harness_span(column, count, LAST_COLUMN)?;
        self.model
            .insert_columns(index, column, count)
            .map_err(crate::error::EditError::Engine)
    }

    /// Deletes `count` columns from the 0-based `column` of `sheet`, shifting the cells left.
    /// Harness only (decisions 0076, 0098); errors as [`Self::delete_rows`] with columns.
    ///
    /// # Errors
    /// As [`Self::delete_rows`], for columns.
    #[cfg(feature = "harness_row_insert")]
    pub fn delete_columns(
        &mut self,
        sheet: SheetId,
        column: u32,
        count: u32,
    ) -> Result<(), crate::error::EditError> {
        let index = self.sheet_index(sheet)?;
        let (column, count) = harness_span(column, count, LAST_COLUMN)?;
        self.model
            .delete_columns(index, column, count)
            .map_err(crate::error::EditError::Engine)
    }

    /// Deletes the table `name` and clears its cells, as Graph's `DELETE tables('T')` does
    /// (T-2341). Harness only (decisions 0076, 0098): the eval replays a case's `delete_table`
    /// sheet op with it. It does not recalculate.
    ///
    /// # Errors
    /// [`EditError::Engine`] when there is no table with that exact name.
    #[cfg(feature = "harness_row_insert")]
    pub fn delete_table(&mut self, name: &str) -> Result<(), crate::error::EditError> {
        self.model
            .delete_table(name)
            .map_err(crate::error::EditError::Engine)
    }

    /// Converts the table `name` to a plain range, as Graph's `convertToRange` does (T-2342): the
    /// cells stay and every structured reference to it becomes the absolute range it resolved to.
    /// Harness only (decisions 0076, 0098). It does not recalculate.
    ///
    /// # Errors
    /// [`EditError::Engine`] when there is no table with that exact name.
    #[cfg(feature = "harness_row_insert")]
    pub fn convert_table_to_range(&mut self, name: &str) -> Result<(), crate::error::EditError> {
        self.model
            .convert_table_to_range(name)
            .map_err(crate::error::EditError::Engine)
    }
}

/// The engine's 1-based start and count for a harness row or column edit at the 0-based `start`
/// of `count` lines in a sheet `last` lines long: at least one line, and no more than lie from
/// `start` to the end.
#[cfg(feature = "harness_row_insert")]
fn harness_span(start: u32, count: u32, last: i32) -> Result<(i32, i32), crate::error::EditError> {
    let start = start
        .checked_add(1)
        .and_then(|s| i32::try_from(s).ok())
        .filter(|s| *s <= last);
    let count = i32::try_from(count)
        .ok()
        .filter(|c| *c >= 1 && start.is_some_and(|s| *c <= last - s + 1));
    match (start, count) {
        (Some(start), Some(count)) => Ok((start, count)),
        _ => Err(crate::error::EditError::Ref(RefError::OutOfBounds)),
    }
}
