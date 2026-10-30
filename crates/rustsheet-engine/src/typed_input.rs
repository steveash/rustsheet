//! Text a user types, which Excel refuses in places where it reads the same text stored.

use std::collections::HashMap;

use crate::expressions::{
    lexer::LexerMode,
    parser::{new_parser_english, stringify::to_english_string, Node},
    types::CellReferenceRC,
};
use crate::model::Model;

/// The A1 text of `text` typed in R1C1 mode into the 1-based `row`, `column` of `sheet`, as
/// Excel's `Range.FormulaR1C1` reads it (`=R1C2+1` in A3 is `=$B$1+1`, `=R[-1]C[1]` is `=B2`);
/// text without a leading `=` is a constant, the same in both modes. `None` when the formula does
/// not parse, which Excel refuses. `worksheets` are the workbook's sheet names; defined names are
/// kept as written and resolve when the A1 text is entered, but a structured reference does not
/// parse here (no tables), so such a formula is `None` (T-3113). The engine's R1C1 grammar is its
/// storage form, which always writes both numbers: a bare `R` or `C` (`RC`, `R[-1]C`) is not read
/// as an offset of 0 (TODO(T-3212)).
#[must_use]
pub fn r1c1_formula_to_a1(
    text: &str,
    worksheets: &[String],
    sheet: &str,
    row: i32,
    column: i32,
) -> Option<String> {
    let Some(formula) = text.strip_prefix('=') else {
        return Some(text.to_string());
    };
    let mut parser = new_parser_english(worksheets.to_vec(), Vec::new(), HashMap::new());
    parser.set_lexer_mode(LexerMode::R1C1);
    let context = CellReferenceRC {
        sheet: sheet.to_string(),
        row,
        column,
    };
    let node = parser.parse(formula, &context);
    if matches!(node, Node::ParseErrorKind { .. }) {
        return None;
    }
    Some(format!("={}", to_english_string(&node, &context)))
}

impl Model {
    /// [`Model::set_user_input`] for text a user types, which Excel refuses where it would read
    /// stored text: a LET name or LAMBDA parameter spelled like a cell (`=LET(f1,1,f1+1)`) is an
    /// error here (decision 0103, T-2497) and binds through `set_user_input`. A formula typed
    /// into a table's data rows fills its column as a calculated column, when
    /// [`Model::calculated_column`] finds the column empty or filled (T-2958, T-3024); a stored
    /// formula never fills. A formula typed just right of a table grows the table by a column
    /// first ([`Model::table_expansion`], T-2968).
    pub fn set_typed_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        // A formula typed just right of a table grows it first, so the new column fills
        // (T-2968); one typed into the header cell only names the column.
        if let Some(expansion) = self.table_expansion(sheet, row, column, &value)? {
            self.expand_table(sheet, &expansion)?;
            if expansion.header {
                return Ok(());
            }
        }
        let fill = self.calculated_column_fill(sheet, row, column)?;
        self.parser.set_typed(true);
        let result = self.set_user_input(sheet, row, column, value);
        self.parser.set_typed(false);
        result?;
        match fill {
            Some(fill) => self.fill_calculated_column(sheet, row, column, &fill),
            None => Ok(()),
        }
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
