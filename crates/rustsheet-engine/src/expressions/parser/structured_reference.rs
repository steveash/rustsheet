//! A structured reference as written (`Sales[[#All],[Qty]]`), kept in the parsed formula next to
//! the area it resolves to (SPEC-0016).

use super::super::lexer::{Lexer, LexerMode};
use super::super::token::{TableReference, TableSpacing, TableSpecifier, TokenType};
use super::{Node, Parser};
use crate::expressions::types::CellReferenceRC;
use crate::language::get_language;
use crate::locale::get_locale;
use crate::table_types::Table;
use crate::types::sheet_names_match;

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
    /// Written without the table name in a cell of the table (`=[@Col]*2`), which names the
    /// containing table (references.md "Structured references: this row, totals and ranges", T-2979).
    pub unqualified: bool,
    /// The range of the table of a `#This Row` reference or of one written without the table
    /// name, as the table list the formula was parsed against had it: the printing cell decides
    /// by it whether the reference prints without the table name (T-3241). `None` for any other
    /// reference. Every change to the table list reparses the formulas, so it stays current.
    pub extent: Option<TableExtent>,
}

/// A table's whole range on its sheet.
#[derive(PartialEq, Clone, Debug)]
pub struct TableExtent {
    /// The table's sheet.
    pub sheet: String,
    /// The first and last columns.
    pub columns: (i32, i32),
    /// The first and last rows, headers and totals row included.
    pub rows: (i32, i32),
    /// The first and last data rows.
    pub data_rows: (i32, i32),
}

impl TableExtent {
    /// The range of `table`, if its `ref` is a range, with its data rows `data_rows` (the whole
    /// range when `None`).
    pub(super) fn of(table: &Table, data_rows: Option<(i32, i32)>) -> Option<Self> {
        let (column_start, row_start, column_end, row_end) = parse_range(&table.reference).ok()?;
        Some(Self {
            sheet: table.sheet_name.clone(),
            columns: (column_start, column_end),
            rows: (row_start, row_end),
            data_rows: data_rows.unwrap_or((row_start, row_end)),
        })
    }

