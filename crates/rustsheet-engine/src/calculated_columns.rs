//! Calculated columns: a formula typed into a table column fills the column, as Excel does
//! (docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table
//! column"; T-2957, T-2958), and one typed just right of a table grows it by a column (T-2968).

use crate::constants::LAST_COLUMN;
use crate::expressions::utils::number_to_column;
use crate::model::Model;
use crate::table_edits::{append_column, bounds};
use crate::types::{Cell, Table};

/// A table that a formula typed just right of it grows by one column (T-2968).
#[derive(Debug, Clone, PartialEq)]
pub struct TableExpansion {
    /// The table, as it is before the expansion.
    pub table: Table,
    /// The new column's cells other than the typed one, (row, column): its header cell and the
    /// data rows the formula fills, which the expansion writes.
    pub cells: Vec<(i32, i32)>,
    /// Whether the typed cell is the header cell: the formula is then not stored, and the header
    /// is the text `0`.
    pub header: bool,
}

impl Model {
    /// The table that typing `value` into (`sheet`, `row`, `column`) grows by one column, as Excel
    /// grows a table when a formula is typed into the cell just right of its header row or a data
    /// row (probes/t2957-expand-right-data, -header: `T` A1:C4 becomes A1:D4). `None` for a
    /// constant, a table with a totals row, or a new column whose other cells are not all blank
    /// or that another table overlaps: those are unprobed, so the cell is typed as before.
    ///
    /// # Errors
    /// When `sheet` does not exist.
    pub fn table_expansion(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<Option<TableExpansion>, String> {
        if !value.starts_with('=') || column > LAST_COLUMN {
            return Ok(None);
        }
        let worksheet = self.workbook.worksheet(sheet)?;
        let sheet_name = worksheet.get_name();
        let on_sheet = || {
            (self.workbook.tables.values())
                .filter(|table| table.is_on_sheet(&sheet_name))
                .filter_map(|table| Some((table, bounds(table)?)))
        };
        let Some((table, b)) = on_sheet().find(|(table, b)| {
            table.totals_row_count == 0 && b.right + 1 == column && (b.top..=b.bottom).contains(&row)
        }) else {
            return Ok(None);
        };
        let overlapped = on_sheet().any(|(_, other)| {
            (other.left..=other.right).contains(&column) && other.top <= b.bottom && b.top <= other.bottom
        });
        let cells: Vec<(i32, i32)> = (b.top..=b.bottom)
            .filter(|&r| r != row)
            .map(|r| (r, column))
            .collect();
        let blank = cells.iter().all(|&(r, c)| {
            matches!(worksheet.cell(r, c), None | Some(Cell::EmptyCell { .. }))
        });
        if overlapped || !blank {
            return Ok(None);
        }
        Ok(Some(TableExpansion {
            table: table.clone(),
            cells,
            header: table.header_row_count > 0 && row == b.top,
        }))
    }

    /// Grows the table of `expansion` (on `sheet`) by its new column, and writes the column's
    /// name into its header cell: `Column1` as for a column insert (probes/t2957-expand-right-data:
    /// D1 is `Column1`), or `0` when the formula was typed into the header cell itself
    /// (-header: `=1+1` typed into D1 leaves the text `0`, not 2 and no formula). Excel's rule for
    /// that `0` is only probed with `=1+1`; a name `0` the table already has falls back to
    /// `ColumnN` (unprobed).
    pub(crate) fn expand_table(&mut self, sheet: u32, expansion: &TableExpansion) -> Result<(), String> {
        let Some(b) = bounds(&expansion.table) else {
            return Ok(());
        };
        let column = b.right + 1;
        let (first, last) = number_to_column(b.left)
            .zip(number_to_column(column))
            .ok_or_else(|| format!("No column {column}"))?;
        let name = expansion.table.name.clone();
        let Some(table) = self.workbook.tables.get_mut(&name) else {
            return Ok(());
        };
        table.reference = format!("{first}{}:{last}{}", b.top, b.bottom);
        let header_text = append_column(&mut table.columns, expansion.header.then_some("0"));
        let has_header = table.header_row_count > 0;
        self.reset_tables();
        if has_header {
            self.prepare_cell_for_user_input(sheet, b.top, column)?;
            self.update_cell_with_text(sheet, b.top, column, &header_text)?;
        }
        Ok(())
    }

    /// Puts `table` back as it was (the facade's undo of [`Model::table_expansion`]'s growth),
    /// and reparses every formula against it.
    pub fn restore_table(&mut self, table: Table) {
        self.workbook.tables.insert(table.name.clone(), table);
        self.reset_tables();
    }

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
        // The column's one formula, and whether a blank cell was met: the other cells must be all
        // blank or all that formula, so blank and formula cells mixed are no calculated column.
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
    /// when the cell holds no formula: a typed constant fills nothing. A target the write guards
    /// refuse (`prepare_cell_for_user_input`) ends the fill with an error and the rows before it
    /// filled; the facade's edit rolls the whole write back.
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
