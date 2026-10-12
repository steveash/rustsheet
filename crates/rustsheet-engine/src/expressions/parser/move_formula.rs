use super::{
    stringify::{stringify_reference, to_string_moved, DisplaceData},
    Node, Reference,
};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::{Area, CellReferenceRC};
use crate::{language::Language, locale::Locale};

pub(crate) fn ref_is_in_area(sheet: u32, row: i32, column: i32, area: &Area) -> bool {
    if area.sheet != sheet {
        return false;
    }
    if row < area.row || row > area.row + area.height - 1 {
        return false;
    }
    if column < area.column || column > area.column + area.width - 1 {
        return false;
    }
    true
}

pub(crate) struct MoveContext<'a> {
    pub source_sheet_name: &'a str,
    pub row: i32,
    pub column: i32,
    pub area: &'a Area,
    pub target_sheet_name: &'a str,
    pub row_delta: i32,
    pub column_delta: i32,
}

/// This implements Excel's cut && paste
/// We are moving a formula in (row, column) to (`row+row_delta`, column + `column_delta`).
/// All references that do not point to a cell in area will be left untouched.
/// All references that point to a cell in area will be displaced
///
/// The text is `stringify`'s, so a moved formula prints exactly as FORMULATEXT does
/// (SPEC-0007 step 5); only its references are rewritten, by [`moved_reference_text`].
pub(crate) fn move_formula(
    node: &Node,
    move_context: &MoveContext,
    locale: &Locale,
    language: &Language,
) -> String {
    to_string_moved(node, move_context, locale, language)
}

/// The text of reference `node` moved by `move_context`, or `None` when `node` is not a
/// cell or range reference: every other node prints as `stringify` prints it.
pub(super) fn moved_reference_text(node: &Node, move_context: &MoveContext) -> Option<String> {
    use self::Node::*;
    let text = match node {
        ReferenceKind {
            sheet_name,
            sheet_index,
            absolute_row,
            absolute_column,
            row,
            column,
        } => {
            let reference_row = if *absolute_row {
                *row
            } else {
                row + move_context.row
            };
            let reference_column = if *absolute_column {
                *column
            } else {
                column + move_context.column
            };

            let new_row;
            let new_column;
            let mut ref_sheet_name = sheet_name;
            let source_sheet_name = &Some(move_context.source_sheet_name.to_string());

            if ref_is_in_area(
                *sheet_index,
                reference_row,
                reference_column,
                move_context.area,
            ) {
                // if the reference is in the area we are moving we want to displace the reference
                new_row = row + move_context.row_delta;
                new_column = column + move_context.column_delta;
            } else {
                // If the reference is not in the area we are moving the reference remains unchanged
                new_row = *row;
                new_column = *column;
                if move_context.target_sheet_name != move_context.source_sheet_name
                    && sheet_name.is_none()
                {
                    ref_sheet_name = source_sheet_name;
                }
            };
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name: ref_sheet_name,
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row,
                    absolute_column: *absolute_column,
                    row: new_row,
                    column: new_column,
                },
                false,
                false,
            )
        }
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
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);

            let reference_row1 = if *absolute_row1 {
                *row1
            } else {
                row1 + move_context.row
            };
            let reference_column1 = if *absolute_column1 {
                *column1
            } else {
                column1 + move_context.column
            };

            let reference_row2 = if *absolute_row2 {
                *row2
            } else {
                row2 + move_context.row
            };
            let reference_column2 = if *absolute_column2 {
                *column2
            } else {
                column2 + move_context.column
            };

            let new_row1;
            let new_column1;
            let new_row2;
            let new_column2;
            let mut ref_sheet_name = sheet_name;
            let source_sheet_name = &Some(move_context.source_sheet_name.to_string());
            if ref_is_in_area(
                *sheet_index,
                reference_row1,
                reference_column1,
                move_context.area,
            ) && ref_is_in_area(
                *sheet_index,
                reference_row2,
                reference_column2,
                move_context.area,
            ) {
                // if the whole range is inside the area we are moving we want to displace the context
                new_row1 = row1 + move_context.row_delta;
                new_column1 = column1 + move_context.column_delta;
                new_row2 = row2 + move_context.row_delta;
                new_column2 = column2 + move_context.column_delta;
            } else {
                // If the reference is not in the area we are moving the context remains unchanged
                new_row1 = *row1;
                new_column1 = *column1;
                new_row2 = *row2;
                new_column2 = *column2;
                if move_context.target_sheet_name != move_context.source_sheet_name
                    && sheet_name.is_none()
                {
                    ref_sheet_name = source_sheet_name;
                }
            };
            let context = CellReferenceRC {
                sheet: move_context.source_sheet_name.to_string(),
                column: move_context.column,
                row: move_context.row,
            };
            let s1 = stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name: ref_sheet_name,
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                    row: new_row1,
                    column: new_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                Some(&context),
                &DisplaceData::None,
                &Reference {
                    sheet_name: &None,
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                    row: new_row2,
                    column: new_column2,
                },
                full_row,
                full_column,
            );
            format!("{s1}:{s2}")
        }
        _ => return None,
    };
    Some(text)
}
