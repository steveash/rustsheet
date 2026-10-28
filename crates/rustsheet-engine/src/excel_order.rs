//! The pass in Excel's fresh-load order (SPEC-0017), run only when a call read the state earlier
//! calls left (`OrderState`): WEEKNUM's stale `return_type` (docs/excel-semantics/dates.md
//! "WEEKNUM's stale `return_type` state") or AMORDEGRC's left-over value (financial.md
//! "AMORDEGRC below period 1").
//!
//! Excel runs the formula cells of a freshly loaded workbook in the reverse of load order: the
//! last sheet first, each sheet bottom row first and each row right to left. A cell that reads a
//! formula cell not yet calculated stops at that read, moves to just after it in the order and
//! runs again from its start when the order reaches it; the stopped run has no effect
//! (recalc.md "The order Excel calculates a freshly loaded workbook in"). The order is only
//! observable through those states, so the default pass keeps its own order and this one runs
//! after it only where a reader ran (`Model::evaluate`).

use crate::{
    evaluation::{key, CellState, OrderState, Restart},
    expressions::types::CellReferenceIndex,
    model::Model,
    types::{ArrayKind, Cell},
};

/// How the run of one cell of the chain ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Outcome {
    /// The cell is calculated.
    Done,
    /// The run stopped at a read of this cell (an index into the chain's cells), not yet
    /// calculated. It had no effect.
    Blocked(u32),
    /// The pass is abandoned (`Restart`): stop.
    Stop,
}

/// No cell: the end of the chain, or a cell that waits on nothing.
const NONE: u32 = u32::MAX;

/// How many moves the chain may make for `cells` cells before the rest of the pass evaluates on
/// demand (SPEC-0017 "Cost"). A chain read in reverse needs `n(n-1)/2` moves, and a cycle would
/// move its cells forever; moves are O(1) and evaluate nothing, so the floor costs tens of
/// milliseconds and keeps chains up to several thousand cells exact. Past it readers can differ
/// from Excel (dates.md, DEVIATION, decision 0014).
pub(crate) fn move_budget(cells: usize) -> u64 {
    4 * cells as u64 + (1 << 24)
}

/// Runs cells `0..len` in Excel's chain order: from `len - 1` down to 0, a blocked cell moved to
/// just after the cell it waits on. `run(ctx, cell, may_block)` runs one cell; with `may_block`
/// false it must evaluate the cell on demand and return `Done`, which is what every cell gets once
/// `budget` moves are spent. A moved cell remembers what it waits on: reached while that is still
/// not calculated, it would stop at the same read with no effect, so it is moved again without
/// running.
pub(crate) fn run_chain<C>(
    ctx: &mut C,
    len: usize,
    mut budget: u64,
    mut run: impl FnMut(&mut C, u32, bool) -> Outcome,
) {
    let Ok(len) = u32::try_from(len) else {
        // Not reachable with a sheet's limits; evaluate on demand rather than truncate.
        return;
    };
    // A singly linked list: cells are only taken from the head and put back after a cell that
    // is still in it.
    let mut next: Vec<u32> = (0..len).map(|i| i.checked_sub(1).unwrap_or(NONE)).collect();
    let mut waits = vec![NONE; len as usize];
    let mut done = vec![false; len as usize];
    let mut head = len.checked_sub(1).unwrap_or(NONE);
    while head != NONE {
        let cell = head;
        head = next[cell as usize];
        let waiting_on = waits[cell as usize];
        let may_block = budget > 0;
        if may_block && waiting_on != NONE && !done[waiting_on as usize] {
            budget -= 1;
            next[cell as usize] = next[waiting_on as usize];
            next[waiting_on as usize] = cell;
            continue;
        }
        match run(ctx, cell, may_block) {
            Outcome::Done => done[cell as usize] = true,
            Outcome::Stop => return,
            Outcome::Blocked(precedent)
                if may_block && precedent != cell && !done[precedent as usize] =>
            {
                budget -= 1;
                waits[cell as usize] = precedent;
                next[cell as usize] = next[precedent as usize];
                next[precedent as usize] = cell;
            }
            // Not expected: `run` blocks only on a cell not calculated yet, never on itself.
            Outcome::Blocked(_) => {
                if run(ctx, cell, false) == Outcome::Stop {
                    return;
                }
                done[cell as usize] = true;
            }
        }
    }
}

