pub(super) mod bound_variables;
mod rename_defined_name;
mod sheet_edits;
mod structured;
mod wrapped;
mod wrong_reference;
use std::fmt::Write;

use super::reference_operators::{
    at_text, callee_operand, intersection_text, spill_operand, tight_operand, union_text,
};
use super::structured_reference::is_unresolved_structured_reference;
use super::{
    super::utils::{name_needs_quoting, quote_name},
    left_chain,
    move_formula::{moved_reference_text, MoveContext},
    value_operands, ArrayNode, Node, Reference,
};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::static_analysis::remove_redundant_implicit_intersection;
use crate::expressions::token::OpUnary;
use crate::language::{get_default_language, Language};
use crate::locale::{get_default_locale, Locale};
use crate::{expressions::types::CellReferenceRC, number_format::to_excel_precision_str};
use bound_variables::{callee_text, prefix_bound_variables, variable_text};
pub(crate) use rename_defined_name::rename_defined_name_in_node;
pub(crate) use sheet_edits::{
    move_sheet_in_node, rename_sheet_in_node, retarget_unqualified_references,
};
pub(crate) use structured::{
    to_localized_string_this_row_lowered, to_rc_lowering_at, to_string_lowering,
    to_string_with_ref_errors,
};
use wrapped::wrapped_text;

pub enum DisplaceData {
    Column {
        sheet: u32,
        column: i32,
        delta: i32,
    },
    Row {
        sheet: u32,
        row: i32,
        delta: i32,
    },
    CellHorizontal {
        sheet: u32,
        row: i32,
        column: i32,
        delta: i32,
    },
    CellVertical {
        sheet: u32,
        row: i32,
        column: i32,
        delta: i32,
    },
    RowMove {
        sheet: u32,
        row: i32,
        delta: i32,
    },
    ColumnMove {
        sheet: u32,
        column: i32,
        delta: i32,
    },
    None,
}

/// A structural edit as the printer applies it (SPEC-0016 "Structural edits").
#[derive(Clone, Copy)]
pub(crate) struct Displacement<'a> {
    /// The displacement.
    pub(crate) data: &'a DisplaceData,
    /// The lowercase names of the tables that follow the edit, whose structured references keep
    /// their text (T-2297).
    pub(crate) kept_tables: &'a [String],
    /// The data blocks of the tables that grow with a row insert at their first data row or at
    /// their totals row (T-2321).
    pub(crate) grown_edges: &'a [DataBlock],
    /// The table columns a column delete removes, as lowercase (table, column) names: a
    /// structured reference to one of them prints `#REF!` (T-2339).
    pub(crate) deleted_columns: &'a [(String, String)],
}

/// The data rows `rows` and columns `columns` of a table on `sheet`, before the edit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataBlock {
    pub(crate) sheet: u32,
    pub(crate) rows: (i32, i32),
    pub(crate) columns: (i32, i32),
}

