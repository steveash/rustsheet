//! Printing a structured reference (SPEC-0016 "Printing" and "Structural edits").

use super::{stringify, DisplaceData, Rewrite, NO_REWRITE};
use crate::expressions::parser::move_formula::ref_is_in_area;
use crate::expressions::parser::{Node, StructuredReference};
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
    crate::expressions::parser::unparsed::stored_text(node)
        .filter(|_| context.is_none())
        .unwrap_or_else(|| {
            let (locale, language) = (get_default_locale(), get_default_language());
            stringify(
                node,
                context,
                &Rewrite::Lower(tables),
                false,
                locale,
                language,
            )
        })
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

/// The printed `Node::StructuredReferenceKind`: its text as written, or the area it resolves to
/// in the export and where an edit changes that area.
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
    // The export prints the area until it writes table parts (SPEC-0016 step 3, T-2196).
    if export_to_excel {
        return print(lowered, rewrite);
    }
    match rewrite {
        Rewrite::Displace(DisplaceData::None) => return reference.text(),
        Rewrite::Lower(tables) => {
            let name = reference.table_name.to_lowercase();
            return if tables.contains(&name) {
                print(lowered, rewrite)
            } else {
                reference.text()
            };
        }
        Rewrite::LowerThisRow => {
            return if matches!(lowered, Node::ThisRowKind { .. }) {
                print(lowered, &NO_REWRITE)
            } else {
                reference.text()
            };
        }
        Rewrite::Displace(_) | Rewrite::Move(_) => {}
    }
    // A structural edit or a cut and paste that changes the area prints the changed area, as
    // before SPEC-0016, since the table itself does not move (the fallback of SPEC-0016
    // "Structural edits"); otherwise the reference stays as written.
    // A `#This Row` area is printed resolved at the cell's row, so its text is kept only where
    // the cell keeps its row too; otherwise it prints the area it read, its text before SPEC-0016
    // step 2 (T-1699).
    let rewritten = print(lowered, rewrite);
    if rewritten == print(lowered, &NO_REWRITE) && !row_may_move(lowered, context, rewrite) {
        reference.text()
    } else {
        rewritten
    }
}

/// Whether the cut and paste `rewrite` moves the formula cell `context` to another row, for a
/// `#This Row` area `lowered` (false for any other node and any other rewrite: a structural edit
/// re-enters the formulas it moves to another row with their `#This Row` areas lowered first,
/// `Model::cell_formula_text`). The cell's sheet is the area's when the area names no sheet; when
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
