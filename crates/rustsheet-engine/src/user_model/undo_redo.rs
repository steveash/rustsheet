#![deny(missing_docs)]

use crate::{
    cf_types::ConditionalFormatting,
    constants::COLUMN_WIDTH_FACTOR,
    expressions::{types::Area, utils::is_valid_block},
    types::{Cell, Style},
    workbook_check::{
        check_cell, check_merged_cells, check_row_record, check_style, check_worksheet,
    },
    UserModel,
};

use crate::user_model::common::selected_sheet_after_move;
use crate::user_model::history::{Diff, DiffList};

impl UserModel {
    pub(super) fn apply_undo_diff_list(&mut self, diff_list: &DiffList) -> Result<(), String> {
        let mut needs_evaluation = false;
        for diff in diff_list.iter().rev() {
            match diff {
                Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    old_value,
                    ..
                } => {
                    needs_evaluation = true;
                    if let Some(cell) = old_value.as_ref() {
                        self.check_restored_cell(*sheet, *row, *column, cell)?;
                    }
                    self.undo_set_cell_value(*sheet, *row, *column, old_value.as_ref().as_ref())?;
                }
                Diff::SetArrayValue {
                    sheet,
                    row,
                    column,
                    width,
                    height,
                    new_value: _,
                    old_values,
                } => {
                    needs_evaluation = true;
                    check_block(*row, *column, *width, *height)?;
                    let restored = restored_cells(*row, *column, *width, *height, old_values);
                    for &(r, c, cell) in &restored {
                        self.check_restored_cell(*sheet, r, c, cell)?;
                    }
                    // Clear all cells in the array formula range (anchor + spill cells).
                    let ws = self.model.workbook.worksheet_mut(*sheet)?;
                    for r in *row..*row + *height {
                        for c in *column..*column + *width {
                            let _ = ws.cell_clear_contents(r, c);
                        }
                    }
                    // Restore all cells that existed before the array formula was placed.
                    for (r, c, cell) in restored {
                        self.model.workbook.worksheet_mut(*sheet)?.update_cell(
                            r,
                            c,
                            cell.clone(),
                        )?;
                    }
                }
                Diff::SetColumnWidth {
                    sheet,
                    column,
                    new_value: _,
                    old_value,
                } => self.model.set_column_width(*sheet, *column, *old_value)?,
                Diff::SetColumnHidden {
                    sheet,
                    column,
                    new_value: _,
                    old_value,
                } => {
                    self.model.set_column_hidden(*sheet, *column, *old_value)?;
                }
                Diff::SetRowHidden {
                    sheet,
                    row,
                    new_value: _,
                    old_value,
                } => {
                    self.model.set_row_hidden(*sheet, *row, *old_value)?;
                }
                Diff::SetRowHeight {
                    sheet,
                    row,
                    new_value: _,
                    old_value,
                } => self.model.set_row_height(*sheet, *row, *old_value)?,
                Diff::RangeClearContents {
                    sheet,
                    row,
                    column,
                    width,
                    height,
                    old_value,
                } => {
                    needs_evaluation = true;
                    check_block(*row, *column, *width, *height)?;
                    let restored = restored_cells(*row, *column, *width, *height, old_value);
                    for &(r, c, cell) in &restored {
                        self.check_restored_cell(*sheet, r, c, cell)?;
                    }
                    for (r, c, cell) in restored {
                        self.model.workbook.worksheet_mut(*sheet)?.update_cell(
                            r,
                            c,
                            cell.clone(),
                        )?;
                    }
                }
                Diff::RangeClearAll {
                    sheet,
                    row,
                    column,
                    width,
                    height,
                    old_value,
                    old_style,
                } => {
                    needs_evaluation = true;
                    check_block(*row, *column, *width, *height)?;
                    let restored = restored_cells(*row, *column, *width, *height, old_value);
                    let mut styled = Vec::with_capacity(restored.len());
                    for (r, c, cell) in restored {
                        self.check_restored_cell(*sheet, r, c, cell)?;
                        // The offsets are not negative: `restored_cells` starts at the corner.
                        let style = usize::try_from(r - *row)
                            .ok()
                            .zip(usize::try_from(c - *column).ok())
                            .and_then(|(i, j)| old_style.get(i)?.get(j))
                            .ok_or_else(|| {
                                format!("cell R{r}C{c} is restored without its style")
                            })?;
                        styled.push((r, c, cell, style));
                    }
                    for (r, c, cell, style) in styled {
                        self.model.workbook.worksheet_mut(*sheet)?.update_cell(
                            r,
                            c,
                            cell.clone(),
                        )?;
                        self.model.set_cell_style(*sheet, r, c, style)?;
                    }
                }
                Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value,
                    new_value: _,
                }
                | Diff::ApplyNamedStyle {
                    sheet,
                    row,
                    column,
                    old_value,
                    name: _,
                } => {
                    if let Some(old_style) = old_value.as_ref() {
                        self.model
                            .set_cell_style(*sheet, *row, *column, old_style)?;
                    } else {
                        // The cell had no explicit style before this operation.
                        // If it still holds formula/value content, only reset the style
                        // index — never touch content, and never call range_clear_all
                        // (which would expand across a dynamic spill range).
                        // If it is truly empty, remove it from sheet_data so that
                        // row/column styles are inherited again.
                        let has_content = self
                            .model
                            .workbook
                            .worksheet(*sheet)
                            .ok()
                            .and_then(|ws| ws.cell(*row, *column))
                            .map(|c| !matches!(c, Cell::EmptyCell { .. }))
                            .unwrap_or(false);
                        if has_content {
                            self.model
                                .set_cell_style(*sheet, *row, *column, &Style::default())?;
                        } else {
                            let area = Area {
                                sheet: *sheet,
                                row: *row,
                                column: *column,
                                width: 1,
                                height: 1,
                            };
                            self.model.range_clear_all(&area)?;
                        }
                    }
                }
                Diff::InsertRows { sheet, row, count } => {
                    self.model.delete_rows(*sheet, *row, *count)?;
                    needs_evaluation = true;
                }
                Diff::DeleteRows {
                    sheet,
                    row,
                    count: _,
                    old_data,
                } => {
                    needs_evaluation = true;
                    let count = restored_count(old_data.len())?;
                    check_block(*row, 1, 1, count)?;
                    let styles = self.model.workbook.styles.cell_xfs.len();
                    for (r, row_data) in (*row..*row + count).zip(old_data) {
                        if let Some(record) = &row_data.row {
                            check_row_record(record, styles)?;
                        }
                        for (c, cell) in &row_data.data {
                            self.check_restored_cell(*sheet, r, *c, cell)?;
                        }
                    }
                    self.model.insert_rows(*sheet, *row, count)?;
                    let worksheet = self.model.workbook.worksheet_mut(*sheet)?;
                    for (r, row_data) in (*row..*row + count).zip(old_data) {
                        if let Some(row_style) = row_data.row.clone() {
                            worksheet.rows.push(row_style);
                        }
                        worksheet.sheet_data.set_row(r, row_data.data.clone());
                    }
                }
                Diff::InsertColumns {
                    sheet,
                    column,
                    count,
                } => {
                    self.model.delete_columns(*sheet, *column, *count)?;
                    needs_evaluation = true;
                }
                Diff::DeleteColumns {
                    sheet,
                    column,
                    count: _,
                    old_data,
                } => {
                    needs_evaluation = true;
                    let count = restored_count(old_data.len())?;
                    check_block(1, *column, count, 1)?;
                    let styles = self.model.workbook.styles.cell_xfs.len();
                    for (c, col_data) in (*column..*column + count).zip(old_data) {
                        if let Some(style) = col_data.column.as_ref().and_then(|col| col.style) {
                            check_style(style, styles).map_err(|e| format!("column {c} {e}"))?;
                        }
                        for (r, cell) in &col_data.data {
                            self.check_restored_cell(*sheet, *r, c, cell)?;
                        }
                    }
                    self.model.insert_columns(*sheet, *column, count)?;
                    let worksheet = self.model.workbook.worksheet_mut(*sheet)?;
                    for (c, col_data) in (*column..*column + count).zip(old_data) {
                        for (row, cell) in &col_data.data {
                            worksheet.update_cell(*row, c, cell.clone())?;
                        }
                        if let Some(col) = &col_data.column {
                            let width = col.width * COLUMN_WIDTH_FACTOR;
                            let style = col.style;
                            let hidden = col.hidden;
                            worksheet.set_column_width_and_style(c, width, hidden, style)?;
                        }
                    }
                }
                Diff::SetFrozenRowsCount {
                    sheet,
                    new_value: _,
                    old_value,
                } => self.model.set_frozen_rows(*sheet, *old_value)?,
                Diff::SetFrozenColumnsCount {
                    sheet,
                    new_value: _,
                    old_value,
                } => self.model.set_frozen_columns(*sheet, *old_value)?,
                Diff::NewSheet { index, name: _ } => {
                    self.model.delete_sheet(*index)?;
                    if *index > 0 {
                        self.set_selected_sheet(*index - 1)?;
                    }
                }
                Diff::DuplicateSheet {
                    source_index,
                    new_index,
                } => {
                    needs_evaluation = true;
                    // Remove the defined names that the duplication created
                    // (they are scoped to the copy's sheet_id) before dropping
                    // the worksheet itself.
                    let new_sheet_id = self.model.workbook.worksheet(*new_index)?.sheet_id;
                    self.model
                        .workbook
                        .defined_names
                        .retain(|dn| dn.sheet_id != Some(new_sheet_id));
                    self.model.delete_sheet(*new_index)?;
                    self.set_selected_sheet(*source_index)?;
                }
                Diff::RenameSheet {
                    index,
                    old_value,
                    new_value: _,
                } => {
                    self.model.rename_sheet_by_index(*index, old_value)?;
                }
                Diff::MoveSheet {
                    sheet_index,
                    new_index,
                    rewritten,
                } => {
                    let selected = self.get_selected_sheet();
                    self.model
                        .unmove_sheet(*sheet_index, *new_index, rewritten)?;
                    let (from, to) = (*new_index, *sheet_index);
                    self.set_selected_sheet(selected_sheet_after_move(selected, from, to))?;
                }
                Diff::SetSheetColor {
                    index,
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_sheet_color(*index, old_value)?;
                }
                Diff::SetShowGridLines {
                    sheet,
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_show_grid_lines(*sheet, *old_value)?;
                }
                Diff::SetTheme {
                    old_value,
                    new_value: _,
                } => {
                    self.model.workbook.theme = *old_value.clone();
                }
                Diff::CreateDefinedName {
                    name,
                    scope,
                    value: _,
                } => {
                    self.model.delete_defined_name(name, *scope)?;
                }
                Diff::DeleteDefinedName {
                    name,
                    scope,
                    old_value,
                } => {
                    self.model.new_defined_name(name, *scope, old_value)?;
                }
                Diff::UpdateDefinedName {
                    name,
                    scope,
                    old_formula,
                    new_name,
                    new_scope,
                    new_formula: _,
                } => {
                    self.model.update_defined_name(
                        new_name,
                        *new_scope,
                        name,
                        *scope,
                        old_formula,
                    )?;
                }
                Diff::SetSheetState {
                    index,
                    old_value,
                    new_value: _,
                } => self.model.set_sheet_state(*index, old_value.clone())?,
                Diff::CellClearFormatting {
                    sheet,
                    row,
                    column,
                    old_style,
                } => {
                    if let Some(value) = old_style.as_ref() {
                        self.model.set_cell_style(*sheet, *row, *column, value)?;
                    } else {
                        let area = Area {
                            sheet: *sheet,
                            row: *row,
                            column: *column,
                            width: 1,
                            height: 1,
                        };
                        self.model.range_clear_all(&area)?;
                    }
                }
                Diff::DeleteSheet {
                    sheet,
                    old_data,
                    tables,
                    rewritten,
                } => {
                    needs_evaluation = true;
                    let styles = self.model.workbook.styles.cell_xfs.len();
                    check_worksheet(old_data, styles)
                        .map_err(|e| format!("Sheet '{}': {e}", old_data.name))?;
                    let (name, id) = (old_data.name.clone(), old_data.sheet_id);
                    self.model.insert_sheet(&name, *sheet, Some(id))?;
                    let worksheet = self.model.workbook.worksheet_mut(*sheet)?;
                    for (row, column, cell) in old_data.sheet_data.cells() {
                        worksheet.update_cell(row, column, cell.clone())?;
                    }
                    worksheet.rows = old_data.rows.clone();
                    worksheet.cols = old_data.cols.clone();
                    worksheet.show_grid_lines = old_data.show_grid_lines;
                    worksheet.frozen_columns = old_data.frozen_columns;
                    worksheet.frozen_rows = old_data.frozen_rows;
                    worksheet.state = old_data.state.clone();
                    worksheet.color = old_data.color.clone();
                    worksheet.merged_cells = old_data.merged_cells.clone();
                    worksheet.shared_formulas = old_data.shared_formulas.clone();
                    self.model.restore_rewritten_formulas(rewritten.clone());
                    self.model.restore_tables(tables.clone());
                    self.set_selected_sheet(*sheet)?;
                }
                Diff::SetColumnStyle {
                    sheet,
                    column,
                    old_value,
                    new_value: _,
                } => match old_value.as_ref() {
                    Some(s) => self.model.set_column_style(*sheet, *column, s)?,
                    None => {
                        self.model.delete_column_style(*sheet, *column)?;
                    }
                },
                Diff::SetRowStyle {
                    sheet,
                    row,
                    old_value,
                    new_value: _,
                } => {
                    if let Some(s) = old_value.as_ref() {
                        self.model.set_row_style(*sheet, *row, s)?;
                    } else {
                        self.model.delete_row_style(*sheet, *row)?;
                    }
                }
                Diff::DeleteColumnStyle {
                    sheet,
                    column,
                    old_value,
                } => {
                    if let Some(s) = old_value.as_ref() {
                        self.model.set_column_style(*sheet, *column, s)?;
                    } else {
                        self.model.delete_column_style(*sheet, *column)?;
                    }
                }
                Diff::DeleteRowStyle {
                    sheet,
                    row,
                    old_value,
                } => {
                    if let Some(s) = old_value.as_ref() {
                        self.model.set_row_style(*sheet, *row, s)?;
                    } else {
                        self.model.delete_row_style(*sheet, *row)?;
                    }
                }
                Diff::MoveColumns {
                    sheet,
                    column,
                    column_count,
                    delta,
                } => {
                    let (moved, back) = moved_back(*column, *delta)?;
                    self.model
                        .move_columns_action(*sheet, moved, *column_count, back)?;
                    needs_evaluation = true;
                }
                Diff::MoveRows {
                    sheet,
                    row,
                    row_count,
                    delta,
                } => {
                    let (moved, back) = moved_back(*row, *delta)?;
                    self.model
                        .move_rows_action(*sheet, moved, *row_count, back)?;
                    needs_evaluation = true;
                }
                Diff::SetLocale {
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_locale(old_value)?;
                }
                Diff::SetTimezone {
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_timezone(old_value)?;
                }
                Diff::SetWorkbookName {
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_workbook_name(old_value);
                }
                Diff::CreateNamedStyle {
                    name,
                    style: _,
                    includes: _,
                } => {
                    self.model.workbook.styles.delete_named_style_entry(name)?;
                }
                Diff::DeleteNamedStyle { name, old_xf_id } => {
                    self.model
                        .workbook
                        .styles
                        .add_named_cell_style(name, *old_xf_id)?;
                }
                Diff::UpdateNamedStyle {
                    name,
                    new_name,
                    old_style,
                    new_style: _,
                    old_includes,
                    new_includes: _,
                } => {
                    self.model
                        .update_named_style(new_name, name, old_style, *old_includes)?;
                }
                Diff::AddConditionalFormatting {
                    sheet, priority, ..
                } => {
                    let ws = self.model.workbook.worksheet_mut(*sheet)?;
                    if let Some(pos) = ws
                        .conditional_formatting
                        .iter()
                        .position(|cf| cf.priority == *priority)
                    {
                        ws.conditional_formatting.remove(pos);
                    }
                    needs_evaluation = true;
                }
                Diff::DeleteConditionalFormatting {
                    sheet,
                    index,
                    old_range,
                    old_rule,
                    old_priority,
                } => {
                    self.model.insert_conditional_formatting_at(
                        *sheet,
                        *index as usize,
                        ConditionalFormatting {
                            range: old_range.clone(),
                            cf_rule: *old_rule.clone(),
                            priority: *old_priority,
                        },
                    )?;
                    needs_evaluation = true;
                }
                Diff::UpdateConditionalFormatting {
                    sheet,
                    index,
                    old_range,
                    old_rule,
                    old_priority,
                    ..
                } => {
                    let ws = self.model.workbook.worksheet_mut(*sheet)?;
                    let i = *index as usize;
                    if i < ws.conditional_formatting.len() {
                        ws.conditional_formatting[i] = ConditionalFormatting {
                            range: old_range.clone(),
                            cf_rule: *old_rule.clone(),
                            priority: *old_priority,
                        };
                    }
                    needs_evaluation = true;
                }
                Diff::SwapConditionalFormattingPriority {
                    sheet,
                    index_a,
                    index_b,
                    priority_a,
                    priority_b,
                } => {
                    // Undo: restore each rule's original priority.
                    let ws = self.model.workbook.worksheet_mut(*sheet)?;
                    if let Some(cf) = ws.conditional_formatting.get_mut(*index_a as usize) {
                        cf.priority = *priority_a;
                    }
                    if let Some(cf) = ws.conditional_formatting.get_mut(*index_b as usize) {
                        cf.priority = *priority_b;
                    }
                    needs_evaluation = true;
                }
                Diff::SetCellLink {
                    sheet,
                    row,
                    column,
                    old_value,
                    new_value: _,
                } => match old_value.as_ref() {
                    Some(link) => self
                        .model
                        .set_cell_link(*sheet, *row, *column, link.clone())?,
                    None => self.model.delete_cell_link(*sheet, *row, *column)?,
                },
                Diff::SetMergedCells {
                    sheet,
                    old_value,
                    new_value: _,
                } => {
                    // Merged cells block spilling, so a change can affect results
                    needs_evaluation = true;
                    check_merged_cells(old_value)?;
                    self.model.workbook.worksheet_mut(*sheet)?.merged_cells = old_value.clone();
                }
            }
        }
        if needs_evaluation {
            self.evaluate_if_not_paused();
        }
        Ok(())
    }

    /// Applies diff list
    pub(super) fn apply_diff_list(&mut self, diff_list: &DiffList) -> Result<(), String> {
        let mut needs_evaluation = false;
        for diff in diff_list {
            match diff {
                Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value,
                    legacy,
                    old_value: _,
                } => {
                    needs_evaluation = true;
                    self.model.reenter_user_input(
                        *sheet,
                        *row,
                        *column,
                        new_value.to_string(),
                        *legacy,
                    )?;
                }
                Diff::SetArrayValue {
                    sheet,
                    row,
                    column,
                    width,
                    height,
                    new_value,
                    old_values: _,
                } => {
                    needs_evaluation = true;
                    self.model.set_user_array_formula(
                        *sheet, *row, *column, *width, *height, new_value,
                    )?;
                }
                Diff::SetColumnWidth {
                    sheet,
                    column,
                    new_value,
                    old_value: _,
                } => {
                    self.model.set_column_width(*sheet, *column, *new_value)?;
                }
                Diff::SetColumnHidden {
                    sheet,
                    column,
                    new_value,
                    old_value: _,
                } => {
                    self.model.set_column_hidden(*sheet, *column, *new_value)?;
                }
                Diff::SetRowHidden {
                    sheet,
                    row,
                    new_value,
                    old_value: _,
                } => {
                    self.model.set_row_hidden(*sheet, *row, *new_value)?;
                }
                Diff::SetRowHeight {
                    sheet,
                    row,
                    new_value,
                    old_value: _,
                } => {
                    self.model.set_row_height(*sheet, *row, *new_value)?;
                }
                Diff::RangeClearContents {
                    sheet,
                    row,
                    column,
                    width,
                    height,
                    old_value: _,
                } => {
                    let area = Area {
                        sheet: *sheet,
                        row: *row,
                        column: *column,
                        width: *width,
                        height: *height,
                    };
                    self.model.range_clear_contents(&area)?;
                    needs_evaluation = true;
                }
                Diff::RangeClearAll {
                    sheet,
                    row,
                    column,
                    width,
                    height,
                    old_value: _,
                    old_style: _,
                } => {
                    let area = Area {
                        sheet: *sheet,
                        row: *row,
                        column: *column,
                        width: *width,
                        height: *height,
                    };
                    self.model.range_clear_all(&area)?;
                    needs_evaluation = true;
                }
                Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: _,
                    new_value,
                } => self
                    .model
                    .set_cell_style(*sheet, *row, *column, new_value)?,
                Diff::ApplyNamedStyle {
                    sheet,
                    row,
                    column,
                    old_value: _,
                    name,
                } => {
                    // The undo restored the cell's pre-apply format, so the
                    // merge can be replayed against the cell's current state.
                    let current_index = self
                        .model
                        .workbook
                        .worksheet(*sheet)?
                        .get_style(*row, *column);
                    let style_index = self
                        .model
                        .workbook
                        .styles
                        .get_style_index_for_applied_style(name, current_index)?;
                    self.model.workbook.worksheet_mut(*sheet)?.set_cell_style(
                        *row,
                        *column,
                        style_index,
                    )?;
                }
                Diff::InsertRows { sheet, row, count } => {
                    self.model.insert_rows(*sheet, *row, *count)?;
                    needs_evaluation = true;
                }
                Diff::DeleteRows {
                    sheet,
                    row,
                    count,
                    old_data: _,
                } => {
                    self.model.delete_rows(*sheet, *row, *count)?;
                    needs_evaluation = true;
                }
                Diff::InsertColumns {
                    sheet,
                    column,
                    count,
                } => {
                    self.model.insert_columns(*sheet, *column, *count)?;
                    needs_evaluation = true;
                }
                Diff::DeleteColumns {
                    sheet,
                    column,
                    count,
                    old_data: _,
                } => {
                    self.model.delete_columns(*sheet, *column, *count)?;
                    needs_evaluation = true;
                }
                Diff::SetFrozenRowsCount {
                    sheet,
                    new_value,
                    old_value: _,
                } => self.model.set_frozen_rows(*sheet, *new_value)?,
                Diff::SetFrozenColumnsCount {
                    sheet,
                    new_value,
                    old_value: _,
                } => self.model.set_frozen_columns(*sheet, *new_value)?,
                Diff::DeleteSheet { sheet, .. } => {
                    self.model.delete_sheet(*sheet)?;
                    if *sheet > 0 {
                        self.set_selected_sheet(*sheet - 1)?;
                    }
                }
                Diff::NewSheet { index, name } => {
                    self.model.insert_sheet(name, *index, None)?;
                    self.set_selected_sheet(*index)?;
                }
                Diff::DuplicateSheet {
                    source_index,
                    new_index,
                } => {
                    needs_evaluation = true;
                    // `duplicate_sheet` is deterministic given the workbook
                    // state, so re-running it reproduces the same copy.
                    self.model.duplicate_sheet(*source_index)?;
                    self.set_selected_sheet(*new_index)?;
                }
                Diff::RenameSheet {
                    index,
                    old_value: _,
                    new_value,
                } => {
                    self.model.rename_sheet_by_index(*index, new_value)?;
                }
                Diff::MoveSheet {
                    sheet_index,
                    new_index,
                    ..
                } => {
                    let selected = self.get_selected_sheet();
                    self.model.move_sheet(*sheet_index, *new_index)?;
                    self.set_selected_sheet(selected_sheet_after_move(
                        selected,
                        *sheet_index,
                        *new_index,
                    ))?;
                }
                Diff::SetSheetColor {
                    index,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_sheet_color(*index, new_value)?;
                }
                Diff::SetShowGridLines {
                    sheet,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_show_grid_lines(*sheet, *new_value)?;
                }
                Diff::SetTheme {
                    old_value: _,
                    new_value,
                } => {
                    self.model.workbook.theme = *new_value.clone();
                }
                Diff::CreateDefinedName { name, scope, value } => {
                    self.model.new_defined_name(name, *scope, value)?;
                }
                Diff::DeleteDefinedName {
                    name,
                    scope,
                    old_value: _,
                } => self.model.delete_defined_name(name, *scope)?,
                Diff::UpdateDefinedName {
                    name,
                    scope,
                    old_formula: _,
                    new_name,
                    new_scope,
                    new_formula,
                } => self.model.update_defined_name(
                    name,
                    *scope,
                    new_name,
                    *new_scope,
                    new_formula,
                )?,
                Diff::SetSheetState {
                    index,
                    old_value: _,
                    new_value,
                } => self.model.set_sheet_state(*index, new_value.clone())?,
                Diff::CellClearFormatting {
                    sheet,
                    row,
                    column,
                    old_style: _,
                } => {
                    self.model
                        .workbook
                        .worksheet_mut(*sheet)?
                        .set_cell_style(*row, *column, 0)?;
                }
                Diff::SetColumnStyle {
                    sheet,
                    column,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_column_style(*sheet, *column, new_value)?;
                }
                Diff::SetRowStyle {
                    sheet,
                    row,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_row_style(*sheet, *row, new_value)?;
                }
                Diff::DeleteColumnStyle {
                    sheet,
                    column,
                    old_value: _,
                } => {
                    self.model.delete_column_style(*sheet, *column)?;
                }
                Diff::DeleteRowStyle {
                    sheet,
                    row,
                    old_value: _,
                } => {
                    self.model.delete_row_style(*sheet, *row)?;
                }
                Diff::MoveColumns {
                    sheet,
                    column,
                    column_count,
                    delta,
                } => {
                    self.model
                        .move_columns_action(*sheet, *column, *column_count, *delta)?;
                    needs_evaluation = true;
                }
                Diff::MoveRows {
                    sheet,
                    row,
                    row_count,
                    delta,
                } => {
                    self.model
                        .move_rows_action(*sheet, *row, *row_count, *delta)?;
                    needs_evaluation = true;
                }
                Diff::SetLocale {
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_locale(new_value)?;
                }
                Diff::SetTimezone {
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_timezone(new_value)?;
                }
                Diff::SetWorkbookName {
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_workbook_name(new_value);
                }
                Diff::CreateNamedStyle {
                    name,
                    style,
                    includes,
                } => {
                    self.model
                        .workbook
                        .styles
                        .create_named_style(name, style, *includes)?;
                }
                Diff::DeleteNamedStyle { name, old_xf_id: _ } => {
                    self.model.workbook.styles.delete_named_style_entry(name)?;
                }
                Diff::UpdateNamedStyle {
                    name,
                    new_name,
                    old_style: _,
                    new_style,
                    old_includes: _,
                    new_includes,
                } => {
                    self.model
                        .update_named_style(name, new_name, new_style, *new_includes)?;
                }
                Diff::AddConditionalFormatting {
                    sheet,
                    range,
                    rule,
                    priority,
                } => {
                    let len = self
                        .model
                        .workbook
                        .worksheet(*sheet)?
                        .conditional_formatting
                        .len();
                    self.model.insert_conditional_formatting_at(
                        *sheet,
                        len,
                        ConditionalFormatting {
                            range: range.clone(),
                            cf_rule: *rule.clone(),
                            priority: *priority,
                        },
                    )?;
                    needs_evaluation = true;
                }
                Diff::DeleteConditionalFormatting { sheet, index, .. } => {
                    self.model
                        .delete_conditional_formatting(*sheet, *index as usize)?;
                    needs_evaluation = true;
                }
                Diff::UpdateConditionalFormatting {
                    sheet,
                    index,
                    new_range,
                    new_rule,
                    ..
                } => {
                    let ws = self.model.workbook.worksheet_mut(*sheet)?;
                    let i = *index as usize;
                    if i < ws.conditional_formatting.len() {
                        ws.conditional_formatting[i].range = new_range.clone();
                        ws.conditional_formatting[i].cf_rule = *new_rule.clone();
                    }
                    needs_evaluation = true;
                }
                Diff::SwapConditionalFormattingPriority {
                    sheet,
                    index_a,
                    index_b,
                    priority_a,
                    priority_b,
                } => {
                    // Apply/redo: swap the two priorities.
                    let ws = self.model.workbook.worksheet_mut(*sheet)?;
                    if let Some(cf) = ws.conditional_formatting.get_mut(*index_a as usize) {
                        cf.priority = *priority_b;
                    }
                    if let Some(cf) = ws.conditional_formatting.get_mut(*index_b as usize) {
                        cf.priority = *priority_a;
                    }
                    needs_evaluation = true;
                }
                Diff::SetCellLink {
                    sheet,
                    row,
                    column,
                    old_value: _,
                    new_value,
                } => match new_value.as_ref() {
                    Some(link) => self
                        .model
                        .set_cell_link(*sheet, *row, *column, link.clone())?,
                    None => self.model.delete_cell_link(*sheet, *row, *column)?,
                },
                Diff::SetMergedCells {
                    sheet,
                    old_value: _,
                    new_value,
                } => {
                    // Merged cells block spilling, so a change can affect results
                    needs_evaluation = true;
                    check_merged_cells(new_value)?;
                    self.model.workbook.worksheet_mut(*sheet)?.merged_cells = new_value.clone();
                }
            }
        }

        if needs_evaluation {
            self.evaluate_if_not_paused();
        }
        Ok(())
    }
}