impl Displacement<'_> {
    /// A displacement that no table follows.
    const fn plain(data: &DisplaceData) -> Displacement<'_> {
        Displacement {
            data,
            kept_tables: &[],
            grown_edges: &[],
            deleted_columns: &[],
        }
    }

    /// The displacements of the two corners of the plain range `rows` x `columns` on sheet
    /// `sheet_index` where a row edit does not displace them one by one; `None` for any other
    /// range, whose corners displace as usual.
    ///
    /// A row delete that takes a range's first or last rows, but not all of them, shrinks it to
    /// the rows left (plain-range-delete-row-first and -last A1:A4, excel-graph, T-2338:
    /// `=SUM(S!C4:C7)` and `=SUM(S!$C$4:$C$7)` become `S!C4:C6` either way), where displacing the
    /// deleted corner alone would give `#REF!`. Only one-row deletes are probed; a wider delete
    /// across an edge shrinks the same way (hand, T-2851).
    ///
    /// Quirk (T-2197, probes/table-edits): a range in the data block of a table that grows with a
    /// row insert at an edge, and touching that edge, grows with the table, where an ordinary
    /// insert at a range's first row shifts it and one just below it leaves it. `=SUM(S!C4:C7)`
    /// onto data rows 4-7 becomes `=SUM(S!C4:C8)` both for the insert above row 4
    /// (table-insert-row-above-first-data A7) and above the totals row 8
    /// (table-insert-row-above-totals A7). Only that exact range is probed; a range inside the
    /// block touching the edge is assumed to grow the same way (hand, unprobed: T-2846).
    fn range_corners(
        &self,
        sheet_index: u32,
        rows: (i32, i32),
        columns: (i32, i32),
    ) -> Option<(DisplaceData, DisplaceData)> {
        let DisplaceData::Row { sheet, row, delta } = *self.data else {
            return None;
        };
        if sheet != sheet_index {
            return None;
        }
        // The corner on row `from` moved to row `to`.
        let shift_to = |from: i32, to: i32| DisplaceData::Row {
            sheet,
            row: from.min(to),
            delta: to - from,
        };
        if delta < 0 {
            let deleted = row..=row - delta - 1;
            return if deleted.contains(&rows.0) && !deleted.contains(&rows.1) {
                Some((shift_to(rows.0, row), shift_to(rows.1, rows.1 + delta)))
            } else if !deleted.contains(&rows.0) && deleted.contains(&rows.1) {
                Some((DisplaceData::None, shift_to(rows.1, row - 1)))
            } else {
                None
            };
        }
        let block = self.grown_edges.iter().find(|block| {
            block.sheet == sheet_index
                && block.rows.0 <= rows.0
                && rows.1 <= block.rows.1
                && block.columns.0 <= columns.0
                && columns.1 <= block.columns.1
        })?;
        // The first row stays (at or above the insert) and the last grows by `delta`.
        let at_edge = (row == block.rows.0 && rows.0 == row)
            || (row == block.rows.1 + 1 && rows.1 == block.rows.1);
        at_edge.then(|| (DisplaceData::None, shift_to(rows.1, rows.1 + delta)))
    }
}

