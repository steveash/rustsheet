//! Circular references with iterative calculation off (recalc.md "A circular reference without
//! iteration keeps the cells' values from before the recalculation"): a cell on a loop, or
//! reading one however indirectly, keeps the value it had, 0 for a formula never evaluated. The
//! evaluator (`evaluation.rs`) finds the loops; this marks them.

use crate::calc_result::CalcResult;
use crate::evaluation::{key, CellKey};
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::{Cell, FormulaValue};

impl Model {
    /// A read closed a loop, or read a circular cell: every cell on the stack
    /// is on the loop or reads it, however indirectly. None of them takes what
    /// its formula would have made of the value ([`Model::circular_value`]), as
    /// Excel's do with iterative calculation off (`=A1+10`, `=ISNUMBER(A1)` and
    /// `=IFERROR(A1,5)` reading a loop, and their readers;
    /// probes/circular-iteration-off, probes/t0445-circular-readers). Only the
    /// cells pushed since the last marking are visited.
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
        if !self.evaluation.abandoned()
            && !self.evaluation.unwinding
            && self.evaluation.circular.contains(&key)
        {
            self.mark_stack_circular();
        }
    }

    /// What a circular cell at `at` stores: the value it held before this
    /// recalculation, which a loaded workbook's cached value supplies, as Excel
    /// keeps it with iterative calculation off (fixtures/circular-keeps-cached-values;
    /// calipers `pv23_022`, whose loop members include one-cell dynamic arrays such as
    /// D88 `=D37`). A formula never evaluated is 0, and so is an anchor that spills,
    /// whose earlier spill no evidence covers.
    pub(crate) fn circular_value(&self, at: CellReferenceIndex) -> CalcResult {
        match self.fetch_cell(at) {
            Some(cell @ Cell::CellFormula { v, .. })
            | Some(cell @ Cell::ArrayFormula { v, r: (1, 1), .. })
                if *v != FormulaValue::Unevaluated =>
            {
                self.get_cell_value(cell, at)
            }
            _ => circular_reference(),
        }
    }

    /// Takes the top cell off the stack.
    pub(crate) fn pop_stack(&mut self) {
        let evaluation = &mut self.evaluation;
        evaluation.stack.pop();
        evaluation.circular_below = evaluation.circular_below.min(evaluation.stack.len());
    }
}

/// What a read that closes a loop gets, and what a circular formula never
/// evaluated or array anchor holds: 0, as in Excel with iterative calculation
/// off (recalc.md "A circular reference without iteration keeps the cells'
/// values from before the recalculation").
pub(crate) fn circular_reference() -> CalcResult {
    CalcResult::Number(0.0)
}
