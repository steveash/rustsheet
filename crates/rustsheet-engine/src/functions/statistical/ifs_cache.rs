//! The per-pass cache of the criteria aggregates (T-0449). SUMIFS, COUNTIFS and their family
//! walked every cell of every criteria range on every call: 2,000 SUMIFS and 2,000 COUNTIFS over
//! 50,000 rows tested 4 * 10^8 criteria. Within one pass a range's cells, once all evaluated,
//! cannot change, so their values are read once, and each criterion's matching cells are found
//! once per range, as a bit set; a call then intersects its criteria's sets and walks the matches.
//!
//! The walks this replaces read lazily: a criteria range's cell is only read when the criteria
//! before it matched, and a sum range's cell only when all of them did, and reading a formula cell
//! evaluates it. A cache that read every cell could evaluate a cell the walk never reaches, closing
//! a cycle or marking a reader circular where the walk would not. So a range is cached only when
//! reading it evaluates nothing: every formula cell of it is already evaluated in this pass
//! (spill cells: their anchor), and none is circular. Until then the walk runs as before, and the
//! check resumes where it stopped (an evaluated cell stays evaluated for the rest of the pass), so
//! it costs each range one scan per pass.
//!
//! The rest of the argument is `search_cache.rs`'s (T-0448): the first read of a cached range
//! records what it saw, so a spill that later writes into it restarts the pass, and every pass
//! starts with an empty cache; nothing is cached under iterative calculation, Excel-order passes
//! or outside a pass. Values read through the cache are the walk's values, and the matches are
//! taken in the walk's order (row by row), so sums round alike and the first error wins alike.
//!
//! Directive 5: ranges between [`MIN_CACHED_CELLS`] and [`MAX_CACHED_CELLS`] cells are cached;
//! all cached values of a pass hold at most [`MAX_CACHED_CELLS`] cells and all match sets at most
//! [`MAX_CACHED_BITS`] bits; past that the walks run as before.

use std::collections::HashMap;
use std::mem::Discriminant;
use std::sync::Arc;

use crate::calc_result::{CalcResult, Range};
use crate::evaluation::{key, CellState, FormulaCell, KeyBuild};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::Cell;

use super::if_ifs::Criterion;

/// Ranges with fewer cells are cheaper to walk again than to cache.
const MIN_CACHED_CELLS: usize = 64;

/// Cells all cached ranges of one pass may hold together; also the largest range cached.
const MAX_CACHED_CELLS: usize = 4_000_000;

/// Bits all match sets of one pass may hold together (4 MB).
const MAX_CACHED_BITS: usize = 32_000_000;

/// A block of one sheet: sheet, top row, left column, bottom row, right column.
type RectKey = (u32, i32, i32, i32, i32);

/// A criterion, as `build_criteria` reads it: two equal keys build the same predicate.
#[derive(PartialEq, Eq, Hash)]
enum CriterionKey {
    Number(u64),
    Text(Vec<u16>),
    Boolean(bool),
    Error(Discriminant<Error>),
    Empty,
}

impl CriterionKey {
    /// The key of `criterion`; `None` for anything else (an array element is already a scalar).
    fn of(criterion: &CalcResult) -> Option<CriterionKey> {
        Some(match criterion {
            CalcResult::Number(x) => CriterionKey::Number(x.to_bits()),
            CalcResult::String(s) => CriterionKey::Text(s.units().collect()),
            CalcResult::Boolean(b) => CriterionKey::Boolean(*b),
            CalcResult::Error { error, .. } => CriterionKey::Error(std::mem::discriminant(error)),
            CalcResult::EmptyCell | CalcResult::EmptyArg => CriterionKey::Empty,
            _ => return None,
        })
    }
}

/// A range's place in the cache.
enum RangeState {
    /// Its values, row by row.
    Ready(Arc<[CalcResult]>),
    /// Not cacheable yet: the cells before `checked` (row by row) can be read without
    /// evaluating anything; the cell at `checked` could not when last looked at.
    Waiting { checked: usize },
    /// Never cacheable in this pass (a circular cell, or over the budget).
    Never,
}

/// The criteria ranges' values and match sets of the current pass.
#[derive(Default)]
pub(crate) struct IfsCache {
    ranges: HashMap<RectKey, RangeState, KeyBuild>,
    matches: HashMap<(RectKey, CriterionKey), Arc<[u64]>>,
    cells: usize,
    bits: usize,
}

impl IfsCache {
    /// Forgets everything: a new pass, or cells whose state was reset.
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
        self.matches.clear();
        self.cells = 0;
        self.bits = 0;
    }
}

/// The cells of `rect` row by row, as positions.
fn rect_cell(rect: RectKey, index: usize) -> CellReferenceIndex {
    let (sheet, top, left, _, right) = rect;
    let width = usize::try_from(right - left + 1).unwrap_or(1).max(1);
    CellReferenceIndex {
        sheet,
        row: top + i32::try_from(index / width).unwrap_or(0),
        column: left + i32::try_from(index % width).unwrap_or(0),
    }
}

fn rect_size(rect: RectKey) -> usize {
    let (_, top, left, bottom, right) = rect;
    let rows = usize::try_from(bottom - top + 1).unwrap_or(0);
    let columns = usize::try_from(right - left + 1).unwrap_or(0);
    rows.saturating_mul(columns)
}

