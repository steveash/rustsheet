//! The budgets of one formula evaluation for multi-area references: how many pairs of areas its
//! intersections may compare and how many steps the walks of its union areas may take, in all
//! (DEVIATION, decision 0014: docs/excel-semantics/references.md "Union area cap").

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
/// walked, one cell handed to the caller, one row found, one unallocated chunk of rows passed
/// over, or [`SLOTS_PER_WALK_STEP`] row slots tested, so the work follows the data visited, not
/// the size of the areas. Past the bound the argument is `#VALUE!` (DEVIATION, decision 0014:
/// references.md "Union area cap"). The message keeps its words ([`TOO_MANY_AREA_CELLS`]).
pub(crate) const MAX_AREA_WALK: u64 = 1 << 24;

/// Row slots of an allocated chunk tested per step of [`MAX_AREA_WALK`]: testing a slot is one
/// comparison over a contiguous array, while a step handed to the caller reads and evaluates a
/// cell, so a slot is charged a fraction of a step (measured in T-1497, references.md "Union area
/// cap").
pub(crate) const SLOTS_PER_WALK_STEP: u64 = 16;

/// Message of the `#VALUE!` for union walks past [`MAX_AREA_WALK`] steps.
pub(crate) const TOO_MANY_AREA_CELLS: &str = "Too many cells in the areas of a reference";

/// What is left of one formula evaluation's budgets of area pairs ([`MAX_AREA_PAIRS`]) and of
/// union walk steps ([`MAX_AREA_WALK`]). [`Model::with_area_budget`] gives each formula run a
/// fresh one, so every argument and operand of the formula draws on the same budget, and its
/// worst-case work does not grow with the number of arguments. Only that call refills it, so
/// every entry point that starts a formula evaluation (`run_cell_formula`,
/// `evaluate_outside_cell`) must go through it; a new one that did not would draw on whatever
/// budget the model holds.
#[derive(Clone, Copy)]
pub(crate) struct AreaBudget {
    pairs: usize,
    walk: u64,
}

impl Default for AreaBudget {
    fn default() -> AreaBudget {
        AreaBudget {
            pairs: MAX_AREA_PAIRS,
            walk: MAX_AREA_WALK,
        }
    }
}

impl Model {
    /// Runs `f`, one formula's evaluation, with a fresh [`AreaBudget`], and gives the budget it
    /// interrupted back afterwards: a cell evaluated while another formula runs has its own
    /// budget and leaves the other's as it was.
    pub(crate) fn with_area_budget<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let outer = std::mem::take(&mut self.area_budget);
        let result = f(self);
        self.area_budget = outer;
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
