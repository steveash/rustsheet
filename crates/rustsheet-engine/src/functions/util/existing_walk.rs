//! The walk of an area over the cells that exist only, for the areas of a union and for a
//! single area that a far cell leaves mostly blank ([`Model::for_each_position_in_area`]):
//! it costs the data, not the area. A union's areas draw on the formula's walk budget (T-1497,
//! T-1502); a single area does not (T-1690).

use crate::{
    area_budget::SLOTS_PER_WALK_STEP,
    calc_result::CalcResult,
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

/// Rows of one column an indexed walk reads from the index at a time: one search, then a step per
/// row, so a dense column costs a read step per this many cells.
const INDEX_BATCH_ROWS: usize = 64;

/// Fewest positions a single area clipped to the used area must hold to be walked over its
/// existing cells instead of position by position ([`Model::walks_sparsely`]): a smaller area
/// costs little either way, and keeps the one-by-one record of its blank cells, which spends
/// nothing from the formula's seen-range cap (a LAMBDA may walk one small area per row).
const MIN_SPARSE_WALK_POSITIONS: i64 = 1 << 16;

/// How many positions per cell its columns hold a single area must have to be walked over its
/// existing cells: a denser one is cheaper walked position by position, which tests no row slots
/// and searches no rows.
const SPARSE_WALK_POSITIONS_PER_CELL: i64 = 4;

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
    /// ([`Self::for_each_existing_position`]) rather than position by position: it holds at least
    /// [`MIN_SPARSE_WALK_POSITIONS`] positions, more than [`SPARSE_WALK_POSITIONS_PER_CELL`] per
    /// cell its columns hold, and the formula can still record the area as read empty in one
    /// rectangle.
    pub(super) fn walks_sparsely(
        &self,
        sheet: u32,
        left: &CellReferenceIndex,
        (row2, column2): (i32, i32),
    ) -> bool {
        let positions =
            i64::from(row2 - left.row + 1).max(0) * i64::from(column2 - left.column + 1).max(0);
        if positions < MIN_SPARSE_WALK_POSITIONS || !self.area_budget.has_seen_ranges(1) {
            return false;
        }
        let Ok(worksheet) = self.workbook.worksheet(sheet) else {
            return false;
        };
        let cells = worksheet.sheet_data.cells_in_columns(left.column, column2);
        let cells = i64::try_from(cells).unwrap_or(i64::MAX);
        positions / SPARSE_WALK_POSITIONS_PER_CELL > cells
    }

    /// [`Self::spend_area_walk`] for a walk drawn from the formula's budget (`budgeted`): the
    /// areas of a union. A single area walked here because it is mostly blank is not: its walk
    /// costs less than the dense one it replaces, which was never charged, so charging it could
    /// only turn a value into `#VALUE!` (a LAMBDA repeating `SUM(A:Z)` over 100,000 cells).
    fn spend_walk_if(
        &mut self,
        budgeted: bool,
        steps: u64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        if budgeted {
            self.spend_area_walk(steps, cell)
        } else {
            Ok(())
        }
    }

    /// The union-area walk of [`Self::for_each_position_in_area`] on one sheet: the existing
    /// cells of the rectangle `left..=right`, in row-major order like the dense walk, so the
    /// first error cell is the same one. When `budgeted` (a union's area), the area, every chunk
    /// and row slot the search looks at and every cell it hands to `f` are paid for before `f`
    /// runs ([`Self::spend_walk_if`]). An area whose cells lie in at
    /// most [`MAX_INDEXED_COLUMNS`] columns is walked through the column index instead
    /// ([`Self::for_each_indexed_position`]), so the rows other columns fill cost it nothing.
    pub(super) fn for_each_existing_position<F>(
        &mut self,
        sheet: u32,
        (left, right): (&CellReferenceIndex, &CellReferenceIndex),
        cell: CellReferenceIndex,
        columns: &mut Vec<i32>,
        budgeted: bool,
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
        // a LAMBDA repeating a union over an empty sheet would grow the record without bound.
        self.spend_walk_if(budgeted, 1, cell)?;
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
            self.spend_walk_if(budgeted, (columns.len() as u64).max(1), cell)?;
            if few {
                return self.for_each_indexed_position(
                    sheet,
                    (left, right),
                    cell,
                    columns,
                    budgeted,
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
            self.spend_walk_if(budgeted, steps, cell)?;
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
            self.spend_walk_if(budgeted, (columns.len() as u64).max(1), cell)?;
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
        columns: &[i32],
        budgeted: bool,
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
            self.spend_walk_if(budgeted, reads, cell)?;
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
            self.spend_walk_if(budgeted, row_columns.len().max(merge) as u64, cell)?;
            for &column in &row_columns {
                f(self, CellReferenceIndex { sheet, row, column })?;
            }
            from = row + 1;
        }
        Ok(())
    }
}
