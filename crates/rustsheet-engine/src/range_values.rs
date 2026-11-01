//! The values of a range read once per pass (T-0449, moved here by T-3534): a range whose cells
//! can all be read without evaluating anything has the same values for the rest of the pass, so
//! its readers share one copy. The criteria aggregates (`ifs_cache.rs`), SUMPRODUCT's masked
//! blocks (T-3230), operator range operands (`evaluate_range`, T-3471) and the running totals
//! (`sum_prefix.rs`, T-3456) read through it.
//!
//! A range is cached only when reading it evaluates nothing (`Model::read_is_inert`): every formula
//! cell of it is already evaluated in this pass (spill cells: their anchor), and none is circular
//! or, under iterative calculation, a cycle member. Until then its readers walk it as before, and
//! the check resumes where it stopped (an evaluated cell stays evaluated for the rest of the pass),
//! so it costs each range one scan per pass. The first read of a cached range records what it saw,
//! so a spill that later writes into it restarts the pass; every pass starts with an empty cache,
//! and nothing is cached where `pass_cache_safe` is false. A block inside a cached range with the
//! same columns (a growing prefix `I$11:I11`) is read from that range.
//!
//! Directive 5: ranges between [`MIN_CACHED_CELLS`] and [`MAX_CACHED_CELLS`] cells are cached, and
//! everything the pass caches derive from cached values (`ifs_cache.rs`'s indexes, block numbers
//! and gathered values) is charged to the same budget of [`MAX_CACHED_CELLS`] cells.

use std::collections::HashMap;
use std::sync::Arc;

use crate::calc_result::{CalcResult, Range};
use crate::evaluation::{key, CellState, FormulaCell, KeyBuild};
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::Cell;

/// Ranges with fewer cells are cheaper to walk again than to cache.
const MIN_CACHED_CELLS: usize = 64;

/// Cells (values, not bytes) all cached ranges of one pass may hold together; also the largest
/// range cached.
pub(crate) const MAX_CACHED_CELLS: usize = 4_000_000;

/// Cached ranges per column span that a block inside them is read from (a bound on the search).
const MAX_PARENTS: usize = 16;

/// A block of one sheet: sheet, top row, left column, bottom row, right column.
pub(crate) type RectKey = (u32, i32, i32, i32, i32);

/// A range's place in the cache (or, in `ifs_cache.rs`, its place at the cells a chain of
/// criteria matches).
pub(crate) enum RangeState {
    /// Its values, row by row.
    Ready(Arc<[CalcResult]>),
    /// Not cacheable yet: the cells before `checked` (row by row) can be read without
    /// evaluating anything; the cell at `checked` could not when last looked at.
    Waiting { checked: usize },
    /// Never cacheable in this pass (a circular cell, or over the budget).
    Never,
}

/// The cells of a block read from a cached range: `values[offset..offset + len]`, where the
/// cached range is `rect`.
pub(crate) struct RangeView {
    pub(crate) rect: RectKey,
    pub(crate) values: Arc<[CalcResult]>,
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

impl RangeView {
    /// The block's `index`-th value, row by row.
    pub(crate) fn get(&self, index: usize) -> Option<&CalcResult> {
        if index < self.len {
            self.values.get(self.offset + index)
        } else {
            None
        }
    }
}

/// The cached ranges of the current pass, and the cells budget every pass cache of derived values
/// draws on.
#[derive(Default)]
pub(crate) struct RangeValues {
    ranges: HashMap<RectKey, RangeState, KeyBuild>,
    /// The cached ranges of each (sheet, left column, right column), as (top row, bottom row).
    parents: HashMap<(u32, i32, i32), Vec<(i32, i32)>>,
    /// Cells charged in this pass, of at most [`MAX_CACHED_CELLS`].
    cells: usize,
}

impl RangeValues {
    /// Forgets everything: a new pass, or cells whose state was reset.
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
        self.parents.clear();
        self.cells = 0;
    }

    /// Charges `cells` to the budget, when they fit; false, charging nothing, when they do not.
    pub(crate) fn charge(&mut self, cells: usize) -> bool {
        match self.cells.checked_add(cells) {
            Some(total) if total <= MAX_CACHED_CELLS => {
                self.cells = total;
                true
            }
            _ => false,
        }
    }

    /// Whether `cells` more fit in the budget.
    pub(crate) fn fits(&self, cells: usize) -> bool {
        self.cells.saturating_add(cells) <= MAX_CACHED_CELLS
    }

    /// How many ranges are cached, and how many wait to be, for the tests of T-0498.
    #[cfg(test)]
    pub(crate) fn counts(&self) -> (usize, usize) {
        let count = |wanted: fn(&RangeState) -> bool| {
            self.ranges.values().filter(|state| wanted(state)).count()
        };
        (
            count(|state| matches!(state, RangeState::Ready(_))),
            count(|state| matches!(state, RangeState::Waiting { .. })),
        )
    }
}

