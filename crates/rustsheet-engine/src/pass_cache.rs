//! The two rules every per-pass cache (`search_cache.rs`, `ifs_cache.rs`, the SUMPRODUCT block
//! cache, `sum_prefix.rs`) shares: when a pass cache may be read or filled at all
//! ([`Model::pass_cache_safe`]), and when reading a cell is inert, so a cache may read it where the
//! lazy walk it replaces might not ([`Model::read_is_inert`], T-0449). One home for both keeps the
//! caches of different function families off each other's files (T-3534).

use crate::evaluation::{key, CellState, FormulaCell};
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::Cell;

impl Model {
    /// Whether the caches of this pass (`search_cache.rs`, `ifs_cache.rs`) may be read or filled
    /// now: inside a pass that is neither abandoned, unwinding, blocked nor in Excel order, so a
    /// value read now stays the pass's value. Under iterative calculation too: a cached range holds
    /// no cycle member (`Model::join_stack`, T-0485). The sweeps run outside a pass.
    pub(crate) fn pass_cache_safe(&self) -> bool {
        let evaluation = &self.evaluation;
        #[cfg(test)]
        if evaluation.no_pass_caches {
            return false;
        }
        evaluation.in_pass
            && !evaluation.abandoned()
            && !evaluation.unwinding
            && !evaluation.excel_order
            && !evaluation.blocking
    }

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
}
