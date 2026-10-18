use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    limits::check_array_size,
    model::Model,
};

impl Model {
    // ── HSTACK ────────────────────────────────────────────────────────────────

    /// `=HSTACK(array1, [array2], ...)`
    ///
    /// Stacks arrays horizontally (side by side) into one array.
    /// All arrays must have the same number of rows, or shorter ones are padded
    /// with #N/A to match the tallest array.
    ///
    /// Each argument is evaluated once (see VSTACK: `read_once` relies on it).
    pub(crate) fn fn_hstack(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut arrays: Vec<Vec<Vec<ArrayNode>>> = Vec::with_capacity(args.len());
        for arg in args {
            match self.eval_to_array(arg, cell) {
                Ok(a) => arrays.push(a),
                Err(e) => return e,
            }
        }

        let max_rows = arrays.iter().map(|a| a.len()).max().unwrap_or(0);
        if max_rows == 0 {
            return CalcResult::Array(vec![]);
        }

        // The padded shape is known from the arguments: refuse one above the array limit before
        // any padding or copying (DEVIATION, arrays.md "Array results above 1,048,576 cells";
        // T-1780). A column next to a row is their product.
        let total_cols = arrays
            .iter()
            .map(|a| a.first().map_or(0, Vec::len))
            .fold(0_usize, usize::saturating_add);
        if let Some((error, message)) = check_array_size(max_rows, total_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let na = ArrayNode::Error(Error::NA);
        let mut result: Vec<Vec<ArrayNode>> = Vec::with_capacity(max_rows);
        for r in 0..max_rows {
            let mut row: Vec<ArrayNode> = Vec::new();
            for arr in &arrays {
                if r < arr.len() {
                    row.extend(arr[r].iter().cloned());
                } else {
                    let ncols = arr.first().map(|r| r.len()).unwrap_or(0);
                    row.extend(std::iter::repeat_n(na.clone(), ncols));
                }
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }

    // ── VSTACK ────────────────────────────────────────────────────────────────

    /// `=VSTACK(array1, [array2], ...)`
    ///
    /// Stacks arrays vertically (one on top of another) into one array.
    /// All arrays must have the same number of columns, or narrower ones are
    /// padded with #N/A to match the widest array.
    ///
    /// Each argument is evaluated once: a LAMBDA parameter read here is moved into the stack, not
    /// copied (`read_once` in functions/logical/lambda.rs relies on it, as for HSTACK).
    pub(crate) fn fn_vstack(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut arrays: Vec<Vec<Vec<ArrayNode>>> = Vec::with_capacity(args.len());
        for arg in args {
            match self.eval_to_array(arg, cell) {
                Ok(a) => arrays.push(a),
                Err(e) => return e,
            }
        }

        let max_cols = arrays
            .iter()
            .filter_map(|a| a.first().map(|r| r.len()))
            .max()
            .unwrap_or(0);

        if max_cols == 0 {
            return CalcResult::Array(vec![]);
        }

        // Refuse a padded shape above the array limit before any padding (T-1780, as HSTACK).
        let total_rows = arrays
            .iter()
            .map(Vec::len)
            .fold(0_usize, usize::saturating_add);
        if let Some((error, message)) = check_array_size(total_rows, max_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let na = ArrayNode::Error(Error::NA);
        let mut result: Vec<Vec<ArrayNode>> = Vec::with_capacity(total_rows);
        for arr in arrays {
            for row in arr {
                let mut padded = row;
                while padded.len() < max_cols {
                    padded.push(na.clone());
                }
                result.push(padded);
            }
        }

        CalcResult::Array(result)
    }
}
