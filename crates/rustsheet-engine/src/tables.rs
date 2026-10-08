//! Workbook tables (`ListObject`s) added through the API; imported tables come from the xlsx
//! reader. Tables matter to evaluation through structured references and through filters
//! (SUBTOTAL skips filter-hidden rows, functions/subtotal.rs).

use crate::{
    expressions::{
        parser::parse_range,
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
    fn reset_tables(&mut self) {
        self.parser.set_tables(self.workbook.tables.clone());
        self.reset_parsed_structures();
    }
}
