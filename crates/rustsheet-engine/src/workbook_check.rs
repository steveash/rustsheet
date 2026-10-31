//! The index check a decoded [`Workbook`] passes before a [`crate::Model`] is built from it.
//!
//! A workbook decoded from bytes is untrusted input (CLAUDE.md directive 5): its indices are
//! whatever the bytes say, and the model reads many of them unchecked, so a dangling one would
//! panic long after loading (T-2882: a cell style of 9999 panics at the next edit, a formula
//! index with no shared formula at the next evaluation). `Model::from_workbook` runs this check
//! first and refuses such a workbook instead.

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::utils::{is_valid_block, is_valid_column_number, is_valid_row};
use crate::limits::check_array_cells;
use crate::types::{Cell, MergedCell, Row, Styles, Workbook, Worksheet};

/// Checks every index of `workbook` that a later read trusts unchecked, in one pass linear in its
/// size. Each invariant names the read that panics without it:
///
/// - styles: `cell_xfs` is not empty (a new cell takes style 0, and `style_is_quote_prefix`
///   indexes `cell_xfs` unchecked); the font, fill and border ids of every `cell_xfs` and
///   `cell_style_xfs` entry are in range (`get_style`, `get_style_by_name` index them); number
///   format ids are not negative (`get_num_fmt` indexes the built-in formats by any id below
///   their count);
/// - worksheets: the storage of the cells (`SheetData::check_layout`: reads index a chunk's rows
///   by the row number); every cell's style (`style_is_quote_prefix`); a formula's index into the
///   sheet's shared formulas (evaluation indexes the parsed formulas by it); an array formula's
///   spill range and a spill cell's anchor within the grid (`anchor + height` and
///   `anchor + width` are computed in `i32`, `actions.rs`);
/// - row records within the grid with a style in range, and column records a range within the
///   grid with a style in range (a row or column style becomes a new cell's style, and row and
///   column inserts shift the records by `+ count` in `i32`); merged ranges at least one cell and
///   within the grid (`merged_cells.rs` walks `row..row + height`).
///
/// Indices read through a checked lookup that tolerates a dangling value are not checked, so a
/// workbook that loads today keeps loading: a shared-string index (it reads as empty text), a
/// conditional format's dxf id, a named style's `xf_id`, a defined name's sheet id, the selected
/// sheet of a view, the frozen panes, and a workbook with no sheets.
pub(crate) fn check_indices(workbook: &Workbook) -> Result<(), String> {
    check_styles(&workbook.styles)?;
    let styles = workbook.styles.cell_xfs.len();
    for worksheet in &workbook.worksheets {
        check_worksheet(worksheet, styles)
            .map_err(|e| format!("Sheet '{}': {e}", worksheet.name))?;
    }
    Ok(())
}

/// Whether `index` is a position in a list of `len` entries.
fn in_range(index: i32, len: usize) -> bool {
    usize::try_from(index).is_ok_and(|index| index < len)
}

fn check_styles(styles: &Styles) -> Result<(), String> {
    if styles.cell_xfs.is_empty() {
        return Err("The workbook has no cell styles".to_string());
    }
    let components = styles
        .cell_xfs
        .iter()
        .map(|xf| (xf.num_fmt_id, xf.font_id, xf.fill_id, xf.border_id))
        .chain(
            styles
                .cell_style_xfs
                .iter()
                .map(|xf| (xf.num_fmt_id, xf.font_id, xf.fill_id, xf.border_id)),
        );
    for (num_fmt_id, font_id, fill_id, border_id) in components {
        if num_fmt_id < 0 {
            return Err(format!("A style has the number format id {num_fmt_id}"));
        }
        if !in_range(font_id, styles.fonts.len()) {
            return Err(format!(
                "A style points to font {font_id}, which does not exist"
            ));
        }
        if !in_range(fill_id, styles.fills.len()) {
            return Err(format!(
                "A style points to fill {fill_id}, which does not exist"
            ));
        }
        if !in_range(border_id, styles.borders.len()) {
            return Err(format!(
                "A style points to border {border_id}, which does not exist"
            ));
        }
    }
    Ok(())
}