impl Model {
    /// The formula cells of the workbook in `(load rank, row, column)` order: the chain starts at
    /// the last one. A sheet keeps its load position after a move (SPEC-0017), so the sheets
    /// are ordered by `Worksheet::load_rank`, not by their tab.
    fn formula_positions(&self) -> Vec<CellReferenceIndex> {
        let mut positions = self.positions_where(|cell| {
            matches!(cell, Cell::CellFormula { .. } | Cell::ArrayFormula { .. })
        });
        positions.sort_unstable_by_key(|position| self.chain_key(*position));
        positions
    }

    /// Where a cell stands in the chain's natural order; the sheet index only breaks a tie
    /// between equal ranks.
    fn chain_key(&self, position: CellReferenceIndex) -> (u32, u32, i32, i32) {
        let rank = self
            .workbook
            .worksheets
            .get(position.sheet as usize)
            .map_or(0, |worksheet| worksheet.load_rank);
        (rank, position.sheet, position.row, position.column)
    }

    /// Everything after the anchors of a pass in Excel order. The anchors ran first, as in the
    /// default pass and with its recursion, so that the restarts, and with them the driver's
    /// proof that the restarts end, stay those of the default pass: no anchor runs after another
    /// cell (where Excel puts a spilling formula is open, T-2070). In the chain nothing nests: a
    /// read of a formula cell not calculated yet stops the run (`Evaluation::blocking`), and the
    /// state the stopped run changed is put back. Returns the reason the pass was abandoned.
    pub(crate) fn run_excel_order(&mut self) -> Option<Restart> {
        let positions = self.formula_positions();
        let budget = move_budget(positions.len());
        // The chain starts as Excel's recalc does, with an empty state: what the anchors' own
        // calls left is not in Excel's order (rule 1; dates.md "Ours").
        self.evaluation.order_state = OrderState::default();
        run_chain(self, positions.len(), budget, |model, index, may_block| {
            let cell = positions[index as usize];
            let state = &mut model.evaluation;
            if state.cells.get(&key(cell)) == Some(&CellState::Evaluated) {
                // An anchor, or a cell an anchor read. At its place in the order a dynamic-array
                // formula empties WEEKNUM's state (gen-m6, dates.md "WEEKNUM's stale
                // `return_type` state"; one that spills is unprobed, T-2070).
                if matches!(
                    model.fetch_cell(cell),
                    Some(Cell::ArrayFormula {
                        kind: ArrayKind::Dynamic,
                        ..
                    })
                ) {
                    // What it does to AMORDEGRC's left-over value is unprobed: left as it is.
                    model.evaluation.order_state.weeknum_start = None;
                }
                return Outcome::Done;
            }
            let order_state = state.order_state;
            state.blocking = may_block;
            state.root = Some(cell);
            model.evaluate_root_cell(cell);
            let state = &mut model.evaluation;
            state.blocking = false;
            if state.abandoned() {
                return Outcome::Stop;
            }
            let Some(precedent) = state.blocked_on.take() else {
                return Outcome::Done;
            };
            state.order_state = order_state;
            let precedent = (precedent.sheet, precedent.row, precedent.column);
            let precedent = model.chain_key(CellReferenceIndex {
                sheet: precedent.0,
                row: precedent.1,
                column: precedent.2,
            });
            match positions.binary_search_by_key(&precedent, |p| model.chain_key(*p)) {
                Ok(precedent) => Outcome::Blocked(precedent as u32),
                Err(_) => Outcome::Blocked(index),
            }
        });
        self.evaluation.restart.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cells that read the cells `reads[i]` in that order, stopping at the first one not done.
    struct Layout {
        reads: Vec<Vec<u32>>,
        done: Vec<bool>,
        order: Vec<u32>,
        runs: usize,
    }

    impl Layout {
        fn new(reads: Vec<Vec<u32>>) -> Self {
            let n = reads.len();
            Layout {
                reads,
                done: vec![false; n],
                order: Vec::new(),
                runs: 0,
            }
        }

        fn run(&mut self, cell: u32, may_block: bool) -> Outcome {
            if self.done[cell as usize] {
                // Run on demand by a cell before it, as the model finds it evaluated.
                return Outcome::Done;
            }
            self.runs += 1;
            if may_block {
                if let Some(&p) = self.reads[cell as usize]
                    .iter()
                    .find(|&&p| !self.done[p as usize])
                {
                    return Outcome::Blocked(p);
                }
            } else {
                // On demand: what it reads first, depth first.
                for p in self.reads[cell as usize].clone() {
                    if !self.done[p as usize] && p != cell {
                        self.run(p, false);
                    }
                }
            }
            self.done[cell as usize] = true;
            self.order.push(cell);
            Outcome::Done
        }
    }

    /// The rule as Excel states it, with a plain list and no shortcuts: take the first cell, run
    /// it, and if it stops at a cell not done, put it just after that cell.
    fn naive(reads: &[Vec<u32>]) -> Vec<u32> {
        let mut list: Vec<u32> = (0..reads.len() as u32).rev().collect();
        let mut done = vec![false; reads.len()];
        let mut order = Vec::new();
        while !list.is_empty() {
            let cell = list.remove(0);
            match reads[cell as usize].iter().find(|&&p| !done[p as usize]) {
                Some(&p) => {
                    let at = list.iter().position(|&c| c == p).unwrap_or(list.len());
                    list.insert(at + 1, cell);
                }
                None => {
                    done[cell as usize] = true;
                    order.push(cell);
                }
            }
        }
        order
    }

    fn chained(reads: &[Vec<u32>], budget: u64) -> Layout {
        let mut layout = Layout::new(reads.to_vec());
        run_chain(
            &mut layout,
            reads.len(),
            budget,
            |layout, cell, may_block| layout.run(cell, may_block),
        );
        layout
    }

    #[test]
    fn matches_the_naive_rule_on_random_acyclic_layouts() {
        let mut seed: u64 = 0x2068;
        let mut random = |below: u64| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % below
        };
        for _ in 0..500 {
            let n = 1 + random(12) as usize;
            // A random rank makes the reads acyclic whatever the positions.
            let mut rank: Vec<u32> = (0..n as u32).collect();
            for i in (1..n).rev() {
                rank.swap(i, random(i as u64 + 1) as usize);
            }
            let reads: Vec<Vec<u32>> = (0..n)
                .map(|i| {
                    let lower: Vec<u32> = (0..n as u32)
                        .filter(|&j| rank[j as usize] < rank[i])
                        .collect();
                    let count = if lower.is_empty() {
                        0
                    } else {
                        random(4) as usize
                    };
                    (0..count)
                        .map(|_| lower[random(lower.len() as u64) as usize])
                        .collect()
                })
                .collect();
            assert_eq!(
                chained(&reads, move_budget(n)).order,
                naive(&reads),
                "{reads:?}"
            );
        }
    }

