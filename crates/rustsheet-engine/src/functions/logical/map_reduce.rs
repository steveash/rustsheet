use super::bycol_byrow::LambdaGrid;
use crate::cast::calc_result_to_element;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    limits::check_array_cells,
    model::Model,
};

impl Model {
    /// `=MAP(array1, [array2, ...], lambda)`
    ///
    /// Applies the lambda element-wise across one or more arrays; smaller arrays pad with `#N/A`.
    /// The lambda receives one scalar value from each array per call.
    pub(crate) fn fn_map(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let n_arrays = args.len() - 1;

        let lambda_result = self.evaluate_node_in_context(&args[n_arrays], cell);
        if lambda_result.is_error() {
            return lambda_result;
        }

        let mut arrays: Vec<LambdaGrid> = Vec::with_capacity(n_arrays);
        for (index, arg) in args[..n_arrays].iter().enumerate() {
            let data = match self.lambda_cells(arg, cell, index > 0, true) {
                Ok(d) => d,
                Err(e) => return e,
            };
            arrays.push(data);
        }

        if arrays
            .iter()
            .any(|arr| arr.rows() == 0 || arr.columns() == 0)
        {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        // Arrays of different sizes pad to the largest rows and columns: an element missing from
        // any array is #N/A and the LAMBDA is not called for it, a scalar or a row next to a column
        // does not broadcast (probes/map-unequal-sizes, excel-graph 2026-10-17; arrays.md "MAP
        // over arrays of different sizes").
        let num_rows = arrays.iter().map(LambdaGrid::rows).max().unwrap_or(0);
        let num_cols = arrays.iter().map(LambdaGrid::columns).max().unwrap_or(0);
        // The padded shape is known before allocating it (DEVIATION, arrays.md "Array results
        // above 1,048,576 cells").
        if let Some((error, message)) = check_array_cells(num_rows, num_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let mut result = vec![vec![ArrayNode::Error(Error::NA); num_cols]; num_rows];
        // One argument buffer for every element.
        let mut values: Vec<CalcResult> = Vec::with_capacity(arrays.len());
        let mut caller = self.lambda_caller(&lambda_result, arrays.len());

        for (i, result_row) in result.iter_mut().enumerate() {
            for (j, slot) in result_row.iter_mut().enumerate() {
                if arrays
                    .iter()
                    .any(|arr| i >= arr.rows() || j >= arr.columns())
                {
                    continue;
                }
                values.clear();
                values.extend(arrays.iter().map(|arr| arr.at(i, j)));
                let cell_result = self.call_lambda_caller(&mut caller, values.drain(..), cell);
                let cell_result = match self.lambda_result_value(cell_result, cell) {
                    Ok(value) => value,
                    Err(error) => return error,
                };
                *slot = calc_result_to_element(cell_result);
            }
        }

        CalcResult::Array(result)
    }

    /// `=REDUCE([initial_value], array, lambda)`
    ///
    /// Accumulates a result by applying the lambda to each element of `array`.
    /// The lambda takes `(accumulator, current_value)` and returns the new accumulator.
    /// With 2 arguments the first element of the array seeds the accumulator.
    pub(crate) fn fn_reduce(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (array_idx, lambda_idx, has_initial) = if args.len() == 3 {
            (1, 2, true)
        } else {
            (0, 1, false)
        };

        let lambda_result = self.evaluate_node_in_context(&args[lambda_idx], cell);
        if lambda_result.is_error() {
            return lambda_result;
        }

        let data = match self.lambda_cells(&args[array_idx], cell, false, false) {
            Ok(d) => d,
            Err(e) => return e,
        };

        let columns = data.columns();
        let count = data.rows() * columns;

        if count == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        let (mut accumulator, start) = if has_initial {
            (self.lambda_initial_value(&args[0], cell), 0)
        } else {
            (data.at(0, 0), 1)
        };

        let mut caller = self
            .lambda_caller(&lambda_result, 2)
            .keeping_trimrange_references();
        for index in start..count {
            accumulator = self.call_lambda_caller(
                &mut caller,
                [accumulator, data.at(index / columns, index % columns)],
                cell,
            );
        }

        accumulator
    }

    /// SCAN's and REDUCE's `initial_value` as the LAMBDA's first accumulator. A cell or range
    /// reference stays a reference, a single cell too, as in the common case of a LAMBDA call's
    /// argument ([`Model::bind_argument`], whose OFFSET, INDIRECT, TRIMRANGE and grid-variable
    /// forms are not taken here): `REDUCE(Data!C7,Data!E12,COUNT)` with C7 the text "1" is 1, and
    /// `REDUCE(Data!A12,Data!C11,PRODUCT)` with both cells TRUE is 0 (gen-m7/2c5ea078f25443a2
    /// A1767, 18b80d3fd292d63a A467; docs/excel-semantics/arrays.md "SCAN and REDUCE keep errors
    /// in the LAMBDA", T-2307). An error goes to the LAMBDA like any value (same entry). A union
    /// and a 3-D span are read as before (unprobed).
    pub(super) fn lambda_initial_value(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if self.is_multi_area(node) || matches!(node.ungrouped(), Node::SheetSpanKind { .. }) {
            return self.evaluate_node_in_context(node, cell);
        }
        self.eval_ref_or_index(node, cell)
    }
}
