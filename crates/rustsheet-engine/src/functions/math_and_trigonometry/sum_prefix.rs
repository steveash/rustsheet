//! Running totals from a per-pass prefix cache (T-3456). `SUM($A$1:A1)` filled down, or
//! `SUM($I11:I11)` filled across, walked its whole prefix in every copy: n copies read about n²/2
//! cells (calipers `pv26_002`: 110,880 such SUMs across 77 columns).
//!
//! SUM of one range walks it row by row within the used area, adds each cell's number
//! (`cell_number`), skips everything else, and ends at the first error cell, whose error is the
//! result (`walk_areas`); one range draws on no walk budget, and its node is charged as one node
//! (SPEC-0024), which this path charges too. So the state after each cell, a
//! [`SumParts`], is all a longer range from the same start needs, and a range that reaches an
//! error cell gives that error. The cache keeps, for a start cell and an axis, the state after
//! each cell read so far, and the error cell where reading stopped.
//!
//! A call extends the cache by reading the new cells in order with `evaluate_cell`, as its walk
//! would: the reads that can evaluate anything are the walk's own. Each cell read must then be
//! readable without effect (`read_is_inert`: evaluated, not circular, not a cycle member): a later
//! call skips reading it, which is the walk's read only when reading it does nothing. If one is
//! not, the cache for that start is dropped and the call goes on reading as the walk does, keeping
//! nothing. As for the criteria caches (`ifs_cache.rs`), the first read of a cell records what
//! it saw, so a spill into it restarts the pass, the cache lives for one pass, and only where
//! `pass_cache_safe`. Directive 5: at most [`MAX_KEPT_STATES`] states per pass, an entry that
//! holds none counted as one; past it the walk
//! runs.

use std::collections::HashMap;

use crate::calc_result::CalcResult;
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceIndex;
use crate::expressions::utils::{is_valid_column_number, is_valid_row};
use crate::functions::util::cell_number;
use crate::model::Model;
use crate::tailed_array::range_corners;

use super::sum::SumParts;

/// States all prefixes of one pass may keep together.
const MAX_KEPT_STATES: usize = 4_000_000;

/// A prefix's start: sheet, row, column, and whether it runs along the row (else down the column).
type PrefixKey = (u32, i32, i32, bool);

/// The states read from one start.
struct Prefix {
    /// The state after each cell read, in order.
    states: Vec<SumParts>,
    /// The error of the cell after the last state, where reading stopped.
    error: Option<CalcResult>,
}

/// The running totals of this pass.
pub(crate) struct SumPrefixes {
    prefixes: HashMap<PrefixKey, Prefix>,
    /// States kept, an entry that holds none (it stopped at an error in its first cell) counted
    /// as one, so that the entries are bounded too.
    kept: usize,
    /// The most `kept` may reach: [`MAX_KEPT_STATES`], lowered by tests.
    pub(crate) limit: usize,
}

impl Default for SumPrefixes {
    fn default() -> Self {
        SumPrefixes {
            prefixes: HashMap::new(),
            kept: 0,
            limit: MAX_KEPT_STATES,
        }
    }
}

/// What an entry of `states` new states adds to `kept`: at least one, for the entry itself.
fn entry_cost(states: usize) -> usize {
    states.max(1)
}

impl SumPrefixes {
    /// Forgets everything: a new pass, or cells whose state was reset.
    pub(crate) fn clear(&mut self) {
        self.prefixes.clear();
        self.kept = 0;
    }

    /// The states kept this pass, for the tests that the cache was used.
    #[cfg(test)]
    pub(crate) fn kept(&self) -> usize {
        self.kept
    }

    /// Keeps `states`, read from `start` on, and the `error` where reading stopped. A cell read
    /// may itself be a running total from this start (`B5 = SUM($B$1:B4)` reads B4), which
    /// extended or dropped the entry meanwhile: the states are kept only where they continue the
    /// entry, else the entry is left as that read left it.
    fn extend(
        &mut self,
        key: PrefixKey,
        start: usize,
        states: Vec<SumParts>,
        error: Option<CalcResult>,
    ) {
        let added = states.len();
        match self.prefixes.get_mut(&key) {
            Some(prefix) if prefix.states.len() == start && prefix.error.is_none() => {
                prefix.states.extend(states);
                prefix.error = error;
                self.kept = self.kept.saturating_add(added);
            }
            None if start == 0 => {
                self.prefixes.insert(key, Prefix { states, error });
                self.kept = self.kept.saturating_add(entry_cost(added));
            }
            _ => {}
        }
    }
}