/// The block `range` covers.
pub(crate) fn rect_of(range: &Range) -> RectKey {
    (
        range.left.sheet,
        range.left.row,
        range.left.column,
        range.right.row,
        range.right.column,
    )
}

/// The position of the `index`-th cell of `rect`, counting row by row.
pub(crate) fn rect_cell(rect: RectKey, index: usize) -> CellReferenceIndex {
    let (sheet, top, left, _, right) = rect;
    let width = usize::try_from(right - left + 1).unwrap_or(1).max(1);
    CellReferenceIndex {
        sheet,
        row: top + i32::try_from(index / width).unwrap_or(0),
        column: left + i32::try_from(index % width).unwrap_or(0),
    }
}

/// How many cells `rect` holds.
fn rect_size(rect: RectKey) -> usize {
    let (_, top, left, bottom, right) = rect;
    let rows = usize::try_from(bottom - top + 1).unwrap_or(0);
    let columns = usize::try_from(right - left + 1).unwrap_or(0);
    rows.saturating_mul(columns)
}

impl Model {
    /// Whether reading `at` evaluates nothing and marks nothing: a constant or a blank, a formula
    /// cell already evaluated in this pass, or a spill cell whose anchor is. `None` for a
    /// circular cell or a cycle member under iterative calculation (whose reader joins it, where
    /// the lazy walk may not read it), which never becomes readable this way.
    pub(crate) fn read_is_inert(&self, at: CellReferenceIndex) -> Option<bool> {
        if self.evaluation.circular.contains(&key(at)) || self.is_cycle_member(at) {
            return None;
        }
        let evaluated = |at: CellReferenceIndex| {
            self.evaluation.cells.get(&key(at)) == Some(&CellState::Evaluated)
        };
        Some(match self.fetch_cell(at) {
            None => true,
            Some(Cell::SpillCell { a, .. }) => {
                let anchor = CellReferenceIndex {
                    sheet: at.sheet,
                    row: a.0,
                    column: a.1,
                };
                if self.is_cycle_member(anchor) {
                    return None;
                }
                matches!(self.fetch_cell(anchor), Some(Cell::ArrayFormula { .. }))
                    && evaluated(anchor)
            }
            Some(cell) => FormulaCell::of(cell).is_none() || evaluated(at),
        })
    }

    /// The values of `rect` row by row, when they can be read without evaluating anything (module
    /// docs), from a cached range that holds it when there is one; `None` when the caller must
    /// walk.
    pub(crate) fn range_values(&mut self, rect: RectKey) -> Option<RangeView> {
        if !self.pass_cache_safe() {
            return None;
        }
        let size = rect_size(rect);
        if !(MIN_CACHED_CELLS..=MAX_CACHED_CELLS).contains(&size) {
            return None;
        }
        let whole = |rect: RectKey, values: &Arc<[CalcResult]>| RangeView {
            rect,
            values: values.clone(),
            offset: 0,
            len: size,
        };
        let (sheet, top, left, bottom, right) = rect;
        let cache = &self.evaluation.range_values;
        let parent = cache
            .parents
            .get(&(sheet, left, right))
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find(|&&(first, last)| first <= top && bottom <= last)
            })
            .map(|&(first, last)| (sheet, first, left, last, right));
        if let Some(parent) = parent {
            if let Some(RangeState::Ready(values)) = cache.ranges.get(&parent) {
                let rows_above = usize::try_from(top - parent.1).unwrap_or(0);
                let width = usize::try_from(right - left + 1).unwrap_or(1);
                return Some(RangeView {
                    offset: rows_above * width,
                    ..whole(parent, values)
                });
            }
        }
        let checked = match cache.ranges.get(&rect) {
            Some(RangeState::Ready(values)) => return Some(whole(rect, values)),
            Some(RangeState::Never) => return None,
            Some(RangeState::Waiting { checked }) => *checked,
            None => 0,
        };
        for index in checked..size {
            let state = match self.read_is_inert(rect_cell(rect, index)) {
                Some(true) => continue,
                Some(false) => RangeState::Waiting { checked: index },
                None => RangeState::Never,
            };
            self.evaluation.range_values.ranges.insert(rect, state);
            return None;
        }
        if !self.evaluation.range_values.fits(size) {
            self.evaluation
                .range_values
                .ranges
                .insert(rect, RangeState::Never);
            return None;
        }
        // Every read is inert: it records what it saw, as the walk's first read does.
        let values: Arc<[CalcResult]> = (0..size)
            .map(|index| self.evaluate_cell(rect_cell(rect, index)))
            .collect();
        if !self.pass_cache_safe() {
            return None;
        }
        let cache = &mut self.evaluation.range_values;
        if !cache.charge(size) {
            return None;
        }
        cache.ranges.insert(rect, RangeState::Ready(values.clone()));
        let blocks = cache.parents.entry((sheet, left, right)).or_default();
        if blocks.len() < MAX_PARENTS {
            blocks.push((top, bottom));
        }
        Some(whole(rect, &values))
    }
}
