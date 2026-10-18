//! The arguments of the text functions that work element by element.

use crate::{
    arithmetic::{bcast_get, MISSING},
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, token::Error},
    functions::{text::util, CellReferenceIndex},
    limits::check_array_size,
};

/// An argument of a text function that works element by element (LEFT, MID,
/// EXACT, FIND, ...): a single scalar value or a 2-D array of values.
/// When any argument is an array (e.g. a SEQUENCE or a multi-cell range) the
/// scalar arguments are broadcast across it and the result is an array.
pub(crate) enum TextArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
    /// An argument that is an error: broadcast like a scalar (an array of errors next to an
    /// array), and returned whole when no argument is an array.
    Error(ArrayNode, CalcResult),
}

impl TextArg {
    /// The (rows, columns) shape of this argument.
    pub(crate) fn dims(&self) -> (usize, usize) {
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
    pub(crate) fn elem(&self, i: usize, j: usize) -> &ArrayNode {
        match self {
            TextArg::Scalar(n) | TextArg::Error(n, _) => n,
            TextArg::Array(a) => bcast_get(a, i, j).unwrap_or(&MISSING),
        }
    }
}

/// Broadcasts `compute` over the (possibly array) `operands`. When every operand
/// is a scalar the result is a single value; otherwise it is an array whose shape
/// is the element-wise maximum of the operand shapes.
///
/// A result above the engine's array limit is the limit's error before any element is computed
/// or allocated: a whole column against a whole row (`=LEFT(A:A,1:1)`) would be 2^34 cells, and
/// a numeric result (FIND, LEN) does not count against the text budget (DEVIATION,
/// docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"; T-1771).
pub(crate) fn broadcast_text(
    cell: CellReferenceIndex,
    operands: &[&TextArg],
    compute: impl Fn(usize, usize) -> Result<ArrayNode, Error>,
) -> CalcResult {
    let mut rows = 1;
    let mut cols = 1;
    for operand in operands {
        let (r, c) = operand.dims();
        rows = rows.max(r);
        cols = cols.max(c);
    }

    // All scalars: return a single value.
    if rows <= 1 && cols <= 1 {
        // Arguments are coerced left to right; the first failure decides (probe
        // error-precedence-text). An error argument is returned as it came only when it is that
        // first failure; "first" is judged by comparing error kinds, which fits the probed cases.
        let computed = compute(0, 0);
        let failure = match &computed {
            Ok(ArrayNode::Error(e)) | Err(e) => Some(e),
            Ok(_) => None,
        };
        for o in operands {
            if let TextArg::Error(_, error @ CalcResult::Error { error: arg, .. }) = o {
                if failure.is_none_or(|e| *e == *arg) {
                    return error.clone();
                }
                break;
            }
        }
        return match computed {
            Ok(ArrayNode::String(s)) => CalcResult::String(s),
            Ok(ArrayNode::Number(n)) => CalcResult::Number(n),
            Ok(ArrayNode::Boolean(b)) => CalcResult::Boolean(b),
            Ok(ArrayNode::Empty) => CalcResult::String(String::new().into()),
            Ok(ArrayNode::Error(e)) | Err(e) => CalcResult::new_error(e, cell, String::new()),
        };
    }

    if let Some((error, message)) = check_array_size(rows, cols) {
        return CalcResult::new_error(error, cell, message);
    }
    let mut total_bytes: usize = 0;
    let mut result = Vec::with_capacity(rows);
    for i in 0..rows {
        let mut data_row = Vec::with_capacity(cols);
        for j in 0..cols {
            let value = compute(i, j).unwrap_or_else(ArrayNode::Error);
            if let ArrayNode::String(s) = &value {
                if !util::within_text_budget(&mut total_bytes, s.len()) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Total text result is too large".to_string(),
                    );
                }
            }
            data_row.push(value);
        }
        result.push(data_row);
    }
    CalcResult::Array(result)
}

#[cfg(test)]
mod tests;
