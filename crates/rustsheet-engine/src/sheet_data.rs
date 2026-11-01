//! The cells of a worksheet.
//!
//! Everything that reads or writes the cells of a sheet goes through
//! [`SheetData`], so that how they are stored is nobody else's business.
//!
//! # How they are stored
//!
//! By position, with no hashing. Rows are grouped in chunks of `CHUNK_ROWS`
//! consecutive rows, and a chunk is allocated the first time one of its rows is
//! used, so a sheet with a single cell in its last row costs one chunk and not
//! a million empty rows. A row is a vector of `(column, cell)` sorted by column.
//!
//! ```text
//!   chunks ──► [ chunk 0 | (none) | chunk 2 | ... ]        row >> CHUNK_BITS
//!                  │
//!                  ▼
//!              [ row 0 | row 1 | (none) | ... ]            row & CHUNK_MASK
//!                           │
//!                           ▼
//!                       [ (A, cell) (C, cell) (D, cell) ]  sorted by column
//! ```
//!
//! Reading a cell is two indexed steps and a short search. Rows and cells come
//! out in natural order, first by row and then by column, without sorting.
//! Evaluation reads every cell several times, and with a hash map per row each
//! read was two hashes and two jumps to unrelated places in memory, which on a
//! large sheet cost more than running the formulas.

use std::collections::{BTreeMap, BTreeSet};

use bitcode::{Decode, Encode};

use crate::constants::LAST_ROW;
use crate::sheet_reads::{PositionHint, ReadLog};
use crate::types::Cell;

mod row;
use row::{on_sheet_column, Row};

const CHUNK_BITS: u32 = 10;
const CHUNK_ROWS: usize = 1 << CHUNK_BITS;
const CHUNK_MASK: usize = CHUNK_ROWS - 1;

/// The cells of a worksheet, by row and column. Both are 1-indexed.
///
/// A row can exist without cells: `set_row` with no cells creates one. The xlsx
/// importer does that for rows that carry a height or a style and nothing
/// else, and the exporter writes a `<row>` element for every row that exists
/// here, so such rows survive a round trip. Removing the last cell of a row
/// with `remove_cell` removes the row.
///
/// Rows and columns outside the sheet (below 1, or beyond the last) hold no
/// cells: writing to them does nothing and reading them finds nothing.
#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct SheetData {
    /// `chunks[i]` holds rows `i * CHUNK_ROWS ..`, if any of them exists.
    chunks: Vec<Option<Box<Chunk>>>,
    /// How many rows exist.
    row_count: usize,
    /// How many cells each column holds, columns with none left out. Its last
    /// key is the last used column, so removals stay O(log columns) however
    /// many rows share that column. Derived from the cells, kept in step by
    /// every write. Not serialised, so bytes cannot forge it:
    /// `Model::from_workbook` rebuilds it with `rebuild_column_counts`.
    #[bitcode(skip)]
    column_counts: BTreeMap<i32, usize>,
    /// The position of every cell as `(column, row)`, in that order, so the rows a column
    /// holds are found without visiting the rows other columns fill: a union walk of a few
    /// columns costs what those columns hold (T-1502). Built from the cells by
    /// `build_column_index` the first time a walk needs it, then kept in step by every write;
    /// until then writes and loads pay nothing for it (an index kept from the start doubled
    /// the cost of filling a sheet). Not serialised, so bytes cannot forge it.
    #[bitcode(skip)]
    column_rows: Option<BTreeSet<(i32, i32)>>,
    /// Counts the writes that add or remove a cell, so a walk that hands cells to code that may
    /// write (a spill) can tell that what it found before is out of date.
    #[bitcode(skip)]
    layout_version: u64,
    /// The last row that exists (0 for none), kept in step by every write, so that `last_row`,
    /// which every clipped walk asks, costs nothing (T-0465). `None` until known, for a sheet
    /// decoded and not yet rebuilt, when `last_row` scans the chunks. Not serialised. Every write
    /// that creates or removes a row (`row_or_insert`, `remove_row`) must keep it.
    #[bitcode(skip)]
    known_last_row: Option<i32>,
    /// What has been read of the cells while a record is on (`sheet_reads.rs`, T-0453). Not
    /// serialised.
    #[bitcode(skip)]
    pub(crate) reads: ReadLog,
    /// Where in its row [`SheetData::cell`] found its last cell (`PositionHint`).
    #[bitcode(skip)]
    hint: PositionHint,
}