/// Checks one worksheet of a workbook whose styles have `styles` cell formats: what
/// [`check_indices`] checks of a sheet. An external diff that restores a deleted sheet
/// passes it too (T-2896).
pub(crate) fn check_worksheet(worksheet: &Worksheet, styles: usize) -> Result<(), String> {
    worksheet.sheet_data.check_layout()?;
    let formulas = worksheet.shared_formulas.len();
    for (row, column, cell) in worksheet.sheet_data.cells() {
        check_cell(row, column, cell, styles, formulas)?;
    }
    for record in &worksheet.rows {
        check_row_record(record, styles)?;
    }
    for record in &worksheet.cols {
        if !is_valid_column_number(record.min)
            || !is_valid_column_number(record.max)
            || record.min > record.max
            || record.style.is_some_and(|style| !in_range(style, styles))
        {
            return Err(format!(
                "the record of columns {}:{} is outside the sheet or has no style",
                record.min, record.max
            ));
        }
    }
    check_merged_cells(&worksheet.merged_cells)
}

/// Checks `cell`, to be stored at (`row`, `column`) of a sheet with `formulas` shared formulas in
/// a workbook with `styles` cell formats: the place is in the grid, the style and the formula
/// exist, and an array formula's spill range and a spill cell's anchor are in the grid.
pub(crate) fn check_cell(
    row: i32,
    column: i32,
    cell: &Cell,
    styles: usize,
    formulas: usize,
) -> Result<(), String> {
    if !is_valid_row(row) || !is_valid_column_number(column) {
        return Err(format!("cell R{row}C{column} is outside the sheet"));
    }
    let style = match *cell {
        Cell::EmptyCell { s }
        | Cell::BooleanCell { s, .. }
        | Cell::NumberCell { s, .. }
        | Cell::ErrorCell { s, .. }
        | Cell::SharedString { s, .. } => s,
        Cell::CellFormula { f, .. } | Cell::ArrayFormula { f, .. } if !in_range(f, formulas) => {
            return Err(format!(
                "cell R{row}C{column} points to formula {f}, which does not exist"
            ));
        }
        Cell::CellFormula { s, .. } => s,
        Cell::ArrayFormula {
            s,
            r: (width, height),
            ..
        } => {
            // An anchor's spill range may be empty, but it may not leave the grid, nor hold more
            // cells than any array may (evaluation fills a CSE anchor's whole range, and the clears
            // walk it; T-2896).
            if !(0..=LAST_COLUMN - column + 1).contains(&width)
                || !(0..=LAST_ROW - row + 1).contains(&height)
            {
                return Err(format!("cell R{row}C{column} spills outside the sheet"));
            }
            let cells = |n: i32| usize::try_from(n).unwrap_or(usize::MAX);
            if check_array_cells(cells(height), cells(width)).is_some() {
                return Err(format!("cell R{row}C{column} spills over too many cells"));
            }
            s
        }
        Cell::SpillCell {
            s,
            a: (anchor_row, anchor_column),
            ..
        } => {
            if !is_valid_row(anchor_row) || !is_valid_column_number(anchor_column) {
                return Err(format!(
                    "cell R{row}C{column} spills from outside the sheet"
                ));
            }
            s
        }
    };
    check_style(style, styles).map_err(|e| format!("cell R{row}C{column} {e}"))
}

/// Checks that `style` is one of the `styles` cell formats of the workbook.
pub(crate) fn check_style(style: i32, styles: usize) -> Result<(), String> {
    if in_range(style, styles) {
        Ok(())
    } else {
        Err(format!("has style {style}, which does not exist"))
    }
}

/// Checks a row record: the row is in the grid and its style exists.
pub(crate) fn check_row_record(record: &Row, styles: usize) -> Result<(), String> {
    if is_valid_row(record.r) && in_range(record.s, styles) {
        Ok(())
    } else {
        Err(format!(
            "the record of row {} is outside the sheet or has no style",
            record.r
        ))
    }
}

/// Checks that every merged range is at least one cell and lies within the grid.
pub(crate) fn check_merged_cells(merged_cells: &[MergedCell]) -> Result<(), String> {
    match merged_cells
        .iter()
        .find(|merged| !is_valid_block(merged.row, merged.column, merged.width, merged.height))
    {
        Some(merged) => Err(format!(
            "the merged range at R{}C{} is outside the sheet",
            merged.row, merged.column
        )),
        None => Ok(()),
    }
}
