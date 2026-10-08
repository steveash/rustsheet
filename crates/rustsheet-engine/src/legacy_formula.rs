//! Formulas as a pre-dynamic-array workbook stores them (a plain `<f>` with no array metadata).

use crate::expressions::parser::static_analysis::add_implicit_intersection;
use crate::expressions::parser::stringify::to_localized_string;
use crate::expressions::types::CellReferenceRC;
use crate::model::Model;

impl Model {
    /// The dynamic-array text of `formula` (with its leading `=`) written as a legacy formula in
    /// cell (`sheet`, `row`, `column`): the implicit intersections a legacy formula applies are
    /// made explicit, `=A1:A2` → `=@A1:A2`, `=ABS(A1:A2)` → `=ABS(@A1:A2)`, exactly as xlsx
    /// import reads a plain `<f>` (`add_implicit_intersection`, T-0983). Excel shows the same
    /// `@` in FORMULATEXT (probes/multiarea-at-legacy H8/H10, oracle excel-graph, T-1096).
    /// Text that is not a formula is returned unchanged.
    ///
    /// # Errors
    /// The sheet does not exist, or the formula is past the parser's limits.
    pub fn legacy_formula_text(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
    ) -> Result<String, String> {
        let Some(body) = formula.strip_prefix('=').filter(|b| !b.is_empty()) else {
            return Ok(formula.to_string());
        };
        let mut node = self.parse_cell_formula(sheet, row, column, body)?;
        add_implicit_intersection(&mut node, true);
        let context = CellReferenceRC {
            sheet: self.workbook.worksheet(sheet)?.get_name(),
            row,
            column,
        };
        Ok(format!(
            "={}",
            to_localized_string(&node, &context, self.locale, self.language)
        ))
    }
}
