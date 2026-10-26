//! A structured reference as written (`Sales[[#All],[Qty]]`), kept in the parsed formula next to
//! the area it resolves to (SPEC-0016).

use super::super::token::{TableReference, TableSpecifier};

/// The source of a structured reference: what [`super::Node::StructuredReferenceKind`] prints
/// back in the storage and in-engine printers.
#[derive(PartialEq, Clone, Debug)]
pub struct StructuredReference {
    /// The table name as written; tables match it ignoring case.
    pub table_name: String,
    /// The item specifier (`#All`, `#Data`, ...), if any.
    pub specifier: Option<TableSpecifier>,
    /// The column or column range, if any.
    pub table_reference: Option<TableReference>,
    /// A bare table name (`ROWS(Sales)`), or `Sales[]`, which the lexer reads as the bare name.
    pub bare: bool,
}

/// The parse error of a structured reference to a table that is not there.
pub(super) const TABLE_NOT_FOUND: &str = "Table not found: ";
/// The parse error of a structured reference to a table whose sheet is not there.
pub(super) const TABLE_SHEET_NOT_FOUND: &str = "table sheet not found: ";
/// The parse error of a structured reference to a column the table does not have.
pub(super) const COLUMN_NOT_FOUND: &str = "Expecting column: ";

/// Whether a parse error with `message` comes from a structured reference that did not
/// resolve. The xlsx export writes such a formula as `#REF!`, so a file never holds a structured
/// reference without the table parts it needs (xlsx-storage.md "Structured references in the
/// file"): only the tables the workbook has are exported, as table parts (T-2196).
pub(crate) fn is_unresolved_structured_reference(message: &str) -> bool {
    [TABLE_NOT_FOUND, TABLE_SHEET_NOT_FOUND, COLUMN_NOT_FOUND]
        .iter()
        .any(|prefix| message.starts_with(prefix))
}

/// Excel's name of an item specifier, in its own case.
fn specifier_text(specifier: &TableSpecifier) -> &'static str {
    match specifier {
        TableSpecifier::All => "[#All]",
        TableSpecifier::Data => "[#Data]",
        TableSpecifier::Headers => "[#Headers]",
        TableSpecifier::ThisRow => "[#This Row]",
        TableSpecifier::Totals => "[#Totals]",
        TableSpecifier::HeadersData => "[#Headers],[#Data]",
        TableSpecifier::DataTotals => "[#Data],[#Totals]",
    }
}

/// A column name between brackets, with Excel's `'` escape before each special character (the
/// inverse of the lexer's `consume_escaped_until`; xlsx-storage.md "Structured references in the
/// file": the quote goes before the character).
fn column_text(name: &str) -> String {
    let mut text = String::with_capacity(name.len() + 2);
    text.push('[');
    for c in name.chars() {
        if matches!(c, '[' | ']' | '#' | '\'' | '@') {
            text.push('\'');
        }
        text.push(c);
    }
    text.push(']');
    text
}

fn table_reference_text(table_reference: &TableReference) -> String {
    match table_reference {
        TableReference::ColumnReference(name) => column_text(name),
        TableReference::RangeReference((left, right)) => {
            format!("{}:{}", column_text(left), column_text(right))
        }
    }
}

impl StructuredReference {
    /// The canonical long form: `T[[#Spec],[Col]]`, `T[[#Spec],[A]:[B]]`, `T[Col]`,
    /// `T[[A]:[B]]`, `T[#Spec]`, `T[]`, or the bare name as written (SPEC-0016 "Printing").
    pub(crate) fn text(&self) -> String {
        let name = &self.table_name;
        if self.bare {
            return name.clone();
        }
        match (&self.specifier, &self.table_reference) {
            (None, None) => format!("{name}[]"),
            (Some(specifier), None) => match specifier {
                TableSpecifier::HeadersData | TableSpecifier::DataTotals => {
                    format!("{name}[{}]", specifier_text(specifier))
                }
                _ => format!("{name}{}", specifier_text(specifier)),
            },
            // `T[Col]`; an empty name or one that starts with a space keeps its own brackets,
            // since `T[]` is the bare name and the lexer skips the space.
            (None, Some(TableReference::ColumnReference(column)))
                if !column.is_empty() && !column.starts_with(char::is_whitespace) =>
            {
                format!("{name}{}", column_text(column))
            }
            (None, Some(reference)) => format!("{name}[{}]", table_reference_text(reference)),
            (Some(specifier), Some(reference)) => format!(
                "{name}[{},{}]",
                specifier_text(specifier),
                table_reference_text(reference)
            ),
        }
    }

    /// The form the xlsx file stores: [`Self::text`], with a bare table name written as `T[]`,
    /// since Excel refuses a file holding a bare `T` as corrupt (xlsx-storage.md "Structured
    /// references in the file").
    pub(crate) fn file_text(&self) -> String {
        if self.bare {
            format!("{}[]", self.table_name)
        } else {
            self.text()
        }
    }
}
