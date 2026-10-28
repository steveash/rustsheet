//! The work budget of one recalculation (SPEC-0028): a ceiling on everything `Model::evaluate`
//! spends, and an allowance per pass that, once overrun, re-runs the round with each formula
//! limited to its share, so which cells give up does not depend on the order the evaluator visits
//! them in (DEVIATION, decision 0014: docs/excel-semantics/recalc.md "One recalculation does at
//! most 2^32 work units, and a pass past 2^30 shares them out").
//!
//! The units are SPEC-0024's (`area_budget.rs`), charged where they are charged now: the hot path
//! (`Model::spend_work`) does not change. They are summed into the pools at the boundaries of
//! formula runs, in `Model::with_area_budget` ([`Model::start_run`]).

use crate::{
    area_budget::AreaBudget,
    calc_result::CalcResult,
    evaluation::FormulaCell,
    expressions::{token::Error, types::CellReferenceIndex},
    model::Model,
    types::Cell,
};

/// Work units the formula runs of one pass may commit before the round is re-run with each
/// formula's share (SPEC-0028): four formula budgets.
pub(crate) const MAX_PASS_WORK: u64 = 1 << 30;

/// Work units one recalculation (`Model::evaluate`) may spend in all, committed or thrown away,
/// in its rounds of passes (SPEC-0028): the time bound of directive 5. Sweeps and data tables do
/// not draw on it yet (T-3009).
pub(crate) const MAX_RECALC_WORK: u64 = 1 << 32;

/// Message of the `#VALUE!` of a formula past its share in share mode (SPEC-0028 section 3).
pub(crate) const TOO_MUCH_RECALC_WORK: &str =
    "Recalculation needs too much work to calculate this formula";

/// Message of the `#VALUE!` of every formula cell once a recalculation spent [`MAX_RECALC_WORK`].
pub(crate) const RECALC_EXHAUSTED: &str = "Recalculation needs too much work";

/// How the formula runs of a recalculation are bounded (SPEC-0028 section 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RecalcMode {
    /// Each formula may spend `MAX_FORMULA_WORK`, as long as the pass allowance lasts.
    Normal,
    /// A round overran the allowance: each formula may spend this many units, its share.
    Share(u64),
    /// The ceiling ran out: every formula cell is `#VALUE!` and nothing more runs.
    Exhausted,
}

/// Which bound was the smallest when a formula run was given its units, and so which one a run
/// that ran out of them ran out of.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum RunBound {
    /// The formula's own limit, `MAX_FORMULA_WORK`: its `#VALUE!` (`TOO_MUCH_WORK`).
    #[default]
    Own,
    /// The formula's share in share mode: its `#VALUE!` ([`TOO_MUCH_RECALC_WORK`]).
    Share,
    /// What the pass may still commit: an overrun, the round goes to share mode.
    Pass,
    /// What the recalculation may still spend: an overrun, the recalculation is exhausted.
    Ceiling,
}

/// What one recalculation has spent and may still spend (SPEC-0028 section 2). It lives in
/// `Evaluation` while `Model::evaluate` runs and is absent outside it, so `evaluate_formula` and
/// conditional formats draw on no pool.
#[derive(Clone, Debug)]
pub(crate) struct RecalcBudget {
    /// Units left of [`MAX_RECALC_WORK`]: everything this recalculation's passes spent.
    pub(crate) left: u64,
    /// Units the current pass committed, against `pass_limit`.
    pub(crate) committed: u64,
    /// Units the current round threw away (abandoned passes, runs cut short), against `pass_limit`.
    pub(crate) thrown_away: u64,
    pub(crate) mode: RecalcMode,
    /// Set when a run stopped at a pool, not at its own limit: the pass is abandoned at once.
    pub(crate) overrun: Option<RunBound>,
    /// [`MAX_PASS_WORK`], or a test's lower limit.
    pub(crate) pass_limit: u64,
    /// [`MAX_RECALC_WORK`], or a test's lower limit.
    #[cfg(test)]
    pub(crate) ceiling: u64,
}

impl RecalcBudget {
    pub(crate) fn new(pass_limit: u64, ceiling: u64) -> RecalcBudget {
        RecalcBudget {
            left: ceiling,
            committed: 0,
            thrown_away: 0,
            mode: RecalcMode::Normal,
            overrun: None,
            pass_limit,
            #[cfg(test)]
            ceiling,
        }
    }

    /// The units spent so far, committed or thrown away.
    #[cfg(test)]
    pub(crate) fn spent(&self) -> u64 {
        self.ceiling - self.left
    }

