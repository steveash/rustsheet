//! An argument broadcast over the positions of a lifted call: a branch of IF over an array
//! condition, IFERROR's and IFNA's fallback, an arm of CHOOSE over an index array.

use crate::cast::calc_result_to_array_node;
use crate::{
    arithmetic::bcast_idx,
    calc_result::{range_dims, CalcResult},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// An argument broadcast over the output: a scalar (every position), a 2-D array (indexed by
/// position with size-1 broadcasting; a ragged out-of-range position → #N/A), or a range no
/// position takes, kept as its extent only. See [`BroadcastArg::at`].
pub(crate) enum BroadcastArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
    /// A range arm no position picks: it sizes the output like any arm (lookup.md "CHOOSE over
    /// an index array broadcasts its value arguments") but its values are never copied.
    Unpicked {
        rows: usize,
        columns: usize,
    },
}

impl BroadcastArg {
    /// Rows and columns; a scalar is 1x1.
    pub(crate) fn dims(&self) -> (usize, usize) {
        match self {
            BroadcastArg::Scalar(_) => (1, 1),
            BroadcastArg::Array(arr) => (arr.len(), arr.first().map_or(0, Vec::len)),
            BroadcastArg::Unpicked { rows, columns } => (*rows, *columns),
        }
    }

    /// The argument at output position (row, col), under Excel's size-1 broadcasting rule: a
    /// scalar covers every position, a length-1 dimension repeats its only element to cover the
    /// whole output extent, and a ragged mismatch (e.g. length 2 against an extent of 3) yields
    /// `#N/A`. An unpicked arm is never read; it gives `#N/A` if it is.
    pub(crate) fn at(&self, row: usize, col: usize) -> ArrayNode {
        match self {
            BroadcastArg::Scalar(node) => node.clone(),
            BroadcastArg::Array(arr) => {
                let rows = arr.len();
                let cols = arr.first().map_or(0, Vec::len);
                match bcast_idx(rows, row)
                    .and_then(|r| arr.get(r))
                    .and_then(|r| bcast_idx(cols, col).and_then(|c| r.get(c)))
                {
                    Some(n) => n.clone(),
                    None => ArrayNode::Error(Error::NA),
                }
            }
            BroadcastArg::Unpicked { .. } => {
                debug_assert!(false, "an unpicked arm is never read");
                ArrayNode::Error(Error::NA)
            }
        }
    }
}

impl Model {
    /// Evaluates a broadcast argument once, as a scalar or a 2-D array; `Err` is a range that
    /// cannot be read.
    pub(crate) fn evaluate_broadcast_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<BroadcastArg, CalcResult> {
        self.evaluate_broadcast_arm(node, cell, true)
    }

    /// [`Model::evaluate_broadcast_arg`] for an arm that a position may or may not pick. An
    /// unpicked range is not copied: its cells are read in the same order as a copy reads them,
    /// so cycles and the dependencies they record are as before, but only its extent is kept
    /// (T-1564).
    pub(crate) fn evaluate_broadcast_arm(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        picked: bool,
    ) -> Result<BroadcastArg, CalcResult> {
        Ok(match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } if picked => {
                BroadcastArg::Array(self.evaluate_range(left, right, cell)?)
            }
            CalcResult::Range { left, right } => {
                self.read_range_cells(left, right, cell)?;
                let (rows, columns) = range_dims(left, right);
                BroadcastArg::Unpicked { rows, columns }
            }
            CalcResult::Array(a) => BroadcastArg::Array(a),
            other => BroadcastArg::Scalar(calc_result_to_array_node(other)),
        })
    }
}
