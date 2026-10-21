//! What OFFSET, INDIRECT and TRIMRANGE share when they lift into an array of references
//! (SPEC-0014 "Producers"): which argument values lift the call, and their conversion to the
//! driver's arguments; and where IF, CHOOSE, LET and variables hand such an array on (rule 7).

use std::borrow::Cow;

use super::drop_take::Edge;
use crate::calc_result::{CalcResult, Range};
use crate::cast::calc_result_to_array_node;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{ArgBuf, LiftedArg};
use crate::functions::util::top_left_of_array;
use crate::functions::Function;
use crate::limits::{check_array_size, too_many_visits};
use crate::model::Model;
use crate::reference_operators::{Picked, Ranges, RefArg, RefElement, RefGrid};

/// The signature of a function implementation, as `Model::evaluate_function` calls it.
type Implementation = fn(&mut Model, &[Node], CellReferenceIndex) -> CalcResult;

/// What one call of a per-element consumer costs (SPEC-0014 "Bounds"), charged for every element
/// to the lift's visit budget (`limits::MAX_LIFTED_CELL_VISITS`) before any element runs.
#[derive(Clone, Copy)]
pub(crate) enum ElementCost {
    /// One visit: ROW, COLUMN, AREAS, CELL, ISFORMULA, N and T read no cells or one.
    Unit,
    /// The cells of the element's references inside the used area, times this many walks:
    /// SUBTOTAL, COUNTIF, SUMIF and COUNTBLANK walk them once, SUMIFS once per criteria pair.
    Cells(u64),
}

impl Function {
    /// The consumers that call themselves once per element of an array of references given
    /// where they take a reference (SPEC-0014 rule 6): which of `count` arguments take one, and
    /// what an element costs. `SUBTOTAL(9,OFFSET(A1,{0;1},0))` spills 1, 100;
    /// `ROW(OFFSET(A1,{0;1},0))` 1, 2 (probes/reference-lift-consumers E57-E78, E97, E109, E165,
    /// E169, E185; -b1c E85-E93; -b1d E97; -b2 E125-E133, E177, E193, E197). COUNTIFS and the
    /// other *IFS stay the value view until T-2065 probes them.
    pub(crate) fn per_reference_element(
        &self,
        count: usize,
    ) -> Option<(fn(usize) -> bool, ElementCost)> {
        let first: fn(usize) -> bool = |index| index == 0;
        Some(match self {
            Function::Row
            | Function::Column
            | Function::Areas
            | Function::Isformula
            | Function::N
            | Function::T => (first, ElementCost::Unit),
            Function::Cell => (|index| index == 1, ElementCost::Unit),
            Function::Countif | Function::Countblank => (first, ElementCost::Cells(1)),
            Function::Subtotal => (|index| index > 0, ElementCost::Cells(1)),
            // The criteria range and the sum range.
            Function::Sumif => (|index| index != 1, ElementCost::Cells(1)),
            // The sum range and every criteria range.
            Function::Sumifs => (
                |index| index == 0 || index % 2 == 1,
                ElementCost::Cells(u64::try_from(count / 2).unwrap_or(u64::MAX).max(1)),
            ),
            _ => return None,
        })
    }
}

/// An absolute reference to `range`, the argument a per-element call reads in place of the
/// array of references it came from.
fn range_node(range: &Range) -> Node {
    Node::RangeKind {
        sheet_name: None,
        sheet_index: range.left.sheet,
        absolute_row1: true,
        absolute_column1: true,
        row1: range.left.row,
        column1: range.left.column,
        absolute_row2: true,
        absolute_column2: true,
        row2: range.right.row,
        column2: range.right.column,
    }
}

/// `args` with every argument at a position in `grids` replaced by that grid's element at
/// (`row`, `column`), broadcast as a lift broadcasts: a grid of one row or one column repeats
/// it, and a position past a shorter grid is `None` (the element is `#N/A`).
fn element_args(
    args: &[Node],
    grids: &[(usize, RefGrid)],
    row: usize,
    column: usize,
) -> Option<Vec<Node>> {
    let mut element_args = args.to_vec();
    for (index, grid) in grids {
        let (rows, columns) = grid.dims();
        let at = |i: usize, n: usize| if n == 1 { 0 } else { i };
        let element = grid.get(at(row, rows), at(column, columns))?;
        let slot = element_args.get_mut(*index)?;
        *slot = match element {
            RefElement::Reference(range) => range_node(range),
            RefElement::Error(error) => Node::ErrorKind(error.clone()),
        };
    }
    Some(element_args)
}

