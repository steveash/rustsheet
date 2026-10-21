//! Workbook tables (`ListObject`s) added through the API; imported tables come from the xlsx
//! reader. Tables matter to evaluation through structured references and through filters
//! (SUBTOTAL skips filter-hidden rows, functions/subtotal.rs).

use crate::{
    expressions::{
        lexer::LexerMode,
        parser::{
            parse_range,
            stringify::{to_english_string, to_string_lowering},
        },
        types::CellReferenceRC,
        utils::{is_valid_identifier, number_to_column},
    },
    model::Model,
    types::{Table, TableColumn, TableStyleInfo},
};

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

    /// The lowercase names of the tables on the sheet `sheet_name`, for [`to_string_lowering`].
    pub(crate) fn table_names_on_sheet(&self, sheet_name: &str) -> Vec<String> {
        self.workbook
            .tables
            .iter()
            .filter(|(_, table)| table.is_on_sheet(sheet_name))
            .map(|(name, _)| name.to_lowercase())
            .collect()
    }

    /// Takes out the tables of the sheet at `sheet_index`, before the sheet is deleted. The
    /// structured references to them in the other sheets' formulas and in defined names are
    /// first rewritten as the areas they stand for, which the delete turns into `#REF!`, as
    /// before SPEC-0016; a sheet added later under the old name brings none of them back.
    pub(crate) fn take_tables_of_sheet(&mut self, sheet_index: u32) -> Vec<(String, Table)> {
        let Ok(sheet_name) = self.workbook.worksheet(sheet_index).map(|w| w.get_name()) else {
            return Vec::new();
        };
        let lowercase = self.table_names_on_sheet(&sheet_name);
        if lowercase.is_empty() {
            return Vec::new();
        }
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for (index, worksheet) in self.workbook.worksheets.iter_mut().enumerate() {
            if index == sheet_index as usize {
                continue;
            }
            let context = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            for formula in &mut worksheet.shared_formulas {
                let node = self.parser.parse(formula, &context);
                *formula = to_string_lowering(&node, None, &lowercase);
            }
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        // Defined names have no cell context; any cell of a remaining sheet does.
        let remaining = self
            .workbook
            .worksheets
            .iter()
            .enumerate()
            .find(|(index, _)| *index != sheet_index as usize)
            .map(|(_, worksheet)| worksheet.get_name());
        for defined_name in &mut self.workbook.defined_names {
            let Some(sheet) = &remaining else {
                break;
            };
            let context = CellReferenceRC {
                sheet: sheet.clone(),
                row: 1,
                column: 1,
            };
            let stored = defined_name.formula.trim_start();
            let (eq, body) = stored.strip_prefix('=').map_or(("", stored), |b| ("=", b));
            let node = self.parser.parse(body, &context);
            let lowered = to_string_lowering(&node, Some(&context), &lowercase);
            if lowered != to_english_string(&node, &context) {
                defined_name.formula = format!("{eq}{lowered}");
            }
        }
        let (taken, kept) = std::mem::take(&mut self.workbook.tables)
            .into_iter()
            .partition(|(_, table)| table.is_on_sheet(&sheet_name));
        self.workbook.tables = kept;
        taken.into_iter().collect()
    }
}
