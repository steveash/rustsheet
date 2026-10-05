use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, model::Model};

use super::array_node_to_calc_result;

/// A two dimensional table for the LOOKUP family of functions. It abstracts over
/// the two possible sources, a range reference or an in-formula array literal, so
/// the lookup logic can be written once for both.
pub(super) enum LookupTable {
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
    Array(Vec<Vec<ArrayNode>>),
}

impl LookupTable {
    pub(super) fn rows(&self) -> i32 {
        match self {
            LookupTable::Range { left, right } => right.row - left.row + 1,
            LookupTable::Array(array) => array.len() as i32,
        }
    }

    pub(super) fn columns(&self) -> i32 {
        match self {
            LookupTable::Range { left, right } => right.column - left.column + 1,
            LookupTable::Array(array) => array.first().map_or(0, |row| row.len()) as i32,
        }
    }

    /// Returns the value at the 0-based `(row, column)` offset within the table.
    /// Callers must ensure the offsets are within bounds.
    pub(super) fn get(
        &self,
        model: &mut Model,
        row: i32,
        column: i32,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self {
            LookupTable::Range { left, .. } => model.evaluate_cell(CellReferenceIndex {
                sheet: left.sheet,
                row: left.row + row,
                column: left.column + column,
            }),
            LookupTable::Array(array) => {
                array_node_to_calc_result(&array[row as usize][column as usize], cell)
            }
        }
    }

    /// Materializes the first row of the table (the search vector for HLOOKUP).
    ///
    /// Columns of a range right of the sheet's last used column are all empty,
    /// so they are not materialized (see `first_column`).
    pub(super) fn first_row(&self, model: &mut Model, cell: CellReferenceIndex) -> Vec<CalcResult> {
        let columns = match self {
            LookupTable::Range { left, .. } => {
                let last_used = model
                    .workbook
                    .worksheet(left.sheet)
                    .map_or(LAST_COLUMN, |ws| ws.last_used_column());
                self.columns().min(last_used - left.column + 1).max(0)
            }
            LookupTable::Array(_) => self.columns(),
        };
        (0..columns)
            .map(|column| self.get(model, 0, column, cell))
            .collect()
    }

    /// Materializes the first column of the table (the search vector for VLOOKUP).
    ///
    /// Rows of a range below the sheet's last used row are all empty, so they are
    /// not materialized: a full-column reference such as `A:B` would otherwise
    /// evaluate a million cells per call. Empty cells never match a lookup value.
    pub(super) fn first_column(
        &self,
        model: &mut Model,
        cell: CellReferenceIndex,
    ) -> Vec<CalcResult> {
        let rows = match self {
            LookupTable::Range { left, .. } => {
                let last_used = model
                    .workbook
                    .worksheet(left.sheet)
                    .map_or(LAST_ROW, |ws| ws.last_used_row());
                self.rows().min(last_used - left.row + 1).max(0)
            }
            LookupTable::Array(_) => self.rows(),
        };
        (0..rows).map(|row| self.get(model, row, 0, cell)).collect()
    }
}
