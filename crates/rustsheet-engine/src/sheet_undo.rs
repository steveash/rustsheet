//! Taking a sheet out of a model and putting it back, so a caller can undo a delete.

use crate::{model::Model, types::Worksheet};

impl Model {
    /// Removes the sheet at `sheet_index` and hands it back, so a caller can undo the delete
    /// with [`Model::restore_sheet`]. Fails like [`Model::delete_sheet`].
    pub fn take_sheet(&mut self, sheet_index: u32) -> Result<Worksheet, String> {
        if self.workbook.worksheets.len() == 1 {
            return Err("Cannot delete only sheet".to_string());
        }
        if sheet_index as usize >= self.workbook.worksheets.len() {
            return Err("Sheet index too large".to_string());
        }
        let sheet = self.workbook.worksheets.remove(sheet_index as usize);
        self.reset_parsed_structures();
        Ok(sheet)
    }

    /// Puts a sheet obtained from [`Model::take_sheet`] back at `sheet_index`.
    /// Fails if the index is out of range or the name/id is already in use.
    pub fn restore_sheet(&mut self, sheet_index: u32, sheet: Worksheet) -> Result<(), String> {
        if sheet_index as usize > self.workbook.worksheets.len() {
            return Err("Sheet index out of range".to_string());
        }
        let name = sheet.get_name().to_uppercase();
        if self
            .workbook
            .worksheets
            .iter()
            .any(|w| w.get_name().to_uppercase() == name || w.sheet_id == sheet.sheet_id)
        {
            return Err("A worksheet already exists with that name".to_string());
        }
        self.workbook.worksheets.insert(sheet_index as usize, sheet);
        self.reset_parsed_structures();
        Ok(())
    }
}
