//! Calculated columns: a formula typed into a table column fills the column, as Excel does
//! (docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table
//! column"; T-2957, T-2958), and one typed just right of a table grows it by a column (T-2968).

use crate::constants::LAST_COLUMN;
use crate::expressions::parser::Node;
use crate::expressions::utils::number_to_column;
use crate::model::Model;
use crate::table_edits::{append_column, bounds, Bounds};
use crate::types::{Cell, Table};

/// A table that a formula typed just right of it grows by one column (T-2968).
#[derive(Debug, Clone, PartialEq)]
pub struct TableExpansion {
    /// The table, as it is before the expansion.
    pub table: Table,
    /// The new column's cells other than the typed one, (row, column): its header cell and the
    /// data rows the formula fills, which the expansion writes.
    pub cells: Vec<(i32, i32)>,
    /// Whether the typed cell is the header cell. Then the formula is not stored and the header is
    /// the text `0`.
    pub header: bool,
}

/// The table column a typed formula fills as a calculated column ([`Model::calculated_column`]).
#[derive(Debug, Clone, PartialEq)]
pub struct CalculatedColumn {
    /// The table, as it is before the formula is typed.
    pub table: Table,
    /// The column's other data rows, (row, column): the cells the formula fills.
    pub targets: Vec<(i32, i32)>,
}

