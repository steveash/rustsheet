//! A structured reference as written (`Sales[[#All],[Qty]]`), kept in the parsed formula next to
//! the area it resolves to (SPEC-0016).

use super::super::lexer::{Lexer, LexerMode};
use super::super::token::{TableReference, TableSpacing, TableSpecifier, TokenType};
use crate::language::get_language;
use crate::locale::get_locale;

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
    /// The whitespace kept inside the outer brackets (T-2801).
    pub spacing: TableSpacing,
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
/// file": the quote goes before the character). `@` is special only first, where it would read as
/// `#This Row`: Excel's own file has `Table9[Totals '[']^&*@'#$,,<>]` (`calc_test_tables/tables.xlsx`
/// Escape!E6).
fn column_text(name: &str) -> String {
    format!("[{}]", escaped_column(name))
}

/// [`column_text`] without its brackets.
fn escaped_column(name: &str) -> String {
    let mut text = String::with_capacity(name.len());
    for (index, c) in name.chars().enumerate() {
        if matches!(c, '[' | ']' | '#' | '\'') || (c == '@' && index == 0) {
            text.push('\'');
        }
        text.push(c);
    }
    text
}

/// Whether a single column prints in the bare form `T[Col]`: an empty name or one that starts
/// with a space keeps its own brackets, since `T[]` is the bare name and the lexer skips the
/// space, and so does one that ends with a space, as Excel stores it (`Table3[[the two ]]`,
/// calc_test_tables/tables.xlsx Hidden!B6).
fn prints_bare(column: &str) -> bool {
    !column.is_empty()
        && !column.starts_with(char::is_whitespace)
        && !column.ends_with(char::is_whitespace)
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
    /// `T[[A]:[B]]`, `T[#Spec]`, `T[]`, or the bare name as written (SPEC-0016 "Printing"), with
    /// the whitespace Excel keeps inside the outer brackets: the runs after `[` and before `]` as
    /// typed, and one space after the comma where any was typed. A single column in padded
    /// brackets drops its own: `T[ [Col] ]` prints `T[ Col ]` (syntax.md "Whitespace beside a
    /// missing argument is kept; a structured reference keeps some and normalizes the rest";
    /// probes/t2786-whitespace-structured-refs-runs F4:F15, T-2801).
    pub(crate) fn text(&self) -> String {
        let name = &self.table_name;
        if self.bare {
            return name.clone();
        }
        let TableSpacing {
            open,
            close,
            comma_space,
        } = &self.spacing;
        let comma = if *comma_space { ", " } else { "," };
        match (&self.specifier, &self.table_reference) {
            (None, None) => format!("{name}[]"),
            (Some(specifier), None) => match specifier {
                TableSpecifier::HeadersData | TableSpecifier::DataTotals => {
                    format!("{name}[{open}{}{close}]", specifier_text(specifier))
                }
                // Padding around a lone specifier is unprobed: it keeps the outer brackets it
                // was typed in.
                _ if self.spacing.is_padded() => {
                    format!("{name}[{open}{}{close}]", specifier_text(specifier))
                }
                _ => format!("{name}{}", specifier_text(specifier)),
            },
            (None, Some(TableReference::ColumnReference(column))) if prints_bare(column) => {
                format!("{name}[{open}{}{close}]", escaped_column(column))
            }
            (None, Some(reference)) => {
                format!("{name}[{open}{}{close}]", table_reference_text(reference))
            }
            (Some(specifier), Some(reference)) => format!(
                "{name}[{open}{}{comma}{}{close}]",
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

/// The `(column1, row1, column2, row2)` of an A1 range such as a table's `ref` (`A1:D6`).
///
/// # Errors
/// When `formula` is not a range.
pub fn parse_range(formula: &str) -> Result<(i32, i32, i32, i32), String> {
    let mut lexer = Lexer::new(
        formula,
        LexerMode::A1,
        #[allow(clippy::expect_used)]
        get_locale("en").expect(""),
        #[allow(clippy::expect_used)]
        get_language("en").expect(""),
    );
    if let TokenType::Range {
        left,
        right,
        sheet: _,
    } = lexer.next_token()
    {
        Ok((left.column, left.row, right.column, right.row))
    } else {
        Err("Not a range".to_string())
    }
}