    /// The smallest of the pools a run may still draw on, and which one it is: the ceiling, and in
    /// normal mode what the pass may still commit. Ties go to the ceiling.
    fn pools_left(&self) -> (u64, RunBound) {
        let pass_left = self.pass_limit.saturating_sub(self.committed);
        if self.mode == RecalcMode::Normal && pass_left < self.left {
            (pass_left, RunBound::Pass)
        } else {
            (self.left, RunBound::Ceiling)
        }
    }

    /// Adds `units` thrown away to the round's sum; past the allowance in normal mode, the
    /// round overruns.
    fn throw_away(&mut self, units: u64) {
        self.thrown_away = self.thrown_away.saturating_add(units);
        if self.mode == RecalcMode::Normal && self.thrown_away > self.pass_limit {
            self.overrun.get_or_insert(RunBound::Pass);
        }
    }
}

/// The share of each of `formulas` formula cells in share mode: `pass_limit / (4 * formulas)`,
/// at least one unit and at most `formula_limit` (SPEC-0028 section 3).
pub(crate) fn share(pass_limit: u64, formulas: u64, formula_limit: u64) -> u64 {
    (pass_limit / formulas.max(1).saturating_mul(4)).clamp(1, formula_limit.max(1))
}

impl Model {
    /// Starts the budget of a recalculation (`Model::evaluate`).
    pub(crate) fn begin_recalc_budget(&mut self) {
        #[cfg(not(test))]
        let (pass_limit, ceiling) = (MAX_PASS_WORK, MAX_RECALC_WORK);
        #[cfg(test)]
        let (pass_limit, ceiling) = self.evaluation.recalc_limits;
        self.evaluation.recalc = Some(RecalcBudget::new(pass_limit, ceiling));
    }

    /// Ends the budget of a recalculation: nothing outside `Model::evaluate` draws on it.
    pub(crate) fn end_recalc_budget(&mut self) {
        #[cfg(test)]
        {
            self.evaluation.last_recalc = self.evaluation.recalc.take();
        }
        self.evaluation.recalc = None;
    }

    /// The fresh budget of a formula run starting now, whose own limit is `formula_limit`, after
    /// the run it interrupts has settled what it spent so far ([`Model::start_run`]): a run in a
    /// pass may spend the least of its own limit (its share in share mode) and what the pools
    /// have left.
    pub(crate) fn start_run(&mut self, formula_limit: u64) -> AreaBudget {
        let outer = &mut self.area_budget;
        let in_pass = self.evaluation.in_pass;
        let Some(recalc) = self.evaluation.recalc.as_mut() else {
            return AreaBudget::for_run(formula_limit, formula_limit, RunBound::Own, false);
        };
        settle(recalc, outer);
        let (own, own_bound) = match recalc.mode {
            RecalcMode::Share(share) if share < formula_limit => (share, RunBound::Share),
            _ => (formula_limit, RunBound::Own),
        };
        if !in_pass {
            return AreaBudget::for_run(formula_limit, own, own_bound, false);
        }
        let (pool, pool_bound) = recalc.pools_left();
        if pool < own {
            AreaBudget::for_run(formula_limit, pool, pool_bound, true)
        } else {
            AreaBudget::for_run(formula_limit, own, own_bound, true)
        }
    }

    /// Ends a formula run whose budget was `inner`: settles what it spent into the pools (thrown
    /// away when the run is unwinding), notes an overrun when it ran out of a pool, and clamps the
    /// run it interrupted to what the pools have left. Returns the formula's error when it ran
    /// out of its own limit or its share.
    pub(crate) fn end_run(
        &mut self,
        inner: &mut AreaBudget,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let unwinding = self.evaluation.unwinding;
        if let Some(recalc) = self.evaluation.recalc.as_mut() {
            settle(recalc, inner);
            if inner.pooled && unwinding {
                // A run cut short by the depth limit runs again: what it spent is thrown away.
                recalc.committed = recalc.committed.saturating_sub(inner.spent);
                recalc.throw_away(inner.spent);
            }
            if inner.work_exceeded && matches!(inner.bound, RunBound::Pass | RunBound::Ceiling) {
                recalc.overrun.get_or_insert(inner.bound);
            }
            let outer = &mut self.area_budget;
            if outer.pooled {
                let (pool, bound) = recalc.pools_left();
                if outer.work > pool {
                    outer.work = pool;
                    outer.bound = bound;
                }
                outer.unsettled_from = outer.work;
                if recalc.overrun.is_some() {
                    // The pass is abandoned: the interrupted run stops at its next charge.
                    outer.work_exceeded = true;
                }
            }
        } else {
            inner.settle_spent();
        }
        if !inner.work_exceeded {
            return None;
        }
        match inner.bound {
            RunBound::Own => Some(crate::area_budget::too_much_work(cell)),
            RunBound::Share => Some(CalcResult::new_error(
                Error::VALUE,
                cell,
                TOO_MUCH_RECALC_WORK.to_string(),
            )),
            // The pass is abandoned; nothing reads this value.
            RunBound::Pass | RunBound::Ceiling => Some(CalcResult::EmptyCell),
        }
    }

