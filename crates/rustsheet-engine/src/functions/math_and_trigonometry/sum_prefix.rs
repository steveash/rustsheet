//! Running totals from a per-pass prefix cache (T-3456). `SUM($A$1:A1)` filled down, or
//! `SUM($I11:I11)` filled across, walked its whole prefix in every copy: n copies read about n²/2
//! cells (calipers `pv26_002`: 110,880 such SUMs across 77 columns).
//!
//! SUM of one range walks it row by row within the used area, adds each cell's number
//! (`cell_number`), skips everything else, and ends at the first error cell, whose error is the
//! result (`walk_areas`); one range draws on no walk budget. So the state after each cell, a
//! [`SumParts`], is all a longer range from the same start needs, and a range that reaches an
//! error cell gives that error. The cache keeps, for a start cell and an axis, the state after
//! each cell read so far, and the error cell where reading stopped.
//!
//! A call extends the cache by reading the new cells in order with `evaluate_cell`, as its walk
//! would: the reads that can evaluate anything are the walk's own. Each cell read must then be
//! readable without effect (`read_is_inert`: evaluated, not circular, not a cycle member): a later
//! call skips reading it, which is the walk's read only when reading it does nothing. If one is
//! not, the cache for that start is dropped and the walk runs, reading again what was read, as it
//! would have. As for the criteria caches (`ifs_cache.rs`), the first read of a cell records what
//! it saw, so a spill into it restarts the pass, the cache lives for one pass, and only where
//! `pass_cache_safe`. Directive 5: at most [`MAX_KEPT_STATES`] states per pass; past it the walk
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
#[derive(Default)]
pub(crate) struct SumPrefixes {
    prefixes: HashMap<PrefixKey, Prefix>,
    kept: usize,
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
}

impl Model {
    /// SUM of the one argument `arg`, from the prefix cache, when it is a range of one row or one
    /// column within the used area; `None` when the walk must run (module docs).
    pub(super) fn prefix_sum(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Option<Result<SumParts, CalcResult>> {
        if !self.pass_cache_safe() {
            return None;
        }
        let (left, right) = range_corners(arg, cell)?;
        let on_grid =
            |at: CellReferenceIndex| is_valid_row(at.row) && is_valid_column_number(at.column);
        if !on_grid(left) || !on_grid(right) {
            return None;
        }
        let (last_row, last_column) = self.used_area_end_of(left.sheet).ok()?;
        if right.row > last_row || right.column > last_column {
            return None;
        }
        let along_row = left.row == right.row;
        if !along_row && left.column != right.column {
            return None;
        }
        let length = usize::try_from(if along_row {
            right.column - left.column + 1
        } else {
            right.row - left.row + 1
        })
        .ok()?;
        let key = (left.sheet, left.row, left.column, along_row);
        let (mut read, stopped) = match self.evaluation.sum_prefixes.prefixes.get(&key) {
            Some(prefix) if prefix.states.len() >= length => {
                return prefix.states.get(length - 1).map(|state| Ok(*state));
            }
            Some(Prefix {
                error: Some(error), ..
            }) => return Some(Err(error.clone())),
            Some(prefix) => (prefix.states.len(), prefix.states.last().copied()),
            None => (0, None),
        };
        let budget = &self.evaluation.sum_prefixes;
        if budget.kept.saturating_add(length - read) > MAX_KEPT_STATES {
            return None;
        }
        let start = read;
        let mut state = stopped.unwrap_or_default();
        let mut states = Vec::with_capacity(length - read);
        let mut error = None;
        while read < length {
            // `read` is below `length`, a length on the grid.
            let offset = i32::try_from(read).ok()?;
            let at = if along_row {
                CellReferenceIndex {
                    column: left.column + offset,
                    ..left
                }
            } else {
                CellReferenceIndex {
                    row: left.row + offset,
                    ..left
                }
            };
            let value = self.evaluate_cell(at);
            if self.read_is_inert(at) != Some(true) || !self.pass_cache_safe() {
                self.evaluation.sum_prefixes.prefixes.remove(&key);
                return None;
            }
            match value {
                error_value @ CalcResult::Error { .. } => {
                    error = Some(error_value);
                    break;
                }
                value => {
                    if let Some(number) = cell_number(&value) {
                        state.add(number);
                    }
                    states.push(state);
                }
            }
            read += 1;
        }
        // A cell read may itself be a running total from this start (`B5 = SUM($B$1:B4)` reads
        // B4), which extended or dropped the entry meanwhile: the states are kept only where they
        // continue the entry, else the entry is left as that read left it.
        let cache = &mut self.evaluation.sum_prefixes;
        let kept = states.len();
        match cache.prefixes.get_mut(&key) {
            Some(prefix) if prefix.states.len() == start && prefix.error.is_none() => {
                prefix.states.extend(states);
                prefix.error.clone_from(&error);
                cache.kept += kept;
            }
            None if start == 0 => {
                cache.prefixes.insert(
                    key,
                    Prefix {
                        states,
                        error: error.clone(),
                    },
                );
                cache.kept += kept;
            }
            _ => {}
        }
        Some(match error {
            Some(error) => Err(error),
            None => Ok(state),
        })
    }
}
