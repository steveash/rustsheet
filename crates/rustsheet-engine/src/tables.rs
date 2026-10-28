//! Workbook tables (`ListObject`s) added through the API; imported tables come from the xlsx
//! reader. Tables matter to evaluation through structured references and through filters
//! (SUBTOTAL skips filter-hidden rows, functions/subtotal.rs).

use crate::{
    expressions::{
        lexer::LexerMode,
        parser::{
            parse_range,
            stringify::{to_rc_lowering_at, to_string_lowering, to_string_with_ref_errors},
            Node,
        },
        types::{Area, CellReferenceRC},
        utils::{is_valid_identifier, number_to_column},
    },
    formula_rewrite::RewrittenFormula,
    model::Model,
    types::{Cell, Table, Worksheet},
};

/// Gives every formula cell of `worksheet` whose shared formula is one of `by_row` (index, parsed
/// formula holding a `#This Row` reference to one of `tables`) the text that formula lowers to on
/// the cell's own row, a shared formula of its own: a duplicated sheet, which gets no table, then
/// reads what each cell read before, `#VALUE!` off the data rows (T-1699). Rows whose text is the
/// same share one formula. Cells on the rows `keep_rows` (inclusive) keep their shared formula.
pub(crate) fn lower_this_row_cells(
    worksheet: &mut Worksheet,
    by_row: &[(usize, Node)],
    tables: &[String],
    keep_rows: Option<(i32, i32)>,
) {
    if by_row.is_empty() {
        return;
    }
    let cells: Vec<(i32, i32, usize)> = worksheet
        .sheet_data
        .cells()
        .filter(|(row, _, _)| keep_rows.is_none_or(|(top, bottom)| *row < top || *row > bottom))
        .filter_map(|(row, column, cell)| {
            let index = usize::try_from(cell.get_formula()?).ok()?;
            Some((row, column, index))
        })
        .collect();
    let mut known: std::collections::HashMap<String, i32> = worksheet
        .shared_formulas
        .iter()
        .enumerate()
        .filter_map(|(index, text)| Some((text.clone(), i32::try_from(index).ok()?)))
        .collect();
    for (row, column, index) in cells {
        let Some((_, node)) = by_row.iter().find(|(i, _)| *i == index) else {
            continue;
        };
        let text = to_rc_lowering_at(node, row, tables);
        let Ok(next) = i32::try_from(worksheet.shared_formulas.len()) else {
            return;
        };
        let new_index = *known.entry(text.clone()).or_insert_with(|| {
            worksheet.shared_formulas.push(text);
            next
        });
        if let Some(Cell::CellFormula { f, .. } | Cell::ArrayFormula { f, .. }) =
            worksheet.cell_mut(row, column)
        {
            *f = new_index;
        }
    }
}

impl Model {
    /// Adds a table on sheet `sheet` over rows `top..=bottom` and columns `left..=right`
    /// (1-based). The first row is the header row: its cells name the columns, so they must be
    /// non-empty and distinct (ignoring case). `has_filters` marks the table's autoFilter as
    /// filtering, which is how an imported table with a filter looks; it hides no row by itself.
    /// Reparses and re-evaluates the workbook so structured references can name the table.
    ///
    /// # Errors
    /// An invalid or taken name (tables and defined names share one namespace), an invalid
    /// sheet, a range without a data row or outside the sheet, a range that overlaps another
    /// table, or an empty or duplicate header cell.
    pub fn add_table(
        &mut self,
        sheet: u32,
        name: &str,
        range: (i32, i32, i32, i32),
        has_filters: bool,
    ) -> Result<(), String> {
        self.add_table_with_totals(sheet, name, range, has_filters, false)
    }