/// Whether an argument's value lifts the call: an array or a range of more than one cell. A
/// one-cell range and a 1x1 array are their value, as before (whether Excel lifts
/// `OFFSET(A1,{1},0)` is unprobed; `Model::lift_references` keeps a 1x1 broadcast scalar too).
pub(super) fn lifts(value: &CalcResult) -> bool {
    match value {
        CalcResult::Range { left, right } => left != right,
        CalcResult::Array(rows) => rows.len() > 1 || rows.first().is_some_and(|row| row.len() > 1),
        _ => false,
    }
}

/// Whether `node` may stand for an array of references (SPEC-0014 rule 7): a call to a producer
/// (OFFSET, INDIRECT, TRIMRANGE), an IF, CHOOSE or LET that may hand one on, or a variable. Read
/// without evaluating, so a consumer asks for the grid only where one can come.
pub(crate) fn may_hand_on_grid(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::NamedVariableKind { .. }
            | Node::FunctionKind {
                kind: Function::Offset
                    | Function::Indirect
                    | Function::Trimrange
                    | Function::If
                    | Function::Choose
                    | Function::Let,
                ..
            }
    )
}

impl Model {
    /// The reference arguments of `args` (positions `takes_reference`) read once where they may
    /// stand for an array of references ([`may_hand_on_grid`]): each one that is a grid is
    /// returned with its position, and each one that is one range on one sheet is replaced by
    /// that range, so the call does not evaluate it again. Anything else stays as written.
    pub(crate) fn reference_grid_args<'a>(
        &mut self,
        args: &'a [Node],
        takes_reference: fn(usize) -> bool,
        cell: CellReferenceIndex,
    ) -> (Cow<'a, [Node]>, Vec<(usize, RefGrid)>) {
        let mut resolved = Cow::Borrowed(args);
        let mut grids = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            if !takes_reference(index) || !may_hand_on_grid(arg) {
                continue;
            }
            match self.evaluate_reference_arg(arg, cell) {
                RefArg::Grid(grid) => grids.push((index, grid)),
                RefArg::Ranges(Ranges::One(range)) if range.left.sheet == range.right.sheet => {
                    if let Some(slot) = resolved.to_mut().get_mut(index) {
                        *slot = range_node(&range);
                    }
                }
                _ => {}
            }
        }
        (resolved, grids)
    }

    /// Calls `implementation` once per element of every array of references among its reference
    /// arguments (SPEC-0014 rule 6), the element in the argument's place, and spills the results
    /// in the grids' broadcast shape; an element that is an error is that error in the
    /// argument's place, and an element's array result is its top-left value. Without such an
    /// argument it is one call. The shape (`check_array_size`) and the elements' cost
    /// ([`ElementCost`], `too_many_visits`) are checked before any element runs.
    pub(crate) fn call_per_reference_element(
        &mut self,
        args: &[Node],
        (takes_reference, cost): (fn(usize) -> bool, ElementCost),
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> CalcResult {
        let (args, grids) = self.reference_grid_args(args, takes_reference, cell);
        if grids.is_empty() {
            return implementation(self, &args, cell);
        }
        let (rows, columns) = grids.iter().fold((1, 1), |(rows, columns), (_, grid)| {
            let (r, c) = grid.dims();
            (rows.max(r), columns.max(c))
        });
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        let mut visits: u64 = 0;
        for (_, grid) in &grids {
            for element in grid.elements() {
                visits = visits.saturating_add(match (cost, element) {
                    (ElementCost::Cells(walks), RefElement::Reference(range)) => {
                        let value = CalcResult::Range {
                            left: range.left,
                            right: range.right,
                        };
                        self.range_walk_cost(&value).saturating_mul(walks)
                    }
                    _ => 1,
                });
            }
        }
        if let Some(error) = too_many_visits(visits, cell) {
            return error;
        }
        let mut output = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut output_row = Vec::with_capacity(columns);
            for column in 0..columns {
                output_row.push(match element_args(&args, &grids, row, column) {
                    Some(element_args) => match implementation(self, &element_args, cell) {
                        CalcResult::Array(array) => {
                            calc_result_to_array_node(top_left_of_array(&array, cell))
                        }
                        value => calc_result_to_array_node(value),
                    },
                    None => ArrayNode::Error(Error::NA),
                });
            }
            output.push(output_row);
        }
        CalcResult::Array(output)
    }

    /// A value argument whose consumer reads an array of references by its elements (SPEC-0014
    /// "Consumers"): a lifted producer is its grid, anything else its value as
    /// `evaluate_node_in_context` gives it, a range as `RefArg::Ranges`. The producers are
    /// OFFSET, INDIRECT, TRIMRANGE, TAKE and DROP ([`Model::reference_producer`]); IF, CHOOSE,
    /// LET and a variable hand a producer's grid on, as they do to an aggregate:
    /// `SUMPRODUCT(IF(TRUE,OFFSET(A1,{0;1},0)))` is 0 as the bare OFFSET is (rule 7;
    /// probes/reference-lift-pass-through E5-E7). Each picks its argument as its value form does
    /// (`if_branch_value`, `choose_arm_value`, LET's body), so nothing else changes.
    pub(crate) fn evaluate_value_or_grid(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> RefArg {
        match node.ungrouped() {
            Node::FunctionKind {
                kind:
                    kind @ (Function::Offset
                    | Function::Indirect
                    | Function::Trimrange
                    | Function::Take
                    | Function::Drop),
                args,
            } => self.reference_producer(kind, args, cell),
            Node::FunctionKind {
                kind: Function::If,
                args,
            } => match self.if_pick(args, cell) {
                Picked::Node(arm) if may_hand_on_grid(arm) => {
                    self.evaluate_value_or_grid(arm, cell)
                }
                Picked::Node(arm) => RefArg::from(self.if_branch_value(arm, cell)),
                Picked::Value(value) => RefArg::Value(value),
            },
            Node::FunctionKind {
                kind: Function::Choose,
                args,
            } => match self.choose_pick(args, cell) {
                Picked::Node(arm) if may_hand_on_grid(arm) => {
                    self.evaluate_value_or_grid(arm, cell)
                }
                Picked::Node(arm) => RefArg::from(self.choose_arm_value(arm, cell)),
                Picked::Value(value) => RefArg::Value(value),
            },
            Node::FunctionKind {
                kind: Function::Let,
                args,
            } => self
                .let_scope(args, cell, |model, body| {
                    model.evaluate_value_or_grid(body, cell)
                })
                .unwrap_or_else(RefArg::Value),
            node @ Node::NamedVariableKind { .. } if self.is_grid_variable(node) => {
                self.variable_reference(node, cell)
            }
            _ => RefArg::from(self.evaluate_node_in_context(node, cell)),
        }
    }

    /// OFFSET, INDIRECT, TRIMRANGE, TAKE or DROP read as a reference argument: one reference (an
    /// unlifted TAKE or DROP: its value, as a value argument reads it), or an array of
    /// references when an argument lifts the call (`kind` is one of the five).
    pub(crate) fn reference_producer(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> RefArg {
        match kind {
            Function::Indirect => self.indirect_reference(args, cell),
            Function::Trimrange => self.trimrange_reference(args, cell, false),
            Function::Take => self.take_drop_reference(args, cell, Edge::Take, false),
            Function::Drop => self.take_drop_reference(args, cell, Edge::Drop, false),
            _ => self.offset_reference(args, cell),
        }
    }

    /// The values of `args`, held inline (no heap allocation for the scalar call, SPEC-0014
    /// "Producers"); a written empty argument is `EmptyArg`.
    pub(super) fn reference_lift_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> ArgBuf<CalcResult> {
        let mut values = ArgBuf::filled(args.len(), || CalcResult::EmptyArg);
        for (slot, arg) in values.as_mut_slice().iter_mut().zip(args) {
            if !matches!(arg, Node::EmptyArgKind) {
                *slot = self.evaluate_node_in_context(arg, cell);
            }
        }
        values
    }

    /// The driver's arguments for `values`: those that [`lifts`] read into their cells, the rest
    /// scalars. An unreadable one (a range across sheets) is the call's error.
    pub(super) fn reference_lift_args(
        &mut self,
        values: ArgBuf<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<Vec<LiftedArg>, CalcResult> {
        let mut lifted = Vec::with_capacity(values.as_slice().len());
        for value in values {
            lifted.push(if lifts(&value) {
                LiftedArg::read(self, value, cell)?
            } else {
                LiftedArg::Scalar(value)
            });
        }
        Ok(lifted)
    }
}
