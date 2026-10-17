//! The result shapes of the reference-returning OFFSET and INDIRECT. Moved out of
//! `static_analysis.rs` (T-1770) to keep it under its size lock.

use super::{Node, StaticResult};

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
