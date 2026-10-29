//! A worksheet table's metadata (`Workbook::tables`) and the autoFilter criteria it keeps from its
//! file (`Table::filter_columns`), which the export writes back and nothing re-applies.

use bitcode::{Decode, Encode};

use crate::types::sheet_names_match;

/// A table on a worksheet (ECMA-376-1:2016 section 18.5.1.2).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Table {
    pub name: String,
    pub display_name: String,
    pub sheet_name: String,
    pub reference: String,
    pub totals_row_count: u32,
    pub header_row_count: u32,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub columns: Vec<TableColumn>,
    pub style_info: TableStyleInfo,
    pub has_filters: bool,
    /// The criteria of the table's autoFilter as the file stored them, kept so an export writes
    /// back the filter Excel read; they hide no row here (filters are not re-applied). Column
    /// inserts and deletes inside the table move them with their columns.
    pub filter_columns: Vec<FilterColumn>,
}

impl Table {
    /// Whether the table is on the sheet `sheet_name` (sheet names match ignoring case).
    pub fn is_on_sheet(&self, sheet_name: &str) -> bool {
        sheet_names_match(&self.sheet_name, sheet_name)
    }
}

/// One column of a table (ECMA-376-1:2016 section 18.5.1.3).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TableColumn {
    pub id: u32,
    pub name: String,
    pub totals_row_label: Option<String>,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub totals_row_function: Option<String>,
    /// The column's calculated-column formula, Excel's `calculatedColumnFormula`. Only whether it
    /// is set is used; its text, the first data row's formula in English without `=`, is an engine
    /// choice, not yet the file form xlsx stores. Set when a typed formula filled the column
    /// (T-3024); only a column that has one is re-filled by a different typed formula, since a
    /// column of identical stored formulas without it is not a calculated column
    /// (docs/excel-semantics/references.md, "Calculated columns", rule 8). Neither read from nor
    /// written to xlsx yet (T-2969, T-3033).
    pub calculated_column_formula: Option<String>,
}

impl Default for TableColumn {
    fn default() -> Self {
        TableColumn {
            id: 0,
            name: "Column".to_string(),
            totals_row_label: None,
            totals_row_function: None,
            data_dxf_id: None,
            header_row_dxf_id: None,
            totals_row_dxf_id: None,
            calculated_column_formula: None,
        }
    }
}

/// A table's style name and banding flags (ECMA-376-1:2016 section 18.5.1.5).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct TableStyleInfo {
    pub name: Option<String>,
    pub show_first_column: bool,
    pub show_last_column: bool,
    pub show_row_stripes: bool,
    pub show_column_stripes: bool,
}

/// One `<filterColumn>` of a table's autoFilter (ECMA-376-1:2016 section 18.3.2.7).
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct FilterColumn {
    /// The 0-based column within the filter's range.
    pub col_id: u32,
    pub criterion: FilterCriterion,
}

/// The criterion of a [`FilterColumn`].
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum FilterCriterion {
    /// `<filters>`: the rows whose text is one of `values`, plus blank rows when `blank`.
    Values { values: Vec<String>, blank: bool },
    /// `<customFilters>`: one or two conditions, all of which must hold when `and`.
    Custom {
        and: bool,
        conditions: Vec<CustomFilter>,
    },
    /// Any other kind (top 10, dynamic, color or icon filters): applied, but its criterion is
    /// not kept, so an export writes the column with no criterion.
    Other,
}

/// One `<customFilter>`: `operator` is the file's attribute (absent means `equal`) and `val`
/// the operand as text, both kept verbatim for the export only; re-applying a filter would need
/// a typed operator.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CustomFilter {
    pub operator: Option<String>,
    pub val: String,
}
