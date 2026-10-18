//! CHOOSECOLS and CHOOSEROWS: the columns or rows of an array picked by a list of indexes.

use crate::functions::util::array_node_to_calc_result;
use crate::limits::check_array_cells;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    // ── CHOOSECOLS ────────────────────────────────────────────────────────────

    /// `=CHOOSECOLS(array, col_num1, [col_num2], ...)`
    ///
    /// Returns the specified columns from an array or range.
    /// Column numbers are 1-based; negative numbers select from the right.
    pub(crate) fn fn_choosecols(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let col_indices = match self.chosen_indices(&args[1..], data[0].len(), data.len(), cell) {
            Ok(indices) => indices,
            Err(error) => return error,
        };

        let result = data
            .iter()
            .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
            .collect();

        CalcResult::Array(result)
    }

    // ── CHOOSEROWS ────────────────────────────────────────────────────────────

    /// `=CHOOSEROWS(array, row_num1, [row_num2], ...)`
    ///
    /// Returns the specified rows from an array or range.
    /// Row numbers are 1-based; negative numbers select from the bottom.
    pub(crate) fn fn_chooserows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() {
            return CalcResult::Array(vec![]);
        }

        let width = data[0].len();
        match self.chosen_indices(&args[1..], data.len(), width, cell) {
            Ok(indices) => CalcResult::Array(indices.iter().map(|&r| data[r].clone()).collect()),
            Err(error) => error,
        }
    }

    /// The 0-based indexes CHOOSECOLS or CHOOSEROWS picks out of `count` columns or rows, each
    /// picked one `other` cells long. Every argument is a list read whole, not lifted: a range or
    /// an array of indexes adds its elements in row-major order, whatever its orientation
    /// (`CHOOSECOLS(A1:B3,A1:A2)` and `CHOOSECOLS(A1:B3,{1,2})` are both 3x2,
    /// `CHOOSEROWS(A1:B3,A1:A2)` is 2x2; probes/lookup-argument-modes F7:G10,
    /// docs/excel-semantics/lookup.md "Lookup and reference function argument modes"). An index
    /// is 1-based, negative from the end; 0 or past `count` is `#VALUE!`, and an error element is
    /// itself. The result is bounded before it is built (CLAUDE.md directive 5).
    fn chosen_indices(
        &mut self,
        args: &[Node],
        count: usize,
        other: usize,
        cell: CellReferenceIndex,
    ) -> Result<Vec<usize>, CalcResult> {
        let count = i64::try_from(count).unwrap_or(i64::MAX);
        let mut indices = Vec::with_capacity(args.len());
        for arg in args {
            let list = self.eval_to_array(arg, cell)?;
            let added = list.len().saturating_mul(list.first().map_or(0, Vec::len));
            if let Some((error, message)) =
                check_array_cells(indices.len().saturating_add(added), other)
            {
                return Err(CalcResult::new_error(error, cell, message));
            }
            for node in list.iter().flatten() {
                let n = self.cast_number_ref(&array_node_to_calc_result(node, cell), cell)? as i64;
                if n == 0 || n.abs() > count {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Index out of range".to_string(),
                    ));
                }
                let index = if n > 0 { n - 1 } else { count + n };
                indices.push(usize::try_from(index).unwrap_or(0));
            }
        }
        Ok(indices)
    }
}
