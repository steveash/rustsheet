//! Circular references with iterative calculation off (recalc.md "A circular reference without
//! iteration is 0"): a cell on a loop, or reading one however indirectly, stores 0. The
//! evaluator (`evaluation.rs`) finds the loops; this marks them.

use crate::calc_result::CalcResult;
use crate::evaluation::{key, CellKey};
use crate::model::Model;

impl Model {
    /// A read closed a loop, or read a circular cell: every cell on the stack
    /// is on the loop or reads it, however indirectly. They all store 0,
    /// whatever their formula would have made of the value, as Excel's do with
    /// iterative calculation off (`=A1+10`, `=ISNUMBER(A1)` and `=IFERROR(A1,5)`
    /// reading a loop, and their readers, are 0; probes/circular-iteration-off,
    /// probes/t0445-circular-readers). Only the cells pushed since the last
    /// marking are visited.
    pub(crate) fn mark_stack_circular(&mut self) {
        let evaluation = &mut self.evaluation;
        let from = evaluation.circular_below.min(evaluation.stack.len());
        for cell in &evaluation.stack[from..] {
            evaluation.circular.insert(key(*cell));
        }
        evaluation.circular_below = evaluation.stack.len();
    }

    /// Marks the stack circular if the cell at `key`, just read, is circular.
    /// Not while the recursion unwinds: cells that have not run yet are then
    /// on the stack above the reader, and the replay reads again.
    pub(crate) fn mark_readers_of(&mut self, key: CellKey) {
        if self.evaluation.restart.is_none()
            && !self.evaluation.unwinding
            && self.evaluation.circular.contains(&key)
        {
            self.mark_stack_circular();
        }
    }

    /// Takes the top cell off the stack.
    pub(crate) fn pop_stack(&mut self) {
        let evaluation = &mut self.evaluation;
        evaluation.stack.pop();
        evaluation.circular_below = evaluation.circular_below.min(evaluation.stack.len());
    }
}

/// What a circular cell holds, and what a read that closes a loop gets: 0, as
/// in Excel with iterative calculation off (recalc.md "A circular reference
/// without iteration is 0").
pub(crate) fn circular_reference() -> CalcResult {
    CalcResult::Number(0.0)
}
