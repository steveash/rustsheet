//! The budgets of one formula evaluation for multi-area references: how many pairs of areas its
//! intersections may compare and how many cells its multi-area arguments may walk, in all
//! (DEVIATION, decision 0014: docs/excel-semantics/references.md "Union area cap").

use crate::{
    calc_result::{CalcResult, Range},
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

/// Most cells the multi-area arguments of one formula evaluation may walk in all
/// ([`AreaBudget`]), counted over their areas clipped to the used area. The area cap bounds the
/// length of an area list, not the work of walking it: `MAX_AREAS` copies of `A:A` fit in a
/// short formula, and a LET-bound union can be passed again and again (`SUM(x,x,...)`). Past the
/// bound the argument is `#VALUE!`, found before any of its cells is walked (DEVIATION, decision
/// 0014: references.md "Union area cap").
pub(crate) const MAX_AREA_CELLS: u64 = 1 << 24;

/// Message of the `#VALUE!` for a multi-area argument of more than [`MAX_AREA_CELLS`] cells.
pub(crate) const TOO_MANY_AREA_CELLS: &str = "Too many cells in the areas of a reference";

/// What is left of one formula evaluation's budgets of area pairs ([`MAX_AREA_PAIRS`]) and of
/// multi-area cells ([`MAX_AREA_CELLS`]). [`Model::with_area_budget`] gives each formula run a
/// fresh one, so every argument and operand of the formula draws on the same budget, and its
/// worst-case work does not grow with the number of arguments. Only that call refills it, so
/// every entry point that starts a formula evaluation (`run_cell_formula`,
/// `evaluate_outside_cell`) must go through it; a new one that did not would draw on whatever
/// budget the model holds.
#[derive(Clone, Copy)]
pub(crate) struct AreaBudget {
    pairs: usize,
    cells: u64,
}

impl Default for AreaBudget {
    fn default() -> AreaBudget {
        AreaBudget {
            pairs: MAX_AREA_PAIRS,
            cells: MAX_AREA_CELLS,
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

    /// Takes `cells` from what is left of the formula's budget of [`MAX_AREA_CELLS`], or gives
    /// the `#VALUE!` of a walk past it and leaves the budget as it was.
    pub(crate) fn spend_area_cells(
        &mut self,
        cells: u64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.area_budget.cells = self.area_budget.cells.checked_sub(cells).ok_or_else(|| {
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

    /// Takes the cells of `areas`, each clipped to its sheet's used area, from what is left of
    /// the formula's budget of [`MAX_AREA_CELLS`], shared by all its multi-area walks, or gives
    /// the `#VALUE!` of a walk past it: arithmetic only, before any cell is read, so `MAX_AREAS`
    /// whole columns over a populated sheet are `#VALUE!` without a walk. One range is not
    /// counted, as a plain range argument never was.
    pub(crate) fn check_area_cells(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        if areas.len() <= 1 {
            return Ok(());
        }
        let left = self.area_budget.cells;
        let mut total: u64 = 0;
        for area in areas {
            let (row2, column2) = self
                .used_area_corner(
                    area.left.sheet,
                    area.left.row,
                    area.left.column,
                    area.right.row,
                    area.right.column,
                )
                .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
            let rows = u64::try_from(row2 - area.left.row + 1).unwrap_or(0);
            let columns = u64::try_from(column2 - area.left.column + 1).unwrap_or(0);
            total = total.saturating_add(rows.saturating_mul(columns));
            if total > left {
                // Past the budget already: the rest of the areas need not be counted, and the
                // partial total fails to spend all the same.
                break;
            }
        }
        self.spend_area_cells(total, cell)
    }
}
