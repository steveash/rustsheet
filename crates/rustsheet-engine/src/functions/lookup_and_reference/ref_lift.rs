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
use crate::functions::aggregate::value_node;
use crate::functions::lift::{ArgBuf, LiftedArg};
use crate::functions::util::top_left_of_array;
use crate::functions::Function;
use crate::limits::check_array_size;
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

/// A consumer that calls itself once per element of an array of references (SPEC-0014 rule 6):
/// which of its arguments take a reference, and what an element costs.
#[derive(Clone, Copy)]
pub(crate) struct PerElement {
    /// Whether the argument at this position takes a reference.
    pub(crate) takes_reference: fn(usize) -> bool,
    cost: ElementCost,
}

impl Function {
    /// The consumers that call themselves once per element of an array of references given
    /// where they take a reference (SPEC-0014 rule 6), for a call of `count` arguments.
    /// `SUBTOTAL(9,OFFSET(A1,{0;1},0))` spills 1, 100; `ROW(OFFSET(A1,{0;1},0))` 1, 2
    /// (probes/reference-lift-consumers E57-E78, E97, E109, E165, E169, E185; -b1c E85-E93;
    /// -b1d E97; -b2 E125-E133, E177, E193, E197). COUNTIFS and the other *IFS stay the value
    /// view until T-2065 probes them.
    pub(crate) fn per_reference_element(&self, count: usize) -> Option<PerElement> {
        let first: fn(usize) -> bool = |index| index == 0;
        let (takes_reference, cost): (fn(usize) -> bool, ElementCost) = match self {
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
        };
        Some(PerElement {
            takes_reference,
            cost,
        })
    }
}

/// An absolute reference to `range`, the argument a per-element call reads in place of the
/// array of references it came from, or of a reference argument read once.
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

/// Whether `node` is an OFFSET or INDIRECT call, which lifts into an array of references when an
/// argument lifts it: the only argument a per-element consumer reads ahead of its call (SPEC-0014
/// rule 6). A TRIMRANGE is not, as SUBTOTAL and AREAS read one as the cells it picks
/// (`SUBTOTAL(9,TRIMRANGE(A1:A2))` is 3, probes/subtotal-reference-operands-b H17); nor are IF,
/// CHOOSE and LET handing a grid on (unprobed).
pub(crate) fn lifts_into_references(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::FunctionKind {
            kind: Function::Offset | Function::Indirect,
            ..
        }
    )
}

/// A node that reads again what `read` holds at no more cost than its cells: one range as an
/// absolute reference, several areas as their union, an array of references as its value view
/// (`#VALUE!` per reference element), an array as a literal, a scalar as a literal. `None` for a
/// 3-D range, an empty value and a LAMBDA.
pub(super) fn put_back(read: &RefArg) -> Option<Node> {
    match read {
        RefArg::Ranges(Ranges::One(range)) if range.left.sheet == range.right.sheet => {
            Some(range_node(range))
        }
        RefArg::Ranges(Ranges::One(_)) => None,
        RefArg::Ranges(areas) => union_node(areas),
        RefArg::Grid(grid) => Some(Node::ArrayKind(
            grid.to_array(ArrayNode::Error(Error::VALUE)),
        )),
        RefArg::Value(CalcResult::Array(rows)) => Some(Node::ArrayKind(rows.clone())),
        RefArg::Value(CalcResult::EmptyCell | CalcResult::EmptyArg) => None,
        RefArg::Value(value) => value_node(value),
    }
}

/// The union of `areas` as a balanced tree of absolute references, so reading it nests no
/// deeper than the logarithm of the count.
fn union_node(areas: &[Range]) -> Option<Node> {
    match areas {
        [] => None,
        [range] => Some(range_node(range)),
        _ => {
            let (left, right) = areas.split_at(areas.len() / 2);
            Some(Node::OpUnionKind {
                left: Box::new(union_node(left)?),
                right: Box::new(union_node(right)?),
                bare: false,
            })
        }
    }
}