#[derive(Encode, Decode, Debug, Clone)]
struct Chunk {
    /// Always `CHUNK_ROWS` long.
    rows: Vec<Option<Row>>,
    /// How many of them exist. A chunk with none is freed.
    used: usize,
}

impl Chunk {
    fn new() -> Self {
        let mut rows = Vec::new();
        rows.resize_with(CHUNK_ROWS, || None);
        Chunk { rows, used: 0 }
    }
}

/// What [`SheetData::next_row_between`] found and what it looked at to find it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RowSearch {
    /// The first existing row of the span, if any.
    pub(crate) row: Option<i32>,
    /// Unallocated chunks passed over.
    pub(crate) absent_chunks: u64,
    /// Row slots of allocated chunks tested.
    pub(crate) slots: u64,
}

/// The chunk and the place in it of a row, if the row is within the sheet.
#[inline]
fn locate(row: i32) -> Option<(usize, usize)> {
    if !(1..=LAST_ROW).contains(&row) {
        return None;
    }
    let row = row as usize;
    Some((row >> CHUNK_BITS, row & CHUNK_MASK))
}

impl SheetData {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn row(&self, row: i32) -> Option<&Row> {
        let (chunk, index) = locate(row)?;
        self.chunks.get(chunk)?.as_ref()?.rows[index].as_ref()
    }

    #[inline]
    fn row_mut(&mut self, row: i32) -> Option<&mut Row> {
        let (chunk, index) = locate(row)?;
        self.chunks.get_mut(chunk)?.as_mut()?.rows[index].as_mut()
    }

    /// The row, created if it does not exist. `None` outside the sheet.
    fn row_or_insert(&mut self, row: i32) -> Option<&mut Row> {
        let (chunk, index) = locate(row)?;
        if chunk >= self.chunks.len() {
            self.chunks.resize_with(chunk + 1, || None);
        }
        let chunk = self.chunks[chunk].get_or_insert_with(|| Box::new(Chunk::new()));
        if chunk.rows[index].is_none() {
            chunk.rows[index] = Some(Row::default());
            chunk.used += 1;
            self.row_count += 1;
            self.known_last_row = Some(match self.known_last_row {
                Some(last) => last.max(row),
                None => self.scan_last_row(),
            });
        }
        self.row_mut(row)
    }

