//! What OFFSET, INDIRECT and TRIMRANGE share when they lift into an array of references
//! (SPEC-0014 "Producers"): which argument values lift the call, and their conversion to the
//! driver's arguments; and where IF, CHOOSE, LET and variables hand such an array on (rule 7).

use crate::calc_result::CalcResult;
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{ArgBuf, LiftedArg};
use crate::functions::Function;
use crate::model::Model;
use crate::reference_operators::{Picked, RefArg};

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
    /// A value argument whose consumer reads an array of references by its elements (SPEC-0014
    /// "Consumers"): a lifted producer is its grid, anything else its value as
    /// `evaluate_node_in_context` gives it, a range as `RefArg::Ranges`. OFFSET, INDIRECT and
    /// TRIMRANGE are the producers so far (TODO(T-1838) TAKE and DROP join them); IF, CHOOSE,
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
                kind: kind @ (Function::Offset | Function::Indirect | Function::Trimrange),
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

    /// OFFSET, INDIRECT or TRIMRANGE read as a reference argument: one reference, or an array of
    /// references when an argument lifts the call (`kind` is one of the three).
    pub(crate) fn reference_producer(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> RefArg {
        match kind {
            Function::Indirect => self.indirect_reference(args, cell),
            Function::Trimrange => self.trimrange_reference(args, cell, false),
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
