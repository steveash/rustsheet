use super::drop_take::sub_range;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Returns true if an `ArrayNode` counts as a blank/empty cell for TRIMRANGE purposes.
fn is_blank(node: &ArrayNode) -> bool {
    matches!(node, ArrayNode::Empty)
}

/// Returns true if every cell in `row` is blank.
fn row_is_empty(row: &[ArrayNode]) -> bool {
    row.iter().all(is_blank)
}

/// Returns true if every row has a blank value at column index `col`.
fn col_is_empty(data: &[Vec<ArrayNode>], col: usize) -> bool {
    data.iter().all(|row| row.get(col).is_none_or(is_blank))
}

/// A trim argument as an integer; a fractional one (even `0.9` or `3.9`) is out of range, like
/// `4` and `-1` (docs/excel-semantics/lookup.md "TRIMRANGE with a fractional trim argument").
fn trim_mode(n: f64) -> i32 {
    if n.fract() == 0.0 && (0.0..=3.0).contains(&n) {
        n as i32
    } else {
        -1
    }
}

impl Model {
    /// `=TRIMRANGE(range, [trim_rows], [trim_cols])`
    ///
    /// Trims blank rows and/or columns from the outer edges of a range or array.
    ///
    /// `trim_rows` / `trim_cols`:
    ///   * 0 – no trimming
    ///   * 1 – trim leading (top / left) blanks only
    ///   * 2 – trim trailing (bottom / right) blanks only
    ///   * 3 – trim both leading and trailing blanks (default)
    ///
    /// Returns `#REF!` if every cell in the range is blank.
    pub(crate) fn fn_trimrange(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.trimrange_impl(args, cell, false)
    }

    /// TRIMRANGE; `as_reference` gives the trimmed part of a single-sheet range argument as a
    /// reference, as an intersection operand sees it: `=A1:A3 TRIMRANGE(A2:B2)` is 2
    /// (probes/intersection-function-operands H14; T-0165).
    pub(crate) fn trimrange_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if args.is_empty() || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let source = self.evaluate_node_in_context(&args[0], cell);
        let reference_corner = match source {
            CalcResult::Range { left, right } if as_reference && left.sheet == right.sheet => {
                Some(left)
            }
            _ => None,
        };
        let data = match self.result_to_array(source, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::new_error(Error::REF, cell, "Range is empty".to_string());
        }

        let trim_rows = if args.len() >= 2 {
            match self.get_number(&args[1], cell) {
                Ok(n) => trim_mode(n),
                Err(e) => return e,
            }
        } else {
            3
        };

        let trim_cols = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(n) => trim_mode(n),
                Err(e) => return e,
            }
        } else {
            3
        };

        if !matches!(trim_rows, 0..=3) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "trim_rows must be 0, 1, 2, or 3".to_string(),
            );
        }
        if !matches!(trim_cols, 0..=3) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "trim_cols must be 0, 1, 2, or 3".to_string(),
            );
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        // Determine row slice bounds
        let row_start = if trim_rows == 1 || trim_rows == 3 {
            (0..num_rows)
                .find(|&r| !row_is_empty(&data[r]))
                .unwrap_or(num_rows)
        } else {
            0
        };
        let row_end = if trim_rows == 2 || trim_rows == 3 {
            (0..num_rows)
                .rev()
                .find(|&r| !row_is_empty(&data[r]))
                .map(|r| r + 1)
                .unwrap_or(0)
        } else {
            num_rows
        };

        // Determine column slice bounds
        let col_start = if trim_cols == 1 || trim_cols == 3 {
            (0..num_cols)
                .find(|&c| !col_is_empty(&data, c))
                .unwrap_or(num_cols)
        } else {
            0
        };
        let col_end = if trim_cols == 2 || trim_cols == 3 {
            (0..num_cols)
                .rev()
                .find(|&c| !col_is_empty(&data, c))
                .map(|c| c + 1)
                .unwrap_or(0)
        } else {
            num_cols
        };

        if row_start >= row_end || col_start >= col_end {
            return CalcResult::new_error(
                Error::REF,
                cell,
                "TRIMRANGE result is empty".to_string(),
            );
        }

        if let Some(left) = reference_corner {
            return sub_range(left, &(row_start..row_end), &(col_start..col_end));
        }

        let result: Vec<Vec<ArrayNode>> = data[row_start..row_end]
            .iter()
            .map(|row| row[col_start..col_end].to_vec())
            .collect();

        CalcResult::Array(result)
    }
}
