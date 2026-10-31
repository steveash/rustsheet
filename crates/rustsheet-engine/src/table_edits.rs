//! Tables that follow a row or column insert or delete (SPEC-0016 "Structural edits", T-2297):
//! an edit wholly before a table moves it, a row insert below its header row and above or at its
//! totals row grows it (T-2321), a row delete inside its data rows that leaves one shrinks it and
//! one of its totals row drops that row (T-2338), so the structured references to it keep their
//! text. Every other edit that crosses a table leaves it
//! as it is (the *fallback*): its references print the displaced area, as before SPEC-0016.

use std::collections::HashSet;

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::parse_range;
use crate::expressions::parser::stringify::{DataBlock, DisplaceData, Displacement};
use crate::expressions::types::Area;
use crate::expressions::utils::number_to_column;
use crate::model::Model;
use crate::types::{FilterColumn, Table, TableColumn};

/// What a structural edit does to the tables of the edited sheet.
#[derive(Debug, Default, PartialEq)]
struct TableEdits {
    /// The lowercase names of the tables whose structured references keep their text: those
    /// that move, grow or shrink with the edit, and those it leaves alone.
    kept: Vec<String>,
    /// The tables that move, grow or shrink, by name.
    resized: Vec<(String, Resized)>,
    /// The table columns a column delete removes, as lowercase (table, column) names: the
    /// structured references to them print `#REF!` (T-2339).
    deleted_columns: Vec<(String, String)>,
    /// The data blocks of the tables that grow with a row insert at their first data row or at
    /// their totals row, onto which plain ranges grow too (T-2321).
    grown_edges: Vec<DataBlock>,
}

/// How an edit changes the rows (or columns) `first..=last` of a table.
#[derive(Debug, PartialEq)]
enum Span {
    /// The edit leaves them where they are.
    Unchanged,
    /// The new first and last row (or column).
    Changed(i32, i32),
    /// The table's totals row is deleted: the table keeps its other rows and has no totals row.
    DropTotals,
    /// The table cannot follow the edit.
    Fallback,
}

/// The rows a row insert (`delta > 0` rows above `at`) or delete (rows `at..at - delta`) gives
/// the table on rows `top..=bottom` with data rows `data_first..=data_last`, by SPEC-0016's
/// edge-case table. An insert below the top row and at or above the bottom row grows the table,
/// so one at the first data row under a header row or at the totals row does too (T-2197
/// table-insert-row-above-first-data and -above-totals: `ROWS(Sales)` 5). An insert at the first
/// row of a table without a header row is a fallback (unprobed, T-2845). A delete inside the data
/// rows, at their edges too, shrinks the table if a data row is left, and one of exactly the totals
/// row drops it (T-2324 table-delete-row-first-data, -last-data, -last-data-totals, -totals; only
/// one-row deletes are probed, a wider one reaching a data edge is hand, T-2848). The first row of
/// a table without a header row is a fallback, as for inserts (T-2845). A
/// delete touching the header row is a fallback (Excel refuses it), and so is one of every data
/// row (`Model::last_data_row` handles the one-row case) or one straddling the table's
/// edges (unprobed).
fn row_span(
    (top, bottom): (i32, i32),
    (data_first, data_last): (i32, i32),
    at: i32,
    delta: i32,
) -> Span {
    if delta > 0 {
        if at > bottom {
            Span::Unchanged
        } else if bottom > LAST_ROW - delta {
            // The insert is allowed (the cells pushed off are empty), the table cannot follow it:
            // an engine decision under directive 5, Excel's answer is unprobed.
            Span::Fallback
        } else if at <= top && at < data_first {
            Span::Changed(top + delta, bottom + delta)
        } else if top < at {
            Span::Changed(top, bottom + delta)
        } else {
            Span::Fallback
        }
    } else {
        let end = at - delta - 1;
        if at > bottom {
            Span::Unchanged
        } else if end < top {
            Span::Changed(top + delta, bottom + delta)
        } else if top < at
            && data_first <= at
            && end <= data_last
            && (data_first < at || end < data_last)
        {
            Span::Changed(top, bottom + delta)
        } else if data_last < bottom && at == bottom && end == bottom {
            Span::DropTotals
        } else {
            Span::Fallback
        }
    }
}