/// Whether `node` is read again at no more cost than its cells: a literal or a written
/// reference.
fn reads_cheaply(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::NumberKind(_)
            | Node::StringKind(_)
            | Node::BooleanKind(_)
            | Node::ErrorKind(_)
            | Node::EmptyArgKind
            | Node::ArrayKind(_)
            | Node::ImplicitIntersection { .. }
            | Node::ReferenceKind { .. }
            | Node::RangeKind { .. }
            | Node::ThisRowKind { .. }
    )
}

/// The element of `grid` at (`row`, `column`) of a broadcast: a grid of one row or one column
/// repeats it, and a position past a shorter grid is `None`.
fn broadcast_element(grid: &RefGrid, row: usize, column: usize) -> Option<&RefElement> {
    let (rows, columns) = grid.dims();
    let at = |i: usize, n: usize| if n == 1 { 0 } else { i };
    grid.get(at(row, rows), at(column, columns))
}

/// Cells in an array value: what reading it once more costs.
fn array_cells(rows: &[Vec<ArrayNode>]) -> u64 {
    rows.iter()
        .map(|row| u64::try_from(row.len()).unwrap_or(u64::MAX))
        .fold(0, u64::saturating_add)
}

/// Whether an argument's value lifts TRIMRANGE: an array or a range of more than one cell. A
/// one-cell range and a 1x1 array are their value, as before (whether Excel lifts a 1x1 array
/// there is unprobed, TODO(T-2347); OFFSET, INDIRECT, TAKE and DROP do, [`lifts_one_by_one`]).
pub(super) fn lifts(value: &CalcResult) -> bool {
    match value {
        CalcResult::Range { left, right } => left != right,
        CalcResult::Array(rows) => rows.len() > 1 || rows.first().is_some_and(|row| row.len() > 1),
        _ => false,
    }
}

/// Whether an argument's value lifts OFFSET, INDIRECT, TAKE or DROP: what [`lifts`] accepts, and
/// a 1x1 array too, which gives one element: `TAKE(A1:B3,{2})` is one element, A1:B2, so ROWS of
/// it is 1 and spilled directly it is `#VALUE!` (probes/m7-lift-take-drop-sortby Sheet1 F13, G13,
/// I13:K13, W65); ROWS of `DROP(A1:B3,{1})` and of `OFFSET(A1:B2,{1},0)` is 1 too
/// (probes/m7-one-cell-operand-lift F6, F7, F16, F17; T-2298), and `INDIRECT("B1",{1})` is
/// `#VALUE!` (probes/t2577-indirect-ref-array A1, A25; T-2577). A one-cell range is still its
/// value.
pub(super) fn lifts_one_by_one(value: &CalcResult) -> bool {
    lifts(value) || matches!(value, CalcResult::Array(_))
}

