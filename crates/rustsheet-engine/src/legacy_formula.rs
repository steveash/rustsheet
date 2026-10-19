//! Formulas as a pre-dynamic-array workbook stores them (a plain `<f>` with no array metadata).

use crate::expressions::parser::static_analysis::add_implicit_intersection;
use crate::expressions::parser::stringify::to_localized_string;
use crate::expressions::parser::Node;
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
        let node = self.legacy_formula_node(sheet, row, column, body)?;
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

    /// Writes `formula` (with its leading `=`) into the cell as a legacy formula: parsed and
    /// rewritten as [`Model::legacy_formula_text`] shows it, and stored as that tree, so what
    /// the rewrite records beyond the text (a legacy N or T, SPEC-0012) is kept. Text that is not
    /// a formula is written as [`Model::set_user_input`] writes it.
    ///
    /// # Errors
    /// The cell cannot be written (merged, inside an array formula), the sheet does not exist,
    /// or the formula is past the parser's limits.
    pub fn set_cell_with_legacy_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
    ) -> Result<(), String> {
        let Some(body) = formula.strip_prefix('=').filter(|b| !b.is_empty()) else {
            return self.set_user_input(sheet, row, column, formula.to_string());
        };
        self.prepare_cell_for_user_input(sheet, row, column)?;
        let node = self.legacy_formula_node(sheet, row, column, body)?;
        let style = self.formula_style_index(sheet, row, column)?;
        self.set_formula_node_with_units(sheet, row, column, node, style)
    }

    /// Whether the formula in the cell holds a legacy N or T marker ([`Node::has_legacy_marker`]).
    /// A cell with no formula holds none.
    pub(crate) fn cell_has_legacy_marker(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<bool, String> {
        let Some(index) = self
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .and_then(|c| c.get_formula())
        else {
            return Ok(false);
        };
        Ok(self
            .parsed_formulas
            .get(sheet as usize)
            .and_then(|formulas| formulas.get(index as usize))
            .is_some_and(|(node, _)| node.has_legacy_marker()))
    }

    /// Writes `input`, the display text an edit (row or column insert or delete, move, fill,
    /// paste) made from a cell's formula, as user input; when that formula held a legacy marker
    /// (`legacy`), through the legacy entry, so the cell stays legacy. Excel keeps a legacy cell
    /// legacy after a row insert above it (probes/sheet-ops/sheet-ops-insert-row-legacy-cell,
    /// excel-graph, T-1933); the other edits are assumed to do the same (SPEC-0012 rule 6). The
    /// re-entry relies on the rewrite being idempotent over its own display text (T-1713).
    ///
    /// # Errors
    /// As [`Model::set_user_input`] and [`Model::set_cell_with_legacy_formula`].
    pub(crate) fn reenter_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        input: String,
        legacy: bool,
    ) -> Result<(), String> {
        if legacy {
            self.set_cell_with_legacy_formula(sheet, row, column, &input)
        } else {
            self.set_user_input(sheet, row, column, input)
        }
    }

    /// Fills cell `target` from cell `source` of `sheet` as a fill handle drag does: the source
    /// formula with its relative references shifted ([`Model::extend_to`]), or its value, written
    /// through [`Model::reenter_user_input`] so a legacy marker is kept. Returns the text written.
    ///
    /// # Errors
    /// As [`Model::extend_to`] and [`Model::reenter_user_input`].
    pub(crate) fn fill_cell(
        &mut self,
        sheet: u32,
        (source_row, source_column): (i32, i32),
        (row, column): (i32, i32),
    ) -> Result<String, String> {
        let text = self.extend_to(sheet, source_row, source_column, row, column)?;
        let legacy = self.cell_has_legacy_marker(sheet, source_row, source_column)?;
        self.reenter_user_input(sheet, row, column, text.clone(), legacy)?;
        Ok(text)
    }

    /// `body` (no `=`) parsed in the cell and rewritten as a legacy formula reads it.
    fn legacy_formula_node(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        body: &str,
    ) -> Result<Node, String> {
        let mut node = self.parse_cell_formula(sheet, row, column, body)?;
        add_implicit_intersection(&mut node, true);
        Ok(node)
    }
}