    /// Whether `cell` lies in the range (sheet names match ignoring case), on a data row if
    /// `data_row`.
    fn holds(&self, cell: &CellReferenceRC, data_row: bool) -> bool {
        let rows = if data_row { self.data_rows } else { self.rows };
        sheet_names_match(&self.sheet, &cell.sheet)
            && (self.columns.0..=self.columns.1).contains(&cell.column)
            && (rows.0..=rows.1).contains(&cell.row)
    }
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
/// `calc_test_tables/tables.xlsx` Hidden!B6).
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
        self.text_with_name(!self.unqualified)
    }

    /// [`Self::text`] as printed in `cell`. A `#This Row` reference written with the table name
    /// prints in its `@` form without the name on a data row of its own table (Excel's
    /// FORMULATEXT of `=T[[#This Row],[Col]]` there is `=[@Col]`: probes/t2800-this-row-whitespace
    /// B3:E3, table-edits/table-insert-row-calc-column-inside-data F!A1:A2); how Excel prints it
    /// in the header or totals row is unprobed (T-3240). One written without the name prints
    /// without it anywhere in the table's range, where it parses, and with it elsewhere, so a
    /// copy, fill or move out of the table, or a resize that leaves the cell outside, gives text
    /// that parses there (T-3241).
    pub(crate) fn text_at(&self, cell: &CellReferenceRC) -> String {
        match &self.extent {
            Some(extent) => self.text_with_name(!extent.holds(cell, !self.unqualified)),
            None => self.text(),
        }
    }

    /// [`Self::text`], with the table name if `named`, without it otherwise.
    fn text_with_name(&self, named: bool) -> String {
        let qualified = self.qualified_text();
        if named {
            return qualified;
        }
        // Excel shows a `#This Row` reference in its `@` form (references.md "Calculated
        // columns" rule 6: FORMULATEXT prints `=[@Col]*2` in every filled row), keeping the
        // padding inside the outer brackets: `T[ [#This Row] , [Col] ]` prints `[ @Col ]`
        // (syntax.md, probes/t2800-this-row-whitespace, T-3069).
        if self.specifier == Some(TableSpecifier::ThisRow) {
            let TableSpacing { open, close, .. } = &self.spacing;
            let inner = match &self.table_reference {
                None => String::new(),
                Some(TableReference::ColumnReference(column)) if prints_bare(column) => {
                    escaped_column(column)
                }
                Some(reference) => table_reference_text(reference),
            };
            return format!("[{open}@{inner}{close}]");
        }
        qualified[self.table_name.len()..].to_string()
    }

    /// The text kept in storage (R1C1), which the parser reads back with no table to infer the
    /// name from: an unqualified reference keeps its table as a `[T]` prefix (`[T][[#This
    /// Row],[Col]]`), so two tables on one sheet never share `[@Col]*2` (T-2979). Any other
    /// reference is [`Self::text`].
    pub(crate) fn stored_text(&self) -> String {
        if self.unqualified {
            let qualified = self.qualified_text();
            format!(
                "[{}]{}",
                self.table_name,
                &qualified[self.table_name.len()..]
            )
        } else {
            self.text()
        }
    }

    /// [`Self::text`] with the table name, whether or not it was written.
    fn qualified_text(&self) -> String {
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
            // A specifier pair keeps its outer brackets, and so does a lone specifier typed with
            // padding inside them (unprobed).
            (Some(specifier), None)
                if self.spacing.is_padded()
                    || matches!(
                        specifier,
                        TableSpecifier::HeadersData | TableSpecifier::DataTotals
                    ) =>
            {
                format!("{name}[{open}{}{close}]", specifier_text(specifier))
            }
            (Some(specifier), None) => format!("{name}{}", specifier_text(specifier)),
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
    ///
    /// A single column in padded brackets keeps its own (`T[ [Col] ]`, not the printed
    /// `T[ Col ]`): Excel loads a stored padded bare column as `#REF!` (syntax.md,
    /// probes/t2800-stored-ws-0, T-3069).
    pub(crate) fn file_text(&self) -> String {
        if self.bare {
            return format!("{}[]", self.table_name);
        }
        let TableSpacing { open, close, .. } = &self.spacing;
        match (&self.specifier, &self.table_reference) {
            (None, Some(TableReference::ColumnReference(column))) if self.spacing.is_padded() => {
                format!("{}[{open}{}{close}]", self.table_name, column_text(column))
            }
            _ => self.qualified_text(),
        }
    }
}

impl Parser {
    /// An operand starting with `[` (the lexer's `LeftBracket`, just read): a structured
    /// reference written without its table name, which names the table holding the formula's
    /// cell, its totals row included (`=[@Col]*2`, `=SUBTOTAL(109,[Col])`; references.md
    /// "Structured references: this row, totals and ranges", T-2979). Stored (R1C1) text carries
    /// the table as a `[T]` prefix (`StructuredReference::stored_text`). Outside every table the
    /// operand is refused, as before.
    pub(super) fn unqualified_structured_reference(&mut self) -> Node {
        let refused =
            |parser: &mut Self| parser.unexpected_operand("Unexpected token: '['".to_string());
        let table_name = if self.lexer.is_a1_mode() {
            let Some(name) = self.containing_table_name() else {
                return refused(self);
            };
            // Back onto the `[` just read (any whitespace before it is already behind).
            self.lexer
                .rewind_to(self.lexer.get_position_index().saturating_sub(1));
            name
        } else {
            let Some(name) = self.lexer.stored_table_prefix() else {
                return refused(self);
            };
            name
        };
        let Ok(TokenType::StructuredReference {
            table_name,
            specifier,
            table_reference,
            spacing,
        }) = self.lexer.consume_structured_reference(&table_name)
        else {
            return refused(self);
        };
        let mut node = self.structured_reference_operand(
            table_name,
            specifier,
            table_reference,
            spacing,
            false,
        );
        if let Node::StructuredReferenceKind { reference, .. } = &mut node {
            reference.unqualified = true;
            if reference.extent.is_none() {
                reference.extent = self
                    .table_named(&reference.table_name)
                    .and_then(|table| TableExtent::of(table, None));
            }
        }
        node
    }

    /// The name of the table whose range (headers and totals row included) holds the formula's
    /// cell, if any.
    fn containing_table_name(&self) -> Option<String> {
        let context = &self.context;
        self.tables.values().find_map(|table| {
            TableExtent::of(table, None)?
                .holds(context, false)
                .then(|| table.name.clone())
        })
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
