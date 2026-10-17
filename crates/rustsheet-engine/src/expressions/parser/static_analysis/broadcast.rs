//! Broadcasting static shapes element-wise, as the operators and the lifted calls do. Moved out
//! of `static_analysis.rs` (T-1770) to keep it under its size lock; `lifted.rs` uses it too.

use super::StaticResult;

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