// A diff list from a peer (`UserModel::apply_external_diffs`) is untrusted (CLAUDE.md
// directive 5, T-2896): the undo of an edit writes the cells, records and ranges it carries
// straight into the sheet, so each is checked against the model as it is when the diff applies,
// with the checks a loaded workbook passes (`workbook_check`).
impl UserModel {
    /// Checks `cell` before an undo restores it at (`row`, `column`) of `sheet`.
    fn check_restored_cell(
        &self,
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
        )
    }
}

/// Refuses a block a diff names that leaves the grid, before `row + height` is computed.
fn check_block(row: i32, column: i32, width: i32, height: i32) -> Result<(), String> {
    if is_valid_block(row, column, width, height) {
        Ok(())
    } else {
        Err(format!("The range at R{row}C{column} is outside the sheet"))
    }
}

/// The number of rows or columns an undo restores, refused when it is no `i32`.
fn restored_count(len: usize) -> Result<i32, String> {
    i32::try_from(len).map_err(|_| "Too many rows or columns to restore".to_string())
}

/// The cells of `old_values`, a `height` by `width` block from (`row`, `column`) that `check_block`
/// accepted, with their places; the cells past the block are not restored.
fn restored_cells(
    row: i32,
    column: i32,
    width: i32,
    height: i32,
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
fn moved_back(first: i32, delta: i32) -> Result<(i32, i32), String> {
    first
        .checked_add(delta)
        .zip(delta.checked_neg())
        .ok_or_else(|| "The move is outside the sheet".to_string())
}