    /// [`Model::add_table`] for a range whose last row is the table's totals row when
    /// `totals_row` is set (`Table[#Totals]`, `Table[#All]`; the cells of that row keep what they
    /// hold). Such a table needs a header row, a data row and the totals row.
    ///
    /// # Errors
    /// As [`Model::add_table`], with the range one row taller for a totals row.
    pub fn add_table_with_totals(
        &mut self,
        sheet: u32,
        name: &str,
        (top, left, bottom, right): (i32, i32, i32, i32),
        has_filters: bool,
        totals_row: bool,
    ) -> Result<(), String> {
        if !is_valid_identifier(name) {
            return Err(format!("Invalid table name '{name}'"));
        }
        let upper = name.to_uppercase();
        if self
            .workbook
            .tables
            .keys()
            .any(|t| t.to_uppercase() == upper)
            || self
                .workbook
                .defined_names
                .iter()
                .any(|d| d.name.to_uppercase() == upper)
        {
            return Err(format!("The name '{name}' is already taken"));
        }
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        let (Some(first), Some(last)) = (number_to_column(left), number_to_column(right)) else {
            return Err("Table columns are outside the sheet".to_string());
        };
        let totals_rows = i32::from(totals_row);
        if top < 1
            || top.saturating_add(totals_rows) >= bottom
            || left > right
            || bottom > crate::constants::LAST_ROW
        {
            return Err("A table needs a header row and at least one data row".to_string());
        }
        let overlaps = self.workbook.tables.values().any(|t| {
            t.sheet_name == sheet_name
                && parse_range(&t.reference).is_ok_and(|(c1, r1, c2, r2)| {
                    c1 <= right && left <= c2 && r1 <= bottom && top <= r2
                })
        });
        if overlaps {
            return Err("A table cannot overlap another table".to_string());
        }
        let mut columns: Vec<String> = Vec::new();
        for column in left..=right {
            let header = self.get_formatted_cell_value(sheet, top, column)?;
            if header.is_empty()
                || columns
                    .iter()
                    .any(|c| c.to_uppercase() == header.to_uppercase())
            {
                return Err(format!("Table header '{header}' is empty or repeated"));
            }
            columns.push(header);
        }
        let table = Table {
            has_filters,
            ..Table::new(
                name,
                &sheet_name,
                &format!("{first}{top}:{last}{bottom}"),
                (1, u32::from(totals_row)),
                columns,
            )
        };
        self.workbook.tables.insert(name.to_string(), table);
        self.reset_tables();
        Ok(())
    }

    /// Removes the table `name` (the undo of [`Model::add_table`]); its cells stay.
    ///
    /// # Errors
    /// There is no table with that exact name.
    pub fn remove_table(&mut self, name: &str) -> Result<(), String> {
        self.workbook
            .tables
            .remove(name)
            .ok_or_else(|| format!("No table named '{name}'"))?;
        self.reset_tables();
        Ok(())
    }

    /// Deletes the table `name` as Graph's `DELETE tables('T')` does: the table and every cell of
    /// its range go (T-2324: `SUM(S!C4:C7)` is 0 afterwards; the rows stay in place, a plain
    /// reference below the table does not shift). Does not recalculate.
    ///
    /// # Errors
    /// There is no table with that exact name, or its range cannot be cleared (it holds part of
    /// an array formula).
    pub fn delete_table(&mut self, name: &str) -> Result<(), String> {
        let table = self
            .workbook
            .tables
            .get(name)
            .ok_or_else(|| format!("No table named '{name}'"))?;
        let sheet = self
            .get_sheet_index_by_name(&table.sheet_name)
            .ok_or_else(|| format!("No sheet named '{}'", table.sheet_name))?;
        let (left, top, right, bottom) = parse_range(&table.reference)
            .map_err(|_| format!("Table '{name}' has an invalid range"))?;
        // Clamped to the used area: the range comes from the file and can span the whole sheet,
        // and the cells past the used area are empty, so clearing them changes nothing.
        let (last_row, last_column) = self.workbook.worksheet(sheet)?.used_area_end();
        let (bottom, right) = (bottom.min(last_row), right.min(last_column));
        if top <= bottom && left <= right {
            self.range_clear_all(&Area {
                sheet,
                row: top,
                column: left,
                width: right - left + 1,
                height: bottom - top + 1,
            })?;
        }
        // Excel rewrites the references to a deleted table to `#REF!` (FORMULATEXT
        // `=SUM(#REF!)`, `ROWS(Sales)` is `#REF!`; T-2340, probes/table-edits/table-delete-table);
        // `remove_table` alone would leave them as parse errors.
        self.rewrite_table_references(&[name.to_lowercase()], None, TableRewrite::RefError);
        self.remove_table(name)
    }

    /// Converts the table `name` to a plain range as Graph's `convertToRange` does: the cells
    /// stay and every structured reference to it, in any sheet's formulas and in defined names,
    /// becomes the absolute range it resolved to (T-2342: `=SUM(Sales[Qty])` is `=SUM(S!$C$4:$C$7)`
    /// in FORMULATEXT, `ROWS(Sales[#All])` stays 5). Does not recalculate.
    ///
    /// # Errors
    /// There is no table with that exact name.
    pub fn convert_table_to_range(&mut self, name: &str) -> Result<(), String> {
        if !self.workbook.tables.contains_key(name) {
            return Err(format!("No table named '{name}'"));
        }
        let tables = [name.to_lowercase()];
        let data_rows = self
            .workbook
            .tables
            .get(name)
            .and_then(|table| Some(crate::table_edits::bounds(table)?.data));
        self.lower_this_row_references(&tables, data_rows);
        self.rewrite_table_references(&tables, None, TableRewrite::Lower);
        self.remove_table(name)
    }

    /// Hands the table list to the parser and reparses every formula against it.
    pub(crate) fn reset_tables(&mut self) {
        self.parser.set_tables(self.workbook.tables.clone());
        self.reset_parsed_structures();
    }