/// The columns a column insert (`delta > 0` columns left of `at`) or delete (columns
/// `at..at - delta`) gives the table on columns `left..=right`: an edit wholly left of it moves
/// it, one wholly right of it leaves it, an insert right of its first column grows it and a
/// delete inside it that leaves a column shrinks it (T-2324 table-insert-column-inside:
/// `COLUMNS(Sales)` 3; table-delete-column-qty and -region: 1). A delete of every column, one
/// straddling an edge (unprobed) or an insert that would push the table off the sheet is a
/// fallback.
fn column_span((left, right): (i32, i32), at: i32, delta: i32) -> Span {
    if at > right {
        return Span::Unchanged;
    }
    if delta > 0 {
        if right > LAST_COLUMN - delta {
            Span::Fallback
        } else if at <= left {
            Span::Changed(left + delta, right + delta)
        } else {
            Span::Changed(left, right + delta)
        }
    } else {
        let end = at - delta - 1;
        if end < left {
            Span::Changed(left + delta, right + delta)
        } else if left <= at && end <= right && (left < at || end < right) {
            // Inside the table, leaving at least one of its columns.
            Span::Changed(left, right + delta)
        } else {
            Span::Fallback
        }
    }
}

/// A table's range, `left..=right` by `top..=bottom`, and its data rows `data.0..=data.1` (below
/// its header row, above its totals row).
pub(crate) struct Bounds {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) data: (i32, i32),
}

/// The [`Bounds`] of `table`; `None` when its reference or row counts do not fit (directive 5).
pub(crate) fn bounds(table: &Table) -> Option<Bounds> {
    let (left, top, right, bottom) = parse_range(&table.reference).ok()?;
    let header = i32::try_from(table.header_row_count).ok()?;
    let totals = i32::try_from(table.totals_row_count).ok()?;
    let data = (top.checked_add(header)?, bottom.checked_sub(totals)?);
    Some(Bounds {
        left,
        top,
        right,
        bottom,
        data,
    })
}

/// What a structural edit does to one table of the edited sheet.
enum Follow {
    /// The table cannot follow the edit.
    Fallback,
    /// The edit leaves the table where it is.
    Unchanged,
    /// The table moves, grows or shrinks.
    Resized(Resized),
}

/// A table that moves, grows or shrinks with a structural edit.
#[derive(Debug, PartialEq)]
struct Resized {
    /// Its new reference.
    reference: String,
    /// Whether it loses its totals row.
    drops_totals: bool,
    /// The columns a column edit inside it inserts or deletes, if any.
    columns: Option<ColumnEdit>,
}

/// The table columns a column edit inside a table inserts (`delta > 0`) or deletes, from the
/// table's column `index` (0 for its first column).
#[derive(Debug, PartialEq, Clone, Copy)]
struct ColumnEdit {
    index: usize,
    delta: i32,
}

impl ColumnEdit {
    /// The number of columns a delete removes; 0 for an insert.
    fn deleted(self) -> usize {
        usize::try_from(-i64::from(self.delta)).unwrap_or(0)
    }
}

/// What the edit `displace_data`, on the sheet of `table`, does to it.
fn follow(table: &Table, displace_data: &DisplaceData) -> Follow {
    followed(table, displace_data).unwrap_or(Follow::Fallback)
}

