//! What was read of a sheet's cells while a record was on ([`SheetData::start_reads`]), so the
//! data tables can tell whether an edit could change their results (T-0453, `data_table_memo.rs`).
//!
//! The record is kept by [`SheetData`] itself, because every read of a cell goes through it:
//! nothing that reads a cell can be left out. It is a superset of what a formula depends on.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::sheet_data::SheetData;

/// Positions and rectangles one record may hold (CLAUDE.md directive 5): past them the record
/// overflows, and its reader may not rely on it.
const MAX_RECORDED_READS: usize = 1 << 18;

/// A rectangle of a sheet, `(row1, column1, row2, column2)`, corners included.
pub(crate) type Area = (i32, i32, i32, i32);

/// What was read of one sheet while the record was on.
#[derive(Debug, Clone, Default)]
pub(crate) struct SheetReads {
    /// Positions read one at a time.
    pub(crate) cells: HashSet<(i32, i32)>,
    /// Rectangles whose cells were looked for: an empty position in one was read as empty.
    pub(crate) areas: HashSet<Area>,
    /// Whether the extent of the sheet (its last row or column) was read.
    pub(crate) extent: bool,
    /// More was read than the record holds.
    pub(crate) overflow: bool,
}

impl SheetReads {
    fn note_cell(&mut self, row: i32, column: i32) {
        if !self.overflow && !self.cells.contains(&(row, column)) {
            self.overflow = self.cells.len() + self.areas.len() >= MAX_RECORDED_READS;
            if !self.overflow {
                self.cells.insert((row, column));
            }
        }
    }

    fn note_area(&mut self, area: Area) {
        if !self.overflow && !self.areas.contains(&area) {
            self.overflow = self.cells.len() + self.areas.len() >= MAX_RECORDED_READS;
            if !self.overflow {
                self.areas.insert(area);
            }
        }
    }
}

/// The record a [`SheetData`] keeps while it is on. Reads take `&SheetData`, hence the interior
/// mutability, and a `Model` must stay `Sync` (the Python bindings share it), hence an atomic and
/// a mutex rather than `Cell`s; `on` keeps the cost of a read while it is off to one load.
#[derive(Debug, Default)]
pub(crate) struct ReadLog {
    on: AtomicBool,
    reads: Mutex<SheetReads>,
}

/// A copy of a sheet records nothing until it is asked to.
impl Clone for ReadLog {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl ReadLog {
    #[inline]
    fn note(&self, f: impl FnOnce(&mut SheetReads)) {
        if self.on.load(Ordering::Relaxed) {
            match self.reads.lock() {
                Ok(mut reads) => f(&mut reads),
                // A poisoned record cannot be trusted: its reader recomputes.
                Err(poisoned) => poisoned.into_inner().overflow = true,
            }
        }
    }

    #[inline]
    pub(crate) fn cell(&self, row: i32, column: i32) {
        self.note(|reads| reads.note_cell(row, column));
    }

    #[inline]
    pub(crate) fn area(&self, area: Area) {
        self.note(|reads| reads.note_area(area));
    }

    /// Every position of rows `row1..=row2`.
    #[inline]
    pub(crate) fn rows(&self, row1: i32, row2: i32) {
        self.area((row1, 1, row2, LAST_COLUMN));
    }

    /// Every position of columns `column1..=column2`.
    #[inline]
    pub(crate) fn columns(&self, column1: i32, column2: i32) {
        self.area((1, column1, LAST_ROW, column2));
    }

    #[inline]
    pub(crate) fn extent(&self) {
        self.note(|reads| reads.extent = true);
    }
}

impl SheetData {
    /// Starts a new record of what is read of the cells ([`Self::take_reads`]).
    pub(crate) fn start_reads(&self) {
        let fresh = SheetReads::default();
        match self.reads.reads.lock() {
            Ok(mut reads) => *reads = fresh,
            Err(poisoned) => *poisoned.into_inner() = fresh,
        }
        self.reads.on.store(true, Ordering::Relaxed);
    }

    /// Stops the record and returns it; `None` if none was on.
    pub(crate) fn take_reads(&self) -> Option<SheetReads> {
        if !self.reads.on.swap(false, Ordering::Relaxed) {
            return None;
        }
        let reads = match self.reads.reads.lock() {
            Ok(mut reads) => std::mem::take(&mut *reads),
            Err(poisoned) => std::mem::take(&mut *poisoned.into_inner()),
        };
        Some(reads)
    }
}
