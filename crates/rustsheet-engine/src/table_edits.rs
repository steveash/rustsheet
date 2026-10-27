//! Tables that follow a row or column insert or delete (SPEC-0016 "Structural edits", T-2297):
//! an edit wholly before a table moves it, a row insert below its header row and above or at its
//! totals row grows it (T-2321), a row delete inside its data rows that leaves one shrinks it and
//! one of its totals row drops that row (T-2338), so the structured references to it keep their
//! text. Every other edit that crosses a table leaves it
//! as it is (the *fallback*): its references print the displaced area, as before SPEC-0016.

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::parse_range;
use crate::expressions::parser::stringify::{DataBlock, DisplaceData, Displacement};
use crate::expressions::types::Area;
use crate::expressions::utils::number_to_column;
use crate::model::Model;
use crate::types::Table;

/// What a structural edit does to the tables of the edited sheet.
#[derive(Debug, Default, PartialEq)]
struct TableEdits {
    /// The lowercase names of the tables whose structured references keep their text: those
    /// that move, grow or shrink with the edit, and those it leaves alone.
    kept: Vec<String>,
    /// The tables that move, grow or shrink, by name, with their new `reference` and whether
    /// they lose their totals row.
    resized: Vec<(String, String, bool)>,
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

/// The columns a column insert or delete gives the table on columns `left..=right`: an edit
/// wholly left of it moves it, one wholly right of it leaves it, and any other changes its column
/// list, which is a fallback (T-2339 grows it on an insert inside).
fn column_span((left, right): (i32, i32), at: i32, delta: i32) -> Span {
    if at > right {
        Span::Unchanged
    } else if (delta > 0 && at <= left && right <= LAST_COLUMN - delta)
        || (delta < 0 && at - delta - 1 < left)
    {
        Span::Changed(left + delta, right + delta)
    } else {
        Span::Fallback
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
    /// The table moves, grows or shrinks to this reference, losing its totals row if set.
    Resized(String, bool),
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
    let (rows, columns) = match *displace_data {
        DisplaceData::Row { row, delta, .. } => {
            (row_span((top, bottom), data, row, delta), Span::Unchanged)
        }
        DisplaceData::Column { column, delta, .. } => {
            (Span::Unchanged, column_span((left, right), column, delta))
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
    let reference = format!("{first}{top}:{last}{bottom}");
    Some(Follow::Resized(reference, drops_totals))
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
        };
        self.displace_cells(displacement)?;
        self.displace_cf_ranges(sheet, displacement);
        self.resize_tables(edits.resized);
        Ok(())
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
                Follow::Resized(reference, drops_totals) => {
                    edits
                        .grown_edges
                        .extend(grown_edge(table, sheet, displace_data));
                    edits.resized.push((name.clone(), reference, drops_totals));
                }
            }
            edits.kept.push(name.to_lowercase());
        }
        edits
    }

    /// Gives the tables `resized` their new reference and reparses every formula against them,
    /// after the walk that printed the formulas (SPEC-0016 "Structural edits" step 3): a formula
    /// whose text the edit kept still holds the area of the table before the edit.
    fn resize_tables(&mut self, resized: Vec<(String, String, bool)>) {
        if resized.is_empty() {
            return;
        }
        for (name, reference, drops_totals) in resized {
            if let Some(table) = self.workbook.tables.get_mut(&name) {
                table.reference = reference;
                if drops_totals {
                    table.totals_row_count = 0;
                }
            }
        }
        self.reset_tables();
    }
}

#[cfg(test)]
mod tests {
    use super::{column_span, row_span, Span};

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
    fn column_edits_move_or_fall_back() {
        // Columns B:C.
        assert_eq!(column_span((2, 3), 1, 1), Span::Changed(3, 4));
        assert_eq!(column_span((2, 3), 2, 1), Span::Changed(3, 4));
        assert_eq!(column_span((2, 3), 3, 1), Span::Fallback);
        assert_eq!(column_span((2, 3), 4, 1), Span::Unchanged);
        assert_eq!(column_span((2, 3), 1, -1), Span::Changed(1, 2));
        assert_eq!(column_span((2, 3), 1, -2), Span::Fallback);
        assert_eq!(column_span((2, 3), 3, -1), Span::Fallback);
        assert_eq!(column_span((2, 3), 4, -1), Span::Unchanged);
    }
}
