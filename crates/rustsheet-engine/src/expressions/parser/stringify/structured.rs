//! Printing a structured reference (SPEC-0016 "Printing" and "Structural edits").

use super::{stringify, DisplaceData, Displacement, Rewrite, NO_REWRITE};
use crate::expressions::parser::move_formula::ref_is_in_area;
use crate::expressions::parser::{Node, StructuredReference};
use crate::expressions::token::{self, TableReference, TableSpecifier};
use crate::expressions::types::CellReferenceRC;
use crate::language::{get_default_language, Language};
use crate::locale::{get_default_locale, Locale};

/// The text of `node`, in R1C1 without `context` and in English A1 with it, with every structured
/// reference to one of `tables` (lowercase names) printed as the area it resolves to. Deleting
/// the sheet of those tables then turns them into `#REF!`, as it does any reference to the sheet.
pub(crate) fn to_string_lowering(
    node: &Node,
    context: Option<&CellReferenceRC>,
    tables: &[String],
) -> String {
    rewritten_text(node, context, &Rewrite::Lower(tables, None))
}

/// The text of `node` printed with `rewrite`, in R1C1 without `context` (a parse error keeps its
/// stored text) and in English A1 with it.
fn rewritten_text(node: &Node, context: Option<&CellReferenceRC>, rewrite: &Rewrite) -> String {
    crate::expressions::parser::unparsed::stored_text(node)
        .filter(|_| context.is_none())
        .unwrap_or_else(|| {
            let (locale, language) = (get_default_locale(), get_default_language());
            stringify(node, context, rewrite, false, locale, language)
        })
}

/// [`to_string_lowering`], but every structured reference to one of `tables` prints as `#REF!`:
/// the text a formula keeps after those tables, or their sheet, are deleted (T-2340,
/// references.md "Structured references under row deletes, column edits, convert, table and
/// sheet delete").
pub(crate) fn to_string_with_ref_errors(
    node: &Node,
    context: Option<&CellReferenceRC>,
    tables: &[String],
) -> String {
    rewritten_text(node, context, &Rewrite::RefError(tables))
}

/// [`to_string_lowering`] in R1C1 for the formula of one cell on `row`: a `#This Row` area to
/// one of `tables` is printed as the area it is on that row (`R3C1`, or `#VALUE!` off the data
/// rows), the text its parse-time lowering had (T-1699).
pub(crate) fn to_rc_lowering_at(node: &Node, row: i32, tables: &[String]) -> String {
    let (locale, language) = (get_default_locale(), get_default_language());
    let rewrite = Rewrite::Lower(tables, Some(row));
    stringify(node, None, &rewrite, false, locale, language)
}

/// [`super::to_localized_string`] with every `#This Row` structured reference printed as the
/// area it is on `context`'s row (`$A$3`, or `#VALUE!` off the data rows): the text of a formula
/// that a structural edit moves to another row, which then reads the row it read, as before
/// SPEC-0016 step 2 (T-1699).
pub(crate) fn to_localized_string_this_row_lowered(
    node: &Node,
    context: &CellReferenceRC,
    locale: &Locale,
    language: &Language,
) -> String {
    stringify(
        node,
        Some(context),
        &Rewrite::LowerThisRow,
        false,
        locale,
        language,
    )
}

/// The printed `Node::ThisRowKind`, which is printed only as a lowered area (a structural-edit
/// fallback): first resolved at the printing cell's row, which gives the text its
/// parse-time lowering had (`$A$3` or `#VALUE!`), then rewritten by `sub`; with no cell, as the
/// columns on the formula's own row (SPEC-0016 "Structural edits", T-1699).
pub(super) fn this_row_text(
    node: &Node,
    context: Option<&CellReferenceRC>,
    rewrite: &Rewrite,
    sub: impl Fn(&Node) -> String,
) -> String {
    let row = match (context, rewrite) {
        (Some(context), _) => Some(context.row),
        (None, Rewrite::Lower(_, row)) => *row,
        (None, _) => None,
    };
    let area = match row {
        Some(row) => node.this_row_at(row),
        None => node.this_row_relative(),
    };
    area.map_or_else(String::new, |area| sub(&area))
}

