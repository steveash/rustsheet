//! Setting a cell to an error value without going through the text parser.

use crate::{expressions::token::Error, model::Model};

impl Model {
    /// Updates the value of a cell with an error value, without parsing text: the engine stores
    /// an error cell of exactly `error`, including kinds a user cannot type as a literal
    /// (`#CONNECT!`, `#BLOCKED!`, `#FIELD!`; docs/excel-semantics/syntax.md "Newer error literals").
    /// It does not change the style, except that it drops a quote prefix.
    ///
    /// See also:
    /// * [Model::update_cell_with_bool()]
    pub fn update_cell_with_error(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        error: Error,
    ) -> Result<(), String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        let new_style_index = if self.workbook.styles.style_is_quote_prefix(style_index) {
            self.workbook
                .styles
                .get_style_without_quote_prefix(style_index)?
        } else {
            style_index
        };
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_with_error(row, column, error, new_style_index)
    }
}
