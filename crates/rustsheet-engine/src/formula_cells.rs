//! Writing a formula into a cell. A typed formula and a legacy one (`set_cell_with_legacy_formula`)
//! are parsed differently and stored through the same node setter, so a legacy tree is stored as
//! it was rewritten, its N/T markers included (SPEC-0012), not re-read from its display text.

use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;

impl Model {
    /// The style a formula written into the cell gets: the cell's, without a quote prefix.
    pub(crate) fn formula_style_index(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<i32, String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        if self.workbook.styles.style_is_quote_prefix(style_index) {
            return self
                .workbook
                .styles
                .get_style_without_quote_prefix(style_index);
        }
        Ok(style_index)
    }

    /// Parses `formula` (no `=`) in the cell and stores it; the index of its shared formula.
    pub(crate) fn set_cell_with_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
        style: i32,
    ) -> Result<i32, String> {
        let parsed_formula = self.parse_cell_formula(sheet, row, column, formula)?;
        self.set_cell_with_formula_node(sheet, row, column, parsed_formula, style)
    }

    /// Stores the parsed formula `node` in the cell, as a dynamic anchor unless its static result
    /// is a scalar; the index of its shared formula.
    pub(crate) fn set_cell_with_formula_node(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        node: Node,
        style: i32,
    ) -> Result<i32, String> {
        let static_result = run_static_analysis_on_node(&node);
        let is_dynamic = !matches!(static_result, StaticResult::Scalar);
        let formula_index = self.shared_formula_index(sheet, node, static_result)?;
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        if is_dynamic {
            worksheet.set_cell_with_dynamic_formula(row, column, formula_index, style, 1, 1)?;
        } else {
            worksheet.set_cell_with_formula(row, column, formula_index, style)?;
        }
        Ok(formula_index)
    }

    /// Stores `node` as `set_user_input` stores a formula: with `style`, then the number format
    /// of the formula's units, if it has any.
    pub(crate) fn set_formula_node_with_units(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        node: Node,
        style: i32,
    ) -> Result<(), String> {
        let formula_index = self.set_cell_with_formula_node(sheet, row, column, node, style)?;
        let cell = CellReferenceIndex { sheet, row, column };
        let (parsed_formula, _static_result) =
            &self.parsed_formulas[sheet as usize][formula_index as usize];
        if let Some(units) = self.compute_node_units(parsed_formula, &cell) {
            let style_index = self
                .workbook
                .styles
                .get_style_with_format(style, &units.get_num_fmt())?;
            let style = self.workbook.styles.get_style(style_index)?;
            self.set_cell_style(sheet, row, column, &style)?;
        }
        Ok(())
    }
}
