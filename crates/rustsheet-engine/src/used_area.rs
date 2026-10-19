//! Clipping of range walks to a sheet's used area: the part of a range past the last row and
//! column that hold anything is blank, so a walk that skips blanks stops there and puts the rest
//! on record as read empty instead (T-0048, T-0049, T-0405).

use crate::model::Model;

/// Fewest blank cells past a sheet's used area that an explicit block must leave out to be cut
/// there ([`Model::clip_block_to_used_area`]): fewer are walked one by one, which costs about as
/// much as the rectangles recording them and spends nothing from the seen-range cap.
const MIN_CLIPPED_CELLS: i64 = 1024;

impl Model {
    /// The bottom-right corner of `sheet`'s used area (`Worksheet::used_area_end`).
    pub(crate) fn used_area_end_of(&self, sheet: u32) -> Result<(i32, i32), String> {
        Ok(self
            .workbook
            .worksheet(sheet)
            .map_err(|_| format!("Invalid worksheet index: '{sheet}'"))?
            .used_area_end())
    }

    /// Clips a whole-column or whole-row range to the sheet's used area, so
    /// that a function walking it visits only the cells that can hold
    /// something. Returns the bottom-right corner to walk to. The cells the
    /// walk will skip are put on record as read empty first: the formula
    /// depends on them, and a spill into them must restart the pass, however
    /// the used area changes while the walk runs.
    pub(crate) fn clip_to_used_area(
        &mut self,
        sheet: u32,
        row1: i32,
        column1: i32,
        row2: i32,
        column2: i32,
    ) -> Result<(i32, i32), String> {
        let corner = self.used_area_corner(sheet, row1, column1, row2, column2)?;
        Ok(self.clip_to_corner(sheet, (row1, column1), (row2, column2), corner))
    }

    /// [`Self::clip_to_used_area`] for any range, not only a whole-column or whole-row one: the
    /// block `row1:column1..=row2:column2` is cut at the sheet's used area, so that a walk of
    /// `Data1:Data999!A2:Z1048575` costs each sheet's data, not the block (T-0405). Only for
    /// walkers that skip blank cells: the cells left out are blank, and are not walked at all.
    /// The returned corner is above or left of the block's first cell when the block lies wholly
    /// past the used area, so that the walk visits nothing.
    ///
    /// An explicit block is cut only when the cut leaves out at least [`MIN_CLIPPED_CELLS`] cells and the
    /// formula's budget of seen-empty rectangles can pay for the record of them: otherwise its
    /// blank cells are walked and recorded one by one, as before, so a LAMBDA reading many small
    /// blocks costs no more than it did and never reaches the seen-range cap on their account
    /// (docs/excel-semantics/references.md "Seen-range cap").
    pub(crate) fn clip_block_to_used_area(
        &mut self,
        sheet: u32,
        row1: i32,
        column1: i32,
        row2: i32,
        column2: i32,
    ) -> Result<(i32, i32), String> {
        // Whole rows and columns are cut as they always were, whatever the budget.
        let whole_lines = self.used_area_corner(sheet, row1, column1, row2, column2)?;
        if whole_lines != (row2, column2) {
            return Ok(self.clip_to_corner(sheet, (row1, column1), (row2, column2), whole_lines));
        }
        let (max_row, max_column) = self.used_area_end_of(sheet)?;
        let corner = (
            row2.min(max_row).max(row1 - 1),
            column2.min(max_column).max(column1 - 1),
        );
        let cells = |(last_row, last_column): (i32, i32)| {
            i64::from(last_row - row1 + 1) * i64::from(last_column - column1 + 1)
        };
        let left_out = cells((row2, column2)) - cells(corner);
        if left_out < MIN_CLIPPED_CELLS || !self.area_budget.has_seen_ranges(2) {
            return Ok((row2, column2));
        }
        Ok(self.clip_to_corner(sheet, (row1, column1), (row2, column2), corner))
    }

    /// Puts on record as read empty the part of the block `first..=last` past `corner`, the
    /// bottom-right corner the walk will stop at, and returns that corner.
    fn clip_to_corner(
        &mut self,
        sheet: u32,
        (row1, column1): (i32, i32),
        (row2, column2): (i32, i32),
        (last_row, last_column): (i32, i32),
    ) -> (i32, i32) {
        if (last_row, last_column) == (row2, column2) {
            return (row2, column2);
        }
        // A block wholly past the used area walks nothing: one record covers it.
        if last_row < row1 || last_column < column1 {
            self.record_seen_empty_range(sheet, row1, column1, row2, column2);
            return (last_row, last_column);
        }
        // The remainder: the rows below the used area over the whole width,
        // and the columns beyond it within the rows that are walked.
        self.record_seen_empty_range(sheet, last_row + 1, column1, row2, column2);
        self.record_seen_empty_range(sheet, row1, last_column + 1, last_row, column2);
        (last_row, last_column)
    }
}
