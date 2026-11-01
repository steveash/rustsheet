use super::lambda::NamedLambdas;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    model::Model,
    reference_operators::RefArg,
    variable_stack::Binding,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// What a LET or a LAMBDA call gives, for [`Model::close_over`]: the LAMBDAs it holds, the one it
/// is or the elements of an array of them (`LET(k,100,SCAN(..,LAMBDA(f,x,LAMBDA(z,f(z)*k+x))))`,
/// T-2039).
pub(crate) trait ScopeResult {
    fn lambdas(&self) -> Vec<usize>;
}

impl ScopeResult for CalcResult {
    fn lambdas(&self) -> Vec<usize> {
        match self {
            CalcResult::Lambda(id) => vec![*id],
            CalcResult::Array(rows) => rows
                .iter()
                .flatten()
                .filter_map(|node| match node {
                    ArrayNode::Lambda(id) => Some(*id),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl ScopeResult for RefArg {
    fn lambdas(&self) -> Vec<usize> {
        match self {
            RefArg::Value(value) => value.lambdas(),
            RefArg::Ranges(_) | RefArg::Grid(_) => Vec::new(),
        }
    }
}

impl ScopeResult for Binding {
    fn lambdas(&self) -> Vec<usize> {
        match self.resolved() {
            Binding::Value(value) | Binding::ReadOnce(value) => value.lambdas(),
            Binding::Omitted
            | Binding::Areas(_)
            | Binding::Span(_)
            | Binding::Grid(_)
            | Binding::Shared(_) => Vec::new(),
        }
    }
}

/// Most cells the LAMBDAs that outlive their scope may capture while one root cell is evaluated
/// (`Model::close_over`; the cells it evaluates as dependencies share the budget, which
/// `Model::evaluate_root_cell` resets): a REDUCE whose steps each return a closure over a fresh
/// array keeps every step's array. Past it a capture is `#NUM!` (DEVIATION, decision 0014:
/// docs/excel-semantics/arrays.md "LAMBDAs capture at most 4,194,304 cells"; Excel unprobed).
/// Four whole columns.
const MAX_CAPTURED_CELLS: usize = 1 << 22;

impl Model {
    /// Gives the LAMBDAs a LET or a LAMBDA call made, and that outlive it, what they read of its
    /// variables `ids` (`mark` is the first lambda id it could have made). Called at the end of
    /// that scope, while `ids` are still bound: a LAMBDA leaves a scope only through its result
    /// (a REDUCE or SCAN accumulator, a LET-bound `mk(5)`), so the LAMBDAs reachable from it, the
    /// result itself and those its captures hold, capture each variable of `ids` their bodies
    /// read, as it is bound now. Excel's closures read those values:
    /// `LET(mk,LAMBDA(a,LAMBDA(b,a+b)),f,mk(1),a,50,f(0))` is 1
    /// (docs/excel-semantics/arrays.md "LAMBDAs that outlive their call",
    /// probes/lambda-escaping-closures, T-0439). A LAMBDA used only inside the scope captures
    /// nothing, so a LAMBDA made on each REDUCE step copies nothing.
    pub(crate) fn close_over(
        &mut self,
        result: &impl ScopeResult,
        ids: &[usize],
        mark: usize,
        cell: CellReferenceIndex,
    ) {
        // Nothing made since `mark` can capture: the result is not walked (a LET giving a large
        // array of numbers costs nothing here).
        if ids.is_empty() || self.last_lambda_id <= mark {
            return;
        }
        let mut pending = result.lambdas();
        let mut seen = HashSet::new();
        while let Some(lambda) = pending.pop() {
            if lambda < mark || !seen.insert(lambda) {
                continue;
            }
            let held = self.lambda_captures.get(&lambda).into_iter().flatten();
            pending.extend(held.flat_map(|(_, binding)| binding.lambdas()));
            let Some(definition) = self.lambdas.get(&lambda) else {
                continue;
            };
            let mut read = Vec::new();
            definition.1.reads_variable(|id| {
                if ids.contains(&id) {
                    read.push(id);
                }
                false
            });
            read.sort_unstable();
            read.dedup();
            for id in read {
                let cells = self.variable_stack.cells_to_share(id);
                self.captured_cells = self.captured_cells.saturating_add(cells);
                let binding = if self.captured_cells > MAX_CAPTURED_CELLS {
                    // DEVIATION (decision 0014): arrays.md "LAMBDAs capture at most 4,194,304 cells".
                    Some(Arc::new(Binding::Value(CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "LAMBDAs capture too many cells".to_string(),
                    ))))
                } else {
                    self.variable_stack.shared(id)
                };
                if let Some(binding) = binding {
                    pending.extend(binding.lambdas());
                    self.lambda_captures
                        .entry(lambda)
                        .or_default()
                        .push((id, binding));
                }
            }
        }
    }

    /// Binds what the LAMBDA `lambda` captured (`close_over`) for one call of it, each
    /// variable that is not bound already, and gives back the ids it bound, for
    /// [`Self::unbind_captures`]. A bound one is in scope still, bound to what was captured: the
    /// same LET or call made the LAMBDA, since every evaluation of a LET, and every call of a
    /// body that made a LAMBDA reading its parameters, binds fresh ids (`call_prepared`). So a
    /// LAMBDA called where it was made copies nothing.
    pub(super) fn bind_captures(
        &mut self,
        lambda: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Vec<usize> {
        let CalcResult::Lambda(id) = lambda else {
            return Vec::new();
        };
        let Some(captures) = self.lambda_captures.get(id) else {
            return Vec::new();
        };
        let mut bound = Vec::new();
        for (variable, binding) in captures {
            if self.variable_stack.get(*variable).is_none() {
                // Shared, not copied: an escaped closure over a large array is called once per
                // element by MAP (`a_closure_over_a_large_array_is_called_without_copying_it`).
                self.variable_stack.bind_shared(*variable, binding, cell);
                bound.push(*variable);
            }
        }
        bound
    }

    /// Unbinds what [`Self::bind_captures`] bound.
    pub(super) fn unbind_captures(&mut self, bound: Vec<usize>) {
        for variable in bound {
            self.variable_stack.remove(variable);
        }
    }
}

/// Slots of `Model::lambdas`' table kept once a root formula's LAMBDAs are dropped: past this the
/// table is given back, so one formula that made millions holds no memory for the rest of the pass
/// (T-2925).
const RETAINED_LAMBDA_SLOTS: usize = 1 << 10;

// The lifetime of the LAMBDAs a formula makes: one root cell's, or one evaluation's.
impl Model {
    /// Evaluates `cell` as a root of the evaluation pass. No LAMBDA outlives the formula that
    /// made it (a cell holding one is `#CALC!`), so the LAMBDAs it made and what they captured
    /// (`Model::close_over`) are dropped with it, not kept for the rest of the pass: each later
    /// formula's calls would otherwise slow down with the map's size (T-2925).
    pub(crate) fn evaluate_root_cell(&mut self, cell: CellReferenceIndex) {
        let mark = self.last_lambda_id;
        self.evaluate_cell(cell);
        self.drop_lambdas_since(mark);
        self.clear_formula_scratch();
    }
    /// Drops what LAMBDAs captured and the nested-array budgets: they belong to one formula, so a
    /// root cell and a whole evaluation both end them.
    fn clear_formula_scratch(&mut self) {
        self.lambda_captures.clear();
        self.captured_cells = 0;
        self.nested_budget = crate::nested_array::NestedBudget::default();
    }
    /// Drops the LAMBDAs made since `last_lambda_id` was `mark`, in time proportional to how many
    /// were made. Ids keep growing (`last_lambda_id` is not reset): a `PreparedLambda`'s
    /// `lambda_mark` compares them (T-2925).
    fn drop_lambdas_since(&mut self, mark: usize) {
        for id in mark..self.last_lambda_id {
            self.lambdas.remove(&id);
            self.eta_lambdas.remove(&id);
        }
        self.named_lambdas.drop_since(mark);
        // A formula that made many leaves the map's table behind; give it back once empty.
        if self.lambdas.is_empty() && self.lambdas.capacity() > RETAINED_LAMBDA_SLOTS {
            self.lambdas = HashMap::new();
        }
    }
    /// Drops every LAMBDA and restarts the ids at 0: a whole evaluation (a pass, a data table) ends,
    /// so no caller holds an id.
    pub(crate) fn clear_lambdas(&mut self) {
        self.lambdas.clear();
        self.eta_lambdas.clear();
        self.named_lambdas = NamedLambdas::default();
        self.clear_formula_scratch();
        self.last_lambda_id = 0;
    }
}
