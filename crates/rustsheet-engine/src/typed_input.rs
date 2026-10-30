//! Text a user types, which Excel refuses in places where it reads the same text stored.

use std::collections::HashMap;

use crate::expressions::{
    lexer::LexerMode,
    parser::{is_limit_error, new_parser_english, stringify::to_english_string, Node},
    types::CellReferenceRC,
};
use crate::model::Model;

/// Start of the error message [`Model::set_typed_user_input`] gives for a formula that does not
/// parse, which Excel refuses typed (Graph PATCH 400 `InvalidArgument`) where it keeps stored text
/// (T-3213). Callers that replay what a user typed leave such a cell as it was (decision 0179).
pub const TYPED_FORMULA_DOES_NOT_PARSE: &str = "Formula does not parse";

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

/// Whether the one `(` that `formula` leaves open opens a call's arguments (`SUM(1`, `LOG10(2`,
/// `SUM(1,,`) rather than a group (`(1`, `1+(2`): typed, Excel closes the first and refuses the
/// second (probes/t3213-typed-parse-errors A2, A3, A6; T-3213). Text inside string literals, quoted
/// sheet names and brackets is skipped.
pub(crate) fn open_parenthesis_is_a_call(formula: &str) -> bool {
    let mut open = Vec::new();
    let mut quote = None;
    let mut brackets = 0_usize;
    let mut previous = None;
    for (index, c) in formula.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') if brackets == 0 => quote = Some(c),
            (None, '[') => brackets += 1,
            (None, ']') => brackets = brackets.saturating_sub(1),
            (None, '(') if brackets == 0 => open.push(previous),
            (None, ')') if brackets == 0 => {
                open.pop();
            }
            _ => {}
        }
        previous = Some((index, c));
    }
    matches!(
        open.as_slice(),
        [Some((_, c))] if c.is_alphanumeric() || matches!(c, '_' | '.')
    )
}

impl Model {
    /// Parses a formula typed into a cell. A formula that fails to parse is
    /// retried with a closing parenthesis (`SUM(A1:A3` => `SUM(A1:A3)`) and
    /// otherwise kept as a parse error, except one past the parser's limits (nested too deep
    /// or too many operators; no `)` helps), which is rejected: the cell could never hold it.
    /// Typed ([`Model::set_typed_user_input`]), only a call's missing `)` is added, and any
    /// other parse error is rejected.
    pub(crate) fn parse_cell_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
    ) -> Result<Node, String> {
        let sheet = self.workbook.worksheet(sheet)?.get_name();
        let cell_reference = &CellReferenceRC { sheet, row, column };
        let mut node = self.parser.parse(formula, cell_reference);
        // Typed, Excel closes a call's parenthesis only, not a group's (`=(1` is refused).
        if matches!(&node, Node::ParseErrorKind { message, .. } if !is_limit_error(message))
            && (!self.parser.is_typed() || open_parenthesis_is_a_call(formula))
        {
            let closed = format!("{formula})");
            let retry = self.parser.parse(&closed, cell_reference);
            if !matches!(retry, Node::ParseErrorKind { .. }) {
                node = retry;
            }
        }
        match node {
            Node::ParseErrorKind { message, .. } if is_limit_error(&message) => {
                Err(format!("Formula: {message}"))
            }
            // Excel refuses a typed formula that does not parse (Graph PATCH 400
            // `InvalidArgument`, probes/t3113-entered-refused, t3213-typed-parse-errors), where it
            // keeps stored text as written (syntax.md "A typed formula that does not parse is
            // refused", T-3213).
            Node::ParseErrorKind { message, .. } if self.parser.is_typed() => {
                Err(format!("{TYPED_FORMULA_DOES_NOT_PARSE}: {message}"))
            }
            node => self.parser.typed_refusal().map_or(Ok(node), Err),
        }
    }

    /// [`Model::set_user_input`] for text a user types, which Excel refuses where it would read
    /// stored text: a LET name or LAMBDA parameter spelled like a cell (`=LET(f1,1,f1+1)`) is an
    /// error here (decision 0103, T-2497) and binds through `set_user_input`. A formula typed
    /// into a table's data rows fills its column as a calculated column, when
    /// [`Model::calculated_column`] finds the column empty or filled (T-2958, T-3024); a stored
    /// formula never fills. A formula or a constant typed just right of a table grows the table
    /// by a column first, and one typed just below it by a row ([`Model::table_expansion`],
    /// T-2968, T-3218, T-3273). A formula that does not parse is an error
    /// starting with [`TYPED_FORMULA_DOES_NOT_PARSE`], and the cell keeps its content (T-3213).
    pub fn set_typed_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        // A formula or constant typed just right of a table grows it first, so the new column
        // fills (T-2968, T-3218); one typed into the header cell only names the column. One typed
        // just below it grows it by a row (T-3273).
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
