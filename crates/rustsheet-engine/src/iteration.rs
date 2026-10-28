//! Iterative calculation (docs/excel-semantics/recalc.md, "Iterative calculation sweeps the cycle
//! in sheet order, in place" and "iterateDelta stops the iterations for the whole workbook").
//!
//! With iteration on, a pass does not make a circular reference 0 (`circular.rs`). A read that closes a
//! cycle returns the cell's value from before the recalculation, and every cell on the evaluation
//! stack at that moment joins the recalculated set: the cycle and the cells that read it. So does
//! every cell that later reads a member. After the pass, the members go back to their values from
//! before the recalculation. They are then swept in row-major sheet order, each reading the values
//! already updated in the sweep (Gauss-Seidel), until a sweep's largest change is strictly below
//! the delta or the count is reached.

use std::collections::{HashMap, HashSet};

use bitcode::{Decode, Encode};

use crate::{
    calc_result::CalcResult,
    evaluation::{key, CellKey as Key},
    expressions::types::CellReferenceIndex,
    model::Model,
    types::{Cell, FormulaValue},
};

/// Iterative calculation settings: a recalculation sweeps the circular references at most
/// `max_iterations` times and stops at the first sweep whose largest change is below
/// `max_change` (docs/excel-semantics/recalc.md, "Iterative calculation").
#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub struct Iteration {
    /// `calcPr@iterateCount`, 100 by default.
    pub max_iterations: u32,
    /// `calcPr@iterateDelta`, 0.001 by default.
    pub max_change: f64,
}

impl Default for Iteration {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            max_change: 0.001,
        }
    }
}

/// Member evaluations all the sweeps of one recalculation may make together (CLAUDE.md
/// directive 5); past it the sweeps stop where they are (recalc.md, DEVIATION). Each run also draws
/// on the recalculation's ceiling (SPEC-0028 section 4), and running out of it stops them the same
/// way.
const MAX_SWEEP_EVALUATIONS: usize = 1 << 25;

/// What a recalculation with iteration on keeps about its circular references.
#[derive(Default)]
pub(crate) struct IterationState {
    /// The recalculated set: cycle cells and the cells that read them.
    members: HashSet<Key>,
    /// Every formula cell as it was before the recalculation.
    prior: HashMap<Key, Cell>,
    /// The members as the first round's sweeps left them.
    settled: HashMap<Key, Cell>,
    /// Calls of `join_stack` so far: a pass cache built while this moved read a member, so its
    /// reader joined, and is not stored (`search_cache.rs`, `ifs_cache.rs`; T-0485).
    joins: u64,
}

impl Model {
    /// Starts a recalculation: with iteration on, remembers every formula cell as it is.
    pub(crate) fn begin_iteration(&mut self) {
        let on = self.iterating();
        let state = &mut self.evaluation.iteration;
        state.members.clear();
        state.prior.clear();
        state.settled.clear();
        if !on {
            return;
        }
        for (sheet, ws) in self.workbook.worksheets.iter().enumerate() {
            for (row, column, cell) in ws.sheet_data.cells() {
                if cell.has_formula() {
                    state
                        .prior
                        .insert((sheet as u32, row, column), cell.clone());
                }
            }
        }
    }

    /// Whether iterative calculation is on. Then no evaluation, on demand ones included (such as
    /// conditional formatting at load), marks a cycle circular.
    pub(crate) fn iterating(&self) -> bool {
        self.workbook.settings.iteration.is_some()
    }

    /// A read closed a cycle at `at` with iteration on: everything on the stack joins the
    /// recalculated set, and the read sees the value from before the recalculation (0 for a new
    /// formula).
    pub(crate) fn read_into_cycle(&mut self, at: CellReferenceIndex) -> CalcResult {
        self.join_stack();
        self.current_value(at)
    }

    /// A read of an evaluated cell: when it is a member, its readers on the stack become members.
    pub(crate) fn read_evaluated(&mut self, at: CellReferenceIndex) {
        if self.iterating() && self.is_cycle_member(at) {
            self.join_stack();
        }
    }

    /// Whether this round found a circular reference to iterate.
    pub(crate) fn has_cycles(&self) -> bool {
        !self.evaluation.iteration.members.is_empty()
    }

    /// Whether `at` is in the recalculated set, so that reading it joins its readers to it.
    pub(crate) fn is_cycle_member(&self, at: CellReferenceIndex) -> bool {
        self.evaluation.iteration.members.contains(&key(at))
    }

    /// [`IterationState::joins`] so far.
    pub(crate) fn cycle_joins(&self) -> u64 {
        self.evaluation.iteration.joins
    }

