//! Tables that follow a row or column insert or delete (SPEC-0016 "Structural edits", T-2297):
//! an edit wholly before a table moves it, a row insert below its header row and above or at its
//! totals row grows it (T-2321) and a row delete strictly inside its data rows shrinks it, so the
//! structured references to it keep their text. Every other edit that crosses a table leaves it
//! as it is (the *fallback*): its references print the displaced area, as before SPEC-0016.

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::parse_range;
use crate::expressions::parser::stringify::{DataBlock, DisplaceData, Displacement};
use crate::expressions::utils::number_to_column;
use crate::model::Model;
use crate::types::Table;

/// What a structural edit does to the tables of the edited sheet.
#[derive(Debug, Default, PartialEq)]
struct TableEdits {
    /// The lowercase names of the tables whose structured references keep their text: those
    /// that move, grow or shrink with the edit, and those it leaves alone.
    kept: Vec<String>,
    /// The tables that move, grow or shrink, by name, with their new `reference`.
    resized: Vec<(String, String)>,
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
    /// The table cannot follow the edit.
    Fallback,
}

/// The rows a row insert (`delta > 0` rows above `at`) or delete (rows `at..at - delta`) gives
/// the table on rows `top..=bottom` with data rows `data_first..=data_last`, by SPEC-0016's
/// edge-case table. An insert below the top row and at or above the bottom row grows the table,
/// so one at the first data row under a header row or at the totals row does too (T-2197
/// table-insert-row-above-first-data and -above-totals: `ROWS(Sales)` 5). An insert at the first
/// row of a table without a header row is a fallback (unprobed, T-2845), and so is a delete at an
/// edge of the data or touching the header or totals row (T-2338).
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
        } else if data_first < at && end < data_last {
            Span::Changed(top, bottom + delta)
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

/// What a structural edit does to one table of the edited sheet.
enum Follow {
    /// The table cannot follow the edit.
    Fallback,
    /// The edit leaves the table where it is.
    Unchanged,
    /// The table moves, grows or shrinks to this reference.
    Resized(String),
}

/// What the edit `displace_data`, on the sheet of `table`, does to it.
fn follow(table: &Table, displace_data: &DisplaceData) -> Follow {
    followed(table, displace_data).unwrap_or(Follow::Fallback)
}

/// [`follow`], with `None` for a fallback.
fn followed(table: &Table, displace_data: &DisplaceData) -> Option<Follow> {
    let (left, top, right, bottom) = parse_range(&table.reference).ok()?;
    let header = i32::try_from(table.header_row_count).ok()?;
    let totals = i32::try_from(table.totals_row_count).ok()?;
    let (rows, columns) = match *displace_data {
        DisplaceData::Row { row, delta, .. } => {
            let data_first = top.checked_add(header)?;
            let data_last = bottom.checked_sub(totals)?;
            let rows = row_span((top, bottom), (data_first, data_last), row, delta);
            (rows, Span::Unchanged)
        }
        DisplaceData::Column { column, delta, .. } => {
            (Span::Unchanged, column_span((left, right), column, delta))
        }
        _ => return None,
    };
    let ((top, bottom), (left, right)) = match (rows, columns) {
        (Span::Fallback, _) | (_, Span::Fallback) => return None,
        (Span::Unchanged, Span::Unchanged) => return Some(Follow::Unchanged),
        (Span::Changed(top, bottom), _) => ((top, bottom), (left, right)),
        (_, Span::Changed(left, right)) => ((top, bottom), (left, right)),
    };
    let (first, last) = (number_to_column(left)?, number_to_column(right)?);
    Some(Follow::Resized(format!("{first}{top}:{last}{bottom}")))
}

/// The data block of `table` on sheet `sheet` when the row insert `displace_data` is at its first
/// data row under a header row or at its totals row, the two edges where Excel grows the table and
/// the plain ranges onto its data with it (T-2197); `None` for any other edit. The caller has
/// checked that the table follows the edit.
fn grown_edge(table: &Table, sheet: u32, displace_data: &DisplaceData) -> Option<DataBlock> {
    let DisplaceData::Row { row, delta, .. } = *displace_data else {
        return None;
    };
    let (left, top, right, bottom) = parse_range(&table.reference).ok()?;
    let data_first = top.checked_add(i32::try_from(table.header_row_count).ok()?)?;
    let data_last = bottom.checked_sub(i32::try_from(table.totals_row_count).ok()?)?;
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
                Follow::Resized(reference) => {
                    edits
                        .grown_edges
                        .extend(grown_edge(table, sheet, displace_data));
                    edits.resized.push((name.clone(), reference));
                }
            }
            edits.kept.push(name.to_lowercase());
        }
        edits
    }

    /// Gives the tables `resized` their new reference and reparses every formula against them,
    /// after the walk that printed the formulas (SPEC-0016 "Structural edits" step 3): a formula
    /// whose text the edit kept still holds the area of the table before the edit.
    fn resize_tables(&mut self, resized: Vec<(String, String)>) {
        if resized.is_empty() {
            return;
        }
        for (name, reference) in resized {
            if let Some(table) = self.workbook.tables.get_mut(&name) {
                table.reference = reference;
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
        for (at, count) in [(2, 2), (3, 1), (4, 1), (7, 1), (6, 2), (4, 4), (1, 10)] {
            assert_eq!(rows(at, -count), Span::Fallback, "delete {at}+{count}");
        }
        assert_eq!(row_span((3, 8), (4, 7), 8, -1), Span::Fallback);
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
