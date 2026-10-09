use crate::functions::Function;

use super::{broadcast_static_result, run_static_analysis_on_node, Node, StaticResult};

/// AGGREGATE spills over an array function_num, options or fourth argument, broadcast against
/// each other (`Model::fn_aggregate`). A multi-cell range function_num or options lifts too
/// (probes/aggregate-range-args); a single cell and a range fourth argument are read, not lifted.
pub(super) fn aggregate_static_result(args: &[Node]) -> StaticResult {
    let lifted = args
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < 2 || (*index == 3 && args.len() == 4))
        .filter(|(index, arg)| match run_static_analysis_on_node(arg) {
            StaticResult::Range(rows, columns) => *index < 2 && (rows, columns) != (1, 1),
            _ => true,
        })
        .map(|(_, arg)| arg);
    broadcast_static_result(lifted)
}

/// SUBTOTAL spills when an argument after the function number evaluates to an array (each
/// element is `#VALUE!` or its own error, `Model::subtotal_reference_args`, T-1163). A written
/// reference is read, not lifted; anything else counts at its shape, a call even when that is
/// a range, since CHOOSE gives a range's shape to its array arms (`CHOOSE(1,{1,2})`). Not
/// `is_reference_operand`: that one accepts calls, which can evaluate to arrays.
pub(super) fn subtotal_static_result(args: &[Node]) -> StaticResult {
    broadcast_static_result(args.iter().skip(1).filter(|arg| !is_read_reference(arg)))
}

/// AREAS spills the same way over its one argument (`AREAS(IF(TRUE,{1,2}))` is two `#VALUE!`,
/// probes/subtotal-reference-operands-b H4, T-1175).
pub(super) fn areas_static_result(args: &[Node]) -> StaticResult {
    broadcast_static_result(args.iter().filter(|arg| !is_read_reference(arg)))
}

/// An argument that SUBTOTAL and AREAS read as a reference instead of lifting over: a written
/// reference, or XLOOKUP, TAKE, DROP and TRIMRANGE, which evaluate to the cells they pick
/// (`Model::evaluate_reference_arg`, probes/subtotal-reference-operands-b H12, H15:H17).
fn is_read_reference(arg: &Node) -> bool {
    matches!(
        arg,
        Node::ReferenceKind { .. }
            | Node::RangeKind { .. }
            | Node::OpRangeKind { .. }
            | Node::OpIntersectKind { .. }
            | Node::OpUnionKind { .. }
            | Node::SheetSpanKind { .. }
            | Node::TableNameKind(_)
            | Node::FunctionKind {
                kind: Function::Xlookup | Function::Take | Function::Drop | Function::Trimrange,
                ..
            }
    )
}