    /// Whether the current pass ran out of a pool, and of which one; clears it.
    pub(crate) fn take_overrun(&mut self) -> Option<RunBound> {
        self.evaluation.recalc.as_mut()?.overrun.take()
    }

    /// Starts a pass: it has committed nothing yet.
    pub(crate) fn begin_pass_budget(&mut self) {
        if let Some(recalc) = self.evaluation.recalc.as_mut() {
            recalc.committed = 0;
            recalc.overrun = None;
        }
    }

    /// Starts a round of passes: it has thrown nothing away yet.
    pub(crate) fn begin_round_budget(&mut self) {
        if let Some(recalc) = self.evaluation.recalc.as_mut() {
            recalc.thrown_away = 0;
        }
    }

    /// The pass just run is thrown away (a restart, or a pass replaced by one in Excel's order):
    /// what it committed counts as thrown away, and past the allowance the round goes on in share
    /// mode.
    pub(crate) fn throw_away_pass(&mut self) {
        let overran = match self.evaluation.recalc.as_mut() {
            Some(recalc) => {
                let committed = std::mem::take(&mut recalc.committed);
                recalc.throw_away(committed);
                recalc.overrun.take().is_some()
            }
            None => false,
        };
        if overran {
            self.enter_share_mode();
        }
    }

    /// Puts the recalculation in share mode, once a round overran its allowance: from now on each
    /// formula run may spend its share (SPEC-0028 section 3).
    pub(crate) fn enter_share_mode(&mut self) {
        let formulas = self.formula_cell_count();
        #[cfg(not(test))]
        let formula_limit = crate::area_budget::MAX_FORMULA_WORK;
        #[cfg(test)]
        let formula_limit = self.area_budget.limit;
        if let Some(recalc) = self.evaluation.recalc.as_mut() {
            if recalc.mode == RecalcMode::Normal {
                recalc.mode = RecalcMode::Share(share(recalc.pass_limit, formulas, formula_limit));
            }
            recalc.thrown_away = 0;
        }
    }

    /// Ends a recalculation that spent its ceiling: every formula cell is `#VALUE!`
    /// ([`RECALC_EXHAUSTED`]), each dynamic anchor spills nothing and its spill cells go. Linear in
    /// the number of cells, and charges nothing.
    pub(crate) fn exhaust_recalc(&mut self) {
        if let Some(recalc) = self.evaluation.recalc.as_mut() {
            recalc.mode = RecalcMode::Exhausted;
        }
        self.evaluation.in_pass = false;
        for position in self.all_positions() {
            let Some(formula_cell) = self.fetch_cell(position).and_then(FormulaCell::of) else {
                continue;
            };
            let error = CalcResult::new_error(Error::VALUE, position, RECALC_EXHAUSTED.to_string());
            // Writing an error into a formula cell that holds one cannot fail; a failure would
            // leave the cell's old value, which is no worse.
            let _ = self.set_cells_with_result(position, formula_cell, &error);
        }
    }

    /// The number of formula cells (`Cell::CellFormula` and `Cell::ArrayFormula` anchors) of the
    /// workbook: the `N` of the share.
    fn formula_cell_count(&self) -> u64 {
        let count: usize = (self.workbook.worksheets.iter())
            .map(|ws| {
                (ws.sheet_data.cells())
                    .filter(|(_, _, cell)| {
                        matches!(cell, Cell::CellFormula { .. } | Cell::ArrayFormula { .. })
                    })
                    .count()
            })
            .sum();
        u64::try_from(count).unwrap_or(u64::MAX)
    }
}

/// Settles what `run` spent since its last settle into `recalc`'s pools: the ceiling, and the
/// pass's committed sum when the run draws on the pass ([`AreaBudget::pooled`]).
fn settle(recalc: &mut RecalcBudget, run: &mut AreaBudget) {
    let spent = run.settle_spent();
    if run.pooled {
        recalc.left = recalc.left.saturating_sub(spent);
        recalc.committed = recalc.committed.saturating_add(spent);
    }
}
