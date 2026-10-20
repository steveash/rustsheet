//! What OFFSET, INDIRECT and TRIMRANGE share when they lift into an array of references
//! (SPEC-0014 "Producers"): which argument values lift the call, and their conversion to the
//! driver's arguments.

use crate::calc_result::CalcResult;
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{ArgBuf, LiftedArg};
use crate::functions::Function;
use crate::model::Model;
use crate::reference_operators::RefArg;

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

impl Model {
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
