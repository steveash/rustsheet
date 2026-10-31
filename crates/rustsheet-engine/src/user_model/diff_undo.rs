//! The undo of the edits whose diffs carry cells, records and areas that are written straight into
//! the sheet. A diff list from a peer (`UserModel::apply_external_diffs`) is untrusted (CLAUDE.md
//! directive 5, T-2896), so each of these is checked against the model as it is when the diff
//! applies, with the checks a loaded workbook passes (`workbook_check`), before anything is written.

use crate::{
    constants::COLUMN_WIDTH_FACTOR,
    expressions::utils::check_block,
    types::{Cell, Style},
    user_model::history::{ColumnData, RowData},
    workbook_check::{check_cell, check_row_record, check_style},
    UserModel,
};

/// A block as (`row`, `column`) of its top left cell and (`width`, `height`).
type Block = ((i32, i32), (i32, i32));

impl UserModel {
    /// Checks `cell` before an undo restores it at (`row`, `column`) of `sheet`, and counts it
    /// into the cells this call restores (`restored_spills`, T-3426).
    pub(super) fn check_restored_cell(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        cell: &Cell,
    ) -> Result<(), String> {
        let formulas = self.model.workbook.worksheet(sheet)?.shared_formulas.len();
        check_cell(
            row,
            column,
            cell,
            self.model.workbook.styles.cell_xfs.len(),
            formulas,
        )?;
        self.restored_spills.add(cell);
        Ok(())
    }

    /// Undoes an array formula entered over the block: clears it and restores `old_values`.
    pub(super) fn undo_array_value(
        &mut self,
        sheet: u32,
        (corner, size): Block,
        old_values: &[Vec<Option<Cell>>],
    ) -> Result<(), String> {
        check_block(corner.0, corner.1, size.0, size.1)?;
        let restored = restored_cells((corner, size), old_values);
        for &(r, c, cell) in &restored {
            self.check_restored_cell(sheet, r, c, cell)?;
        }
        // Clear the cells of the array formula (anchor and spill cells); only those that exist,
        // since the block can be the whole sheet.
        let ws = self.model.workbook.worksheet_mut(sheet)?;
        for (r, c) in ws.sheet_data.places_in_block(corner, size) {
            let _ = ws.cell_clear_contents(r, c);
        }
        // Restore all cells that existed before the array formula was placed.
        for (r, c, cell) in restored {
            ws.update_cell(r, c, cell.clone())?;
        }
        Ok(())
    }

    /// Undoes a clear of the block's contents, or of everything when `old_style` holds the styles
    /// the cells had: restores `old_value` (and those styles).
    pub(super) fn undo_range_clear(
        &mut self,
        sheet: u32,
        (corner, size): Block,
        old_value: &[Vec<Option<Cell>>],
        old_style: Option<&Vec<Vec<Style>>>,
    ) -> Result<(), String> {
        check_block(corner.0, corner.1, size.0, size.1)?;
        let mut restored = Vec::new();
        for (r, c, cell) in restored_cells((corner, size), old_value) {
            self.check_restored_cell(sheet, r, c, cell)?;
            let style = match old_style {
                // The offsets are not negative: `restored_cells` starts at the corner.
                Some(styles) => Some(
                    usize::try_from(r - corner.0)
                        .ok()
                        .zip(usize::try_from(c - corner.1).ok())
                        .and_then(|(i, j)| styles.get(i)?.get(j))
                        .ok_or_else(|| format!("cell R{r}C{c} is restored without its style"))?,
                ),
                None => None,
            };
            restored.push((r, c, cell, style));
        }
        for (r, c, cell, style) in restored {
            let worksheet = self.model.workbook.worksheet_mut(sheet)?;
            worksheet.update_cell(r, c, cell.clone())?;
            if let Some(style) = style {
                self.model.set_cell_style(sheet, r, c, style)?;
            }
        }
        Ok(())
    }

    /// Undoes a delete of the rows from `row`: inserts them again and restores `old_data`.
    pub(super) fn undo_delete_rows(
        &mut self,
        sheet: u32,
        row: i32,
        old_data: &[RowData],
    ) -> Result<(), String> {
        let count = restored_count(old_data.len())?;
        check_block(row, 1, 1, count)?;
        let styles = self.model.workbook.styles.cell_xfs.len();
        for (r, row_data) in (row..row + count).zip(old_data) {
            if let Some(record) = &row_data.row {
                check_row_record(record, styles)?;
            }
            for (c, cell) in &row_data.data {
                self.check_restored_cell(sheet, r, *c, cell)?;
            }
        }
        self.model.insert_rows(sheet, row, count)?;
        let worksheet = self.model.workbook.worksheet_mut(sheet)?;
        for (r, row_data) in (row..row + count).zip(old_data) {
            if let Some(row_style) = row_data.row.clone() {
                worksheet.rows.push(row_style);
            }
            worksheet.sheet_data.set_row(r, row_data.data.clone());
        }
        Ok(())
    }

    /// Undoes a delete of the columns from `column`: inserts them again and restores `old_data`.
    pub(super) fn undo_delete_columns(
        &mut self,
        sheet: u32,
        column: i32,
        old_data: &[ColumnData],
    ) -> Result<(), String> {
        let count = restored_count(old_data.len())?;
        check_block(1, column, count, 1)?;
        let styles = self.model.workbook.styles.cell_xfs.len();
        for (c, col_data) in (column..column + count).zip(old_data) {
            if let Some(style) = col_data.column.as_ref().and_then(|col| col.style) {
                check_style(style, styles).map_err(|e| format!("column {c} {e}"))?;
            }
            for (r, cell) in &col_data.data {
                self.check_restored_cell(sheet, *r, c, cell)?;
            }
        }
        self.model.insert_columns(sheet, column, count)?;
        let worksheet = self.model.workbook.worksheet_mut(sheet)?;
        for (c, col_data) in (column..column + count).zip(old_data) {
            for (row, cell) in &col_data.data {
                worksheet.update_cell(*row, c, cell.clone())?;
            }
            if let Some(col) = &col_data.column {
                let width = col.width * COLUMN_WIDTH_FACTOR;
                worksheet.set_column_width_and_style(c, width, col.hidden, col.style)?;
            }
        }
        Ok(())
    }
}

/// The number of rows or columns an undo restores, refused when it is no `i32`.
fn restored_count(len: usize) -> Result<i32, String> {
    i32::try_from(len).map_err(|_| "Too many rows or columns to restore".to_string())
}

/// The cells of `old_values`, laid over a block `check_block` accepted, with their places; the
/// cells past the block are not restored.
fn restored_cells(
    ((row, column), (width, height)): Block,
    old_values: &[Vec<Option<Cell>>],
) -> Vec<(i32, i32, &Cell)> {
    (row..row + height)
        .zip(old_values)
        .flat_map(|(r, cells)| {
            (column..column + width)
                .zip(cells)
                .filter_map(move |(c, cell)| cell.as_ref().map(|cell| (r, c, cell)))
        })
        .collect()
}

/// Where the undo of a move by `delta` starts and how far it moves back, refused when either
/// is no `i32`.
pub(super) fn moved_back(first: i32, delta: i32) -> Result<(i32, i32), String> {
    first
        .checked_add(delta)
        .zip(delta.checked_neg())
        .ok_or_else(|| "The move is outside the sheet".to_string())
}
