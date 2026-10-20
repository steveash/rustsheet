//! The LET variables and LAMBDA parameters in scope, and what each is bound to.

use std::collections::HashMap;

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{token::Error, types::CellReferenceIndex},
    limits::MAX_SIZE,
    reference_operators::{
        several_areas, three_d_needs_one_sheet, too_many_areas, Areas, RefGrid, MAX_AREAS,
    },
};

/// What a LET variable or a LAMBDA parameter is bound to: a value (a single reference
/// included), the areas of a union, a 3-D span, or an array of references. Only reference
/// context reads the last three as such ([`Model::variable_reference`]); everywhere else they are `#VALUE!`, as `LET(x,(A1,B2),x)`
/// is (probes/multiarea-let H3) and as the bare span is (`LET(x,Data1:Data3!A1,x)`,
/// `INDEX(x,1)`, `ROWS(x)`: probes/3d-ref-let-bound-span, T-1158).
#[derive(Clone)]
pub(crate) enum Binding {
    Value(CalcResult),
    /// A LAMBDA parameter's value that the body reads exactly once per call
    /// (`Model::call_prepared`): that read takes the value instead of copying it, so an array
    /// accumulator handed through REDUCE is not deep-copied on each step (T-1588).
    ReadOnce(CalcResult),
    Areas(Areas),
    /// A range whose corners sit on two sheets: an aggregate reads it like the written span
    /// (`LET(x,Data1:Data3!A1,SUM(x))` is 111, probes/3d-ref-let-bound-span C1:C3).
    Span(Range),
    /// A lifted OFFSET, INDIRECT or TRIMRANGE (SPEC-0014 rule 7): INDEX, ROWS and the aggregates
    /// read its elements through the variable (`LET(x,OFFSET(A1,{0;1},0),INDEX(x,2,1))` is A2's
    /// value, probes/reference-lift-consumers E89); a value read is its value view.
    Grid(RefGrid),
}

impl Binding {
    /// The binding where a value or a single reference is needed.
    pub(crate) fn value(&self, cell: CellReferenceIndex) -> CalcResult {
        match self {
            Binding::Value(value) | Binding::ReadOnce(value) => value.clone(),
            Binding::Areas(_) => several_areas(cell),
            Binding::Span(_) => three_d_needs_one_sheet(cell),
            Binding::Grid(grid) => grid.to_values(),
        }
    }

    /// How many areas the binding holds: those of a union, none for anything else.
    pub(crate) fn held_areas(&self) -> usize {
        match self {
            Binding::Areas(areas) => areas.len(),
            _ => 0,
        }
    }

    fn grid(&self) -> Option<&RefGrid> {
        match self {
            Binding::Grid(grid) => Some(grid),
            _ => None,
        }
    }
}

/// Most areas the LET variables and LAMBDA parameters in scope may hold together. One union is
/// at most [`MAX_AREAS`] areas, but each binding may hold a fresh one
/// (`LET(x1,(A1,A2),x2,(x1,x1),x3,(x2,A1),...)`), so memory would grow with formula length;
/// past this total a binding is `#VALUE!` (DEVIATION, decision 0014: references.md "Union area
/// cap").
const MAX_HELD_AREAS: usize = 4 * MAX_AREAS;

/// Most elements the arrays of references bound in scope may hold together, apart from
/// [`MAX_HELD_AREAS`], which would refuse one legal grid of 2^20 elements (SPEC-0014
/// "Pass-through"). A grid is charged once however many bindings share it
/// (`LET(x,r,y,x,..)` copies no element); past this total a binding is `#VALUE!` (DEVIATION,
/// decision 0014: lookup.md "Arrays of references held by LET variables are bounded").
const MAX_HELD_GRID_ELEMENTS: usize = 4 * MAX_SIZE;

/// The LET variables and LAMBDA parameters in scope, by variable id, with the areas their
/// union bindings hold, which may not pass [`MAX_HELD_AREAS`], and the elements their distinct
/// grids hold, which may not pass [`MAX_HELD_GRID_ELEMENTS`].
#[derive(Clone, Default)]
pub(crate) struct VariableStack {
    bindings: HashMap<usize, Binding>,
    held_areas: usize,
    held_grid_elements: usize,
}

impl VariableStack {
    pub(crate) fn get(&self, id: usize) -> Option<&Binding> {
        self.bindings.get(&id)
    }

    /// What `id` holds where a value is wanted: [`Binding::value`], except that a
    /// [`Binding::ReadOnce`] value is moved out. What it leaves behind is an error, not a value, so
    /// that a second read (a broken read-once analysis) shows instead of silently reading 0.
    pub(crate) fn read_value(&mut self, id: usize, cell: CellReferenceIndex) -> Option<CalcResult> {
        let binding = self.bindings.get_mut(&id)?;
        if let Binding::ReadOnce(_) = binding {
            let read_twice = CalcResult::new_error(
                Error::ERROR,
                cell,
                "A LAMBDA parameter read once was read again".to_string(),
            );
            if let Binding::ReadOnce(value) = std::mem::replace(binding, Binding::Value(read_twice))
            {
                return Some(value);
            }
        }
        Some(binding.value(cell))
    }

