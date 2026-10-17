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
    // ── WRAPCOLS ──────────────────────────────────────────────────────────────

    /// `=WRAPCOLS(vector, wrap_count, [pad_with])`
    ///
    /// Wraps the provided row or column vector by columns after a specified
    /// number of values. The vector is read top-to-bottom then left-to-right.
    ///   * `wrap_count` – number of values per column (must be >= 1)
    ///   * `pad_with`   – value used to pad the last column; defaults to #N/A
    pub(crate) fn fn_wrapcols(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let flat = match self.flatten_vector(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let wrap_count = match self.get_wrap_count(args, flat.len(), cell) {
            Ok(n) => n,
            Err(e) => return e,
        };

        let pad = match self.get_pad_with(args, flat.len() % wrap_count != 0, cell) {
            Ok(p) => p,
            Err(e) => return e,
        };

        let n = flat.len();
        let num_cols = n.div_ceil(wrap_count);
        // The padded shape is wrap_count rows: refuse one above the array limit before
        // allocating it (DEVIATION, arrays.md "Array results above 1,048,576 cells"; T-1786).
        if let Some((error, message)) = check_array_size(wrap_count, num_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let mut result: Vec<Vec<ArrayNode>> = vec![Vec::with_capacity(num_cols); wrap_count];
        for (i, val) in flat.into_iter().enumerate() {
            result[i % wrap_count].push(val);
        }
        // Pad the last (partial) column
        for row in &mut result {
            while row.len() < num_cols {
                row.push(pad.clone());
            }
        }

        CalcResult::Array(result)
    }

    // ── WRAPROWS ──────────────────────────────────────────────────────────────

    /// `=WRAPROWS(vector, wrap_count, [pad_with])`
    ///
    /// Wraps the provided row or column vector by rows after a specified number
    /// of values. The vector is read top-to-bottom then left-to-right.
    ///   * `wrap_count` – number of values per row (must be >= 1)
    ///   * `pad_with`   – value used to pad the last row; defaults to #N/A
    pub(crate) fn fn_wraprows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let flat = match self.flatten_vector(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let wrap_count = match self.get_wrap_count(args, flat.len(), cell) {
            Ok(n) => n,
            Err(e) => return e,
        };

        let pad = match self.get_pad_with(args, flat.len() % wrap_count != 0, cell) {
            Ok(p) => p,
            Err(e) => return e,
        };

        // The padded shape is wrap_count columns: refuse one above the array limit before
        // allocating a row (T-1786, as WRAPCOLS).
        if let Some((error, message)) =
            check_array_size(flat.len().div_ceil(wrap_count), wrap_count)
        {
            return CalcResult::new_error(error, cell, message);
        }

        let mut result: Vec<Vec<ArrayNode>> = Vec::new();
        let mut chunk = Vec::with_capacity(wrap_count);
        for val in flat {
            chunk.push(val);
            if chunk.len() == wrap_count {
                result.push(chunk);
                chunk = Vec::with_capacity(wrap_count);
            }
        }
        if !chunk.is_empty() {
            while chunk.len() < wrap_count {
                chunk.push(pad.clone());
            }
            result.push(chunk);
        }

        CalcResult::Array(result)
    }

    // ── Shared helpers ────────────────────────────────────────────────────────

    /// Flatten the first argument of WRAPCOLS/WRAPROWS into a 1-D vector.
    /// Returns an error if the first argument is not a 1-D vector or range.
    fn flatten_vector(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<ArrayNode>, CalcResult> {
        let data = self.eval_to_array(&args[0], cell)?;
        let is_row_vec = data.len() == 1;
        let is_col_vec = data.iter().all(|r| r.len() == 1);
        if !is_row_vec && !is_col_vec {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "First argument must be a 1-D vector".to_string(),
            ));
        }
        let flat: Vec<ArrayNode> = if is_row_vec {
            data.into_iter().next().unwrap_or_default()
        } else {
            data.into_iter().map(|mut r| r.remove(0)).collect()
        };
        Ok(flat)
    }

    /// The `wrap_count` argument (2nd arg of WRAPCOLS/WRAPROWS) for a vector of `len` values: at
    /// least 1, and a count above the vector's length does not pad, it is the length
    /// (probes/dynamic-array-reshape-and-isomitted, arrays.md "WRAPROWS and WRAPCOLS"; T-1786).
    fn get_wrap_count(
        &mut self,
        args: &[Node],
        len: usize,
        cell: CellReferenceIndex,
    ) -> Result<usize, CalcResult> {
        // Below 1 is checked before truncation: 0.5 and 0 are `#NUM!` (arrays.md "WRAPCOLS of a
        // single value is not padded"; gen-m7/f68fafa0ca2eba26 A39, gen-m9/50f10c4062fb2e98 A21).
        let wrap_count = self.get_number(&args[1], cell)?;
        if wrap_count.is_nan() || wrap_count < 1.0 {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "wrap_count must be >= 1".to_string(),
            ));
        }
        Ok((wrap_count as usize).min(len.max(1)))
    }

    /// Parse the optional `pad_with` argument; `padded` says some cell is filled with it
    /// (3rd arg of WRAPCOLS/WRAPROWS).
    fn get_pad_with(
        &mut self,
        args: &[Node],
        padded: bool,
        cell: CellReferenceIndex,
    ) -> Result<ArrayNode, CalcResult> {
        if args.len() == 3 {
            let v = self.evaluate_node_in_context(&args[2], cell);
            // An error pad_with that nothing is padded with is the result
            // (gen-m9/675763979c1defb6 A6: `WRAPCOLS(D6:D6,2,VALUE("x"))` is `#VALUE!`), while
            // one that fills a padded cell stays in that cell (WRAPCOLS_ROWS.xlsx E10, NA()).
            if !padded && v.is_error() {
                return Err(v);
            }
            Ok(match v {
                CalcResult::Number(n) => ArrayNode::Number(n),
                CalcResult::Boolean(b) => ArrayNode::Boolean(b),
                CalcResult::String(s) => ArrayNode::String(s),
                CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Number(0.0),
                CalcResult::Error { error, .. } => ArrayNode::Error(error),
                _ => ArrayNode::Error(Error::VALUE),
            })
        } else {
            Ok(ArrayNode::Error(Error::NA))
        }
    }
}
