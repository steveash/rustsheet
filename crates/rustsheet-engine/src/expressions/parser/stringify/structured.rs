//! Printing a structured reference (SPEC-0016 "Printing" and "Structural edits").

use super::{stringify, DisplaceData, Rewrite, NO_REWRITE};
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
        Rewrite::Displace(_) | Rewrite::Move(_) => {}
    }
    // A structural edit or a cut and paste that changes the area prints the changed area, as
    // before SPEC-0016, since the table itself does not move (the fallback of SPEC-0016
    // "Structural edits"); otherwise the reference stays as written.
    let rewritten = print(lowered, rewrite);
    if rewritten == print(lowered, &NO_REWRITE) {
        reference.text()
    } else {
        rewritten
    }
}
