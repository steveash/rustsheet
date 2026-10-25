//! Workbook tables (`ListObject`s) added through the API; imported tables come from the xlsx
//! reader. Tables matter to evaluation through structured references and through filters
//! (SUBTOTAL skips filter-hidden rows, functions/subtotal.rs).

use crate::{
    expressions::{
        lexer::LexerMode,
        parser::{
            parse_range,
            stringify::{to_english_string, to_rc_lowering_at, to_string_with_ref_errors},
            Node,
        },
        types::{Area, CellReferenceRC},
        utils::{is_valid_identifier, number_to_column},
    },
    model::Model,
    types::{Cell, Table, TableColumn, TableStyleInfo, Worksheet},
};

/// Gives every formula cell of `worksheet` whose shared formula is one of `by_row` (index, parsed
/// formula holding a `#This Row` reference to one of `tables`) the text that formula lowers to on
/// the cell's own row, a shared formula of its own: a duplicated sheet, which gets no table, then
/// reads what each cell read before, `#VALUE!` off the data rows (T-1699). Rows whose text is the
/// same share one formula.
pub(crate) fn lower_this_row_cells(
    worksheet: &mut Worksheet,
    by_row: &[(usize, Node)],
    tables: &[String],
) {
    if by_row.is_empty() {
        return;
    }
    let cells: Vec<(i32, i32, usize)> = worksheet
        .sheet_data
        .cells()
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
        (top, left, bottom, right): (i32, i32, i32, i32),
        has_filters: bool,
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
        if top < 1 || top >= bottom || left > right || bottom > crate::constants::LAST_ROW {
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
        let mut columns: Vec<TableColumn> = Vec::new();
        for (id, column) in (1..).zip(left..=right) {
            let header = self.get_formatted_cell_value(sheet, top, column)?;
            if header.is_empty()
                || columns
                    .iter()
                    .any(|c| c.name.to_uppercase() == header.to_uppercase())
            {
                return Err(format!("Table header '{header}' is empty or repeated"));
            }
            columns.push(TableColumn {
                id,
                name: header,
                ..TableColumn::default()
            });
        }
        let table = Table {
            name: name.to_string(),
            display_name: name.to_string(),
            sheet_name,
            reference: format!("{first}{top}:{last}{bottom}"),
            totals_row_count: 0,
            header_row_count: 1,
            header_row_dxf_id: None,
            data_dxf_id: None,
            totals_row_dxf_id: None,
            columns,
            // As the xlsx reader defaults a table without `tableStyleInfo`.
            style_info: TableStyleInfo {
                name: None,
                show_first_column: false,
                show_last_column: false,
                show_row_stripes: true,
                show_column_stripes: false,
            },
            has_filters,
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
        self.rewrite_table_references(&[name.to_lowercase()], None);
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
        let rewritten = self.rewrite_table_references(&lowercase, Some(sheet_index as usize));
        let (taken, kept) = std::mem::take(&mut self.workbook.tables)
            .into_iter()
            .partition(|(_, table)| table.is_on_sheet(&sheet_name));
        self.workbook.tables = kept;
        (taken.into_iter().collect(), rewritten)
    }

    /// Prints `#REF!` for every structured reference to one of `tables` (lowercase names) in the
    /// formulas of every sheet but `skip_sheet` and in the defined names, and returns what it
    /// changed. Does not reparse.
    fn rewrite_table_references(
        &mut self,
        tables: &[String],
        skip_sheet: Option<usize>,
    ) -> Vec<RewrittenFormula> {
        let mut rewritten = Vec::new();
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for (index, worksheet) in self.workbook.worksheets.iter_mut().enumerate() {
            if Some(index) == skip_sheet {
                continue;
            }
            let context = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            for (formula_index, formula) in worksheet.shared_formulas.iter_mut().enumerate() {
                let node = self.parser.parse(formula, &context);
                let text = to_string_with_ref_errors(&node, None, tables);
                if text != *formula {
                    rewritten.push(RewrittenFormula {
                        place: FormulaPlace::Shared {
                            sheet_id: worksheet.sheet_id,
                            index: formula_index,
                        },
                        original: std::mem::replace(formula, text.clone()),
                        rewritten: text,
                    });
                }
            }
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        // Defined names have no cell context; any cell of a remaining sheet does.
        let remaining = self
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(index, _)| Some(*index) != skip_sheet)
            .map(|(_, worksheet)| worksheet.get_name());
        let Some(sheet) = remaining else {
            return rewritten;
        };
        let context = CellReferenceRC {
            sheet,
            row: 1,
            column: 1,
        };
        for (index, defined_name) in self.workbook.defined_names.iter_mut().enumerate() {
            let stored = defined_name.formula.trim_start();
            let (eq, body) = stored.strip_prefix('=').map_or(("", stored), |b| ("=", b));
            let node = self.parser.parse(body, &context);
            let text = to_string_with_ref_errors(&node, Some(&context), tables);
            if text != to_english_string(&node, &context) {
                let text = format!("{eq}{text}");
                rewritten.push(RewrittenFormula {
                    place: FormulaPlace::DefinedName(index),
                    original: std::mem::replace(&mut defined_name.formula, text.clone()),
                    rewritten: text,
                });
            }
        }
        rewritten
    }

    /// Puts back the formulas [`Model::take_tables_of_sheet`] rewrote, where they still hold the
    /// rewritten text (the undo of a sheet delete). Does not reparse.
    pub(crate) fn restore_table_references(&mut self, rewritten: Vec<RewrittenFormula>) {
        for RewrittenFormula {
            place,
            original,
            rewritten,
        } in rewritten
        {
            let slot = match place {
                FormulaPlace::Shared { sheet_id, index } => self
                    .workbook
                    .worksheets
                    .iter_mut()
                    .find(|worksheet| worksheet.sheet_id == sheet_id)
                    .and_then(|worksheet| worksheet.shared_formulas.get_mut(index)),
                FormulaPlace::DefinedName(index) => self
                    .workbook
                    .defined_names
                    .get_mut(index)
                    .map(|defined_name| &mut defined_name.formula),
            };
            if let Some(slot) = slot.filter(|slot| **slot == rewritten) {
                *slot = original;
            }
        }
    }
}

/// A formula that a table or sheet delete rewrote, kept so the undo can put it back.
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub(crate) struct RewrittenFormula {
    place: FormulaPlace,
    original: String,
    rewritten: String,
}

/// Where a [`RewrittenFormula`] lives.
#[derive(Debug, Clone, Copy, bitcode::Encode, bitcode::Decode)]
enum FormulaPlace {
    /// A shared formula of the sheet with this id.
    Shared { sheet_id: u32, index: usize },
    /// A defined name, by its index in the workbook's list.
    DefinedName(usize),
}