/// A SUM argument the cache can serve: a range of one row or one column of two cells or more, on
/// the grid, within the used area.
struct PrefixRange {
    key: PrefixKey,
    left: CellReferenceIndex,
    along_row: bool,
    /// Its cells, as an offset bound and as a count.
    span: i32,
    length: usize,
}

impl PrefixRange {
    /// The `offset`-th cell from the start.
    fn cell(&self, offset: i32) -> CellReferenceIndex {
        if self.along_row {
            CellReferenceIndex {
                column: self.left.column + offset,
                ..self.left
            }
        } else {
            CellReferenceIndex {
                row: self.left.row + offset,
                ..self.left
            }
        }
    }
}

impl Model {
    /// `arg` as a [`PrefixRange`], or `None` when the walk must run (module docs).
    fn prefix_range(&self, arg: &Node, cell: CellReferenceIndex) -> Option<PrefixRange> {
        let (left, right) = range_corners(arg, cell)?;
        // A range operand of a tailed argument reads as its view says (`tailed_operand`).
        if self.tailed_leaf_view(arg, cell).is_some() {
            return None;
        }
        let on_grid =
            |at: CellReferenceIndex| is_valid_row(at.row) && is_valid_column_number(at.column);
        if !on_grid(left) || !on_grid(right) {
            return None;
        }
        let (last_row, last_column) = self.used_area_end_of(left.sheet).ok()?;
        if right.row > last_row || right.column > last_column {
            return None;
        }
        // One cell is no run along either axis, and its walk reads that one cell: were it keyed
        // by one axis, a run along the other from the same start would miss its entry.
        if left == right {
            return None;
        }
        let along_row = left.row == right.row;
        if !along_row && left.column != right.column {
            return None;
        }
        let span = if along_row {
            right.column - left.column + 1
        } else {
            right.row - left.row + 1
        };
        Some(PrefixRange {
            key: (left.sheet, left.row, left.column, along_row),
            left,
            along_row,
            span,
            length: usize::try_from(span).ok()?,
        })
    }

    /// SUM of the one argument `arg`, from the prefix cache, when it is a [`PrefixRange`]; `None`
    /// when the walk must run (module docs).
    pub(super) fn prefix_sum(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Option<Result<SumParts, CalcResult>> {
        if !self.pass_cache_safe() {
            return None;
        }
        let range = self.prefix_range(arg, cell)?;
        let (key, length) = (range.key, range.length);
        let cache = &self.evaluation.sum_prefixes;
        let entry = cache.prefixes.get(&key);
        let read = entry.map_or(0, |prefix| prefix.states.len());
        // A hit keeps nothing new; a read keeps its new states, or a new entry at least one.
        let hit = entry.is_some_and(|prefix| read >= length || prefix.error.is_some());
        let cost = match entry {
            None => entry_cost(length),
            Some(_) => length.saturating_sub(read),
        };
        if !hit && cache.kept.saturating_add(cost) > cache.limit {
            return None;
        }
        let first = i32::try_from(read).ok()?;
        // The walk evaluates the range node first, which charges one node (SPEC-0024); from here
        // on this call is the walk, so a read that cannot be cached is still this call's read.
        if self.spend_node(cell).is_err() {
            // The walk's own charge fails the same way.
            return None;
        }
        let mut state = match self.evaluation.sum_prefixes.prefixes.get(&key) {
            Some(prefix) if prefix.states.len() >= length => {
                return prefix.states.get(length - 1).map(|state| Ok(*state));
            }
            Some(Prefix {
                error: Some(error), ..
            }) => return Some(Err(error.clone())),
            Some(prefix) => prefix.states.last().copied().unwrap_or_default(),
            None => SumParts::default(),
        };
        let mut caching = true;
        let mut states = Vec::with_capacity(length - read);
        let mut error = None;
        for offset in first..range.span {
            let at = range.cell(offset);
            let value = self.evaluate_cell(at);
            if caching && (self.read_is_inert(at) != Some(true) || !self.pass_cache_safe()) {
                // A later call could not skip this read: nothing from this start is kept.
                self.evaluation.sum_prefixes.prefixes.remove(&key);
                caching = false;
            }
            if let CalcResult::Error { .. } = value {
                error = Some(value);
                break;
            }
            if let Some(number) = cell_number(&value) {
                state.add(number);
            }
            if caching {
                states.push(state);
            }
        }
        let answer = match &error {
            Some(error) => Err(error.clone()),
            None => Ok(state),
        };
        if caching {
            self.evaluation
                .sum_prefixes
                .extend(key, read, states, error);
        }
        Some(answer)
    }
}
