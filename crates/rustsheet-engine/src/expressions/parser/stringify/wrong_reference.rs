//! Printing a reference whose sheet the parser did not find (`WrongReferenceKind`,
//! `WrongRangeKind`): a deleted sheet, or an external one.

use super::{open_range_flags, stringify_reference, DisplaceData};
use crate::expressions::parser::{Node, Reference};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceRC;
use crate::expressions::utils::quote_name;

/// The sheet printed in place of a missing one: none, after a `#REF!` prefix.
const NO_SHEET: &Option<String> = &None;

/// A reference to a sheet that does not exist (a deleted one) is displayed as `#REF!` before the
/// cell or range, as Excel prints `=SUM(#REF!C4:C7)` for `S!C4:C7` once S is deleted (T-2340,
/// probes/table-edits/table-delete-sheet); the lexer reads that text back as `#REF!`. The stored
/// R1C1 text and the export keep the name, so an undo that restores the sheet restores the
/// reference. An external reference (`[1]Uses!A1`) is not a missing sheet.
fn is_missing_sheet(
    sheet_name: Option<&str>,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> bool {
    sheet_name.is_some_and(|name| !name.starts_with('[')) && context.is_some() && !export_to_excel
}

/// `#REF!` written for a deleted reference, after the sheet prefix Excel keeps (`S!#REF!`) when
/// there is one (references.md "A sheet-qualified #REF! is #REF!", T-2855).
pub(super) fn ref_error_text(sheet_name: Option<&str>) -> String {
    match sheet_name {
        Some(name) => format!("{}!{}", quote_name(name), Error::REF),
        None => Error::REF.to_string(),
    }
}

/// The text of a `WrongReferenceKind` or `WrongRangeKind` node (empty for any other node).
pub(super) fn text(
    node: &Node,
    context: Option<&CellReferenceRC>,
    export_to_excel: bool,
) -> String {
    let corner = |sheet_name, row, column, absolute_row, absolute_column, full_row, full_column| {
        stringify_reference(
            context,
            &DisplaceData::None,
            &Reference {
                sheet_name,
                sheet_index: 0, // HACK
                row,
                column,
                absolute_row,
                absolute_column,
            },
            full_row,
            full_column,
        )
    };
    let missing = |sheet_name: &Option<String>| {
        is_missing_sheet(sheet_name.as_deref(), context, export_to_excel)
    };
    match node {
        Node::WrongReferenceKind {
            sheet_name,
            column,
            row,
            absolute_row,
            absolute_column,
        } => {
            let (prefix, sheet_name) = if missing(sheet_name) {
                ("#REF!", NO_SHEET)
            } else {
                ("", sheet_name)
            };
            let text = corner(
                sheet_name,
                *row,
                *column,
                *absolute_row,
                *absolute_column,
                false,
                false,
            );
            format!("{prefix}{text}")
        }
        Node::WrongRangeKind {
            sheet_name,
            row1,
            column1,
            absolute_row1,
            absolute_column1,
            row2,
            column2,
            absolute_row2,
            absolute_column2,
        } => {
            let (full_row, full_column) = open_range_flags(
                (*absolute_row1, *row1, *absolute_row2, *row2),
                (*absolute_column1, *column1, *absolute_column2, *column2),
            );
            let (prefix, sheet_name) = if missing(sheet_name) {
                ("#REF!", NO_SHEET)
            } else {
                ("", sheet_name)
            };
            let s1 = corner(
                sheet_name,
                *row1,
                *column1,
                *absolute_row1,
                *absolute_column1,
                full_row,
                full_column,
            );
            let s2 = corner(
                &None,
                *row2,
                *column2,
                *absolute_row2,
                *absolute_column2,
                full_row,
                full_column,
            );
            format!("{prefix}{s1}:{s2}")
        }
        _ => String::new(),
    }
}
