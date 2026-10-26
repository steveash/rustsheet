//! The walks of the cells that exist in a rectangle of one sheet: the union-area walk of T-1497
//! ([`Model::for_each_existing_position`], through the column index when the cells lie in few
//! columns), the *IF(S) criteria walks built on it ([`Model::stored_walk`], T-2711), and the
//! walk of a single aggregate range that a far cell leaves mostly blank ([`Model::walks_sparsely`],
//! T-1690).

use crate::{
    area_budget::SLOTS_PER_WALK_STEP,
    calc_result::{CalcResult, Range},
    expressions::{token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Most columns holding cells that a union area may have to be walked through the column index
/// ([`Model::for_each_indexed_position`]); a wider one is walked by rows. The merge compares the
/// next row of every column still being merged for each row it hands on, and a row is charged at
/// least a step per four of them, so with this few columns the comparisons stay a small part of
/// a row's cost.
const MAX_INDEXED_COLUMNS: usize = 16;

/// Fewest rows of the sheet a union area must span to be walked through the column index
/// ([`Model::for_each_existing_position`]): one chunk of row storage.
const MIN_INDEXED_ROWS: i32 = 1024;

/// Most positions per stored cell of its columns a [`Model::for_each_stored_position`] walk, or a
/// single aggregate range ([`Model::walks_sparsely`]), may visit densely: past it the block is
/// mostly blank, and walking the cells that exist is cheaper.
const DENSE_WALK_FACTOR: u64 = 4;

/// Fewest positions a single area clipped to the used area must hold to be walked over its
/// existing cells instead of position by position ([`Model::walks_sparsely`]): a smaller area
/// costs little either way, and keeps the one-by-one record of its blank cells, which spends
/// nothing from the formula's seen-range cap (a LAMBDA may walk one small area per row).
const MIN_SPARSE_WALK_POSITIONS: u64 = 1 << 16;

/// Rows of one column an indexed walk reads from the index at a time: one search, then a step per
/// row, so a dense column costs a read step per this many cells.
const INDEX_BATCH_ROWS: usize = 64;

/// Positions from `first` to `last` along one side of an area, 0 when `last` is before `first`.
pub(super) fn span(first: i32, last: i32) -> u64 {
    u64::try_from(i64::from(last) - i64::from(first) + 1).unwrap_or(0)
}

/// Whether a walk of existing cells ([`Model::for_each_existing_position`]) draws on the formula's
/// union walk budget ([`MAX_AREA_WALK`](crate::area_budget::MAX_AREA_WALK)).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum WalkBudget {
    /// A union area: nothing else bounds how many a formula walks (T-1497).
    Charged,
    /// A single range of a caller that bounds its own walks (the *IF(S) criteria walks, priced
    /// by the lift budget), and that must not fail where the dense walk it replaces did not; so
    /// is a mostly blank aggregate range ([`Model::walks_sparsely`]), whose walk costs less than
    /// the dense one, which was never charged (a LAMBDA repeating `SUM(A:Z)` over 100,000 cells).
    Free,
}

/// How [`Model::stored_walk`] walks a range to visit the cells stored in it.
#[derive(Clone, Copy)]
pub(crate) enum StoredWalk {
    /// Every position of the range's top-left cell to `(row2, column2)`, its block within the
    /// used area (none when the corner is above or left of it).
    Dense { row2: i32, column2: i32 },
    /// The cells that exist only.
    Sparse,
}

/// The rows of one column of an indexed walk that were read from the index, from `next` on not
/// yet handed on.
struct ColumnBatch {
    column: i32,
    rows: Vec<i32>,
    next: usize,
}

impl ColumnBatch {
    fn head(&self) -> Option<i32> {
        self.rows.get(self.next).copied()
    }

    /// Whether the index may hold rows past the ones read: the read was full and is used up.
    fn needs_read(&self) -> bool {
        self.next == self.rows.len() && self.rows.len() == INDEX_BATCH_ROWS
    }
}

impl Model {
    /// Whether the area from `left` to the corner `(row2, column2)` on `sheet`, a single area
    /// already clipped to the used area, is walked over its existing cells
    /// ([`Self::for_each_existing_position`], [`WalkBudget::Free`]) rather than position by
    /// position: it holds at least [`MIN_SPARSE_WALK_POSITIONS`] positions, more than
    /// [`DENSE_WALK_FACTOR`] per cell its columns hold (T-1690). The area is put on record as
    /// read empty in one rectangle, also once the formula's seen-range cap is spent, when the
    /// formula is `#VALUE!` ([`MAX_SEEN_RANGES`](crate::area_budget::MAX_SEEN_RANGES)): the dense
    /// walk it would fall back to costs every position of the area and a record per blank one,
    /// unbounded in time and memory (directive 5, T-2710).
    pub(super) fn walks_sparsely(
        &self,
        sheet: u32,
        left: &CellReferenceIndex,
        (row2, column2): (i32, i32),
    ) -> bool {
        let positions = span(left.row, row2) * span(left.column, column2);
        if positions < MIN_SPARSE_WALK_POSITIONS {
            return false;
        }
        let Ok(worksheet) = self.workbook.worksheet(sheet) else {
            return false;
        };
        let cells = worksheet.sheet_data.cells_in_columns(left.column, column2);
        positions > cells.saturating_mul(DENSE_WALK_FACTOR)
    }

    /// [`Self::spend_area_walk`] for a [`WalkBudget::Charged`] walk; a free walk spends nothing.
    fn spend_walk(
        &mut self,
        budget: WalkBudget,
        steps: u64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        match budget {
            WalkBudget::Charged => self.spend_area_walk(steps, cell),
            WalkBudget::Free => Ok(()),
        }
    }

    /// How to walk `range` (one sheet) so as to visit every cell stored in it, in row-major
    /// order, and maybe some blank positions too, for a caller that reads a blank position as
    /// blank: when the range's block within the used area is at most [`DENSE_WALK_FACTOR`] times
    /// the cells stored in its columns (plus [`MIN_INDEXED_ROWS`]), every position of that block
    /// ([`StoredWalk::Dense`], the cheaper walk of well-filled data); otherwise only the cells that
    /// exist ([`StoredWalk::Sparse`], [`Self::for_each_existing_cell`]). Either way the work is at
    /// most a few times the cells stored in the range's columns plus the rows or chunks searched,
    /// whatever the range's or the used area's extent (T-2711). The whole range is put on record
    /// as read empty in one piece, so a spill into it restarts the pass.
    pub(crate) fn stored_walk(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
    ) -> Result<StoredWalk, CalcResult> {
        let Range { left, right } = range;
        let sheet = left.sheet;
        let sheet_error = |message| CalcResult::new_error(Error::ERROR, cell, message);
        let (row2, column2) = self
            .clip_block_to_used_area(sheet, left.row, left.column, right.row, right.column)
            .map_err(sheet_error)?;
        let area = u64::try_from(row2 - left.row + 1).unwrap_or(0)
            * u64::try_from(column2 - left.column + 1).unwrap_or(0);
        let stored = self
            .workbook
            .worksheet(sheet)
            .map_err(sheet_error)?
            .sheet_data
            .cells_in_columns(left.column, right.column);
        if area <= stored.saturating_mul(DENSE_WALK_FACTOR) + MIN_INDEXED_ROWS as u64 {
            self.record_seen_empty_range(sheet, left.row, left.column, right.row, right.column);
            Ok(StoredWalk::Dense { row2, column2 })
        } else {
            Ok(StoredWalk::Sparse)
        }
    }

    /// The [`StoredWalk::Sparse`] walk of `range`: calls `f` with the position of every cell that
    /// exists in it, through [`Self::for_each_existing_position`], which records the range as read
    /// empty. It does not draw on the union walk budget ([`WalkBudget::Free`]): its callers price
    /// their walks.
    pub(crate) fn for_each_existing_cell<F>(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let mut columns = Vec::new();
        self.for_each_existing_position(
            range.left.sheet,
            (&range.left, &range.right),
            cell,
            (&mut columns, WalkBudget::Free),
            f,
        )
    }

    /// Calls `f`, in row-major order, with every position of the walk [`Self::stored_walk`]
    /// chooses for `range`: every cell stored in it, and maybe some blank positions too.
    pub(crate) fn for_each_stored_position<F>(
        &mut self,
        range: &Range,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        match self.stored_walk(range, cell)? {
            StoredWalk::Dense { row2, column2 } => {
                let sheet = range.left.sheet;
                for row in range.left.row..=row2 {
                    for column in range.left.column..=column2 {
                        f(self, CellReferenceIndex { sheet, row, column })?;
                    }
                }
                Ok(())
            }
            StoredWalk::Sparse => self.for_each_existing_cell(range, cell, f),
        }
    }

    /// The union-area walk of [`Self::for_each_position_in_area`] on one sheet: the existing
    /// cells of the rectangle `left..=right`, in row-major order like the dense walk, so the
    /// first error cell is the same one. A [`WalkBudget::Charged`] walk pays for the area, every
    /// chunk and row slot the search looks at and every cell it hands to `f` before `f` runs. An
    /// area whose cells lie in at most [`MAX_INDEXED_COLUMNS`] columns is walked through the
    /// column index instead ([`Self::for_each_indexed_position`]), so the rows other columns fill
    /// cost it nothing.
    pub(super) fn for_each_existing_position<F>(
        &mut self,
        sheet: u32,
        (left, right): (&CellReferenceIndex, &CellReferenceIndex),
        cell: CellReferenceIndex,
        (columns, budget): (&mut Vec<i32>, WalkBudget),
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let sheet_error = |message| CalcResult::new_error(Error::ERROR, cell, message);
        // The positions not handed to `f` are empty. Recording the whole rectangle covers the
        // existing cells too, which is harmless: a spill can only write where nothing but its
        // own spill cells stands, and those read as empty until it has run.
        // The area itself is a step: its record must be paid for even when it finds nothing, or
        // a LAMBDA repeating a union over an empty sheet would grow the record without bound. A
        // free single area is not charged: a mostly blank aggregate range holds at least
        // 65,536 positions ([`Self::walks_sparsely`]), so its record takes one rectangle of the
        // seen-range cap per such range, and a rectangle already on record is free.
        self.spend_walk(budget, 1, cell)?;
        self.record_seen_empty_range(sheet, left.row, left.column, right.row, right.column);
        let data = &mut self
            .workbook
            .worksheet_mut(sheet)
            .map_err(sheet_error)?
            .sheet_data;
        // Only an area over more rows of the sheet than one chunk gains from the index: the row
        // walk of a shorter one is a chunk's slots at most. Such an area builds the index, once
        // per sheet: a pass over its cells, after which every write keeps it in step. Finding
        // the area's columns is then a search of the index per column found (one when none).
        if right.row.min(data.last_row()) - left.row >= MIN_INDEXED_ROWS {
            data.build_column_index();
            let few = data.columns_between(left.column, right.column, MAX_INDEXED_COLUMNS, columns);
            self.spend_walk(budget, (columns.len() as u64).max(1), cell)?;
            if few {
                return self.for_each_indexed_position(
                    sheet,
                    (left, right),
                    cell,
                    (columns, budget),
                    f,
                );
            }
        }
        // Slots are paid for in whole steps as they add up, so a dense column costs a step per
        // cell and one more per `SLOTS_PER_WALK_STEP` rows.
        let mut slots: u64 = 0;
        let mut from = left.row;
        while from <= right.row {
            let search = self
                .workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data
                .next_row_between(from, right.row);
            let paid = slots.div_ceil(SLOTS_PER_WALK_STEP);
            slots += search.slots;
            let steps = search.absent_chunks + (slots.div_ceil(SLOTS_PER_WALK_STEP) - paid);
            self.spend_walk(budget, steps, cell)?;
            let Some(row) = search.row else {
                break;
            };
            self.workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data
                .columns_in_row_between(row, left.column, right.column, columns);
            // A row costs a step even with no cell in the area's columns: finding them is a
            // search of the row.
            self.spend_walk(budget, (columns.len() as u64).max(1), cell)?;
            for &column in columns.iter() {
                f(self, CellReferenceIndex { sheet, row, column })?;
            }
            from = row + 1;
        }
        Ok(())
    }

    /// The walk of [`Self::for_each_existing_position`] through the column index, for an area
    /// whose cells lie in `columns` (ascending, at most [`MAX_INDEXED_COLUMNS`] of them): each
    /// column's rows are read from the index [`INDEX_BATCH_ROWS`] at a time and merged in
    /// row-major order, so only the area's own cells are visited. A read is a step and every
    /// cell handed to `f` is a step (a row at least one per four columns still being merged), paid
    /// before `f` runs. `f` may write to the sheet (a spill
    /// anchored in the area); each of `columns` is then read again from the next row on, a step
    /// each, as the row walk finds each next row afresh. The set of columns is fixed when the walk
    /// starts: a spill into a column of the area that held no cell is not walked, and is caught
    /// by the seen-empty record of the area, which restarts the pass.
    fn for_each_indexed_position<F>(
        &mut self,
        sheet: u32,
        (left, right): (&CellReferenceIndex, &CellReferenceIndex),
        cell: CellReferenceIndex,
        (columns, budget): (&[i32], WalkBudget),
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, CellReferenceIndex) -> Result<(), CalcResult>,
    {
        let sheet_error = |message| CalcResult::new_error(Error::ERROR, cell, message);
        let mut batches: Vec<ColumnBatch> = columns
            .iter()
            .map(|&column| ColumnBatch {
                column,
                rows: Vec::new(),
                next: 0,
            })
            .collect();
        let mut row_columns = Vec::with_capacity(columns.len());
        let mut version = None;
        let mut from = left.row;
        loop {
            let data = &self
                .workbook
                .worksheet(sheet)
                .map_err(sheet_error)?
                .sheet_data;
            let stale = version != Some(data.layout_version());
            version = Some(data.layout_version());
            let mut reads = 0;
            for batch in &mut batches {
                if stale || batch.needs_read() {
                    data.rows_in_column_between(
                        batch.column,
                        (from, right.row),
                        INDEX_BATCH_ROWS,
                        &mut batch.rows,
                    );
                    batch.next = 0;
                    reads += 1;
                }
            }
            self.spend_walk(budget, reads, cell)?;
            // A column with no rows left is done (a write by `f` into it is caught by the
            // area's read-empty record, as for a column that held none at the start).
            batches.retain(|batch| batch.head().is_some());
            let Some(row) = batches.iter().filter_map(ColumnBatch::head).min() else {
                break;
            };
            row_columns.clear();
            for batch in &mut batches {
                if batch.head() == Some(row) {
                    row_columns.push(batch.column);
                    batch.next += 1;
                }
            }
            // A row is a step per cell handed on, and at least a step per four columns still
            // being merged, which each row compares.
            let merge = batches.len().div_ceil(4);
            self.spend_walk(budget, row_columns.len().max(merge) as u64, cell)?;
            for &column in &row_columns {
                f(self, CellReferenceIndex { sheet, row, column })?;
            }
            from = row + 1;
        }
        Ok(())
    }
}
