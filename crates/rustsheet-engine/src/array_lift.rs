//! Element-wise array lifting shared by the function categories (text, math, ...): a scalar or
//! 2-D array argument and the broadcast of a per-element computation over them.
use crate::{
    arithmetic::bcast_idx,
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// An argument of an element-wise function that works element by element (LEFT, MID,
/// EXACT, FIND, ...): a single scalar value or a 2-D array of values.
/// When any argument is an array (e.g. a SEQUENCE or a multi-cell range) the
/// scalar arguments are broadcast across it and the result is an array.
pub(crate) enum LiftArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
}

/// What an array has beyond its end, when it is used next to a longer one.
static MISSING: ArrayNode = ArrayNode::Error(Error::NA);

impl LiftArg {
    /// The (rows, columns) shape of this argument.
    pub(crate) fn dims(&self) -> (usize, usize) {
        match self {
            LiftArg::Scalar(_) => (1, 1),
            LiftArg::Array(a) => (a.len(), a.first().map(|r| r.len()).unwrap_or(0)),
        }
    }

    /// Returns the element at (`i`, `j`), broadcasting scalars and length-1 dimensions.
    ///
    /// Where the array is too short for that position the element is `#N/A`,
    /// as in Excel. It is an error like any other: if an earlier argument of
    /// the function is an error too, that one comes first.
    pub(crate) fn elem(&self, i: usize, j: usize) -> &ArrayNode {
        match self {
            LiftArg::Scalar(n) => n,
            LiftArg::Array(a) => bcast_idx(a.len(), i)
                .and_then(|ri| a.get(ri))
                .and_then(|row| bcast_idx(row.len(), j).and_then(|cj| row.get(cj)))
                .unwrap_or(&MISSING),
        }
    }
}

/// Broadcasts `compute` over the (possibly array) `operands`. When every operand
/// is a scalar the result is a single value; otherwise it is an array whose shape
/// is the element-wise maximum of the operand shapes.
pub(crate) fn broadcast_lift(
    cell: CellReferenceIndex,
    operands: &[&LiftArg],
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
        return match compute(0, 0) {
            Ok(ArrayNode::String(s)) => CalcResult::String(s),
            Ok(ArrayNode::Number(n)) => CalcResult::Number(n),
            Ok(ArrayNode::Boolean(b)) => CalcResult::Boolean(b),
            Ok(ArrayNode::Empty) => CalcResult::String(String::new()),
            Ok(ArrayNode::Error(e)) | Err(e) => CalcResult::new_error(e, cell, String::new()),
        };
    }

    let mut total_bytes: usize = 0;
    let mut result = Vec::with_capacity(rows);
    for i in 0..rows {
        let mut data_row = Vec::with_capacity(cols);
        for j in 0..cols {
            let value = compute(i, j).unwrap_or_else(ArrayNode::Error);
            if let ArrayNode::String(s) = &value {
                if !within_text_budget(&mut total_bytes, s.len()) {
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

/// Cap on the total bytes of text one broadcast may produce. Excel has no such
/// limit that we have measured; this bounds allocation on untrusted input
/// (DEVIATION, decision 0014; docs/excel-semantics/text.md).
const MAX_BROADCAST_TEXT_BYTES: usize = 64 * 1024 * 1024;

/// Adds `len` to the running `total`; false once it exceeds the broadcast cap.
pub(crate) fn within_text_budget(total: &mut usize, len: usize) -> bool {
    *total = total.saturating_add(len);
    *total <= MAX_BROADCAST_TEXT_BYTES
}

impl Model {
    /// Evaluates an argument of an element-wise function into a scalar value or a 2-D array of
    /// values. References to a single cell collapse to a scalar; multi-cell ranges
    /// and array literals/spills become arrays that the result is broadcast over.
    pub(crate) fn lift_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LiftArg, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            err @ CalcResult::Error { .. } => Err(err),
            CalcResult::Range { left, right } => {
                Ok(LiftArg::Array(self.evaluate_range(left, right)))
            }
            CalcResult::Array(a) => Ok(LiftArg::Array(a)),
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expecting a value".to_string(),
            )),
            other => Ok(LiftArg::Scalar(calc_result_to_array_node(other))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::within_text_budget;

    #[test]
    fn budget_allows_exactly_64_mib() {
        let mut total = 0;
        // 2048 results of 32 KiB make exactly 64 MiB; the next one crosses the cap.
        assert!((0..2048).all(|_| within_text_budget(&mut total, 32768)));
        assert!(!within_text_budget(&mut total, 1));
    }
}