    /// Turns every [`Binding::ReadOnce`] into a copied [`Binding::Value`]. IF over an array
    /// condition evaluates both branches, where `read_once` (functions/logical/lambda.rs) counts
    /// a read in each as one, since a scalar condition evaluates only one (T-1593).
    pub(crate) fn copy_read_once(&mut self) {
        for binding in self.bindings.values_mut() {
            if let Binding::ReadOnce(value) = binding {
                let value = std::mem::replace(value, CalcResult::EmptyArg);
                *binding = Binding::Value(value);
            }
        }
    }

    /// Binds `id`; a union that would pass [`MAX_HELD_AREAS`], or a grid that would pass
    /// [`MAX_HELD_GRID_ELEMENTS`], is bound to `#VALUE!` instead.
    pub(crate) fn insert(&mut self, id: usize, binding: Binding, cell: CellReferenceIndex) {
        let held = self.held_areas + binding.held_areas();
        let new_elements = binding
            .grid()
            .filter(|grid| !self.holds(grid))
            .map_or(0, |grid| grid.elements().len());
        let held_elements = self.held_grid_elements.saturating_add(new_elements);
        let binding = if held > MAX_HELD_AREAS {
            Binding::Value(too_many_areas(cell))
        } else if held_elements > MAX_HELD_GRID_ELEMENTS {
            Binding::Value(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Too many references held by LET variables".to_string(),
            ))
        } else {
            self.held_areas = held;
            self.held_grid_elements = held_elements;
            binding
        };
        if let Some(old) = self.bindings.insert(id, binding) {
            self.release(&old);
        }
    }

    pub(crate) fn remove(&mut self, id: usize) {
        if let Some(old) = self.bindings.remove(&id) {
            self.release(&old);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.bindings.clear();
        self.held_areas = 0;
        self.held_grid_elements = 0;
    }

    /// Whether a binding in scope shares `grid`'s elements, which are then charged already.
    fn holds(&self, grid: &RefGrid) -> bool {
        self.bindings
            .values()
            .filter_map(Binding::grid)
            .any(|held| held.shares_elements(grid))
    }

    /// Uncharges a binding no longer in scope: its areas, and its grid's elements unless another
    /// binding still shares them.
    fn release(&mut self, old: &Binding) {
        self.held_areas -= old.held_areas();
        if let Some(grid) = old.grid() {
            if !self.holds(grid) {
                self.held_grid_elements -= grid.elements().len();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference_operators::RefElement;

    fn grid(elements: usize) -> RefGrid {
        RefGrid::new(elements, 1, vec![RefElement::Error(Error::NA); elements])
            .expect("a non-empty grid")
    }

    fn is_grid(stack: &VariableStack, id: usize) -> bool {
        matches!(stack.get(id), Some(Binding::Grid(_)))
    }

    // Four whole-column grids fill the budget; a fifth new one, however small, is `#VALUE!`,
    // while clones of a held grid cost nothing (SPEC-0014 "Pass-through", DEVIATION).
    #[test]
    fn held_grids_are_charged_once_and_bounded() {
        let cell = CellReferenceIndex {
            sheet: 0,
            row: 1,
            column: 1,
        };
        let mut stack = VariableStack::default();
        let first = grid(MAX_SIZE);
        stack.insert(0, Binding::Grid(first.clone()), cell);
        for id in 1..4 {
            stack.insert(id, Binding::Grid(grid(MAX_SIZE)), cell);
        }
        for id in 4..8 {
            stack.insert(id, Binding::Grid(first.clone()), cell);
            assert!(is_grid(&stack, id), "a clone is not charged");
        }
        stack.insert(8, Binding::Grid(grid(1)), cell);
        assert!(
            !is_grid(&stack, 8),
            "past the budget the binding is #VALUE!"
        );
        assert!(matches!(
            stack.get(8).map(|binding| binding.value(cell)),
            Some(CalcResult::Error {
                error: Error::VALUE,
                ..
            })
        ));
        // The first grid stays charged while a clone of it is in scope.
        stack.remove(0);
        stack.insert(9, Binding::Grid(grid(1)), cell);
        assert!(!is_grid(&stack, 9));
        // Once no binding holds it, its elements are free again.
        for id in 4..8 {
            stack.remove(id);
        }
        stack.insert(10, Binding::Grid(grid(1)), cell);
        assert!(is_grid(&stack, 10));
        assert_eq!(stack.held_grid_elements, 3 * MAX_SIZE + 1);
    }
}
