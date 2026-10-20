use crate::functions::Function;

use super::{run_static_analysis_on_node, Node, StaticResult};

/// RANK's `ref` (`ArgKind::ReferenceByCaller`) must be a reference: an array that is not one
/// (`IF(A1:C1>0,A1:C1)`) broadcasts as a `#VALUE!` array of that array's shape, which spills
/// (docs/excel-semantics/statistics.md, "RANK of a non-reference array"). A `ref` whose shape is
/// `Unknown` (INDEX, CHOOSE, OFFSET, a defined name, `A1:<name>`) may still evaluate to such an
/// array, so its shape is `Unknown` too: a reference ranks to a 1x1 result, an array spills
/// (probes/rank-unknown-shape-ref).
pub(super) fn reference_by_caller_shape(reference: &Node) -> StaticResult {
    reference_parameter_static_result(Some(reference))
}

/// COUNTBLANK of an array is one `#VALUE!` per element (docs/excel-semantics/references.md,
/// "COUNTBLANK of a value that is not a reference is #VALUE!"; probes/countblank-array-values).
pub(super) fn countblank_static_result(args: &[Node]) -> StaticResult {
    reference_parameter_static_result(args.first())
}

/// The result shape of a function whose parameter `reference` must be a reference (RANK's
/// `ref`, COUNTBLANK's `range`): an array that is not one makes a `#VALUE!` array of its shape,
/// and a `reference` of unknown shape may still be such an array.
fn reference_parameter_static_result(reference: Option<&Node>) -> StaticResult {
    let Some(reference) = reference else {
        return StaticResult::Scalar;
    };
    match run_static_analysis_on_node(reference) {
        shape @ (StaticResult::Array(..) | StaticResult::Unknown) => shape,
        // CHOOSE with an array index lifts to an array of values, though its shape reads as a
        // range (`COUNTBLANK(CHOOSE({1,2},D1:D2,D3:D4))` is a 2x2 of `#VALUE!`,
        // probes/countblank-array-values H20).
        StaticResult::Range(..) if is_choose_with_array_index(reference) => StaticResult::Unknown,
        _ => StaticResult::Scalar,
    }
}

/// Whether `node` is a CHOOSE call whose index is not a scalar.
fn is_choose_with_array_index(node: &Node) -> bool {
    match node.ungrouped() {
        Node::FunctionKind {
            kind: Function::Choose,
            args,
        } => args.first().is_some_and(|index| {
            !matches!(run_static_analysis_on_node(index), StaticResult::Scalar)
        }),
        _ => false,
    }
}