/// [`follow`], with `None` for a fallback.
fn followed(table: &Table, displace_data: &DisplaceData) -> Option<Follow> {
    let Bounds {
        left,
        top,
        right,
        bottom,
        data,
    } = bounds(table)?;
    let (rows, columns, column_edit) = match *displace_data {
        DisplaceData::Row { row, delta, .. } => (
            row_span((top, bottom), data, row, delta),
            Span::Unchanged,
            None,
        ),
        DisplaceData::Column { column, delta, .. } => {
            let span = column_span((left, right), column, delta);
            // An insert right of the first column, or a delete from the first column on, is
            // inside the table: it changes the column list, not only the table's place.
            let inside = (delta > 0 && left < column) || (delta < 0 && left <= column);
            let edit = (inside && matches!(span, Span::Changed(..))).then(|| ColumnEdit {
                index: usize::try_from(column - left).unwrap_or(usize::MAX),
                delta,
            });
            (Span::Unchanged, span, edit)
        }
        _ => return None,
    };
    let ((top, bottom), (left, right), drops_totals) = match (rows, columns) {
        // `column_span` never drops a totals row.
        (Span::Fallback, _) | (_, Span::Fallback | Span::DropTotals) => return None,
        (Span::Unchanged, Span::Unchanged) => return Some(Follow::Unchanged),
        (Span::DropTotals, _) => ((top, bottom.checked_sub(1)?), (left, right), true),
        (Span::Changed(top, bottom), _) => ((top, bottom), (left, right), false),
        (_, Span::Changed(left, right)) => ((top, bottom), (left, right), false),
    };
    let (first, last) = (number_to_column(left)?, number_to_column(right)?);
    Some(Follow::Resized(Resized {
        reference: format!("{first}{top}:{last}{bottom}"),
        drops_totals,
        columns: column_edit,
    }))
}

/// The data block of `table` on sheet `sheet` when the row insert `displace_data` is at its first
/// data row under a header row or at its totals row, the two edges where Excel grows the table and
/// the plain ranges onto its data with it (T-2197); `None` for any other edit. The caller has
/// checked that the table follows the edit.
fn grown_edge(table: &Table, sheet: u32, displace_data: &DisplaceData) -> Option<DataBlock> {
    let DisplaceData::Row { row, delta, .. } = *displace_data else {
        return None;
    };
    let Bounds {
        left,
        top,
        right,
        bottom,
        data: (data_first, data_last),
    } = bounds(table)?;
    let at_edge = (row == data_first && top < row) || (row == data_last + 1 && row <= bottom);
    (delta > 0 && at_edge && data_first <= data_last).then_some(DataBlock {
        sheet,
        rows: (data_first, data_last),
        columns: (left, right),
    })
}

/// Inserts or deletes the table columns `edit` names in `columns`. An inserted column is named
/// `ColumnN`, N the least number no column of the table has yet: Excel names one inserted column
/// `Column1`, and a second `Column2` (T-2339 table-insert-column-inside-header,
/// table-insert-two-columns-inside-header); gaps, case clashes and a multi-column insert are a
/// hand generalisation (T-2899).
fn edit_columns(columns: &mut Vec<TableColumn>, edit: ColumnEdit) {
    let index = edit.index.min(columns.len());
    if edit.delta < 0 {
        let end = index.saturating_add(edit.deleted()).min(columns.len());
        columns.drain(index..end);
        return;
    }
    let mut next_id = columns.iter().map(|c| c.id).max().unwrap_or(0);
    let taken: HashSet<String> = columns.iter().map(|c| c.name.to_lowercase()).collect();
    let mut number = 0u32;
    for offset in 0..usize::try_from(edit.delta).unwrap_or(0) {
        next_id = next_id.saturating_add(1);
        let name = loop {
            number = number.saturating_add(1);
            let name = format!("Column{number}");
            if !taken.contains(&name.to_lowercase()) {
                break name;
            }
        };
        columns.insert(
            index.saturating_add(offset),
            TableColumn {
                id: next_id,
                name,
                ..TableColumn::default()
            },
        );
    }
}

