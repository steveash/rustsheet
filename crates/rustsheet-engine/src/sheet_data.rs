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
use crate::types::Cell;

const CHUNK_BITS: u32 = 10;
const CHUNK_ROWS: usize = 1 << CHUNK_BITS;
const CHUNK_MASK: usize = CHUNK_ROWS - 1;

/// Rows this short are searched from the left; longer ones by bisection.
const LINEAR_SEARCH_LIMIT: usize = 8;

/// The cells of a worksheet, by row and column. Both are 1-indexed.
///
/// A row can exist without cells: `set_row` with no cells creates one. The xlsx
/// importer does that for rows that carry a height or a style and nothing
/// else, and the exporter writes a `<row>` element for every row that exists
/// here, so such rows survive a round trip. Removing the last cell of a row
/// with `remove_cell` removes the row.
///
/// Rows outside the sheet, below 1 or beyond the last row, cannot hold cells:
/// writing to them does nothing and reading them finds nothing.
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
}

#[derive(Encode, Decode, Debug, Clone)]
struct Chunk {
    /// Always `CHUNK_ROWS` long.
    rows: Vec<Option<Row>>,
    /// How many of them exist. A chunk with none is freed.
    used: usize,
}

/// The cells of a row with their column, sorted by column.
#[derive(Encode, Decode, Debug, Clone, Default, PartialEq)]
struct Row {
    cells: Vec<(i32, Cell)>,
}

impl Chunk {
    fn new() -> Self {
        let mut rows = Vec::new();
        rows.resize_with(CHUNK_ROWS, || None);
        Chunk { rows, used: 0 }
    }
}

impl Row {
    /// Where the column is, or where it would go.
    #[inline]
    fn position(&self, column: i32) -> Result<usize, usize> {
        if self.cells.len() <= LINEAR_SEARCH_LIMIT {
            for (index, (c, _)) in self.cells.iter().enumerate() {
                if *c >= column {
                    return if *c == column { Ok(index) } else { Err(index) };
                }
            }
            Err(self.cells.len())
        } else {
            self.cells.binary_search_by_key(&column, |(c, _)| *c)
        }
    }