/// How the printer rewrites the references of a formula: displaced by a structural edit, or
/// moved by a cut and paste. Everything else prints the same either way, so a moved formula
/// has the text FORMULATEXT shows (SPEC-0007 step 5).
enum Rewrite<'a> {
    /// Displaced by a structural edit; the structured references to the tables named here
    /// (lowercase) keep their text, since those tables move, grow or shrink with the edit
    /// (SPEC-0016 "Structural edits", T-2297).
    Displace(Displacement<'a>),
    Move(&'a MoveContext<'a>),
    /// Prints the area of every structured reference to one of these tables (lowercase names)
    /// instead of its text: the tables of a sheet being deleted (SPEC-0016 "Sheet rename, delete
    /// and restore"). A `#This Row` area is resolved at the cell's row, or at the given row when
    /// the formula is printed in R1C1 for one cell (T-1699).
    Lower(&'a [String], Option<i32>),
    /// Prints every `#This Row` as its area on the printing cell's row (T-1699).
    LowerThisRow,
    /// Prints `#REF!` for every structured reference to one of these tables (lowercase names):
    /// the tables a table or sheet delete removes, as Excel rewrites them (T-2340).
    RefError(&'a [String]),
}

/// Whether an R1C1 range is a whole row (`1:1`, absolute rows 1 to the last) and a whole column
/// (`A:A`): open ranges such as SUM(A:A) are stored as normal ranges (`R1C[0]:R1048576C[0]`), so
/// the A1 text strips the corners they do not print.
pub(crate) fn open_range_flags(
    rows: (bool, i32, bool, i32),
    columns: (bool, i32, bool, i32),
) -> (bool, bool) {
    let (absolute_row1, row1, absolute_row2, row2) = rows;
    let (absolute_column1, column1, absolute_column2, column2) = columns;
    (
        absolute_row1 && absolute_row2 && row1 == 1 && row2 == LAST_ROW,
        absolute_column1 && absolute_column2 && column1 == 1 && column2 == LAST_COLUMN,
    )
}

const NO_REWRITE: Rewrite<'static> = Rewrite::Displace(Displacement::plain(&DisplaceData::None));

/// This is the internal mode in IronCalc
/// Formulas internally are stored in R1C1 format, the locale and language are always "en"
pub fn to_rc_format(node: &Node) -> String {
    // A parse error is stored marked, so a re-parse in R1C1 does not read its A1 text (T-1805).
    super::unparsed::stored_text(node).unwrap_or_else(|| {
        let (locale, language) = (get_default_locale(), get_default_language());
        stringify(node, None, &NO_REWRITE, false, locale, language)
    })
}

pub fn to_english_string(node: &Node, context: &CellReferenceRC) -> String {
    let (locale, language) = (get_default_locale(), get_default_language());
    stringify(node, Some(context), &NO_REWRITE, false, locale, language)
}

/// This is the mode used to display the formula in the UI
pub fn to_localized_string(
    node: &Node,
    context: &CellReferenceRC,
    locale: &Locale,
    language: &Language,
) -> String {
    stringify(node, Some(context), &NO_REWRITE, false, locale, language)
}

/// This is the mode used to export the formula to Excel
/// Internally the locale and language are always "en"
pub fn to_excel_string(node: &Node, context: &CellReferenceRC) -> String {
    excel_string(node, context, false)
}

/// Like [`to_excel_string`], for the formula of an array, CSE or dynamic.
///
/// No implicit intersection is added to the formula of an array when it is
/// read, by Excel or by the import, so none of its `@` is redundant: they are
/// all written, as `_xlfn.SINGLE`.
pub fn to_excel_array_formula_string(node: &Node, context: &CellReferenceRC) -> String {
    excel_string(node, context, true)
}

fn excel_string(node: &Node, context: &CellReferenceRC, is_array_formula: bool) -> String {
    let (locale, language) = (get_default_locale(), get_default_language());
    // The internal representation stores every implicit intersection as `@`,
    // without the `automatic` flag. Drop the operators that Excel would re-insert
    // automatically on import; the rest are kept as `_xlfn.SINGLE` while
    // stringifying. See `remove_redundant_implicit_intersection`.
    let mut node = node.clone();
    if !is_array_formula {
        remove_redundant_implicit_intersection(&mut node, true);
    }
    prefix_bound_variables(&mut node);
    stringify(&node, Some(context), &NO_REWRITE, true, locale, language)
}

pub fn to_string_displaced(
    node: &Node,
    context: &CellReferenceRC,
    displace_data: &DisplaceData,
    locale: &Locale,
    language: &Language,
) -> String {
    let displacement = Displacement::plain(displace_data);
    to_string_displaced_keeping(node, context, displacement, locale, language)
}

/// [`to_string_displaced`] for a structural edit that moves, grows or shrinks the tables
/// `displacement.kept_tables` (lowercase names): a structured reference to one of them keeps its
/// text, and one to any other table prints the area it is displaced to where that area changes
/// (SPEC-0016 "Structural edits", T-2297).
pub(crate) fn to_string_displaced_keeping(
    node: &Node,
    context: &CellReferenceRC,
    displacement: Displacement,
    locale: &Locale,
    language: &Language,
) -> String {
    let rewrite = Rewrite::Displace(displacement);
    stringify(node, Some(context), &rewrite, false, locale, language)
}

/// The text of `node` cut from the cell `move_context` names and pasted by its deltas: the
/// references into the moved area follow it (see [`super::move_formula::move_formula`]).
pub(crate) fn to_string_moved(
    node: &Node,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    let context = CellReferenceRC {
        sheet: move_context.source_sheet_name.to_string(),
        column: move_context.column,
        row: move_context.row,
    };
    let rewrite = Rewrite::Move(move_context);
    stringify(node, Some(&context), &rewrite, false, locale, language)
}

/// The sheet prefix of a 3-D reference, `First:Last`, quoted as one name when either sheet
/// needs quotes (`'Data2:Q 4'!A1`); a one-sheet span is the plain name (probes/3d-ref-one-sheet-span-lift B6).
pub(crate) fn sheet_span_text(first: &str, last: &str, one_sheet: bool) -> String {
    let span = format!("{first}:{last}");
    match (
        one_sheet,
        name_needs_quoting(first) || name_needs_quoting(last),
    ) {
        (true, _) => quote_name(first),
        (false, false) => span,
        (false, true) => format!("'{}'", span.replace('\'', "''")),
    }
}

/// Converts a local reference to a string applying some displacement if needed.
/// It uses A1 style if context is not None. If context is None it uses R1C1 style
/// If `full_row` is true then the row details will be omitted in the A1 case
/// If `full_column` is true then column details will be omitted.
pub(crate) fn stringify_reference(
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    reference: &Reference,
    full_row: bool,
    full_column: bool,
) -> String {
    let sheet_name = reference.sheet_name;
    let sheet_index = reference.sheet_index;
    let absolute_row = reference.absolute_row;
    let absolute_column = reference.absolute_column;
    let row = reference.row;
    let column = reference.column;
    match context {
        Some(context) => {
            let mut row = if absolute_row { row } else { row + context.row };
            let mut column = if absolute_column {
                column
            } else {
                column + context.column
            };
            match displace_data {
                DisplaceData::Row {
                    sheet,
                    row: displace_row,
                    delta,
                } => {
                    if sheet_index == *sheet && !full_row {
                        if *delta < 0 {
                            if &row >= displace_row {
                                if row < displace_row - *delta {
                                    return "#REF!".to_string();
                                }
                                row += *delta;
                            }
                        } else if &row >= displace_row {
                            row += *delta;
                        }
                    }
                }
                DisplaceData::Column {
                    sheet,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && !full_column {
                        if *delta < 0 {
                            if &column >= displace_column {
                                if column < displace_column - *delta {
                                    return "#REF!".to_string();
                                }
                                column += *delta;
                            }
                        } else if &column >= displace_column {
                            column += *delta;
                        }
                    }
                }
                DisplaceData::CellHorizontal {
                    sheet,
                    row: displace_row,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && displace_row == &row {
                        if *delta < 0 {
                            if &column >= displace_column {
                                if column < displace_column - *delta {
                                    return "#REF!".to_string();
                                }
                                column += *delta;
                            }
                        } else if &column >= displace_column {
                            column += *delta;
                        }
                    }
                }
                DisplaceData::CellVertical {
                    sheet,
                    row: displace_row,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && displace_column == &column {
                        if *delta < 0 {
                            if &row >= displace_row {
                                if row < displace_row - *delta {
                                    return "#REF!".to_string();
                                }
                                row += *delta;
                            }
                        } else if &row >= displace_row {
                            row += *delta;
                        }
                    }
                }
                DisplaceData::RowMove {
                    sheet,
                    row: move_row,
                    delta,
                } => {
                    if sheet_index == *sheet {
                        if row == *move_row {
                            row += *delta;
                        } else if *delta > 0 {
                            // Moving the row downwards
                            if row > *move_row && row <= *move_row + *delta {
                                // Intermediate rows move up by one position
                                row -= 1;
                            }
                        } else if *delta < 0 {
                            // Moving the row upwards
                            if row < *move_row && row >= *move_row + *delta {
                                // Intermediate rows move down by one position
                                row += 1;
                            }
                        }
                    }
                }
                DisplaceData::ColumnMove {
                    sheet,
                    column: move_column,
                    delta,
                } => {
                    if sheet_index == *sheet {
                        if column == *move_column {
                            column += *delta;
                        } else if *delta > 0 {
                            // Moving the column to the right
                            if column > *move_column && column <= *move_column + *delta {
                                // Intermediate columns move left by one position
                                column -= 1;
                            }
                        } else if *delta < 0 {
                            // Moving the column to the left
                            if column < *move_column && column >= *move_column + *delta {
                                // Intermediate columns move right by one position
                                column += 1;
                            }
                        }
                    }
                }
                DisplaceData::None => {}
            }
            if row < 1 {
                return "#REF!".to_string();
            }
            let mut row_abs = if absolute_row {
                format!("${row}")
            } else {
                format!("{row}")
            };
            let column = match crate::expressions::utils::number_to_column(column) {
                Some(s) => s,
                None => return "#REF!".to_string(),
            };
            let mut col_abs = if absolute_column {
                format!("${column}")
            } else {
                column
            };
            if full_row {
                row_abs = "".to_string();
            }
            if full_column {
                col_abs = "".to_string();
            }
            match &sheet_name {
                Some(name) => {
                    format!("{}!{}{}", quote_name(name), col_abs, row_abs)
                }
                None => {
                    format!("{col_abs}{row_abs}")
                }
            }
        }
        None => {
            // Written into one buffer: the R1C1 text of every reference of every loaded formula
            // is printed here (T-0499).
            let mut text = String::with_capacity(24);
            if let Some(name) = sheet_name {
                text.push_str(&quote_name(name));
                text.push('!');
            }
            // Writing to a `String` cannot fail.
            let _ = if absolute_row {
                write!(text, "R{row}")
            } else {
                write!(text, "R[{row}]")
            };
            let _ = if absolute_column {
                write!(text, "C{column}")
            } else {
                write!(text, "C[{column}]")
            };
            text
        }
    }
}

/// The separator of function arguments, and of union operands, in `locale`: `,`, or `;`
/// where the decimal separator is not `.`.
pub(crate) fn argument_separator(locale: &Locale) -> &'static str {
    if locale.numbers.symbols.decimal == "." {
        ","
    } else {
        ";"
    }
}

/// `number` as a formula prints it, with the decimal separator of `locale`.
fn format_number_locale(number: f64, locale: &Locale) -> String {
    let s = to_excel_precision_str(number);
    let decimal = &locale.numbers.symbols.decimal;
    if decimal == "." {
        s
    } else {
        s.replace('.', decimal)
    }
}

/// The printed element `node` of an array constant.
fn to_string_array_node(node: &ArrayNode, locale: &Locale, language: &Language) -> String {
    match node {
        ArrayNode::Boolean(true) => language.booleans.r#true.to_uppercase(),
        ArrayNode::Boolean(false) => language.booleans.r#false.to_uppercase(),
        ArrayNode::Number(number) => format_number_locale(*number, locale),
        ArrayNode::String(value) => format!("\"{value}\""),
        ArrayNode::Error(kind) => format!("{kind}"),
        ArrayNode::Empty => "0".to_string(),
        ArrayNode::Lambda(_) => "#CALC!".to_string(), // never parsed: what a cell shows
        ArrayNode::Nested(_) => "#VALUE!".to_string(), // never parsed: what a cell shows
    }
}

fn format_function(
    name: &str,
    args: &Vec<Node>,
    context: Option<&CellReferenceRC>,
    rewrite: &Rewrite,
    export_to_excel: bool,
    locale: &Locale,
    language: &Language,
) -> String {
    // One buffer for the call: rebuilding the argument text at each argument copied it once
    // per argument, a hot spot on loading long formulas (T-0499).
    let arg_separator = argument_separator(locale);
    let mut text = String::with_capacity(name.len() + 2 + 16 * args.len());
    text.push_str(name);
    text.push('(');
    for (index, el) in args.iter().enumerate() {
        if index > 0 {
            text.push_str(arg_separator);
        }
        text.push_str(&stringify(
            el,
            context,
            rewrite,
            export_to_excel,
            locale,
            language,
        ));
    }
    text.push(')');
    text
}

// There is just one representation in the AST (Abstract Syntax Tree) of a formula.
// But three different ways to convert it to a string.
//
// To stringify a formula we need a "context", that is in which cell are we doing the "stringifying"
//
// But there are three ways to stringify a formula:
//
// * To show it to the IronCalc user
// * To store internally
// * To export to Excel
//
// There are, of course correspondingly three "modes" when parsing a formula.
//
// The internal representation is the more different as references are stored in the RC representation.
// The the AST of the formula is kept close to this representation we don't need a context
//
// In the export to Excel representation certain things are different:
// * We add a _xlfn. in front of some (more modern) functions
// * We remove the Implicit Intersection operator when it is automatic and add _xlfn.SINGLE when it is not
//
// Examples:
// * =A1+B2
// * =RC+R1C1
// * =A1+B1

fn stringify(
    node: &Node,
    context: Option<&CellReferenceRC>,
    rewrite: &Rewrite,
    export_to_excel: bool,
    locale: &Locale,
    language: &Language,
) -> String {
    use self::Node::*;
    let displace_data = match rewrite {
        Rewrite::Displace(displacement) => displacement.data,
        Rewrite::Move(move_context) => {
            if let Some(text) = moved_reference_text(node, move_context) {
                return text;
            }
            &DisplaceData::None
        }
        Rewrite::Lower(..) | Rewrite::LowerThisRow | Rewrite::RefError(_) => &DisplaceData::None,
    };
    let sub = |n: &Node| stringify(n, context, rewrite, export_to_excel, locale, language);
    match node {
        BooleanKind(value) => {
            if *value {
                language.booleans.r#true.to_string()
            } else {
                language.booleans.r#false.to_string()
            }
        }
        NumberKind(number) => format_number_locale(*number, locale),
        StringKind(value) => format!("\"{value}\""),
        WrongReferenceKind { .. } | WrongRangeKind { .. } => {
            wrong_reference::text(node, context, export_to_excel)
        }
        ReferenceKind {
            sheet_name,
            sheet_index,
            column,
            row,
            absolute_row,
            absolute_column,
        } => stringify_reference(
            context,
            displace_data,
            &Reference {
                sheet_name,
                sheet_index: *sheet_index,
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
            },
            false,
            false,
        ),
        RangeKind {
            sheet_name,
            sheet_index,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } => {
            let (full_row, full_column) = open_range_flags(
                (*absolute_row1, *row1, *absolute_row2, *row2),
                (*absolute_column1, *column1, *absolute_column2, *column2),
            );
            let grown = match (rewrite, context) {
                (Rewrite::Displace(displacement), Some(context)) if !full_row && !full_column => {
                    let at =
                        |absolute, value, origin| if absolute { value } else { value + origin };
                    displacement.range_corners(
                        *sheet_index,
                        (
                            at(*absolute_row1, *row1, context.row),
                            at(*absolute_row2, *row2, context.row),
                        ),
                        (
                            at(*absolute_column1, *column1, context.column),
                            at(*absolute_column2, *column2, context.column),
                        ),
                    )
                }
                _ => None,
            };
            let (displace1, displace2) = match &grown {
                Some((first, last)) => (first, last),
                None => (displace_data, displace_data),
            };
            let s1 = stringify_reference(
                context,
                displace1,
                &Reference {
                    sheet_name,
                    sheet_index: *sheet_index,
                    row: *row1,
                    column: *column1,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                context,
                displace2,
                &Reference {
                    sheet_name: &None,
                    sheet_index: *sheet_index,
                    row: *row2,
                    column: *column2,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                },
                full_row,
                full_column,
            );
            // Quirk (T-2855, probes/table-edits plain-range-delete-row-first F!A6, excel-graph):
            // a wholly deleted range prints one `S!#REF!`, not a `#REF!` per corner.
            if s1 == "#REF!" && s2 == "#REF!" {
                return wrong_reference::ref_error_text(sheet_name.as_deref());
            }
            format!("{s1}:{s2}")
        }
        // Rows and columns inserted or moved on one sheet of the span do not shift the span's
        // cells (edit semantics are not modelled yet), so no displacement reaches `reference`.
        SheetSpanKind {
            first_sheet,
            last_sheet,
            first_index,
            last_index,
            reference,
        } => format!(
            "{}!{}",
            sheet_span_text(first_sheet, last_sheet, first_index == last_index),
            stringify(
                reference,
                context,
                &NO_REWRITE,
                export_to_excel,
                locale,
                language
            )
        ),
        OpRangeKind { left, right } => {
            let operand = |n: &Node| tight_operand(sub(n), n);
            format!("{}:{}", operand(left), operand(right))
        }
        OpIntersectKind { left, right } => intersection_text(left, right, &sub),
        OpUnionKind { left, right, .. } => union_text(&sub(left), &sub(right), locale),
        OpConcatenateKind { .. }
        | CompareKind { .. }
        | OpSumKind { .. }
        | OpProductKind { .. }
        | OpPowerKind { .. } => {
            // A loop down the chain's left operands, not recursion (T-0128).
            let (links, first) = left_chain(node);
            let mut text = sub(first);
            for link in links {
                text = operator_text(link, text, &sub);
            }
            text
        }
        NamedFunctionKind { name, args, id: _ } => format_function(
            &callee_text(name, context),
            args,
            context,
            rewrite,
            export_to_excel,
            locale,
            language,
        ),
        FunctionKind { kind, args } => {
            let name = match kind.legacy_spelling() {
                // Only the R1C1 storage text (no context) shows a legacy N or T (SPEC-0012).
                Some(spelling) if context.is_none() => spelling.to_string(),
                _ if export_to_excel => kind.to_xlsx_string(),
                _ => kind.to_localized_name(language),
            };
            format_function(
                &name,
                args,
                context,
                rewrite,
                export_to_excel,
                locale,
                language,
            )
        }
        ArrayKind(args) => {
            let mut first_row = true;
            let mut matrix_string = String::new();
            let row_separator = if locale.numbers.symbols.decimal == "." {
                ';'
            } else {
                '/'
            };
            let col_separator = if row_separator == ';' { ',' } else { ';' };

            for row in args {
                if !first_row {
                    matrix_string.push(row_separator);
                } else {
                    first_row = false;
                }
                let mut first_column = true;
                let mut row_string = String::new();
                for el in row {
                    if !first_column {
                        row_string.push(col_separator);
                    } else {
                        first_column = false;
                    }
                    row_string.push_str(&to_string_array_node(el, locale, language));
                }
                matrix_string.push_str(&row_string);
            }
            format!("{{{matrix_string}}}")
        }
        DefinedNameKind((name, scope, _)) => crate::expressions::utils::defined_name_text(
            name,
            *scope,
            context.is_none() || export_to_excel,
        ),
        NamedVariableKind { name, id: _ } => variable_text(name, context),
        UnaryKind {
            kind: OpUnary::Minus,
            ..
        }
        | UnaryPlus { .. }
        | Group { .. }
        | Spaced { .. } => wrapped_text(node, sub),
        UnaryKind {
            kind: OpUnary::Percentage,
            right,
        } => format!("{}%", wrap(sub(right), binding(right) < binding(node))),
        ErrorKind(kind) => format!("{kind}"),
        SheetRefErrorKind { sheet_name } => wrong_reference::ref_error_text(Some(sheet_name)),
        ParseErrorKind { message, .. }
            if export_to_excel && is_unresolved_structured_reference(message) =>
        {
            "#REF!".to_string()
        }
        ParseErrorKind { formula, .. } => formula.to_string(),
        EmptyArgKind => "".to_string(),
        ThisRowKind { .. } => structured::this_row_text(node, context, rewrite, sub),
        StructuredReferenceKind { reference, lowered } => structured::reference_text(
            reference,
            lowered,
            context,
            rewrite,
            export_to_excel,
            (locale, language),
        ),
        SpillRangeOperator { child } => {
            if export_to_excel {
                return format!("_xlfn.ANCHORARRAY({})", sub(child));
            };
            format!("{}#", spill_operand(sub(child), child))
        }
        LambdaDefKind { parameters, body } => {
            let lambda_name = if export_to_excel {
                "_xlfn.LAMBDA"
            } else {
                "LAMBDA"
            };
            let arg_sep = argument_separator(locale);
            let mut parts: Vec<String> = parameters
                .iter()
                .map(|p| {
                    if export_to_excel {
                        if p.is_optional {
                            format!("_xlop.{}", p.name)
                        } else {
                            format!("_xlpm.{}", p.name)
                        }
                    } else if p.is_optional {
                        format!("[{}]", variable_text(&p.name, context))
                    } else {
                        variable_text(&p.name, context)
                    }
                })
                .collect();
            parts.push(sub(body));
            format!("{}({})", lambda_name, parts.join(arg_sep))
        }
        LambdaCallKind { lambda, args } => {
            // When the callee is a plain named variable (unknown identifier used as a function),
            // use lowercase to distinguish from real function calls in R1C1 format and
            // to match the display behaviour of InvalidFunctionKind.
            let lambda_str = match lambda.as_ref() {
                Node::NamedVariableKind { name, id: _ } => name.to_lowercase(),
                other => callee_operand(sub(other), other),
            };
            let call_args: Vec<String> = args.iter().map(sub).collect();
            format!(
                "{}({})",
                lambda_str,
                call_args.join(argument_separator(locale))
            )
        }
        ImplicitIntersection {
            automatic: _,
            child,
        } => {
            if export_to_excel {
                // Redundant operators have already been stripped by
                // `remove_redundant_implicit_intersection`; whatever remains is
                // meaningful and must be exported explicitly. A written group prints inside
                // SINGLE's own parentheses, `_xlfn.SINGLE((Z1+(Z2)))`, as Excel stores `=@(Z1+(Z2))`
                // (docs/excel-semantics/xlsx-storage.md, SPEC-0007).
                return format!("_xlfn.SINGLE({})", sub(child));
            }
            // The space binds tighter than `@`, so `@A1:A3 A2:B2` is `@(A1:A3 A2:B2)`; Excel's
            // FORMULATEXT prints it without parentheses (T-1098).
            at_text(child, &sub)
        }
    }
}

/// How tightly the printed `node` binds as an operand of a value operator, loosest first: the
/// parser's chain levels (`ChainLevel`), then `%`, then `-` (`-A1%` is `(-A1)%`, `-A1^2` is
/// `(-A1)^2`), then everything else (constants, references, reference operators, `@`, calls,
/// written groups), which never needs parentheses.
#[derive(PartialEq, PartialOrd)]
enum Binding {
    Compare,
    Concat,
    Sum,
    Product,
    Power,
    Percent,
    Minus,
    Operand,
}

fn binding(node: &Node) -> Binding {
    // Whitespace binds as the operand it is written around.
    match node.unwrap_spaced() {
        Node::CompareKind { .. } => Binding::Compare,
        Node::OpConcatenateKind { .. } => Binding::Concat,
        Node::OpSumKind { .. } => Binding::Sum,
        Node::OpProductKind { .. } => Binding::Product,
        Node::OpPowerKind { .. } => Binding::Power,
        Node::UnaryKind {
            kind: OpUnary::Percentage,
            ..
        } => Binding::Percent,
        Node::UnaryKind {
            kind: OpUnary::Minus,
            ..
        }
        | Node::UnaryPlus { .. } => Binding::Minus,
        _ => Binding::Operand,
    }
}

/// `text` in parentheses when `parens`.
fn wrap(text: String, parens: bool) -> String {
    if parens {
        format!("({text})")
    } else {
        text
    }
}

/// The text of value operator `node` (`+ - * / ^ &` or a comparison) whose left operand prints
/// as `left_text`: the operator and its right operand appended. The one precedence rule: an
/// operand gets parentheses only where the bare text would parse back to a different tree, a
/// looser left operand, or a right operand no tighter than `node` (the operators are left
/// associative). A parsed tree carries its written parentheses as [`Node::Group`]s, so it
/// prints as written: `=SUM(A1)^2`, `=2^-A1`, `=A1^2^3` (probes/operand-wrap-text,
/// probes/power-chain-text). One link of the loop that prints a chain (T-0128).
fn operator_text(node: &Node, left_text: String, sub: &dyn Fn(&Node) -> String) -> String {
    use self::Node::*;
    let Some((left, right)) = value_operands(node) else {
        return left_text;
    };
    let operator: &dyn std::fmt::Display = match node {
        OpConcatenateKind { .. } => &"&",
        CompareKind { kind, .. } => kind,
        OpSumKind { kind, .. } => kind,
        OpProductKind { kind, .. } => kind,
        OpPowerKind { .. } => &"^",
        _ => return left_text,
    };
    let level = binding(node);
    // Appended to the chain's text so far rather than copied into a new string at every link.
    let mut text = wrap(left_text, binding(left) < level);
    let _ = write!(text, "{operator}");
    text.push_str(&wrap(sub(right), binding(right) <= level));
    text
}
