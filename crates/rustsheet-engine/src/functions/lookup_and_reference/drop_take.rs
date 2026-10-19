use std::ops::Range;

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    reference_operators::Picked,
};

/// Whether a call keeps (TAKE) or removes (DROP) the rows and columns it counts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edge {
    Take,
    Drop,
}

/// The indices out of `len` that `edge` keeps for a count `n`: from the start when `n` is
/// positive, from the end when it is negative. A count larger than `len` keeps (TAKE) or
/// removes (DROP) everything.
fn edge_span(edge: Edge, len: usize, n: i64) -> Range<usize> {
    let count = usize::try_from(n.unsigned_abs())
        .unwrap_or(usize::MAX)
        .min(len);
    match (edge, n >= 0) {
        (Edge::Take, true) => 0..count,
        (Edge::Take, false) => len - count..len,
        (Edge::Drop, true) => count..len,
        (Edge::Drop, false) => 0..len - count,
    }
}

impl Model {
    // ── TAKE ──────────────────────────────────────────────────────────────────

    /// `=TAKE(array, rows, [cols])`
    ///
    /// Returns the first (or last, when negative) N rows and/or M columns.
    ///   * rows > 0 → take from the top;  rows < 0 → take from the bottom
    ///   * cols > 0 → take from the left; cols < 0 → take from the right
    ///   * If |rows| or |cols| exceeds the array size, all rows/cols are returned.
    pub(crate) fn fn_take(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.take_drop_impl(args, cell, Edge::Take, false)
    }

    // ── DROP ──────────────────────────────────────────────────────────────────

    /// `=DROP(array, rows, [cols])`
    ///
    /// Returns the array with the first (or last, when negative) N rows and/or M columns removed.
    ///   * rows > 0 → drop from the top;  rows < 0 → drop from the bottom
    ///   * cols > 0 → drop from the left; cols < 0 → drop from the right
    pub(crate) fn fn_drop(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.take_drop_impl(args, cell, Edge::Drop, false)
    }