    #[test]
    fn a_running_total_in_reverse_moves_quadratically_and_runs_each_cell_twice() {
        // Cell i reads cell i - 1: the last cell is first in the chain.
        let reads: Vec<Vec<u32>> = (0..50u32)
            .map(|i| i.checked_sub(1).into_iter().collect())
            .collect();
        let layout = chained(&reads, move_budget(50));
        assert_eq!(layout.order, (0..50).collect::<Vec<_>>());
        // Every cell but the first stops once; moves of blocked cells run nothing.
        assert_eq!(layout.runs, 50 + 49);
    }

    #[test]
    fn past_the_budget_the_rest_runs_on_demand() {
        let reads: Vec<Vec<u32>> = (0..50u32)
            .map(|i| i.checked_sub(1).into_iter().collect())
            .collect();
        let layout = chained(&reads, 10);
        assert_eq!(layout.order.len(), 50);
        assert!(layout.done.iter().all(|&d| d));
    }

    #[test]
    fn a_cycle_ends_at_the_budget() {
        let reads = vec![vec![1], vec![0], vec![]];
        let mut layout = Layout::new(reads.clone());
        // On demand, a cycle is the caller's to detect: here a read of a cell being run is skipped.
        run_chain(&mut layout, 3, 100, |layout, cell, may_block| {
            if may_block {
                layout.run(cell, true)
            } else {
                layout.done[cell as usize] = true;
                layout.order.push(cell);
                Outcome::Done
            }
        });
        assert_eq!(layout.order.len(), 3);
        assert!(layout.runs <= 3 + 100);
    }
}
