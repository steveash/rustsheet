//! The budgets of one formula evaluation for multi-area references: how many pairs of areas its
//! intersections may compare and how many steps the walks of its union areas may take, in all
//! (DEVIATION, decision 0014: docs/excel-semantics/references.md "Union area cap"), and how many
//! rectangles it may put on record as read empty (references.md "Seen-range cap").

use crate::{
    calc_result::CalcResult,
    expressions::{token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Most pairs of areas one formula evaluation may compare ([`AreaBudget`]). Each intersection
/// compares at most `MAX_AREAS` pairs, but a chain of cuts (`(U1 U2) A1 A1 A1 ...`) repeats that
/// once per three characters of formula, and a LET-bound union repeats it per argument
/// (`SUM(IF(1,u u A1),IF(1,u u A1),...)`), so the whole formula shares this budget.
pub(crate) const MAX_AREA_PAIRS: usize = 1 << 20;

/// Message of the `#VALUE!` for an expression comparing more than [`MAX_AREA_PAIRS`] pairs.
pub(crate) const TOO_MANY_AREA_PAIRS: &str = "Too many areas to intersect";

/// Most steps the walks of the union areas of one formula evaluation may take in all
/// ([`AreaBudget`]). The area cap bounds the length of an area list, not the work of walking it:
/// `MAX_AREAS` copies of `A:A` fit in a short formula, a LET-bound union can be passed again and
/// again (`SUM(x,x,...)`), and a LAMBDA can walk one per row of a MAP. A union area is walked
/// over its existing cells only ([`Model::for_each_position_in_area`]), and each step is one area
/// walked, one cell handed to the caller, one search of the sheet's column index (an area with
/// cells in few columns walks them through it), one row found, one unallocated chunk of rows
/// passed over, or [`SLOTS_PER_WALK_STEP`] row slots tested, so the work follows the data
/// visited, not the size of the areas. Past the bound the argument is `#VALUE!` (DEVIATION,
/// decision 0014: references.md "Union area cap"). The message keeps its words
/// ([`TOO_MANY_AREA_CELLS`]).
pub(crate) const MAX_AREA_WALK: u64 = 1 << 24;

/// Row slots of an allocated chunk tested per step of [`MAX_AREA_WALK`]: testing a slot is one
/// comparison over a contiguous array, while a step handed to the caller reads and evaluates a
/// cell, so a slot is charged a fraction of a step (measured in T-1497, references.md "Union area
/// cap").
pub(crate) const SLOTS_PER_WALK_STEP: u64 = 16;

/// Message of the `#VALUE!` for union walks past [`MAX_AREA_WALK`] steps.
pub(crate) const TOO_MANY_AREA_CELLS: &str = "Too many cells in the areas of a reference";

/// Most rectangles one formula evaluation may put on record as read empty in a pass
/// ([`Model::record_seen_empty_range`]). A rectangle already on record for the same root is
/// free, so a walk repeated by a LAMBDA costs once (T-1497); a new one needs a reference no
/// earlier read of the pass produced, such as one per row of `MAKEARRAY(..,LAMBDA(r,c,
/// SUM(OFFSET(1:1,r-1,0,c))))`, and nothing else bounds how many a formula makes. Past the bound
/// the formula is `#VALUE!` (DEVIATION, decision 0014: references.md "Seen-range cap").
pub(crate) const MAX_SEEN_RANGES: usize = 1 << 16;

/// Message of the `#VALUE!` for a formula past [`MAX_SEEN_RANGES`].
pub(crate) const TOO_MANY_SEEN_RANGES: &str = "Too many distinct ranges read";

/// What is left of one formula evaluation's budgets of area pairs ([`MAX_AREA_PAIRS`]), of union
/// walk steps ([`MAX_AREA_WALK`]) and of seen-empty rectangles ([`MAX_SEEN_RANGES`]).
/// [`Model::with_area_budget`] gives each formula run a fresh one, so every argument and operand of
/// the formula draws on the same budget, and its worst-case work does not grow with the number of
/// arguments. Only that call refills it, so every entry point that starts a formula evaluation
/// (`run_cell_formula`, `evaluate_outside_cell`) must go through it; a new one that did not would
/// draw on whatever budget the model holds.
#[derive(Clone, Copy)]
pub(crate) struct AreaBudget {
    pairs: usize,
    walk: u64,
    seen_ranges: usize,
    /// Whether a rectangle went unrecorded for want of [`MAX_SEEN_RANGES`]: the formula's value
    /// may then depend on a read the pass does not know of, so it is not used.
    seen_ranges_exceeded: bool,
}

impl AreaBudget {
    /// Whether `count` more rectangles fit in what is left of the budget of [`MAX_SEEN_RANGES`]:
    /// a walk that may instead record its cells one by one asks before it clips (T-0405).
    pub(crate) fn has_seen_ranges(&self, count: usize) -> bool {
        self.seen_ranges >= count
    }

    /// Takes one rectangle from what is left of the budget of [`MAX_SEEN_RANGES`], or, past it,
    /// marks the formula's run as exceeded ([`Model::with_area_budget`]) and returns `false`: the
    /// rectangle must not be recorded.
    pub(crate) fn spend_seen_range(&mut self) -> bool {
        match self.seen_ranges.checked_sub(1) {
            Some(left) => {
                self.seen_ranges = left;
                true
            }
            None => {
                self.seen_ranges_exceeded = true;
                false
            }
        }
    }
}

impl Default for AreaBudget {
    fn default() -> AreaBudget {
        AreaBudget {
            pairs: MAX_AREA_PAIRS,
            walk: MAX_AREA_WALK,
            seen_ranges: MAX_SEEN_RANGES,
            seen_ranges_exceeded: false,
        }
    }
}

impl Model {
    /// Runs `f`, one formula's evaluation in `cell`, with a fresh [`AreaBudget`], and gives the
    /// budget it interrupted back afterwards: a cell evaluated while another formula runs has its
    /// own budget and leaves the other's as it was. A run that read more rectangles than
    /// [`MAX_SEEN_RANGES`] lets it record is `#VALUE!`, whatever `f` returned: a spill into a
    /// rectangle it could not record would not restart the pass, so its value cannot be trusted,
    /// and an error here, unlike one at the read, cannot be caught (`IFERROR`) and replaced by a
    /// value that still depends on the read.
    pub(crate) fn with_area_budget(
        &mut self,
        cell: CellReferenceIndex,
        f: impl FnOnce(&mut Self) -> CalcResult,
    ) -> CalcResult {
        let outer = std::mem::take(&mut self.area_budget);
        let result = f(self);
        let exceeded = self.area_budget.seen_ranges_exceeded;
        self.area_budget = outer;
        if exceeded {
            return CalcResult::new_error(Error::VALUE, cell, TOO_MANY_SEEN_RANGES.to_string());
        }
        result
    }

    /// Takes `steps` from what is left of the formula's budget of [`MAX_AREA_WALK`], or gives
    /// the `#VALUE!` of a walk past it and leaves the budget as it was.
    pub(crate) fn spend_area_walk(
        &mut self,
        steps: u64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.area_budget.walk = self.area_budget.walk.checked_sub(steps).ok_or_else(|| {
            CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREA_CELLS.to_string())
        })?;
        Ok(())
    }

    /// Takes `pairs` from what is left of the formula's budget of [`MAX_AREA_PAIRS`], or gives
    /// the `#VALUE!` of an intersection past it and leaves the budget as it was.
    pub(crate) fn spend_area_pairs(
        &mut self,
        pairs: usize,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.area_budget.pairs = self.area_budget.pairs.checked_sub(pairs).ok_or_else(|| {
            CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREA_PAIRS.to_string())
        })?;
        Ok(())
    }
}