/// Moves the kept autoFilter criteria with their columns through the column edit `edit`, as
/// [`edit_columns`] moves the columns: a criterion right of the edit shifts by its delta, and one
/// on a deleted column goes with it. Without this, an export would write a criterion on the wrong
/// column, or past the table's last one.
fn edit_filter_columns(filters: &mut Vec<FilterColumn>, edit: ColumnEdit) {
    let index = u32::try_from(edit.index).unwrap_or(u32::MAX);
    if edit.delta < 0 {
        let deleted = u32::try_from(edit.deleted()).unwrap_or(u32::MAX);
        let end = index.saturating_add(deleted);
        filters.retain(|f| f.col_id < index || f.col_id >= end);
        for filter in filters.iter_mut().filter(|f| f.col_id >= end) {
            filter.col_id -= deleted;
        }
    } else {
        let inserted = u32::try_from(edit.delta).unwrap_or(0);
        for filter in filters.iter_mut().filter(|f| f.col_id >= index) {
            filter.col_id = filter.col_id.saturating_add(inserted);
        }
    }
}

/// Appends one column to `columns` and returns its name: `name` when no column of the table has
/// that name (compared ignoring case), else `ColumnN` as [`edit_columns`] names an inserted
/// column.
pub(crate) fn append_column(columns: &mut Vec<TableColumn>, name: Option<&str>) -> String {
    let free = name.filter(|name| {
        let lower = name.to_lowercase();
        !columns.iter().any(|c| c.name.to_lowercase() == lower)
    });
    let index = columns.len();
    edit_columns(columns, ColumnEdit { index, delta: 1 });
    let Some(column) = columns.last_mut() else {
        // `edit_columns` has just appended the column.
        return String::new();
    };
    if let Some(name) = free {
        name.clone_into(&mut column.name);
    }
    column.name.clone()
}

/// The header cells (row, column, name) of the columns the insert `edit` added to `table`, whose
/// reference and column list are already edited; none for a delete or a table without a header
/// row.
fn inserted_headers(table: &Table, edit: ColumnEdit) -> Vec<(i32, i32, String)> {
    let Some(b) = bounds(table) else {
        return Vec::new();
    };
    if edit.delta <= 0 || table.header_row_count == 0 {
        return Vec::new();
    }
    let count = usize::try_from(edit.delta).unwrap_or(0);
    (b.left..=b.right)
        .zip(&table.columns)
        .skip(edit.index)
        .take(count)
        .map(|(column, c)| (b.top, column, c.name.clone()))
        .collect()
}

impl Model {
    /// Rewrites every formula (cells on every sheet, and the conditional formats of `sheet`) for
    /// the structural edit `displace_data` on `sheet`, then moves, grows or shrinks the tables
    /// that follow it.
    pub(crate) fn displace_formulas(
        &mut self,
        sheet: u32,
        displace_data: &DisplaceData,
    ) -> Result<(), String> {
        let edits = self.table_edits(displace_data);
        let displacement = Displacement {
            data: displace_data,
            kept_tables: &edits.kept,
            grown_edges: &edits.grown_edges,
            deleted_columns: &edits.deleted_columns,
        };
        self.displace_cells(displacement)?;
        self.displace_cf_ranges(sheet, displacement);
        self.resize_tables(sheet, edits.resized)
    }

    /// The table on `sheet` whose only data row, under a header row, is `row`, when the row delete
    /// of `row_count` rows at `row` is exactly that row: Excel then keeps the row and clears it, so
    /// the table keeps one blank data row and nothing below it moves (T-2324
    /// table-delete-row-all-data: `ROWS(Sales)` 1, `SUM(Sales[Qty])` 0, the plain `S!C4:C4` kept;
    /// table-delete-row-all-data-below, T-2338: `=S!C10` reads `=S!C7` after four deletes at row 4,
    /// three rows up, not four). Only the table's own cells of that row are cleared; what Excel does
    /// to the row's cells outside the table, and to a table with a totals row, is unprobed (T-2848).
    pub(crate) fn last_data_row(&self, sheet: u32, row: i32, row_count: i32) -> Option<Area> {
        if row_count != 1 {
            return None;
        }
        let sheet_name = self.workbook.worksheet(sheet).ok()?.get_name();
        self.workbook.tables.values().find_map(|table| {
            let b = bounds(table)?;
            let only = table.is_on_sheet(&sheet_name) && b.top < row && b.data == (row, row);
            only.then_some(Area {
                sheet,
                row,
                column: b.left,
                width: b.right.checked_sub(b.left)?.checked_add(1)?,
                height: 1,
            })
        })
    }