    /// TAKE or DROP; `as_reference` gives the part of a single-sheet range argument as a
    /// reference, as an intersection operand sees it: `=A1:A3 TAKE(A2:B3,1)` is 2
    /// (probes/intersection-function-operands H11, H12; T-0165). `array` is evaluated once
    /// (`read_once` in functions/logical/lambda.rs relies on it).
    pub(crate) fn take_drop_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        edge: Edge,
        as_reference: bool,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let source = if as_reference {
            self.evaluate_picked_source(&args[0], cell, Self::evaluate_node_in_context)
        } else {
            self.evaluate_node_in_context(&args[0], cell)
        };
        if let CalcResult::Range { left, right } = source {
            if as_reference && left.sheet == right.sheet {
                let num_rows = usize::try_from(right.row - left.row + 1).unwrap_or(0);
                let num_cols = usize::try_from(right.column - left.column + 1).unwrap_or(0);
                return match self.edge_spans(args, cell, edge, num_rows, num_cols) {
                    Ok((rows, columns)) => sub_range(left, &rows, &columns),
                    Err(error) => error,
                };
            }
        }
        let data = match self.result_to_array(source, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }
        match self.edge_spans(args, cell, edge, data.len(), data[0].len()) {
            Ok((rows, columns)) => {
                let result: Vec<Vec<ArrayNode>> = data[rows]
                    .iter()
                    .map(|row| row[columns.clone()].to_vec())
                    .collect();
                CalcResult::Array(result)
            }
            Err(error) => error,
        }
    }

    /// The array argument of a TAKE, DROP or TRIMRANGE read as a reference (below SUBTOTAL, AREAS,
    /// ISREF or an intersection): an XLOOKUP, TAKE, DROP or TRIMRANGE there is the cells it picks,
    /// also as the arm a scalar IF, CHOOSE, IFS or SWITCH picks ([`Self::pick_step`]) or as a
    /// LET body, however deeply they nest:
    /// `SUBTOTAL(9,TAKE(XLOOKUP(1,J1:J2,H1:I2),1,1))` is 3 and
    /// `SUBTOTAL(9,TAKE(IF(TRUE,CHOOSE(1,XLOOKUP(..))),1,1))` too, while an XLOOKUP that lifts or
    /// returns an array gives values (probes/take-drop-trimrange-over-xlookup, -b; T-1998;
    /// `TAKE(IFS(TRUE,XLOOKUP(..)),1,1)` too, probes/take-drop-over-ifs-switch-picks, T-2071).
    /// Any other node is read as `read` says (TAKE's own read, or the picking function's read of
    /// its arm). Each condition, index and binding is read once; bounded by the parser's depth cap.
    pub(super) fn evaluate_picked_source(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        read: fn(&mut Self, &Node, CellReferenceIndex) -> CalcResult,
    ) -> CalcResult {
        let Node::FunctionKind { kind, args } = node.ungrouped() else {
            return read(self, node, cell);
        };
        if let Some(picked) = self.pick_step(kind, args, cell) {
            // The picked arm, when not a reference form, reads as the picking function reads it.
            let read_arm: fn(&mut Self, &Node, CellReferenceIndex) -> CalcResult = match kind {
                Function::If => Self::if_branch_value,
                Function::Choose => Self::choose_arm_value,
                _ => Self::evaluate_node_in_context,
            };
            return match picked {
                Picked::Node(arm) => self.evaluate_picked_source(arm, cell, read_arm),
                Picked::Value(value) => value,
            };
        }
        match kind {
            Function::Let => self
                .let_scope(args, cell, |model, body| {
                    model.evaluate_picked_source(body, cell, Self::evaluate_node_in_context)
                })
                .unwrap_or_else(|error| error),
            _ if kind.picks_reference() => self
                .evaluate_reference_form(kind, args, cell)
                .unwrap_or_else(|| read(self, node, cell)),
            _ => read(self, node, cell),
        }
    }

    /// The rows and columns out of `num_rows` x `num_cols` that TAKE or DROP keeps, read from
    /// its count arguments; an error for a zero TAKE count or a DROP that removes everything.
    fn edge_spans(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        edge: Edge,
        num_rows: usize,
        num_cols: usize,
    ) -> Result<(Range<usize>, Range<usize>), CalcResult> {
        let rows_arg = self.get_number(&args[1], cell)? as i64;
        if edge == Edge::Take && rows_arg == 0 {
            return Err(CalcResult::new_error(
                Error::CALC,
                cell,
                "rows cannot be 0".to_string(),
            ));
        }
        let rows = edge_span(edge, num_rows, rows_arg);
        let columns = if args.len() == 3 {
            let cols_arg = self.get_number(&args[2], cell)? as i64;
            if edge == Edge::Take && cols_arg == 0 {
                return Err(CalcResult::new_error(
                    Error::CALC,
                    cell,
                    "cols cannot be 0".to_string(),
                ));
            }
            edge_span(edge, num_cols, cols_arg)
        } else {
            0..num_cols
        };
        if rows.is_empty() || columns.is_empty() {
            return Err(CalcResult::new_error(
                Error::CALC,
                cell,
                "TAKE or DROP result is empty".to_string(),
            ));
        }
        Ok((rows, columns))
    }
}

/// The reference to `rows` x `columns` of the area whose top left corner is `left`.
pub(crate) fn sub_range(
    left: CellReferenceIndex,
    rows: &Range<usize>,
    columns: &Range<usize>,
) -> CalcResult {
    // Both spans index an area of at most LAST_ROW x LAST_COLUMN cells, so they fit an i32.
    let offset = |n: usize| i32::try_from(n).unwrap_or(i32::MAX);
    CalcResult::Range {
        left: CellReferenceIndex {
            sheet: left.sheet,
            row: left.row + offset(rows.start),
            column: left.column + offset(columns.start),
        },
        right: CellReferenceIndex {
            sheet: left.sheet,
            row: left.row + offset(rows.end) - 1,
            column: left.column + offset(columns.end) - 1,
        },
    }
}
