use super::bycol_byrow::LambdaGrid;
use crate::cast::calc_result_to_array_node;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// `=MAP(array1, [array2, ...], lambda)`
    ///
    /// Applies the lambda element-wise across one or more arrays of equal dimensions.
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
        for arg in &args[..n_arrays] {
            let data = match self.lambda_cells(arg, cell) {
                Ok(d) => d,
                Err(e) => return e,
            };
            arrays.push(data);
        }

        if arrays[0].rows() == 0 || arrays[0].columns() == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        let num_rows = arrays[0].rows();
        let num_cols = arrays[0].columns();

        for arr in &arrays[1..] {
            if arr.rows() != num_rows || arr.columns() != num_cols {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "MAP: all arrays must have the same dimensions".to_string(),
                );
            }
        }

        let mut result = vec![vec![ArrayNode::Empty; num_cols]; num_rows];
        // One argument buffer for every element.
        let mut values: Vec<CalcResult> = Vec::with_capacity(arrays.len());
        let mut caller = self.lambda_caller(&lambda_result, arrays.len());

        for (i, result_row) in result.iter_mut().enumerate() {
            for (j, slot) in result_row.iter_mut().enumerate() {
                values.clear();
                values.extend(arrays.iter().map(|arr| arr.at(i, j)));
                let cell_result = self.call_lambda_caller(&mut caller, values.drain(..), cell);
                let cell_result = self.lambda_result_value(cell_result, cell);
                *slot = calc_result_to_array_node(cell_result);
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

        let data = match self.lambda_cells(&args[array_idx], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        let columns = data.columns();
        let count = data.rows() * columns;

        if count == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        let (mut accumulator, start) = if has_initial {
            let init = self.evaluate_node_in_context(&args[0], cell);
            if init.is_error() {
                return init;
            }
            (init, 0)
        } else {
            (data.at(0, 0), 1)
        };

        let mut caller = self.lambda_caller(&lambda_result, 2);
        for index in start..count {
            accumulator = self.call_lambda_caller(
                &mut caller,
                [accumulator, data.at(index / columns, index % columns)],
                cell,
            );
            if accumulator.is_error() {
                return accumulator;
            }
        }

        accumulator
    }
}
