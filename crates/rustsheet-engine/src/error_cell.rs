//! Setting a cell to an error value without going through the text parser.

use crate::{expressions::token::Error, model::Model};

impl Model {
    /// The style index of a cell with its quote prefix dropped: what a typed number, boolean or
    /// error value takes, since only text needs the prefix. A new module holds this and
    /// [`Model::update_cell_with_error`] because `model.rs` may only shrink (file-size ratchet).
    pub(crate) fn style_index_without_quote_prefix(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<i32, String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        if self.workbook.styles.style_is_quote_prefix(style_index) {
            self.workbook
                .styles
                .get_style_without_quote_prefix(style_index)
        } else {
            Ok(style_index)
        }
    }

    /// Updates the value of a cell with an error value, without parsing text: the engine stores
    /// an error cell of exactly `error`, including kinds a user cannot type as a literal
    /// (`#CONNECT!`, `#BLOCKED!`, `#FIELD!`; docs/excel-semantics/syntax.md "Newer error literals").
    /// It does not change the style, except that it drops a quote prefix.
    ///
    /// See also:
    /// * [`Model::update_cell_with_bool()`]
    pub fn update_cell_with_error(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        error: Error,
    ) -> Result<(), String> {
        let new_style_index = self.style_index_without_quote_prefix(sheet, row, column)?;
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_with_error(row, column, error, new_style_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{expressions::token::Error, test::util::new_empty_model};

    #[test]
    fn stores_the_error_and_drops_a_quote_prefix() {
        let mut model = new_empty_model();
        model.update_cell_with_text(0, 1, 1, "=1").expect("text");
        model
            .update_cell_with_error(0, 1, 1, Error::CONNECT)
            .expect("error");
        assert_eq!(
            model.get_formatted_cell_value(0, 1, 1).expect("formatted"),
            "#CONNECT!"
        );
        let style = model.get_cell_style_index(0, 1, 1).expect("style");
        assert!(!model.workbook.styles.style_is_quote_prefix(style));
    }

    #[test]
    fn rejects_an_invalid_cell() {
        let mut model = new_empty_model();
        assert_eq!(
            model.update_cell_with_error(0, 0, 1, Error::NA),
            Err("Incorrect row or column".to_string())
        );
        assert!(model.update_cell_with_error(9, 1, 1, Error::NA).is_err());
    }
}