/// The printed `Node::StructuredReferenceKind`: its text as written (in its file form in the
/// export), or the area it resolves to where an edit changes that area.
pub(super) fn reference_text(
    reference: &StructuredReference,
    lowered: &Node,
    context: Option<&CellReferenceRC>,
    rewrite: &Rewrite,
    export_to_excel: bool,
    (locale, language): (&Locale, &Language),
) -> String {
    let print = |node: &Node, rewrite: &Rewrite| {
        stringify(node, context, rewrite, export_to_excel, locale, language)
    };
    // With no cell the text is the stored R1C1 one, which keeps an unqualified reference's table.
    let written = || {
        if context.is_none() {
            reference.stored_text()
        } else {
            reference.text()
        }
    };
    // The export writes every table as a table part, so the file keeps the reference, in the
    // form Excel stores (SPEC-0016 step 3, T-2196), `#Totals` of a table without a totals row
    // included (Excel keeps `SUM(Table2[#Totals])`, cached `#REF!`: calc_test_tables/tables.xlsx
    // Sheet2!F4). Any other reference that lowers to an error comes from table metadata that does
    // not fit (directive 5), and is written as that error, which it evaluates to.
    if export_to_excel {
        return match lowered {
            Node::ErrorKind(_) if reference.specifier != Some(TableSpecifier::Totals) => {
                print(lowered, rewrite)
            }
            _ => reference.file_text(),
        };
    }
    match rewrite {
        Rewrite::Displace(Displacement {
            data: DisplaceData::None,
            ..
        }) => return written(),
        // A `#This Row` area keeps today's rule below: whether Excel then reads the cell's row
        // of the moved table, or the row it read, is unprobed (T-2845).
        Rewrite::Displace(Displacement {
            kept_tables,
            deleted_columns,
            ..
        }) if !matches!(lowered, Node::ThisRowKind { .. })
            && kept_tables.contains(&reference.table_name.to_lowercase()) =>
        {
            // A column delete inside the table turns a reference to a deleted column into
            // `#REF!` (T-2324 table-delete-column-qty: `=SUM(#REF!)`, -region: `=COUNTA(#REF!)`).
            // A column range that loses one end is printed as the area it is displaced to below,
            // as under a fallback: what Excel does to it is unprobed.
            match deleted_column(reference, deleted_columns) {
                DeletedColumn::Whole => return token::Error::REF.to_string(),
                DeletedColumn::None => return written(),
                DeletedColumn::RangeEnd => {}
            }
        }
        Rewrite::Lower(tables, _) => {
            let name = reference.table_name.to_lowercase();
            return if tables.contains(&name) {
                print(lowered, rewrite)
            } else {
                written()
            };
        }
        Rewrite::RefError(tables) => {
            let name = reference.table_name.to_lowercase();
            return if tables.contains(&name) {
                token::Error::REF.to_string()
            } else {
                written()
            };
        }
        Rewrite::LowerThisRow => {
            return if matches!(lowered, Node::ThisRowKind { .. }) {
                print(lowered, &NO_REWRITE)
            } else {
                written()
            };
        }
        Rewrite::Displace(..) | Rewrite::Move(_) => {}
    }
    // A structural edit the table does not follow, or a cut and paste, that changes the area
    // prints the changed area, as before SPEC-0016 (the fallback of SPEC-0016 "Structural
    // edits"); otherwise the reference stays as written.
    // A `#This Row` area is printed resolved at the cell's row, so its text is kept only where
    // the cell keeps its row too; otherwise it prints the area it read, its text before SPEC-0016
    // step 2 (T-1699).
    let rewritten = print(lowered, rewrite);
    if rewritten == print(lowered, &NO_REWRITE) && !row_may_move(lowered, context, rewrite) {
        written()
    } else {
        rewritten
    }
}

/// How a column delete inside a table touches a structured reference to it.
enum DeletedColumn {
    /// It names no deleted column.
    None,
    /// It names one deleted column, or a range both of whose ends are deleted.
    Whole,
    /// It names a column range one of whose ends is deleted.
    RangeEnd,
}

/// How the columns `deleted` (lowercase (table, column) names) touch `reference`.
fn deleted_column(reference: &StructuredReference, deleted: &[(String, String)]) -> DeletedColumn {
    if deleted.is_empty() {
        return DeletedColumn::None;
    }
    let table = reference.table_name.to_lowercase();
    let is_deleted = |column: &str| {
        let column = column.to_lowercase();
        deleted.iter().any(|(t, c)| *t == table && *c == column)
    };
    match &reference.table_reference {
        Some(TableReference::ColumnReference(column)) if is_deleted(column) => DeletedColumn::Whole,
        Some(TableReference::RangeReference((first, last))) => {
            match (is_deleted(first), is_deleted(last)) {
                (true, true) => DeletedColumn::Whole,
                (false, false) => DeletedColumn::None,
                _ => DeletedColumn::RangeEnd,
            }
        }
        _ => DeletedColumn::None,
    }
}

/// Whether the cut and paste `rewrite` moves the formula cell `context` to another row, for a
/// `#This Row` area `lowered` (false for any other node and any other rewrite: a structural edit
/// re-enters the formulas it moves to another row with their `#This Row` areas lowered first,
/// `Model::moved_text`). The cell's sheet is the area's when the area names no sheet; when
/// it names one, the cell is on another sheet, which may be the cut one.
fn row_may_move(lowered: &Node, context: Option<&CellReferenceRC>, rewrite: &Rewrite) -> bool {
    let (
        Node::ThisRowKind {
            sheet_name,
            sheet_index,
            ..
        },
        Some(context),
        Rewrite::Move(moved),
    ) = (lowered, context, rewrite)
    else {
        return false;
    };
    let on_the_cut_sheet = match sheet_name {
        None => moved.area.sheet == *sheet_index,
        Some(_) => moved.area.sheet != *sheet_index,
    };
    moved.row_delta != 0
        && on_the_cut_sheet
        && ref_is_in_area(moved.area.sheet, context.row, context.column, moved.area)
}
