//! Broadcasting static shapes element-wise, as the operators and the lifted calls do. Moved out
//! of `static_analysis.rs` (T-1770, and the operator chain in T-1713) to keep it under its size
//! lock; `lifted.rs` uses it too.

use super::{run_static_analysis_on_node, Node, StaticResult};
use crate::expressions::parser::{left_chain, value_operands};

/// [`broadcast_static_result`](super::broadcast_static_result) of shapes already analysed: scalar
/// if all are, `Unknown` if any is, else an array of the largest counts.
pub(super) fn broadcast_shapes(shapes: impl IntoIterator<Item = StaticResult>) -> StaticResult {
    let mut shape: Option<(i32, i32)> = None;
    for result in shapes {
        match result {
            StaticResult::Scalar => {}
            StaticResult::Unknown => return StaticResult::Unknown,
            // An array or a range (`counts` is never `None` here).
            other => {
                let (a, b) = other.counts().unwrap_or((1, 1));
                let (n, m) = shape.unwrap_or((0, 0));
                shape = Some((n.max(a), m.max(b)));
            }
        }
    }
    match shape {
        Some((n, m)) => StaticResult::Array(n, m),
        None => StaticResult::Scalar,
    }
}

/// The shape of a chain of value operators, broadcast link by link in a loop (T-0128).
pub(super) fn static_analysis_operator_chain(node: &Node) -> StaticResult {
    let (links, first) = left_chain(node);
    let mut result = run_static_analysis_on_node(first);
    for link in links {
        if let Some((_, right)) = value_operands(link) {
            result = static_analysis_op_results(result, run_static_analysis_on_node(right));
        }
    }
    result
}

/// An operator over a range gives an array of the range's counts.
fn static_analysis_op_results(lhs: StaticResult, rhs: StaticResult) -> StaticResult {
    match (lhs, rhs) {
        (StaticResult::Scalar, StaticResult::Scalar) => StaticResult::Scalar,
        (_, StaticResult::Unknown) | (StaticResult::Unknown, _) => StaticResult::Unknown,
        (lhs, rhs) => {
            // A scalar operand counts as 1x1.
            let (a1, b1) = lhs.counts().unwrap_or((1, 1));
            let (a2, b2) = rhs.counts().unwrap_or((1, 1));
            StaticResult::Array(a1.max(a2), b1.max(b2))
        }
    }
}