    /// What the structural edit `displace_data` does to the tables of the sheet it edits. Tables
    /// on other sheets are untouched and in neither list; a row or column move or a cell shift
    /// leaves every table a fallback.
    fn table_edits(&self, displace_data: &DisplaceData) -> TableEdits {
        let sheet = match displace_data {
            DisplaceData::Row { sheet, .. } | DisplaceData::Column { sheet, .. } => *sheet,
            _ => return TableEdits::default(),
        };
        let Ok(sheet_name) = self.workbook.worksheet(sheet).map(|w| w.get_name()) else {
            return TableEdits::default();
        };
        let mut edits = TableEdits::default();
        for (name, table) in &self.workbook.tables {
            if !table.is_on_sheet(&sheet_name) {
                continue;
            }
            match follow(table, displace_data) {
                Follow::Fallback => continue,
                Follow::Unchanged => {}
                Follow::Resized(resized) => {
                    edits
                        .grown_edges
                        .extend(grown_edge(table, sheet, displace_data));
                    if let Some(edit) = resized.columns {
                        let deleted = table.columns.iter().skip(edit.index).take(edit.deleted());
                        edits
                            .deleted_columns
                            .extend(deleted.map(|c| (name.to_lowercase(), c.name.to_lowercase())));
                    }
                    edits.resized.push((name.clone(), resized));
                }
            }
            edits.kept.push(name.to_lowercase());
        }
        edits
    }

    /// Gives the tables `resized` (on `sheet`) their new reference and column list, and reparses
    /// every formula against them, after the walk that printed the formulas (SPEC-0016
    /// "Structural edits" step 3): a formula whose text the edit kept still holds the area of the
    /// table before the edit. A column inserted inside a table with a header row gets its name
    /// written in its header cell, as Excel does (T-2339 table-insert-column-inside-header:
    /// `INDIRECT("S!C3")` is "Column1").
    fn resize_tables(&mut self, sheet: u32, resized: Vec<(String, Resized)>) -> Result<(), String> {
        if resized.is_empty() {
            return Ok(());
        }
        let mut headers = Vec::new();
        for (name, resized) in resized {
            if let Some(table) = self.workbook.tables.get_mut(&name) {
                table.reference = resized.reference;
                if resized.drops_totals {
                    table.totals_row_count = 0;
                }
                if let Some(edit) = resized.columns {
                    edit_columns(&mut table.columns, edit);
                    edit_filter_columns(&mut table.filter_columns, edit);
                    headers.extend(inserted_headers(table, edit));
                }
            }
        }
        for (row, column, name) in headers {
            self.update_cell_with_text(sheet, row, column, &name)?;
        }
        self.reset_tables();
        Ok(())
    }

