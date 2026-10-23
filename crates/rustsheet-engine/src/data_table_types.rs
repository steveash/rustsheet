//! What-if data table metadata of a worksheet (`Worksheet::data_tables`); the evaluation is in
//! `data_tables.rs`.

use bitcode::{Decode, Encode};

/// A what-if data table (Excel's `TABLE()`, docs/excel-semantics/recalc.md). Each result cell holds
/// the value of a formula of the sheet with the input cell or cells replaced by values. For a
/// one-variable table the values come from the column left of `range` (`column_input`) or the
/// row above it (`row_input`), and the formulas are in the row above (respectively the column to
/// the left). A two-variable table has both inputs and one formula, at the corner above and left
/// of `range`.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct DataTable {
    /// The result cells: top row, left column, bottom row, right column (1-based).
    pub range: (i32, i32, i32, i32),
    /// The input cell the values of the row above `range` replace.
    pub row_input: Option<DataTableInput>,
    /// The input cell the values of the column left of `range` replace.
    pub column_input: Option<DataTableInput>,
}

impl DataTable {
    /// The text Excel shows for every result cell, e.g. `{=TABLE(,A1)}`: the row input first.
    #[must_use]
    pub fn formula_text(&self) -> String {
        let input = |i: Option<DataTableInput>| match i {
            None => String::new(),
            Some(DataTableInput::Deleted) => "#REF!".to_string(),
            Some(DataTableInput::Cell(row, column)) => {
                crate::expressions::utils::number_to_column(column)
                    .map(|c| format!("{c}{row}"))
                    .unwrap_or_default()
            }
        };
        format!(
            "{{=TABLE({},{})}}",
            input(self.row_input),
            input(self.column_input)
        )
    }
}

/// An input cell of a [`DataTable`], on the table's own sheet.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataTableInput {
    /// The cell at (row, column).
    Cell(i32, i32),
    /// The input cell was deleted (`del1`/`del2`): every result is `#REF!`.
    Deleted,
}
