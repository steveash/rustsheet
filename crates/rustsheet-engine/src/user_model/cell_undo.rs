//! Undo of a single cell value edit.

use crate::{
    types::{ArrayKind, Cell},
    UserModel,
};

impl UserModel {
    /// Restores `old_value` in the cell, first clearing the spill cells of a dynamic array
    /// anchor that the edit replaced.
    pub(super) fn undo_set_cell_value(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Option<&Cell>,
    ) -> Result<(), String> {
        // If the current cell is a dynamic ArrayFormula anchor, its
        // spill cells are not tracked in the diff (they are created
        // as a side-effect of evaluate()). Clear them now so that
        // after undo evaluate() re-spills into a clean area.
        let spill_dims = self
            .model
            .workbook
            .worksheet(sheet)
            .ok()
            .and_then(|ws| ws.cell(row, column))
            .and_then(|cell| match cell {
                Cell::ArrayFormula {
                    kind: ArrayKind::Dynamic,
                    r,
                    ..
                } => Some(*r),
                _ => None,
            });
        if let Some((w, h)) = spill_dims {
            let ws = self.model.workbook.worksheet_mut(sheet)?;
            for r in row..row + h {
                for c in column..column + w {
                    if r == row && c == column {
                        continue;
                    }
                    if matches!(ws.cell(r, c), Some(Cell::SpillCell { a, .. }) if *a == (row, column))
                    {
                        let _ = ws.cell_clear_contents(r, c);
                    }
                }
            }
        }
        match old_value.cloned() {
            Some(value) => {
                self.model
                    .workbook
                    .worksheet_mut(sheet)?
                    .update_cell(row, column, value)?;
            }
            None => {
                if spill_dims.is_some() {
                    // The spill cells were already cleared above; only
                    // the anchor itself remains.  range_clear_all would
                    // re-expand to the full spill range and erase cells
                    // that were just restored by earlier diffs in this
                    // same undo operation (e.g. the cut source that
                    // overlaps the paste target's spill area).
                    let _ = self
                        .model
                        .workbook
                        .worksheet_mut(sheet)?
                        .cell_clear_contents(row, column);
                } else {
                    self.model
                        .workbook
                        .worksheet_mut(sheet)?
                        .cell_clear_contents(row, column)?;
                }
            }
        }
        Ok(())
    }
}