    /// Refuses an insert of `count` rows (`rows`) or columns at `at` that would push a cell of a
    /// table on `sheet` past the last row (column), even when the cells pushed off are empty: Excel
    /// answers 409 `InsertDeleteConflict` (excel-graph 2026-10-28, T-2845). A table that merely ends
    /// exactly at the last row afterwards is unprobed; no table cell leaves the sheet, so it is allowed.
    pub(crate) fn refuse_table_off_sheet(
        &self,
        sheet: u32,
        at: i32,
        count: i32,
        rows: bool,
    ) -> Result<(), String> {
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        let (limit, what) = if rows {
            (LAST_ROW, "rows")
        } else {
            (LAST_COLUMN, "columns")
        };
        let off = (self.workbook.tables.values())
            .filter(|table| table.is_on_sheet(&sheet_name))
            .filter_map(bounds)
            .any(|b| {
                let last = if rows { b.bottom } else { b.right };
                last >= at && last.saturating_add(count) > limit
            });
        if off {
            return Err(format!(
                "Cannot insert {what} because that would move cells in a table off the sheet"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_column, column_span, edit_columns, edit_filter_columns, row_span, ColumnEdit, Span,
    };
    use crate::types::{FilterColumn, FilterCriterion, TableColumn};

    /// Header row 3, data rows 4-7, no totals row.
    fn rows(at: i32, delta: i32) -> Span {
        row_span((3, 7), (4, 7), at, delta)
    }

    #[test]
    fn row_inserts_follow_the_edge_case_table() {
        assert_eq!(rows(1, 1), Span::Changed(4, 8));
        assert_eq!(rows(3, 2), Span::Changed(5, 9));
        assert_eq!(rows(4, 1), Span::Changed(3, 8));
        assert_eq!(rows(5, 1), Span::Changed(3, 8));
        assert_eq!(rows(7, 1), Span::Changed(3, 8));
        assert_eq!(rows(8, 1), Span::Unchanged);
        // With a totals row 8 (data 4-7), an insert at the totals row grows the table (T-2321).
        assert_eq!(row_span((3, 8), (4, 7), 8, 1), Span::Changed(3, 9));
        assert_eq!(row_span((3, 8), (4, 7), 9, 1), Span::Unchanged);
        // Without a header row the first row is a data row: an insert there is a fallback, since
        // Excel grows a table with a header on an insert at its first data row (T-2845 probes it).
        assert_eq!(row_span((3, 7), (3, 7), 3, 1), Span::Fallback);
        assert_eq!(row_span((3, 7), (3, 7), 2, 1), Span::Changed(4, 8));
        // A table that would leave the sheet does not follow.
        let last = crate::constants::LAST_ROW;
        assert_eq!(row_span((3, last), (4, last), 1, 1), Span::Fallback);
    }

    #[test]
    fn row_deletes_follow_the_edge_case_table() {
        assert_eq!(rows(1, -2), Span::Changed(1, 5));
        assert_eq!(rows(5, -1), Span::Changed(3, 6));
        assert_eq!(rows(5, -2), Span::Changed(3, 5));
        assert_eq!(rows(8, -1), Span::Unchanged);
        // The data edges shrink it too (T-2338).
        assert_eq!(rows(4, -1), Span::Changed(3, 6));
        assert_eq!(rows(7, -1), Span::Changed(3, 6));
        assert_eq!(rows(6, -2), Span::Changed(3, 5));
        // Without a header row the first row is a data row: a fallback, as for inserts (T-2845).
        assert_eq!(row_span((3, 7), (3, 7), 3, -1), Span::Fallback);
        // The header row, every data row at once, or across an edge of the table: a fallback.
        for (at, count) in [(2, 2), (3, 1), (4, 4), (1, 10)] {
            assert_eq!(rows(at, -count), Span::Fallback, "delete {at}+{count}");
        }
        // With a totals row 8: deleting it drops it, deleting the last data row shrinks.
        assert_eq!(row_span((3, 8), (4, 7), 8, -1), Span::DropTotals);
        assert_eq!(row_span((3, 8), (4, 7), 7, -1), Span::Changed(3, 7));
        assert_eq!(row_span((3, 8), (4, 7), 7, -2), Span::Fallback);
    }

    #[test]
    fn column_edits_move_grow_shrink_or_fall_back() {
        // Columns B:C.
        assert_eq!(column_span((2, 3), 1, 1), Span::Changed(3, 4));
        assert_eq!(column_span((2, 3), 2, 1), Span::Changed(3, 4));
        // An insert right of the first column grows it (T-2339).
        assert_eq!(column_span((2, 3), 3, 1), Span::Changed(2, 4));
        assert_eq!(column_span((2, 3), 3, 2), Span::Changed(2, 5));
        assert_eq!(column_span((2, 3), 4, 1), Span::Unchanged);
        assert_eq!(column_span((2, 3), 1, -1), Span::Changed(1, 2));
        // A delete inside that leaves a column shrinks it, at either edge (T-2339).
        assert_eq!(column_span((2, 3), 3, -1), Span::Changed(2, 2));
        assert_eq!(column_span((2, 3), 2, -1), Span::Changed(2, 2));
        assert_eq!(column_span((2, 4), 3, -1), Span::Changed(2, 3));
        // Every column, or across an edge: a fallback.
        assert_eq!(column_span((2, 3), 2, -2), Span::Fallback);
        assert_eq!(column_span((2, 3), 1, -2), Span::Fallback);
        assert_eq!(column_span((2, 3), 3, -2), Span::Fallback);
        assert_eq!(column_span((2, 3), 4, -1), Span::Unchanged);
        // A table that would leave the sheet does not follow.
        let last = crate::constants::LAST_COLUMN;
        assert_eq!(column_span((2, last), 3, 1), Span::Fallback);
    }

    fn named(names: &[&str]) -> Vec<TableColumn> {
        (1..)
            .zip(names)
            .map(|(id, name)| TableColumn {
                id,
                name: (*name).to_string(),
                ..TableColumn::default()
            })
            .collect()
    }

    fn names(columns: &[TableColumn]) -> Vec<(u32, &str)> {
        columns.iter().map(|c| (c.id, c.name.as_str())).collect()
    }

    /// A typed header names the appended column unless the table has that name already; then the
    /// column is `ColumnN` (T-2968).
    #[test]
    fn appended_columns_take_a_free_name_or_column_n() {
        let mut columns = named(&["Col", "Other"]);
        assert_eq!(append_column(&mut columns, None), "Column1");
        assert_eq!(append_column(&mut columns, Some("0")), "0");
        assert_eq!(append_column(&mut columns, Some("0")), "Column2");
        assert_eq!(
            names(&columns),
            [
                (1, "Col"),
                (2, "Other"),
                (3, "Column1"),
                (4, "0"),
                (5, "Column2")
            ]
        );
    }

    #[test]
    fn column_edits_insert_named_columns_and_delete() {
        let mut columns = named(&["Region", "Qty"]);
        edit_columns(&mut columns, ColumnEdit { index: 1, delta: 2 });
        assert_eq!(
            names(&columns),
            [(1, "Region"), (3, "Column1"), (4, "Column2"), (2, "Qty")]
        );
        // A name the table has is skipped.
        let mut columns = named(&["Column1", "Qty"]);
        edit_columns(&mut columns, ColumnEdit { index: 1, delta: 1 });
        assert_eq!(
            names(&columns),
            [(1, "Column1"), (3, "Column2"), (2, "Qty")]
        );
        let mut columns = named(&["Region", "Qty", "Price"]);
        edit_columns(
            &mut columns,
            ColumnEdit {
                index: 0,
                delta: -2,
            },
        );
        assert_eq!(names(&columns), [(3, "Price")]);
    }

    /// Kept filter criteria follow their columns through an insert and a delete (T-2838).
    #[test]
    fn filter_criteria_follow_their_columns() {
        let filter = |col_id| FilterColumn {
            col_id,
            criterion: FilterCriterion::Other,
        };
        let ids = |filters: &[FilterColumn]| filters.iter().map(|f| f.col_id).collect::<Vec<_>>();
        let mut filters = vec![filter(0), filter(2)];
        edit_filter_columns(&mut filters, ColumnEdit { index: 1, delta: 2 });
        assert_eq!(ids(&filters), [0, 4]);
        edit_filter_columns(&mut filters, ColumnEdit { index: 4, delta: 1 });
        assert_eq!(ids(&filters), [0, 5]);
        // Deleting columns 1-2 shifts the criterion on 5 to 3; deleting column 0 drops its own.
        edit_filter_columns(
            &mut filters,
            ColumnEdit {
                index: 1,
                delta: -2,
            },
        );
        assert_eq!(ids(&filters), [0, 3]);
        edit_filter_columns(
            &mut filters,
            ColumnEdit {
                index: 0,
                delta: -1,
            },
        );
        assert_eq!(ids(&filters), [2]);
    }
}
