//! The result shapes of the reference-returning OFFSET, INDIRECT and CHOOSE. Moved out of
//! `static_analysis.rs` (T-1770, T-1874) to keep it under its size lock.

use super::broadcast::broadcast_shapes;
use super::{run_static_analysis_on_node, Node, StaticResult};

pub(super) fn static_analysis_offset(args: &[Node]) -> StaticResult {
    // If first argument is a single cell reference and there are no4th and 5th argument,
    // or they are 1, then it is a scalar
    let arg_count = args.len();
    if arg_count < 3 {
        // Actually an error
        return StaticResult::Scalar;
    }
    if !matches!(args[0], Node::ReferenceKind { .. }) {
        return StaticResult::Unknown;
    }
    if arg_count == 3 {
        return StaticResult::Scalar;
    }
    if !matches!(args[3], Node::NumberKind(f) if f == 1.0) {
        return StaticResult::Unknown;
    }
    if arg_count == 4 {
        return StaticResult::Scalar;
    }
    if !matches!(args[4], Node::NumberKind(f) if f == 1.0) {
        return StaticResult::Unknown;
    }
    // Both height and width are explicitly 1, so OFFSET will return a single cell
    StaticResult::Scalar
}

pub(super) fn static_analysis_indirect(_args: &[Node]) -> StaticResult {
    // We will always insert the @, but we don't need to do that in every scenario`
    StaticResult::Unknown
}

/// CHOOSE is a reference function: with a range arm the call itself is reference-shaped, so
/// legacy import wraps the whole CALL in `@` (Excel upgrades `=CHOOSE(2,A1:A10,B1:B10)` to
/// `=@CHOOSE(...)`). Its arms' counts become the extents of that range shape. An index that is an
/// array or a multi-cell range lifts CHOOSE to an array of values, even over range arms, so the
/// call is an array: a `LiftArrays` argument lifts over it rather than refusing a range
/// (`IMABS(CHOOSE({1,2},H1:H2,I1:I2))` is 2x2, probes/lift-arrays-choose-arms A21, T-1874). Each
/// argument is analysed once, so nested calls stay linear.
pub(super) fn choose_static_result(args: &[Node]) -> StaticResult {
    let mut shapes = args.iter().map(run_static_analysis_on_node);
    let index = shapes.next().unwrap_or(StaticResult::Scalar);
    let scalar_index = matches!(index, StaticResult::Scalar | StaticResult::Range(0, 0));
    match broadcast_shapes(std::iter::once(index).chain(shapes)) {
        StaticResult::Array(n, m) if scalar_index => StaticResult::Range(n - 1, m - 1),
        other => other,
    }
}
