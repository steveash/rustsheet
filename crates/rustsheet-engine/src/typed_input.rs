//! Text a user types, which Excel refuses in places where it reads the same text stored.

use crate::model::Model;

impl Model {
    /// [`Model::set_user_input`] for text a user types, which Excel refuses where it would read
    /// stored text: a LET name or LAMBDA parameter spelled like a cell (`=LET(f1,1,f1+1)`) is an
    /// error here (decision 0103, T-2497) and binds through `set_user_input`. A formula typed
    /// into a table's data rows fills its column as a calculated column, when
    /// [`Model::calculated_column_targets`] finds the column empty or filled (T-2958); a stored
    /// formula never fills.
    pub fn set_typed_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        let targets = self.calculated_column_targets(sheet, row, column)?;
        self.parser.set_typed(true);
        let result = self.set_user_input(sheet, row, column, value);
        self.parser.set_typed(false);
        result?;
        self.fill_calculated_column(sheet, row, column, &targets)
    }

    /// [`Model::set_user_input`] for a formula as a file stores it: one naming a table or column
    /// the workbook lacks is `#REF!` instead of a parse error (T-2322).
    pub fn set_loaded_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        self.parser.set_loaded(true);
        let result = self.set_user_input(sheet, row, column, value);
        self.parser.set_loaded(false);
        result
    }
}
