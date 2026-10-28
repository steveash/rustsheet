//! Calculated columns: a formula typed into a table column fills the column, as Excel does
//! (docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table
//! column"; T-2957, T-2958).

use crate::model::Model;
use crate::table_edits::bounds;
use crate::types::Cell;

impl Model {
    /// The cells a formula typed into (`sheet`, `row`, `column`) fills as its table's calculated
    /// column: every other data row of the cell's table column, when those cells are all blank or
    /// all hold one formula. Empty when the cell is not in a table's data rows, or when another
    /// data cell of the column holds a value, a different formula, or is blank beside formulas:
    /// Excel fills an empty column and re-fills a filled one, but leaves a column with an
    /// exception (a typed constant) alone (probes/t2957-*). The totals row is never filled.
    ///
    /// # Errors
    /// When `sheet` does not exist.
    pub fn calculated_column_targets(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Vec<(i32, i32)>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let sheet_name = worksheet.get_name();
        let Some(data) = self
            .workbook
            .tables
            .values()
            .filter(|table| table.is_on_sheet(&sheet_name))
            .filter_map(bounds)
            .find(|b| (b.left..=b.right).contains(&column) && (b.data.0..=b.data.1).contains(&row))
            .map(|b| b.data)
        else {
            return Ok(Vec::new());
        };
        // The column's one formula, and whether a blank cell was met: blank and formula cells
        // mixed are no calculated column either.
        let mut formula = None;
        let mut blank = false;
        let mut targets = Vec::new();
        for r in data.0..=data.1 {
            if r == row {
                continue;
            }
            match worksheet.cell(r, column) {
                None | Some(Cell::EmptyCell { .. }) => blank = true,
                Some(cell) => match (cell.get_formula(), formula) {
                    (Some(f), None) => formula = Some(f),
                    (Some(f), Some(g)) if f == g => {}
                    _ => return Ok(Vec::new()),
                },
            }
            if blank && formula.is_some() {
                return Ok(Vec::new());
            }
            targets.push((r, column));
        }
        Ok(targets)
    }

    /// Writes the formula of (`sheet`, `row`, `column`) into `targets`, the cells
    /// [`Model::calculated_column_targets`] found before it was typed, each with its own style.
    /// The formula is stored relative to its cell, so an A1 reference fills shifted, like a fill
    /// down (`=A2*2` gives `=A3*2` a row below; probes/t2957-a1-reference). Nothing is written
    /// when the cell holds no formula: a typed constant fills nothing.
    pub(crate) fn fill_calculated_column(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        targets: &[(i32, i32)],
    ) -> Result<(), String> {
        let Some(formula_index) = self
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .and_then(Cell::get_formula)
        else {
            return Ok(());
        };
        for &(r, c) in targets {
            self.prepare_cell_for_user_input(sheet, r, c)?;
            let style = self.formula_style_index(sheet, r, c)?;
            self.set_formula_index_with_units(sheet, r, c, formula_index, style)?;
        }
        Ok(())
    }
}
