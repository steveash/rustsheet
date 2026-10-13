//! The LET variables and LAMBDA parameters in scope, and what each is bound to.

use std::collections::HashMap;

use crate::{
    calc_result::{CalcResult, Range},
    expressions::types::CellReferenceIndex,
    reference_operators::{
        several_areas, three_d_needs_one_sheet, too_many_areas, Areas, MAX_AREAS,
    },
};

/// What a LET variable or a LAMBDA parameter is bound to: a value (a single reference
/// included), the areas of a union, or a 3-D span. Only reference context reads the last two as
/// such ([`Model::variable_reference`]); everywhere else they are `#VALUE!`, as `LET(x,(A1,B2),x)`
/// is (probes/multiarea-let H3) and as the bare span is (`LET(x,Data1:Data3!A1,x)`,
/// `INDEX(x,1)`, `ROWS(x)`: probes/3d-ref-let-bound-span, T-1158).
#[derive(Clone)]
pub(crate) enum Binding {
    Value(CalcResult),
    Areas(Areas),
    /// A range whose corners sit on two sheets: an aggregate reads it like the written span
    /// (`LET(x,Data1:Data3!A1,SUM(x))` is 111, probes/3d-ref-let-bound-span C1:C3).
    Span(Range),
}

impl Binding {
    /// The binding where a value or a single reference is needed.
    pub(crate) fn value(&self, cell: CellReferenceIndex) -> CalcResult {
        match self {
            Binding::Value(value) => value.clone(),
            Binding::Areas(_) => several_areas(cell),
            Binding::Span(_) => three_d_needs_one_sheet(cell),
        }
    }

    /// How many areas the binding holds: those of a union, none for a value or a span.
    pub(crate) fn held_areas(&self) -> usize {
        match self {
            Binding::Value(_) | Binding::Span(_) => 0,
            Binding::Areas(areas) => areas.len(),
        }
    }
}

/// Most areas the LET variables and LAMBDA parameters in scope may hold together. One union is
/// at most [`MAX_AREAS`] areas, but each binding may hold a fresh one
/// (`LET(x1,(A1,A2),x2,(x1,x1),x3,(x2,A1),...)`), so memory would grow with formula length;
/// past this total a binding is `#VALUE!` (DEVIATION, decision 0014: references.md "Union area
/// cap").
const MAX_HELD_AREAS: usize = 4 * MAX_AREAS;

/// The LET variables and LAMBDA parameters in scope, by variable id, with the areas their
/// union bindings hold, which may not pass [`MAX_HELD_AREAS`].
#[derive(Clone, Default)]
pub(crate) struct VariableStack {
    bindings: HashMap<usize, Binding>,
    held_areas: usize,
}

impl VariableStack {
    pub(crate) fn get(&self, id: usize) -> Option<&Binding> {
        self.bindings.get(&id)
    }

    /// Binds `id`; a union that would pass [`MAX_HELD_AREAS`] is bound to `#VALUE!` instead.
    pub(crate) fn insert(&mut self, id: usize, binding: Binding, cell: CellReferenceIndex) {
        let held = self.held_areas + binding.held_areas();
        let binding = if held > MAX_HELD_AREAS {
            Binding::Value(too_many_areas(cell))
        } else {
            self.held_areas = held;
            binding
        };
        if let Some(old) = self.bindings.insert(id, binding) {
            self.held_areas -= old.held_areas();
        }
    }

    pub(crate) fn remove(&mut self, id: usize) {
        if let Some(old) = self.bindings.remove(&id) {
            self.held_areas -= old.held_areas();
        }
    }

    pub(crate) fn clear(&mut self) {
        self.bindings.clear();
        self.held_areas = 0;
    }
}