/// Whether `node` may stand for an array of references (SPEC-0014 rule 7): a call to a producer
/// (OFFSET, INDIRECT, TRIMRANGE), an IF, CHOOSE or LET that may hand one on, or a variable. Read
/// without evaluating, so a consumer asks for the grid only where one can come.
pub(super) fn may_hand_on_grid(node: &Node) -> bool {
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
    /// The reference arguments of `args` (positions `takes_reference`) that are a lifting call
    /// ([`lifts_into_references`]), each read once: each one that is an array of references is
    /// returned with its position, and every other one is put back as what it gave
    /// ([`put_back`]), so the call never evaluates it again (a nest of such calls stays linear).
    /// What cannot be put back (a 3-D range, an empty value) is `#VALUE!`, as one sheet's
    /// reference is wanted.
    pub(crate) fn reference_grid_args<'a>(
        &mut self,
        args: &'a [Node],
        takes_reference: fn(usize) -> bool,
        cell: CellReferenceIndex,
    ) -> (Cow<'a, [Node]>, Vec<(usize, RefGrid)>) {
        let mut resolved = Cow::Borrowed(args);
        let mut grids = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            if !takes_reference(index) || !lifts_into_references(arg) {
                continue;
            }
            let read = self.evaluate_reference_arg(arg, cell);
            if let RefArg::Grid(grid) = read {
                grids.push((index, grid));
            } else if let Some(slot) = resolved.to_mut().get_mut(index) {
                *slot = put_back(&read).unwrap_or(Node::ErrorKind(Error::VALUE));
            }
        }
        (resolved, grids)
    }

    /// Calls the implementation of `kind`: through [`Self::call_per_reference_element`] for a
    /// per-element consumer ([`Function::per_reference_element`]), directly otherwise.
    pub(crate) fn call_by_reference_elements(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> CalcResult {
        match kind.per_reference_element(args.len()) {
            Some(per_element) => {
                self.call_per_reference_element(args, per_element, cell, implementation)
            }
            None => implementation(self, args, cell),
        }
    }

    /// Calls `implementation` once per element of every array of references among its reference
    /// arguments (SPEC-0014 rule 6), the element in the argument's place, and spills the results
    /// in the grids' broadcast shape (a position past a shorter grid is `#N/A`); an element that
    /// is an error is that error in the argument's place, and an element's array result is its
    /// top-left value. Without such an argument it is one call.
    ///
    /// The other arguments are the same for every element, so each is read once, before any
    /// element runs, and put back as a literal, an array or an absolute reference; then the shape
    /// (`check_array_size`) and the cost of every element are checked (directive 5): the cells
    /// its references walk ([`ElementCost`]) and those the other arguments read. An argument that
    /// cannot be put back (a 3-D range, a LAMBDA, an empty value) makes the call `#VALUE!`
    /// (unprobed in Excel). Out of line: inlined into `evaluate_function`, its locals made the
    /// frame of every function call, and of every nested LAMBDA call, larger (T-0444).
    #[inline(never)]
    pub(crate) fn call_per_reference_element(
        &mut self,
        args: &[Node],
        per_element: PerElement,
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> CalcResult {
        let (resolved, grids) = self.reference_grid_args(args, per_element.takes_reference, cell);
        if grids.is_empty() {
            return implementation(self, &resolved, cell);
        }
        let (rows, columns) = grids.iter().fold((1, 1), |(rows, columns), (_, grid)| {
            let (r, c) = grid.dims();
            (rows.max(r), columns.max(c))
        });
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        let walks = match per_element.cost {
            ElementCost::Unit => 0,
            ElementCost::Cells(walks) => walks,
        };
        let mut element_args = resolved.into_owned();
        // What the arguments that are not grids cost each element.
        let mut shared: u64 = 1;
        for (index, arg) in element_args.iter_mut().enumerate() {
            if grids.iter().any(|(at, _)| *at == index) {
                continue;
            }
            let Some(cells) = self.read_once(arg, cell) else {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "An argument beside an array of references".to_string(),
                );
            };
            // A reference argument is walked; a value argument is read.
            let reads = if (per_element.takes_reference)(index) {
                cells.saturating_mul(walks)
            } else {
                cells
            };
            shared = shared.saturating_add(reads);
        }
        let mut visits: u64 = 0;
        for row in 0..rows {
            for column in 0..columns {
                let mut element_cost = shared;
                for (_, grid) in &grids {
                    if let (Some(RefElement::Reference(range)), true) =
                        (broadcast_element(grid, row, column), walks > 0)
                    {
                        let value = CalcResult::Range {
                            left: range.left,
                            right: range.right,
                        };
                        let cells = self.range_walk_cost(&value);
                        element_cost = element_cost.saturating_add(cells.saturating_mul(walks));
                    }
                }
                visits = visits.saturating_add(element_cost);
            }
        }
        if let Some(error) = self.spend_visits(visits, cell) {
            return error;
        }
        let mut output = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut output_row = Vec::with_capacity(columns);
            for column in 0..columns {
                output_row.push(self.call_on_element(
                    &mut element_args,
                    &grids,
                    (row, column),
                    cell,
                    implementation,
                ));
            }
            output.push(output_row);
        }
        CalcResult::Array(output)
    }

    /// One element's call: each grid's element at `at` put in its argument's place.
    fn call_on_element(
        &mut self,
        element_args: &mut [Node],
        grids: &[(usize, RefGrid)],
        (row, column): (usize, usize),
        cell: CellReferenceIndex,
        implementation: Implementation,
    ) -> ArrayNode {
        for (index, grid) in grids {
            let (Some(element), Some(slot)) = (
                broadcast_element(grid, row, column),
                element_args.get_mut(*index),
            ) else {
                return ArrayNode::Error(Error::NA);
            };
            *slot = match element {
                RefElement::Reference(range) => range_node(range),
                RefElement::Error(error) => Node::ErrorKind(error.clone()),
            };
        }
        match implementation(self, element_args, cell) {
            CalcResult::Array(array) => calc_result_to_array_node(top_left_of_array(&array, cell)),
            value => calc_result_to_array_node(value),
        }
    }

    /// Reads an argument that is the same for every element once, and puts it back as what it
    /// gave ([`put_back`]). Returns the cells a further read costs (a reference's cells inside
    /// the used area, an array's cells, 1 for a literal), or `None` when the value cannot be put
    /// back.
    fn read_once(&mut self, arg: &mut Node, cell: CellReferenceIndex) -> Option<u64> {
        let value = if reads_cheaply(arg) {
            self.evaluate_node_with_reference(arg, cell)
        } else {
            let value = self.evaluate_node_in_context(arg, cell);
            let read = RefArg::from(value.clone());
            *arg = put_back(&read)?;
            value
        };
        Some(match &value {
            CalcResult::Range { left, right } if left.sheet != right.sheet => return None,
            CalcResult::Range { .. } => self.range_walk_cost(&value),
            CalcResult::Array(rows) => array_cells(rows).max(1),
            _ => 1,
        })
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

    /// OFFSET, INDIRECT, TRIMRANGE, TAKE, DROP or VSTACK read as a reference argument: one
    /// reference (an unlifted TAKE or DROP: its value, as a value argument reads it), or an array
    /// of references when an argument lifts the call, or VSTACK stacks such arrays
    /// ([`Model::vstack_reference`]; otherwise VSTACK's value). `kind` is one of the six.
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
            Function::Vstack => match self.vstack_reference(args, cell) {
                Some(stacked) => stacked,
                None => RefArg::from(self.fn_vstack(args, cell)),
            },
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
        self.reference_lift_values_read(args, cell, Self::evaluate_node_in_context)
    }

    /// [`Self::reference_lift_values`] with each argument read by `read`.
    pub(super) fn reference_lift_values_read(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        read: fn(&mut Self, &Node, CellReferenceIndex) -> CalcResult,
    ) -> ArgBuf<CalcResult> {
        let mut values = ArgBuf::filled(args.len(), || CalcResult::EmptyArg);
        for (slot, arg) in values.as_mut_slice().iter_mut().zip(args) {
            if !matches!(arg, Node::EmptyArgKind) {
                *slot = read(self, arg, cell);
            }
        }
        values
    }

    /// The driver's arguments for `values`: those that `lifts_value` accepts ([`lifts`] or
    /// [`lifts_one_by_one`]: the reason it is a parameter) read into their cells, the rest
    /// scalars. An unreadable one (a range across sheets) is the call's error.
    pub(super) fn reference_lift_args(
        &mut self,
        values: ArgBuf<CalcResult>,
        lifts_value: fn(&CalcResult) -> bool,
        cell: CellReferenceIndex,
    ) -> Result<Vec<LiftedArg>, CalcResult> {
        let mut lifted = Vec::with_capacity(values.as_slice().len());
        for value in values {
            lifted.push(if lifts_value(&value) {
                LiftedArg::read(self, value, cell)?
            } else {
                LiftedArg::Scalar(value)
            });
        }
        Ok(lifted)
    }
}
