//! Whether a structural edit may go ahead: the checks [`Model::delete_rows`] and
//! [`Model::delete_columns`] make before any change, and that `UserModel::delete_rows` makes before
//! its table branch clears a row (T-3259).

use crate::constants::LAST_ROW;
use crate::model::{CellStructure, Model};

impl Model {
    /// Why the delete of `row_count` rows at `row` on `sheet` is refused, if it is: a count below 1,
    /// a row or a span outside the sheet, or an array formula the delete would cut. Where the delete
    /// takes a table's every data row ([`Model::kept_data_row`]) it clears the first row and deletes
    /// only the rest, so that shorter delete must not cut an array formula either (one starting in
    /// the kept row). Checked before any change, by [`Model::delete_rows`] and by
    /// `UserModel::delete_rows`, both of which clear that row before they delete (T-3259).
    pub(crate) fn check_delete_rows(
        &self,
        sheet: u32,
        row: i32,
        row_count: i32,
    ) -> Result<(), String> {
        if row_count <= 0 {
            return Err("Please use insert rows instead".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        if row_count > LAST_ROW - row + 1 {
            return Err("Cannot delete rows beyond the last row of the sheet".to_string());
        }
        let cuts = !self.can_delete_rows(sheet, row, row_count)?
            || (row_count > 1
                && self.kept_data_row(sheet, row, row_count).is_some()
                && !self.can_delete_rows(sheet, row + 1, row_count - 1)?);
        if cuts {
            return Err("Cannot delete rows because that would break an array formula".to_string());
        }
        Ok(())
    }

    // Returns true if deleting rows [row, row + row_count - 1] would not break any
    // array formula. An array formula must be either fully inside the deleted range
    // or fully outside it; any partial overlap is rejected.
    fn can_delete_rows(&self, sheet: u32, row: i32, row_count: i32) -> Result<bool, String> {
        let row_end = row + row_count; // exclusive upper bound
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .cells()
                .map(|(r, c, _)| (r, c))
                .collect()
        };
        for (r, c) in cell_coords {
            if let CellStructure::ArrayFormula { range: (_, height) } =
                self.get_cell_structure(sheet, r, c)?
            {
                // Formula row span: [r, r + height - 1]
                let overlaps = r < row_end && r + height > row;
                let contained = r >= row && r + height <= row_end;
                if overlaps && !contained {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    // Returns true if deleting columns [column, column + column_count - 1] would not
    // break any array formula.
    pub(crate) fn can_delete_columns(
        &self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<bool, String> {
        let col_end = column + column_count; // exclusive upper bound
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .cells()
                .map(|(r, c, _)| (r, c))
                .collect()
        };
        for (r, c) in cell_coords {
            if let CellStructure::ArrayFormula { range: (width, _) } =
                self.get_cell_structure(sheet, r, c)?
            {
                // Formula column span: [c, c + width - 1]
                let overlaps = c < col_end && c + width > column;
                let contained = c >= column && c + width <= col_end;
                if overlaps && !contained {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}
