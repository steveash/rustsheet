//! Formulas evaluated outside a cell: the rules and thresholds of conditional formats. Each one
//! is a formula evaluation of its own, with its own area budget (`area_budget.rs`), as a cell's
//! formula is (`run_cell_formula`).

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::Node,
        types::{CellReferenceIndex, CellReferenceRC},
    },
    model::Model,
};

impl Model {
    /// Evaluates `node` in the context of `cell` as a formula of its own, outside any cell's
    /// formula: with a fresh area budget, so that one conditional format evaluated over many
    /// cells does not spend one budget for all of them.
    pub(crate) fn evaluate_outside_cell(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.with_area_budget(|model| model.evaluate_node_in_context(node, cell))
    }

    /// Evaluates a formula string on a sheet, returning the numeric result.
    /// Assumes the workbook has already been evaluated (cell values are up-to-date).
    /// Returns `None` if the formula is invalid or does not produce a number.
    pub(crate) fn evaluate_formula(&mut self, formula: &str, sheet: u32) -> Option<f64> {
        let body = formula.trim().strip_prefix('=').unwrap_or(formula.trim());
        if body.is_empty() {
            return None;
        }
        let sheet_name = self.workbook.worksheets.get(sheet as usize)?.get_name();
        let context_rc = CellReferenceRC {
            sheet: sheet_name,
            row: 1,
            column: 1,
        };
        let node = self.parse_internal_formula(body, &context_rc);
        let context_index = CellReferenceIndex {
            sheet,
            row: 1,
            column: 1,
        };
        match self.evaluate_outside_cell(&node, context_index) {
            CalcResult::Number(n) => Some(n),
            _ => None,
        }
    }
}
