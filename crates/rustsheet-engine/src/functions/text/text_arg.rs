//! The arguments of the text functions that work element by element.

use crate::{
    arithmetic::{bcast_idx, MISSING},
    calc_result::CalcResult,
    expressions::parser::ArrayNode,
};

/// An argument of a text function that works element by element (LEFT, MID,
/// EXACT, FIND, ...): a single scalar value or a 2-D array of values.
/// When any argument is an array (e.g. a SEQUENCE or a multi-cell range) the
/// scalar arguments are broadcast across it and the result is an array.
pub(super) enum TextArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
    /// An argument that is an error: broadcast like a scalar (an array of errors next to an
    /// array), and returned whole when no argument is an array.
    Error(ArrayNode, CalcResult),
}

impl TextArg {
    /// The (rows, columns) shape of this argument.
    pub(super) fn dims(&self) -> (usize, usize) {
        match self {
            TextArg::Scalar(_) | TextArg::Error(..) => (1, 1),
            TextArg::Array(a) => (a.len(), a.first().map(|r| r.len()).unwrap_or(0)),
        }
    }

    /// Returns the element at (`i`, `j`), broadcasting scalars and length-1 dimensions.
    ///
    /// Where the array is too short for that position the element is `#N/A`,
    /// as in Excel. It is an error like any other: if an earlier argument of
    /// the function is an error too, that one comes first.
    pub(super) fn elem(&self, i: usize, j: usize) -> &ArrayNode {
        match self {
            TextArg::Scalar(n) | TextArg::Error(n, _) => n,
            TextArg::Array(a) => bcast_idx(a.len(), i)
                .and_then(|ri| a.get(ri))
                .and_then(|row| bcast_idx(row.len(), j).and_then(|cj| row.get(cj)))
                .unwrap_or(&MISSING),
        }
    }
}