    /// Points the tables of the sheet named `old_name` at `new_name` and hands the list to the
    /// parser, so the reparse after a sheet rename still finds them (SPEC-0016 "Sheet rename,
    /// delete and restore").
    pub(crate) fn rename_tables_sheet(&mut self, old_name: &str, new_name: &str) {
        for table in self.workbook.tables.values_mut() {
            if table.is_on_sheet(old_name) {
                new_name.clone_into(&mut table.sheet_name);
            }
        }
        self.parser.set_tables(self.workbook.tables.clone());
    }

    /// The tables on the sheet `sheet_name`, by name.
    pub(crate) fn tables_on_sheet(&self, sheet_name: &str) -> Vec<(String, Table)> {
        self.workbook
            .tables
            .iter()
            .filter(|(_, table)| table.is_on_sheet(sheet_name))
            .map(|(name, table)| (name.clone(), table.clone()))
            .collect()
    }

    /// Puts back tables taken out with their sheet (the undo of a sheet delete), and reparses.
    /// A table added since under the same name keeps its place.
    pub(crate) fn restore_tables(&mut self, tables: Vec<(String, Table)>) {
        for (name, table) in tables {
            self.workbook.tables.entry(name).or_insert(table);
        }
        self.reset_tables();
    }

    /// The lowercase names of the tables on the sheet `sheet_name`, for the delete rewrite.
    pub(crate) fn table_names_on_sheet(&self, sheet_name: &str) -> Vec<String> {
        self.workbook
            .tables
            .iter()
            .filter(|(_, table)| table.is_on_sheet(sheet_name))
            .map(|(name, _)| name.to_lowercase())
            .collect()
    }

    /// Takes out the tables of the sheet at `sheet_index`, before the sheet is deleted, with the
    /// formulas that referred to them. Their structured references in the other sheets' formulas
    /// and in defined names become `#REF!`, as in Excel (`=SUM(#REF!)`, T-2340,
    /// probes/table-edits/table-delete-sheet); a sheet added later under the old name brings
    /// none of them back, and [`Model::restore_table_references`] puts the text back.
    pub(crate) fn take_tables_of_sheet(
        &mut self,
        sheet_index: u32,
    ) -> (Vec<(String, Table)>, Vec<RewrittenFormula>) {
        let Ok(sheet_name) = self.workbook.worksheet(sheet_index).map(|w| w.get_name()) else {
            return (Vec::new(), Vec::new());
        };
        let lowercase = self.table_names_on_sheet(&sheet_name);
        if lowercase.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let rewritten = self.rewrite_table_references(
            &lowercase,
            Some(sheet_index as usize),
            TableRewrite::RefError,
        );
        let (taken, kept) = std::mem::take(&mut self.workbook.tables)
            .into_iter()
            .partition(|(_, table)| table.is_on_sheet(&sheet_name));
        self.workbook.tables = kept;
        (taken.into_iter().collect(), rewritten)
    }

    /// Gives every cell off the table's `data_rows` whose formula holds a `#This Row` reference
    /// to one of `tables` the text that reference lowers to there (`lower_this_row_cells`), so it
    /// stays `#VALUE!` after the convert as in Excel (S!F1, S!F9 and T!A1 of
    /// probes/table-edits/table-convert-names-this-row, T-2643): a shared formula has no row.
    /// Cells on the data rows keep the shared formula, which lowers to the relative form Excel
    /// prints (`=$C4*2`, table-convert-same-sheet).
    fn lower_this_row_references(&mut self, tables: &[String], data_rows: Option<(i32, i32)>) {
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for worksheet in &mut self.workbook.worksheets {
            let context = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            let by_row: Vec<(usize, Node)> = worksheet
                .shared_formulas
                .iter()
                .enumerate()
                .filter_map(|(index, formula)| {
                    let node = self.parser.parse(formula, &context);
                    node.has_this_row_to(tables).then_some((index, node))
                })
                .collect();
            lower_this_row_cells(worksheet, &by_row, tables, data_rows);
        }
        self.parser.set_lexer_mode(LexerMode::A1);
    }

    /// Prints `#REF!` for every structured reference to one of `tables` (lowercase names) in the
    /// formulas of every sheet but `skip_sheet` and in the defined names, and returns what it
    /// changed. Does not reparse.
    fn rewrite_table_references(
        &mut self,
        tables: &[String],
        skip_sheet: Option<usize>,
        rewrite: TableRewrite,
    ) -> Vec<RewrittenFormula> {
        self.rewrite_formulas(skip_sheet, |node, context| match rewrite {
            TableRewrite::RefError => to_string_with_ref_errors(&node, context, tables),
            TableRewrite::Lower => to_string_lowering(&node, context, tables),
        })
    }
}

/// How [`Model::rewrite_table_references`] prints a structured reference to a removed table.
#[derive(Clone, Copy)]
enum TableRewrite {
    /// `#REF!`: the table is deleted.
    RefError,
    /// The absolute area it resolved to: the table is converted to a range.
    Lower,
}