impl Model {
    /// Whether the cache may be read or filled now (`search_cache_safe`'s conditions).
    fn ifs_cache_safe(&self) -> bool {
        let evaluation = &self.evaluation;
        evaluation.in_pass
            && evaluation.restart.is_none()
            && !evaluation.unwinding
            && !evaluation.excel_order
            && !evaluation.blocking
            && !self.iterating()
    }

    /// Whether reading `at` evaluates nothing and marks nothing: a constant or a blank, a formula
    /// cell already evaluated in this pass, or a spill cell whose anchor is. `None` for a
    /// circular cell, which never becomes readable this way.
    fn read_is_inert(&self, at: CellReferenceIndex) -> Option<bool> {
        if self.evaluation.circular.contains(&key(at)) {
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
                matches!(self.fetch_cell(anchor), Some(Cell::ArrayFormula { .. }))
                    && evaluated(anchor)
            }
            Some(cell) => FormulaCell::of(cell).is_none() || evaluated(at),
        })
    }

    /// The values of `rect` row by row, when they can be read without evaluating anything (module
    /// docs); `None` when the caller must walk.
    fn ifs_range_values(&mut self, rect: RectKey) -> Option<Arc<[CalcResult]>> {
        if !self.ifs_cache_safe() {
            return None;
        }
        let size = rect_size(rect);
        if !(MIN_CACHED_CELLS..=MAX_CACHED_CELLS).contains(&size) {
            return None;
        }
        let checked = match self.evaluation.ifs_cache.ranges.get(&rect) {
            Some(RangeState::Ready(values)) => return Some(values.clone()),
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
            self.evaluation.ifs_cache.ranges.insert(rect, state);
            return None;
        }
        if self.evaluation.ifs_cache.cells + size > MAX_CACHED_CELLS {
            self.evaluation
                .ifs_cache
                .ranges
                .insert(rect, RangeState::Never);
            return None;
        }
        // Every read is inert: it records what it saw, as the walk's first read does.
        let values: Arc<[CalcResult]> = (0..size)
            .map(|index| self.evaluate_cell(rect_cell(rect, index)))
            .collect();
        if !self.ifs_cache_safe() {
            return None;
        }
        let cache = &mut self.evaluation.ifs_cache;
        cache.cells += size;
        cache.ranges.insert(rect, RangeState::Ready(values.clone()));
        Some(values)
    }

    /// The cells of `values` (the cached values of `rect`) that `matches`, as a bit set, from the
    /// cache when `criterion` was tested on `rect` before in this pass.
    fn ifs_matches(
        &mut self,
        rect: RectKey,
        values: &[CalcResult],
        criterion: &CalcResult,
        matches: &dyn Fn(&CalcResult) -> bool,
    ) -> Arc<[u64]> {
        let key = CriterionKey::of(criterion).map(|k| (rect, k));
        if let Some(bits) = key
            .as_ref()
            .and_then(|k| self.evaluation.ifs_cache.matches.get(k))
        {
            return bits.clone();
        }
        let mut bits = vec![0u64; values.len().div_ceil(64)];
        for (index, value) in values.iter().enumerate() {
            if matches(value) {
                bits[index / 64] |= 1u64 << (index % 64);
            }
        }
        let bits: Arc<[u64]> = bits.into();
        let cache = &mut self.evaluation.ifs_cache;
        if let Some(key) = key {
            if cache.bits + values.len() <= MAX_CACHED_BITS {
                cache.bits += values.len();
                cache.matches.insert(key, bits.clone());
            }
        }
        bits
    }

    /// The cells (row by row within `rects[0]`'s shape) that every criterion matches, as a bit set:
    /// `rects[k]` is the block of the k-th criteria range the walk would read, `criteria[k]` its
    /// criterion and `matchers[k]` its predicate. `None` when a range is not cacheable yet.
    pub(super) fn ifs_all_matches(
        &mut self,
        rects: &[Range],
        criteria: &[CalcResult],
        matchers: &[Criterion<'_>],
    ) -> Option<Vec<u64>> {
        let mut all: Option<Vec<u64>> = None;
        for ((range, criterion), matches) in rects.iter().zip(criteria).zip(matchers) {
            let rect = (
                range.left.sheet,
                range.left.row,
                range.left.column,
                range.right.row,
                range.right.column,
            );
            let values = self.ifs_range_values(rect)?;
            let bits = self.ifs_matches(rect, &values, criterion, matches.as_ref());
            match all.as_mut() {
                None => all = Some(bits.to_vec()),
                Some(all) => all.iter_mut().zip(bits.iter()).for_each(|(a, b)| *a &= b),
            }
        }
        all
    }

    /// The cached values of `range` (`ifs_range_values`) for the SUMIFS-shaped walks' value range.
    pub(super) fn ifs_values(&mut self, range: &Range) -> Option<Arc<[CalcResult]>> {
        self.ifs_range_values((
            range.left.sheet,
            range.left.row,
            range.left.column,
            range.right.row,
            range.right.column,
        ))
    }
}

/// The indices of the set bits of `bits`, ascending (row by row).
pub(super) fn set_bits(bits: &[u64]) -> impl Iterator<Item = usize> + '_ {
    bits.iter().enumerate().flat_map(|(word_index, &word)| {
        let mut word = word;
        std::iter::from_fn(move || {
            (word != 0).then(|| {
                let bit = word.trailing_zeros() as usize;
                word &= word - 1;
                word_index * 64 + bit
            })
        })
    })
}

/// How many bits of `bits` are set.
pub(super) fn count_bits(bits: &[u64]) -> usize {
    bits.iter().map(|word| word.count_ones() as usize).sum()
}