    fn insert(&mut self, column: i32, cell: Cell) -> Option<Cell> {
        // Rows are mostly filled from left to right.
        if self.cells.last().is_none_or(|(last, _)| *last < column) {
            self.cells.push((column, cell));
            return None;
        }
        match self.position(column) {
            Ok(index) => Some(std::mem::replace(&mut self.cells[index].1, cell)),
            Err(index) => {
                self.cells.insert(index, (column, cell));
                None
            }
        }
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
        }
        chunk.rows[index].as_mut()
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
        let row = self.row(row)?;
        let index = row.position(column).ok()?;
        Some(&row.cells[index].1)
    }

    /// The cell at a position, to be changed in place.
    #[inline]
    pub fn cell_mut(&mut self, row: i32, column: i32) -> Option<&mut Cell> {
        let row = self.row_mut(row)?;
        let index = row.position(column).ok()?;
        Some(&mut row.cells[index].1)
    }

    /// Puts a cell at a position and returns the one that was there.
    pub fn set_cell(&mut self, row: i32, column: i32, cell: Cell) -> Option<Cell> {
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
        self.row_count == 0
    }

    /// Every cell of the sheet with its row and column, in natural order:
    /// by row, and within a row by column.
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32, &Cell)> {
        self.rows_in_order().flat_map(|(row, row_data)| {
            row_data
                .cells
                .iter()
                .map(move |(column, cell)| (row, *column, cell))
        })
    }

    /// The number of the last row that exists, or 0 for an empty sheet. Cheap: it
    /// looks at whole chunks, so callers can bound whole-column walks by it.
    pub(crate) fn last_row(&self) -> i32 {
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
        self.column_counts.keys().next_back().copied().unwrap_or(0)
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
        self.layout_version = self.layout_version.wrapping_add(1);
    }

    /// Builds the column index from the cells if it is not built yet. One pass over the cells
    /// and a sort, paid once per sheet: afterwards every write keeps it in step.
    pub(crate) fn build_column_index(&mut self) {
        if self.column_rows.is_none() {
            // Collected and sorted, then built in one pass: far cheaper than one insert per cell.
            self.column_rows = Some(self.cells().map(|(row, column, _)| (column, row)).collect());
        }
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
        let Some(index) = &self.column_rows else {
            return false;
        };
        let mut from = column1;
        while from <= column2 {
            let Some(&(column, _)) = index
                .range((from, i32::MIN)..=(column2, i32::MAX))
                .next()
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
    /// in ascending order: one search of the index, then a step per row. None when the index is
    /// not built ([`Self::build_column_index`]).
    pub(crate) fn rows_in_column_between(
        &self,
        column: i32,
        (from, to): (i32, i32),
        limit: usize,
        rows: &mut Vec<i32>,
    ) {
        rows.clear();
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
        self.rows_in_order().map(|(row, _)| row).collect()
    }

    /// The columns of a row that have cells, in ascending order.
    pub fn columns_in_row(&self, row: i32) -> Vec<i32> {
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

    /// The cells of a row with their column, in ascending order of column.
    pub fn cells_in_row(&self, row: i32) -> Vec<(i32, &Cell)> {
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
        // Collected at its exact size: a row grown cell by cell from empty
        // starts with room for four, and a sheet with a million short rows
        // paid for that four times over.
        let mut cells: Vec<(i32, Cell)> = cells.into_iter().collect();
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
mod test {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn number(v: f64) -> Cell {
        Cell::NumberCell { v, s: 0 }
    }

    #[test]
    fn next_row_between_skips_absent_rows_and_chunks() {
        let mut data = SheetData::new();
        data.set_cell(1, 2, number(1.0));
        data.set_cell(10_000, 3, number(1.0));
        let first = data.next_row_between(1, LAST_ROW);
        assert_eq!(
            (first.row, first.absent_chunks, first.slots),
            (Some(1), 0, 1)
        );
        // Rows 2..=1023 of chunk 0, chunks 1..=8 absent, then 785 slots of chunk 9.
        let second = data.next_row_between(2, LAST_ROW);
        assert_eq!(second.row, Some(10_000));
        assert_eq!((second.absent_chunks, second.slots), (8, 1022 + 785));
        // Past the last chunk the search stops: the chunks after it are never looked at.
        let none = data.next_row_between(10_001, LAST_ROW);
        assert_eq!((none.row, none.absent_chunks, none.slots), (None, 0, 239));
        assert_eq!(data.next_row_between(3, 2), RowSearch::default());
        assert_eq!(data.next_row_between(2, 9_999).row, None);
    }

    #[test]
    fn columns_in_row_between_keeps_the_span() {
        let mut data = SheetData::new();
        for column in [1, 3, 5, 7, 9, 11, 13, 15, 17, 19] {
            data.set_cell(4, column, number(1.0));
        }
        let mut columns = vec![99];
        data.columns_in_row_between(4, 4, 15, &mut columns);
        assert_eq!(columns, [5, 7, 9, 11, 13, 15]);
        data.columns_in_row_between(4, 2, 2, &mut columns);
        assert!(columns.is_empty());
        data.columns_in_row_between(5, 1, 20, &mut columns);
        assert!(columns.is_empty());
    }

    #[test]
    fn last_row_spans_chunks() {
        let mut data = SheetData::new();
        assert_eq!(data.last_row(), 0);
        data.set_cell(1, 1, number(1.0));
        assert_eq!(data.last_row(), 1);
        data.set_cell(1024, 1, number(1.0)); // first row of the second chunk
        assert_eq!(data.last_row(), 1024);
        data.set_cell(5000, 3, number(1.0));
        assert_eq!(data.last_row(), 5000);
        data.remove_cell(5000, 3);
        assert_eq!(data.last_row(), 1024);
    }

    #[test]
    fn last_column_survives_clearing_a_wide_sheet() {
        let mut data = SheetData::new();
        for row in 1..=20_000 {
            data.set_cell(row, 3, number(1.0));
            data.set_cell(row, 8, number(1.0));
        }
        data.set_cell(1, 8, number(2.0)); // a replace does not count twice
        for row in 1..=20_000 {
            data.remove_cell(row, 8);
        }
        assert_eq!(data.last_column(), 3);
        for row in 1..=20_000 {
            data.remove_row(row);
        }
        assert_eq!(data.last_column(), 0);
    }

    #[test]
    fn last_column_is_rebuilt_after_decoding() {
        let mut data = SheetData::new();
        data.set_cell(2, 6, number(1.0));
        data.set_cell(9, 3, number(1.0));
        let mut decoded: SheetData = bitcode::decode(&bitcode::encode(&data)).unwrap();
        // the counts are not part of the bytes
        assert_eq!(decoded.last_column(), 0);
        decoded.rebuild_column_counts();
        assert_eq!(decoded.last_column(), 6);
        decoded.remove_cell(2, 6);
        assert_eq!(decoded.last_column(), 3);
    }

    #[test]
    fn the_column_index_follows_every_write() {
        let mut data = SheetData::new();
        let mut rows = Vec::new();
        let mut columns = Vec::new();
        for row in 1..=9_000 {
            data.set_cell(row, 7, number(1.0));
        }
        // not built yet: no columns, and the caller walks by rows
        assert!(!data.columns_between(1, 16_384, 8, &mut columns));
        assert!(columns.is_empty());
        data.build_column_index();
        data.set_cell(3, 2, number(1.0));
        data.set_cell(9_500, 2, number(1.0));
        data.set_cell(2, 3, number(1.0));
        data.rows_in_column_between(2, (1, LAST_ROW), 64, &mut rows);
        assert_eq!(rows, [3, 9_500]);
        data.rows_in_column_between(2, (4, 9_499), 64, &mut rows);
        assert!(rows.is_empty());
        data.rows_in_column_between(7, (10, LAST_ROW), 3, &mut rows);
        assert_eq!(rows, [10, 11, 12]);
        // an empty span is no rows, not a panic
        data.rows_in_column_between(7, (10, 9), 3, &mut rows);
        assert!(rows.is_empty());
        assert!(data.columns_between(1, 16_384, 8, &mut columns));
        assert_eq!(columns, [2, 3, 7]);
        assert!(!data.columns_between(1, 16_384, 2, &mut columns));
        assert_eq!(columns, [2, 3]);
        assert!(data.columns_between(4, 6, 8, &mut columns));
        assert!(columns.is_empty());
        // removing a cell, a row, or replacing a row keeps it in step, and each such write
        // changes the layout version
        let version = data.layout_version();
        data.remove_cell(3, 2);
        assert_ne!(data.layout_version(), version);
        data.set_row(2, vec![(4, number(1.0))]);
        data.remove_row(9_500);
        data.rows_in_column_between(2, (1, LAST_ROW), 64, &mut rows);
        assert!(rows.is_empty());
        assert!(data.columns_between(1, 16_384, 8, &mut columns));
        assert_eq!(columns, [4, 7]);
        // replacing a cell's value does not
        let version = data.layout_version();
        data.set_cell(2, 4, number(2.0));
        assert_eq!(data.layout_version(), version);
    }

    #[test]
    fn last_column_is_the_rightmost_cell() {
        let mut data = SheetData::new();
        assert_eq!(data.last_column(), 0);
        data.set_cell(1, 2, number(1.0));
        data.set_cell(5000, 7, number(1.0));
        data.set_cell(3, 4, number(1.0));
        assert_eq!(data.last_column(), 7);
        data.remove_cell(5000, 7);
        assert_eq!(data.last_column(), 4);
        // removing a whole row, or replacing it, lowers it too
        data.set_cell(9, 12, number(1.0));
        assert_eq!(data.last_column(), 12);
        data.set_row(9, vec![(5, number(1.0))]);
        assert_eq!(data.last_column(), 5);
        data.set_row(9, Vec::new());
        assert_eq!(data.last_column(), 4);
        data.set_cell(9, 12, number(1.0));
        data.remove_row(9);
        assert_eq!(data.last_column(), 4);
        data.remove_row(1);
        data.remove_row(3);
        assert_eq!(data.last_column(), 0);
    }

    #[test]
    fn set_get_replace_remove() {
        let mut data = SheetData::new();
        assert!(data.is_empty());
        assert_eq!(data.set_cell(3, 2, number(1.0)), None);
        assert_eq!(data.set_cell(3, 2, number(2.0)), Some(number(1.0)));
        assert_eq!(data.cell(3, 2), Some(&number(2.0)));
        assert_eq!(data.cell(3, 1), None);
        assert_eq!(data.cell(4, 2), None);
        *data.cell_mut(3, 2).unwrap() = number(5.0);
        assert_eq!(data.remove_cell(3, 2), Some(number(5.0)));
        assert_eq!(data.remove_cell(3, 2), None);
        // the last cell of the row took the row with it
        assert!(data.is_empty());
        assert!(data.rows().is_empty());
    }

    #[test]
    fn natural_order_whatever_the_order_of_insertion() {
        let mut data = SheetData::new();
        // two chunks, and a row longer than the linear search limit, filled
        // from the right
        for column in (1..=20).rev() {
            data.set_cell(2000, column, number(column as f64));
        }
        data.set_cell(5, 7, number(0.0));
        data.set_cell(5, 3, number(0.0));
        data.set_cell(1, 9, number(0.0));
        let positions: Vec<(i32, i32)> = data.cells().map(|(r, c, _)| (r, c)).collect();
        let mut sorted = positions.clone();
        sorted.sort_unstable();
        assert_eq!(positions, sorted);
        assert_eq!(positions.len(), 23);
        assert_eq!(data.rows(), vec![1, 5, 2000]);
        assert_eq!(data.columns_in_row(5), vec![3, 7]);
        assert_eq!(data.cell(2000, 13), Some(&number(13.0)));
        assert_eq!(data.cells_in_row(2000).len(), 20);
        assert!(data.columns_in_row(6).is_empty());
    }

    #[test]
    fn a_row_can_exist_without_cells() {
        let mut data = SheetData::new();
        data.set_row(10, Vec::new());
        assert!(!data.is_empty());
        assert_eq!(data.rows(), vec![10]);
        assert_eq!(data.cells().count(), 0);
        data.remove_row(10);
        assert!(data.is_empty());
    }

    #[test]
    fn set_row_replaces_and_the_last_duplicate_stays() {
        let mut data = SheetData::new();
        data.set_cell(4, 1, number(1.0));
        data.set_row(
            4,
            vec![(5, number(5.0)), (2, number(2.0)), (5, number(6.0))],
        );
        assert_eq!(data.columns_in_row(4), vec![2, 5]);
        assert_eq!(data.cell(4, 5), Some(&number(6.0)));
        assert_eq!(data.cell(4, 1), None);
    }

    #[test]
    fn rows_outside_the_sheet_hold_nothing() {
        let mut data = SheetData::new();
        for row in [0, -3, LAST_ROW + 1, i32::MAX, i32::MIN] {
            assert_eq!(data.set_cell(row, 1, number(1.0)), None);
            data.set_row(row, vec![(1, number(1.0))]);
            assert_eq!(data.cell(row, 1), None);
            data.remove_row(row);
        }
        assert!(data.is_empty());
        // the first and the last row of the sheet are fine
        data.set_cell(1, 1, number(1.0));
        data.set_cell(LAST_ROW, 1, number(2.0));
        assert_eq!(data.rows(), vec![1, LAST_ROW]);
    }

    #[test]
    fn equality_is_about_cells_not_about_chunks() {
        let mut a = SheetData::new();
        let mut b = SheetData::new();
        a.set_cell(1, 1, number(1.0));
        b.set_cell(1, 1, number(1.0));
        // `a` has had a cell far away, and no longer has
        a.set_cell(900_000, 4, number(9.0));
        a.remove_cell(900_000, 4);
        assert_eq!(a, b);
        b.set_cell(1, 2, number(1.0));
        assert_ne!(a, b);
    }
}