    /// Every row that exists with its number, in ascending order.
    fn rows_in_order(&self) -> impl Iterator<Item = (i32, &Row)> {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(chunk_index, chunk)| Some((chunk_index, chunk.as_ref()?)))
            .flat_map(|(chunk_index, chunk)| {
                chunk
                    .rows
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, row)| {
                        let number = (chunk_index << CHUNK_BITS) | index;
                        Some((number as i32, row.as_ref()?))
                    })
            })
    }

    /// The cell at a position, if there is one.
    #[inline]
    pub fn cell(&self, row: i32, column: i32) -> Option<&Cell> {
        self.reads.cell(row, column);
        let row = self.row(row)?;
        if let Some((_, cell)) = row.cells.get(self.hint.get()).filter(|(c, _)| *c == column) {
            return Some(cell);
        }
        let index = row.position(column).ok()?;
        self.hint.set(index);
        Some(&row.cells[index].1)
    }

    /// The cell at a position, to be changed in place.
    #[inline]
    pub fn cell_mut(&mut self, row: i32, column: i32) -> Option<&mut Cell> {
        self.reads.cell(row, column);
        let row = self.row_mut(row)?;
        let index = row.position(column).ok()?;
        Some(&mut row.cells[index].1)
    }

    /// Puts a cell at a position and returns the one that was there.
    pub fn set_cell(&mut self, row: i32, column: i32, cell: Cell) -> Option<Cell> {
        if !on_sheet_column(column) {
            return None;
        }
        let previous = self.row_or_insert(row)?.insert(column, cell);
        if previous.is_none() {
            self.index_cell(row, column);
        }
        previous
    }

    /// Removes the cell at a position and returns it.
    pub fn remove_cell(&mut self, row: i32, column: i32) -> Option<Cell> {
        let row_data = self.row_mut(row)?;
        let index = row_data.position(column).ok()?;
        let (_, cell) = row_data.cells.remove(index);
        if row_data.cells.is_empty() {
            self.remove_row(row);
        }
        self.unindex_cell(row, column);
        Some(cell)
    }

    /// True if the sheet has no rows.
    pub fn is_empty(&self) -> bool {
        self.reads.extent();
        self.row_count == 0
    }

    /// Every cell of the sheet with its row and column, in natural order:
    /// by row, and within a row by column.
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32, &Cell)> {
        self.reads.rows(1, LAST_ROW);
        self.rows_in_order().flat_map(|(row, row_data)| {
            row_data
                .cells
                .iter()
                .map(move |(column, cell)| (row, *column, cell))
        })
    }

    /// The number of the last row that exists, or 0 for an empty sheet. Kept by every write, so
    /// callers can bound whole-column walks by it.
    pub(crate) fn last_row(&self) -> i32 {
        self.reads.extent();
        self.known_last_row.unwrap_or_else(|| self.scan_last_row())
    }

    /// [`Self::last_row`] from the chunks: the last chunk's slots, from the end.
    fn scan_last_row(&self) -> i32 {
        for (chunk_index, chunk) in self.chunks.iter().enumerate().rev() {
            let Some(chunk) = chunk else { continue };
            if let Some(index) = chunk.rows.iter().rposition(Option::is_some) {
                return ((chunk_index << CHUNK_BITS) | index) as i32;
            }
        }
        0
    }

    /// The number of the last column that holds a cell, or 0 for an empty sheet.
    /// O(log columns), so callers can bound whole-row walks by it.
    pub(crate) fn last_column(&self) -> i32 {
        self.reads.extent();
        self.column_counts.keys().next_back().copied().unwrap_or(0)
    }

    /// The number of cells in the columns `column1..=column2`, all rows: a search of the column
    /// counts and a step per column that holds a cell.
    pub(crate) fn cells_in_columns(&self, column1: i32, column2: i32) -> u64 {
        if column1 > column2 {
            return 0;
        }
        self.column_counts
            .range(column1..=column2)
            .map(|(_, &count)| count as u64)
            .sum()
    }

    /// Recounts the columns from the cells, for a sheet that was decoded. The column index is
    /// left to be built when a walk needs it.
    pub(crate) fn rebuild_column_counts(&mut self) {
        let mut counts = BTreeMap::new();
        for (_, column, _) in self.cells() {
            *counts.entry(column).or_insert(0) += 1;
        }
        self.column_counts = counts;
        self.column_rows = None;
        self.known_last_row = Some(self.scan_last_row());
        self.layout_version = self.layout_version.wrapping_add(1);
    }

    /// Checks the storage of a decoded sheet, which the bytes control: every read indexes a chunk's
    /// rows by the row number unchecked, and the counts are trusted by every write. Holds when
    /// there are at most as many chunks as the grid has, each `CHUNK_ROWS` rows long with `used`
    /// counting them, rows only within `1..=LAST_ROW`, `row_count` counting every row, and the
    /// columns of a row strictly ascending within `1..=LAST_COLUMN`. One pass, linear in the
    /// chunks and cells.
    pub(crate) fn check_layout(&self) -> Result<(), String> {
        if self.chunks.len() > (LAST_ROW as usize >> CHUNK_BITS) + 1 {
            return Err("more row chunks than the sheet has rows".to_string());
        }
        let mut rows = 0;
        for (chunk_index, chunk) in self.chunks.iter().enumerate() {
            let Some(chunk) = chunk else { continue };
            if chunk.rows.len() != CHUNK_ROWS {
                return Err(format!(
                    "row chunk {chunk_index} is not {CHUNK_ROWS} rows long"
                ));
            }
            let mut used = 0;
            for (index, row) in chunk.rows.iter().enumerate() {
                let Some(row) = row else { continue };
                let number = (chunk_index << CHUNK_BITS) | index;
                if number == 0 || number > LAST_ROW as usize {
                    return Err(format!("row {number} is outside the sheet"));
                }
                let mut previous = 0;
                for &(column, _) in &row.cells {
                    if column <= previous || !on_sheet_column(column) {
                        return Err(format!(
                            "row {number}: column {column} is out of order or outside the sheet"
                        ));
                    }
                    previous = column;
                }
                used += 1;
            }
            if used != chunk.used {
                return Err(format!("row chunk {chunk_index} miscounts its rows"));
            }
            rows += used;
        }
        if rows != self.row_count {
            return Err("the sheet miscounts its rows".to_string());
        }
        Ok(())
    }

    /// Builds the column index from the cells if it is not built yet. One pass over the cells
    /// and a sort, paid once per sheet: afterwards every write keeps it in step.
    pub(crate) fn build_column_index(&mut self) {
        if self.column_rows.is_none() {
            // Collected and sorted, then built in one pass: far cheaper than one insert per cell.
            self.column_rows = Some(self.cells().map(|(row, column, _)| (column, row)).collect());
        }
    }

    /// Whether the column index is built, for tests of when a walk builds it.
    #[cfg(test)]
    pub(crate) fn has_column_index(&self) -> bool {
        self.column_rows.is_some()
    }

    fn index_cell(&mut self, row: i32, column: i32) {
        *self.column_counts.entry(column).or_insert(0) += 1;
        if let Some(index) = &mut self.column_rows {
            index.insert((column, row));
        }
        self.layout_version = self.layout_version.wrapping_add(1);
    }

    fn unindex_cell(&mut self, row: i32, column: i32) {
        if let Some(count) = self.column_counts.get_mut(&column) {
            *count -= 1;
            if *count == 0 {
                self.column_counts.remove(&column);
            }
        }
        if let Some(index) = &mut self.column_rows {
            index.remove(&(column, row));
        }
        self.layout_version = self.layout_version.wrapping_add(1);
    }

    /// Changes whenever a cell is added or removed (wrapping, so equal means unchanged only
    /// between reads close together, as a walk's are).
    pub(crate) fn layout_version(&self) -> u64 {
        self.layout_version
    }

    /// Replaces `columns` with the columns of `column1..=column2` that hold a cell, in ascending
    /// order, unless there are more than `limit`: then `false`, with `columns` holding the first
    /// `limit` of them. Each column found is one search of the index. `false` with no columns
    /// when the index is not built ([`Self::build_column_index`]).
    pub(crate) fn columns_between(
        &self,
        column1: i32,
        column2: i32,
        limit: usize,
        columns: &mut Vec<i32>,
    ) -> bool {
        columns.clear();
        self.reads.columns(column1, column2);
        let Some(index) = &self.column_rows else {
            return false;
        };
        let mut from = column1;
        while from <= column2 {
            let Some(&(column, _)) = index.range((from, i32::MIN)..=(column2, i32::MAX)).next()
            else {
                break;
            };
            if columns.len() == limit {
                return false;
            }
            columns.push(column);
            let Some(next) = column.checked_add(1) else {
                break;
            };
            from = next;
        }
        true
    }

    /// Replaces `rows` with the first `limit` rows of `from..=to` at which `column` holds a cell,
    /// in ascending order: one search of the index, then a step per row. `rows` is left empty
    /// when the index is not built ([`Self::build_column_index`]).
    pub(crate) fn rows_in_column_between(
        &self,
        column: i32,
        (from, to): (i32, i32),
        limit: usize,
        rows: &mut Vec<i32>,
    ) {
        rows.clear();
        self.reads.area((from, column, to, column));
        let Some(index) = &self.column_rows else {
            return;
        };
        if from > to {
            return;
        }
        rows.extend(
            index
                .range((column, from)..=(column, to))
                .take(limit)
                .map(|&(_, row)| row),
        );
    }

    /// The rows that exist, in ascending order.
    pub fn rows(&self) -> Vec<i32> {
        self.reads.rows(1, LAST_ROW);
        self.rows_in_order().map(|(row, _)| row).collect()
    }

    /// The columns of a row that have cells, in ascending order.
    pub fn columns_in_row(&self, row: i32) -> Vec<i32> {
        self.reads.rows(row, row);
        match self.row(row) {
            Some(row_data) => row_data.cells.iter().map(|(column, _)| *column).collect(),
            None => Vec::new(),
        }
    }

    /// The first row of `from..=to` that exists, if any, with the work of finding it: how many
    /// unallocated chunks were passed over and how many row slots of allocated chunks were
    /// tested. A walk over sparse rows calls it again from the row after the one found, so its
    /// total work is one pass over the chunks of the span, whatever the number of rows in it.
    pub(crate) fn next_row_between(&self, from: i32, to: i32) -> RowSearch {
        let mut search = RowSearch::default();
        self.reads.rows(from, to);
        if from > to {
            return search;
        }
        let (Some((mut chunk_index, mut index)), Some((last_chunk, last_index))) =
            (locate(from.max(1)), locate(to.min(LAST_ROW)))
        else {
            return search;
        };
        while chunk_index <= last_chunk && chunk_index < self.chunks.len() {
            if let Some(chunk) = &self.chunks[chunk_index] {
                let end = if chunk_index == last_chunk {
                    last_index
                } else {
                    CHUNK_MASK
                };
                for slot in index..=end {
                    search.slots += 1;
                    if chunk.rows[slot].is_some() {
                        search.row = Some(((chunk_index << CHUNK_BITS) | slot) as i32);
                        return search;
                    }
                }
            } else {
                search.absent_chunks += 1;
            }
            chunk_index += 1;
            index = 0;
        }
        search
    }

    /// Replaces `columns` with the columns of `row` in `column1..=column2` that have cells, in
    /// ascending order. The row is searched once, as [`Self::cell`] would search it.
    pub(crate) fn columns_in_row_between(
        &self,
        row: i32,
        column1: i32,
        column2: i32,
        columns: &mut Vec<i32>,
    ) {
        columns.clear();
        self.reads.area((row, column1, row, column2));
        let Some(row_data) = self.row(row) else {
            return;
        };
        let start = row_data.position(column1).unwrap_or_else(|index| index);
        columns.extend(
            row_data.cells[start..]
                .iter()
                .map(|(column, _)| *column)
                .take_while(|column| *column <= column2),
        );
    }

    /// The places of the cells that exist in the `height` by `width` block from (`row`, `column`),
    /// row by row: a walk costs the cells there are, not the area, which can be the whole sheet
    /// (T-2896).
    pub(crate) fn places_in_block(
        &self,
        (row, column): (i32, i32),
        (width, height): (i32, i32),
    ) -> Vec<(i32, i32)> {
        let (last_row, last_column) = (
            row.saturating_add(height - 1),
            column.saturating_add(width - 1),
        );
        let mut places = Vec::new();
        let mut columns = Vec::new();
        let mut from = row;
        while let Some(found) = self.next_row_between(from, last_row).row {
            self.columns_in_row_between(found, column, last_column, &mut columns);
            places.extend(columns.iter().map(|&c| (found, c)));
            from = found + 1;
        }
        places
    }

    /// The cells of a row with their column, in ascending order of column.
    pub fn cells_in_row(&self, row: i32) -> Vec<(i32, &Cell)> {
        self.reads.rows(row, row);
        match self.row(row) {
            Some(row_data) => row_data
                .cells
                .iter()
                .map(|(column, cell)| (*column, cell))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Removes every cell of a row, and the row.
    pub fn remove_row(&mut self, row: i32) {
        let Some((chunk_index, index)) = locate(row) else {
            return;
        };
        let Some(Some(chunk)) = self.chunks.get_mut(chunk_index) else {
            return;
        };
        if let Some(removed) = chunk.rows[index].take() {
            chunk.used -= 1;
            self.row_count -= 1;
            if chunk.used == 0 {
                self.chunks[chunk_index] = None;
            }
            if self.known_last_row == Some(row) {
                self.known_last_row = Some(self.scan_last_row());
            }
            for (column, _) in &removed.cells {
                self.unindex_cell(row, *column);
            }
        }
    }

    /// Replaces the cells of a row. The row exists afterwards, even with no
    /// cells. If a column is given twice the last one stays.
    pub fn set_row(&mut self, row: i32, cells: impl IntoIterator<Item = (i32, Cell)>) {
        let Some(row_data) = self.row_or_insert(row) else {
            return;
        };
        // Collected at its exact size: a row grown cell by cell from empty starts with room for
        // four, and a sheet with a million short rows paid for that four times over.
        let on_sheet = |(column, _): &(i32, Cell)| on_sheet_column(*column);
        let mut cells: Vec<(i32, Cell)> = cells.into_iter().filter(on_sheet).collect();
        cells.sort_by_key(|(column, _)| *column);
        // Of two cells in the same column the later one stays: it is moved
        // into the slot of the earlier one, which `dedup_by` then keeps.
        cells.dedup_by(|later, earlier| {
            if later.0 == earlier.0 {
                std::mem::swap(&mut later.1, &mut earlier.1);
                true
            } else {
                false
            }
        });
        cells.shrink_to_fit();
        let old = std::mem::replace(&mut row_data.cells, cells);
        let new_columns: Vec<i32> = row_data.cells.iter().map(|(column, _)| *column).collect();
        for (column, _) in &old {
            self.unindex_cell(row, *column);
        }
        for column in new_columns {
            self.index_cell(row, column);
        }
    }
}

/// Two sheets are equal when they have the same rows and the same cells.
/// Which chunks happen to be allocated is not part of that.
impl PartialEq for SheetData {
    fn eq(&self, other: &Self) -> bool {
        self.row_count == other.row_count && self.rows_in_order().eq(other.rows_in_order())
    }
}

#[cfg(test)]
mod test;