impl Model {
    /// The tables on the sheet named `sheet_name` whose range fits, with their [`Bounds`].
    fn table_bounds_on_sheet<'a>(
        &'a self,
        sheet_name: &'a str,
    ) -> impl Iterator<Item = (&'a Table, Bounds)> + 'a {
        (self.workbook.tables.values())
            .filter(move |table| table.is_on_sheet(sheet_name))
            .filter_map(|table| Some((table, bounds(table)?)))
    }

    /// The table that typing `value` into (`sheet`, `row`, `column`) grows by one column, as Excel
    /// grows a table when a formula is typed into the cell just right of its header row or a data
    /// row (probes/t2957-expand-right-data, -header: `T` A1:C4 becomes A1:D4). `None` for a
    /// constant, a table with a totals row, or a new column whose other cells are not all blank
    /// or that another table overlaps: those are unprobed, so the cell is typed as before. `None`
    /// too for a formula that does not parse there as typed: Excel refuses such an entry and
    /// changes nothing.
    ///
    /// # Errors
    /// When `sheet` does not exist.
    pub fn table_expansion(
        &mut self,
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
        let on_sheet = || self.table_bounds_on_sheet(&sheet_name);
        let Some((table, b)) = on_sheet().find(|(table, b)| {
            table.totals_row_count == 0
                && b.right + 1 == column
                && (b.top..=b.bottom).contains(&row)
        }) else {
            return Ok(None);
        };
        let overlapped = on_sheet().any(|(_, other)| {
            (other.left..=other.right).contains(&column)
                && other.top <= b.bottom
                && b.top <= other.bottom
        });
        let cells: Vec<(i32, i32)> = (b.top..=b.bottom)
            .filter(|&r| r != row)
            .map(|r| (r, column))
            .collect();
        let blank = cells
            .iter()
            .all(|&(r, c)| matches!(worksheet.cell(r, c), None | Some(Cell::EmptyCell { .. })));
        if overlapped || !blank {
            return Ok(None);
        }
        let header = table.header_row_count > 0 && row == b.top;
        let expansion = TableExpansion {
            table: table.clone(),
            // Typed into the header cell, the formula writes no data row.
            cells: if header { Vec::new() } else { cells },
            header,
        };
        Ok(self
            .parses_as_typed(sheet, row, column, value)
            .then_some(expansion))
    }

    /// Whether `value`, a formula, parses as typed into (`sheet`, `row`, `column`): no parse
    /// error and no refusal of typed text (decision 0103).
    fn parses_as_typed(&mut self, sheet: u32, row: i32, column: i32, value: &str) -> bool {
        let Some(formula) = self.formula_without_prefix(value) else {
            return false;
        };
        self.parser.set_typed(true);
        let parsed = self.parse_cell_formula(sheet, row, column, formula);
        self.parser.set_typed(false);
        matches!(parsed, Ok(node) if !matches!(node, Node::ParseErrorKind { .. }))
    }

    /// Grows the table of `expansion` (on `sheet`) by its new column, and writes the column's
    /// name into its header cell: `Column1` as for a column insert (probes/t2957-expand-right-data:
    /// D1 is `Column1`), or `0` when the formula was typed into the header cell itself
    /// (-header: `=1+1` typed into D1 leaves the text `0`, not 2 and no formula). Excel's rule for
    /// that `0` is only probed with `=1+1`; a name `0` the table already has falls back to
    /// `ColumnN` (unprobed).
    pub(crate) fn expand_table(
        &mut self,
        sheet: u32,
        expansion: &TableExpansion,
    ) -> Result<(), String> {
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
            let written = self
                .prepare_cell_for_user_input(sheet, b.top, column)
                .and_then(|()| self.update_cell_with_text(sheet, b.top, column, &header_text));
            if written.is_err() {
                // No grown table without its header: put it back.
                self.restore_table(expansion.table.clone());
            }
            written?;
        }
        Ok(())
    }

    /// Puts `table` back as it was (the facade's undo of [`Model::table_expansion`]'s growth),
    /// and reparses every formula against it.
    pub fn restore_table(&mut self, table: Table) {
        self.workbook.tables.insert(table.name.clone(), table);
        self.reset_tables();
    }

    /// The table, as it is now, whose column a formula typed into (`sheet`, `row`, `column`)
    /// fills or re-fills as a calculated column, and the cells it fills, every other data row of
    /// the column (row, column). The table is one in whose data rows the cell is, when the
    /// column's other data cells are all blank (Excel fills an empty column), or all hold one
    /// formula and the column is a calculated column (`calculated_column_formula`: Excel re-fills
    /// a filled column, but not a column of identical stored formulas, rule 8 of
    /// docs/excel-semantics/references.md "Calculated columns"). `None` when another data cell
    /// holds a value or a different formula (Excel leaves a column with an exception, a typed
    /// constant, alone), or is blank beside formulas (probes/t2957-*, t2983-*). The typed formula
    /// makes the column a calculated column, so the facade keeps this table for its rollback.
    ///
    /// # Errors
    /// When `sheet` does not exist.
    pub fn calculated_column(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<CalculatedColumn>, String> {
        let Some(fill) = self.calculated_column_fill(sheet, row, column)? else {
            return Ok(None);
        };
        Ok(
            (self.workbook.tables.get(&fill.table)).map(|table| CalculatedColumn {
                table: table.clone(),
                targets: fill.targets,
            }),
        )
    }

    /// What a formula typed into (`sheet`, `row`, `column`) fills ([`Model::calculated_column`]).
    /// The totals row is never filled.
    pub(crate) fn calculated_column_fill(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<ColumnFill>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let sheet_name = worksheet.get_name();
        let Some((table, b)) = self.table_bounds_on_sheet(&sheet_name).find(|(_, b)| {
            (b.left..=b.right).contains(&column) && (b.data.0..=b.data.1).contains(&row)
        }) else {
            return Ok(None);
        };
        let Ok(index) = usize::try_from(column - b.left) else {
            return Ok(None);
        };
        let calculated =
            (table.columns.get(index)).is_some_and(|c| c.calculated_column_formula.is_some());
        // The column's one formula, and whether a blank cell was met: the other cells must be all
        // blank or all that formula, so blank and formula cells mixed are no calculated column.
        let mut formula = None;
        let mut blank = false;
        let mut targets = Vec::new();
        for r in b.data.0..=b.data.1 {
            if r == row {
                continue;
            }
            match worksheet.cell(r, column) {
                None | Some(Cell::EmptyCell { .. }) => blank = true,
                Some(cell) => match (cell.get_formula(), formula) {
                    (Some(f), None) => formula = Some(f),
                    (Some(f), Some(g)) if f == g => {}
                    _ => return Ok(None),
                },
            }
            if blank && formula.is_some() {
                return Ok(None);
            }
            targets.push((r, column));
        }
        // Identical formulas the column holds without being a calculated column were stored, not
        // filled by typing (probes/t2983-identical-stored-refill: only the typed cell changes).
        if formula.is_some() && !calculated {
            return Ok(None);
        }
        Ok(Some(ColumnFill {
            table: table.name.clone(),
            index,
            first_row: b.data.0,
            targets,
        }))
    }

    /// Writes the formula of (`sheet`, `row`, `column`) into `fill`'s targets, the cells
    /// [`Model::calculated_column_fill`] found before it was typed, each with its own style, and
    /// makes the column a calculated column (`calculated_column_formula`, the first data row's
    /// text in English; only whether it is set is read). The formula is stored relative to its
    /// cell, so an A1 reference fills shifted, like a fill down (`=A2*2` gives `=A3*2` a row
    /// below; probes/t2957-a1-reference). Nothing is written when the cell holds no formula: a typed
    /// constant fills nothing. A target the write guards refuse (`prepare_cell_for_user_input`)
    /// ends the fill with an error and the rows before it filled; the facade's edit rolls the
    /// whole write back.
    pub(crate) fn fill_calculated_column(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        fill: &ColumnFill,
    ) -> Result<(), String> {
        let Some(formula_index) = self
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .and_then(Cell::get_formula)
        else {
            return Ok(());
        };
        for &(r, c) in &fill.targets {
            self.prepare_cell_for_user_input(sheet, r, c)?;
            let style = self.formula_style_index(sheet, r, c)?;
            self.set_formula_index_with_units(sheet, r, c, formula_index, style)?;
        }
        let text = self.get_english_cell_formula(sheet, fill.first_row, column)?;
        let text = text.map(|text| text.strip_prefix('=').unwrap_or(&text).to_string());
        if let Some(table_column) = (self.workbook.tables.get_mut(&fill.table))
            .and_then(|table| table.columns.get_mut(fill.index))
        {
            table_column.calculated_column_formula = text;
        }
        Ok(())
    }
}

/// The calculated-column fill of one typed cell ([`Model::calculated_column_fill`]).
pub(crate) struct ColumnFill {
    /// The table's name.
    table: String,
    /// The column's index in the table.
    index: usize,
    /// The table's first data row.
    first_row: i32,
    /// The other data rows of the column, (row, column): the cells the formula fills.
    targets: Vec<(i32, i32)>,
}