    /// Joins the stack to the recalculated set. A cached read skips the reads that would join its
    /// reader, so a cached range holds no member: its build is not stored when it joined (the
    /// count moved), and a cell joins only while it is on the stack, before it is evaluated, where
    /// a range is cached only once every cell of it is (T-0485).
    fn join_stack(&mut self) {
        let state = &mut self.evaluation;
        state.iteration.joins += 1;
        state
            .iteration
            .members
            .extend(state.stack.iter().map(|c| key(*c)));
    }

    /// Starts the second round of passes, after data tables: the members are collected afresh.
    /// The data tables' substituted evaluations read a cycle's current values too.
    pub(crate) fn restart_iteration_round(&mut self) {
        self.evaluation.iteration.members.clear();
    }

    /// Ends a recalculation: drops what it remembered.
    pub(crate) fn end_iteration(&mut self) {
        let state = &mut self.evaluation.iteration;
        state.prior.clear();
        state.settled.clear();
    }

    /// Ends a round of passes: the members go back to their values from before the
    /// recalculation and, when `sweep`, are swept as the iteration settings say. Without `sweep`
    /// (the round after data tables) they get the values the first round's sweeps settled on.
    pub(crate) fn iterate_cycles(&mut self, sweep: bool) {
        let Some(settings) = self.workbook.settings.iteration else {
            return;
        };
        let mut members: Vec<Key> = self.evaluation.iteration.members.iter().copied().collect();
        members.sort_unstable();
        let positions: Vec<CellReferenceIndex> = members
            .iter()
            .map(|&(sheet, row, column)| CellReferenceIndex { sheet, row, column })
            .collect();
        for at in &positions {
            let state = &self.evaluation.iteration;
            let start = if sweep {
                None
            } else {
                state.settled.get(&key(*at))
            };
            if let Some(mut cell) = start.or_else(|| state.prior.get(&key(*at))).cloned() {
                // A formula never evaluated starts its iterations at 0, as in Excel.
                if let Cell::CellFormula { v, .. } | Cell::ArrayFormula { v, .. } = &mut cell {
                    if *v == FormulaValue::Unevaluated {
                        *v = FormulaValue::Number(0.0);
                    }
                }
                if let Ok(ws) = self.workbook.worksheet_mut(at.sheet) {
                    ws.sheet_data.set_cell(at.row, at.column, cell);
                }
            }
        }
        if sweep {
            self.sweep(&positions, settings.max_iterations, settings.max_change);
            for at in &positions {
                if let Some(cell) = self.fetch_cell(*at).cloned() {
                    self.evaluation.iteration.settled.insert(key(*at), cell);
                }
            }
        }
    }

    fn sweep(&mut self, members: &[CellReferenceIndex], max_iterations: u32, max_change: f64) {
        self.evaluation.in_pass = false;
        self.evaluation.root = None;
        let mut budget = MAX_SWEEP_EVALUATIONS;
        for _ in 0..max_iterations {
            let mut largest = 0.0_f64;
            for &at in members {
                if budget == 0 {
                    return;
                }
                budget -= 1;
                let before = self.current_value(at);
                self.evaluation.cells.remove(&key(at));
                self.evaluation.clear_pass_caches();
                self.evaluate_cell(at);
                if self.take_overrun().is_some() {
                    // The ceiling ran out: the run stored nothing, so the member keeps its value
                    // from before it, and the sweeps stop as at `MAX_SWEEP_EVALUATIONS`.
                    return;
                }
                let after = self.current_value(at);
                largest = largest.max(change(&before, &after));
            }
            if largest < max_change {
                return;
            }
        }
    }

    /// The value a member holds now, 0 for a formula never evaluated.
    fn current_value(&self, at: CellReferenceIndex) -> CalcResult {
        match self.fetch_cell(at) {
            Some(Cell::CellFormula {
                v: FormulaValue::Unevaluated,
                ..
            })
            | Some(Cell::ArrayFormula {
                v: FormulaValue::Unevaluated,
                ..
            }) => CalcResult::Number(0.0),
            Some(cell) => self.get_cell_value(cell, at),
            None => CalcResult::EmptyCell,
        }
    }
}

/// How much a member changed in a sweep: the difference of two numbers, 0 for equal values
/// otherwise, and infinitely much for any other change (it never counts as converged).
fn change(before: &CalcResult, after: &CalcResult) -> f64 {
    match (before, after) {
        (CalcResult::Number(a), CalcResult::Number(b)) => {
            let d = (a - b).abs();
            if d.is_nan() {
                f64::INFINITY
            } else {
                d
            }
        }
        (CalcResult::String(a), CalcResult::String(b)) if a == b => 0.0,
        (CalcResult::Boolean(a), CalcResult::Boolean(b)) if a == b => 0.0,
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => 0.0,
        (CalcResult::Error { error: a, .. }, CalcResult::Error { error: b, .. }) if a == b => 0.0,
        _ => f64::INFINITY,
    }
}
