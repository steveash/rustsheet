//! Writing a formula into a cell. A typed formula and a legacy one (`set_cell_with_legacy_formula`)
//! are parsed differently and stored through the same node setter, so a legacy tree is stored as
//! it was rewritten, its N/T markers included (SPEC-0012), not re-read from its display text.

use std::sync::Arc;

use crate::expressions::parser::static_analysis::{
    legacy_formula_is_dynamic, run_static_analysis_on_node, StaticResult,
};
use crate::expressions::parser::stringify::{
    to_localized_string, to_localized_string_this_row_lowered,
};
use crate::expressions::parser::Node;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::language::Language;
use crate::locale::Locale;
use crate::model::Model;
use crate::types::{ArrayKind, Cell};

/// A printer of a parsed formula in the active locale and language, at a cell.
type FormulaPrinter = fn(&Node, &CellReferenceRC, &Locale, &Language) -> String;

/// A shared formula as a sheet stores it: the tree evaluation and analysis read, with no
/// whitespace in it, its static result and, for a formula written with whitespace, the tree as
/// written, which the cell's formula text prints (SPEC-0023; see [`ParsedFormula::written`]).
/// Evaluation never sees a [`Node::Spaced`], so no argument-shape test has to peel one.
pub struct ParsedFormula {
    /// The tree evaluation and analysis read, with no whitespace in it.
    pub evaluated: Arc<Node>,
    /// The static analysis of [`evaluated`](Self::evaluated).
    pub static_result: StaticResult,
    /// The tree as written, whitespace included, for a formula written with whitespace.
    pub written: Option<Arc<Node>>,
}

impl ParsedFormula {
    /// The tree the formula text prints: as written, whitespace included.
    pub fn written(&self) -> &Node {
        self.written.as_deref().unwrap_or(&self.evaluated)
    }
}

/// `node`, a formula as parsed, stored as a [`ParsedFormula`].
pub(crate) fn stored_formula(node: Node) -> ParsedFormula {
    let written = node.has_spaces().then(|| Arc::new(node.clone()));
    let mut node = node;
    if written.is_some() {
        node.strip_spaces();
    }
    let static_result = run_static_analysis_on_node(&node);
    ParsedFormula {
        evaluated: Arc::new(node),
        static_result,
        written,
    }
}

impl Model {
    /// The tree the formula text of shared formula `formula_index` of `sheet` prints.
    pub(crate) fn written_formula(&self, sheet: u32, formula_index: i32) -> Result<&Node, String> {
        Ok(self
            .parsed_formulas
            .get(sheet as usize)
            .ok_or("missing sheet")?
            .get(formula_index as usize)
            .ok_or("missing formula")?
            .written())
    }

    /// The formula in (`sheet`, `row`, `column`), `=` first, as `printer` prints it at the cell;
    /// `None` for a cell without a formula. [`Model::get_cell_formula`] and
    /// [`Model::moved_text`] differ only in their printer.
    pub(crate) fn formula_text_with(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
        printer: FormulaPrinter,
    ) -> Result<Option<String>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let Some(formula_index) = worksheet.cell(row, column).and_then(|c| c.get_formula()) else {
            return Ok(None);
        };
        let formula = self.written_formula(sheet, formula_index)?;
        let cell_ref = CellReferenceRC {
            sheet: worksheet.get_name(),
            row,
            column,
        };
        let text = printer(formula, &cell_ref, self.locale, self.language);
        Ok(Some(format!("={text}")))
    }

    /// [`Model::get_cell_formula`] for a formula a structural edit moves `row_delta` rows; to
    /// another row, every `#This Row` reference in it is printed as the area it is on the
    /// cell's row (`$A$3`, or `#VALUE!` off the data rows), so the moved formula keeps reading the
    /// row it read, as before SPEC-0016 step 2 (T-1699).
    pub(crate) fn moved_text(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
        row_delta: i32,
    ) -> Result<Option<String>, String> {
        let printer: FormulaPrinter = if row_delta == 0 {
            to_localized_string
        } else {
            to_localized_string_this_row_lowered
        };
        self.formula_text_with(sheet, row, column, printer)
    }

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
        let formula_index = self.shared_formula_index(sheet, node)?;
        self.set_cell_with_formula_index(sheet, row, column, formula_index, style)?;
        Ok(formula_index)
    }

    /// Makes each plain formula cell whose formula Excel spills even when it is stored without the
    /// dynamic-array flag ([`legacy_formula_is_dynamic`]) a dynamic anchor, as typing the formula
    /// would ([`Self::set_cell_with_formula_index`]): an xlsx `<f>` with no `cm`, or a decoded
    /// `CellFormula`, holding `SIN(A1#)` or `SEQUENCE(3)` (probes/legacy-formula-spill-ref, T-3467).
    /// Without it such a cell reached evaluation as a scalar cell with an array result.
    pub(crate) fn promote_dynamic_formula_cells(&mut self) {
        for (sheet, worksheet) in self.workbook.worksheets.iter_mut().enumerate() {
            let Some(formulas) = self.parsed_formulas.get(sheet) else {
                continue;
            };
            let dynamic = |f: i32| {
                usize::try_from(f)
                    .ok()
                    .and_then(|f| formulas.get(f))
                    .is_some_and(|parsed| {
                        !matches!(parsed.static_result, StaticResult::Scalar)
                            && legacy_formula_is_dynamic(&parsed.evaluated)
                    })
            };
            let promoted: Vec<(i32, i32)> = worksheet
                .sheet_data
                .cells()
                .filter_map(|(row, column, cell)| match cell {
                    Cell::CellFormula { f, .. } if dynamic(*f) => Some((row, column)),
                    _ => None,
                })
                .collect();
            for (row, column) in promoted {
                if let Some(cell) = worksheet.sheet_data.cell_mut(row, column) {
                    if let Cell::CellFormula { f, s, v } = cell.clone() {
                        *cell = Cell::ArrayFormula {
                            f,
                            s,
                            r: (1, 1),
                            kind: ArrayKind::Dynamic,
                            v,
                        };
                    }
                }
            }
        }
    }

    /// Stores the sheet's shared formula `formula_index` in the cell, as a dynamic anchor unless
    /// its static result is a scalar.
    fn set_cell_with_formula_index(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula_index: i32,
        style: i32,
    ) -> Result<(), String> {
        let is_dynamic = !matches!(
            self.parsed_formulas[sheet as usize][formula_index as usize].static_result,
            StaticResult::Scalar
        );
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        if is_dynamic {
            worksheet.set_cell_with_dynamic_formula(row, column, formula_index, style, 1, 1)
        } else {
            worksheet.set_cell_with_formula(row, column, formula_index, style)
        }
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
        let formula_index = self.shared_formula_index(sheet, node)?;
        self.set_formula_index_with_units(sheet, row, column, formula_index, style)
    }

    /// [`Model::set_formula_node_with_units`] for a formula the sheet already shares, by index.
    pub(crate) fn set_formula_index_with_units(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula_index: i32,
        style: i32,
    ) -> Result<(), String> {
        self.set_cell_with_formula_index(sheet, row, column, formula_index, style)?;
        let cell = CellReferenceIndex { sheet, row, column };
        let parsed_formula =
            &self.parsed_formulas[sheet as usize][formula_index as usize].evaluated;
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
