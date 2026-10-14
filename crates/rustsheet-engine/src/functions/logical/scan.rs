use crate::{
    calc_result::CalcResult,
    expressions::{parser::ArrayNode, parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

fn calc_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Number(n) => ArrayNode::Number(n),
        CalcResult::Boolean(b) => ArrayNode::Boolean(b),
        CalcResult::String(s) => ArrayNode::String(s),
        CalcResult::Error { error, .. } => ArrayNode::Error(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
        _ => ArrayNode::Error(Error::VALUE),
    }
}

impl Model {
    /// `=SCAN([initial_value], array, lambda)`
    ///
    /// Like REDUCE but returns an array of all intermediate accumulated values
    /// rather than just the final one. The output has the same shape as `array`.
    /// With 2 arguments the first element seeds the accumulator (not included in output).
    pub(crate) fn fn_scan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

        if data.rows() == 0 || data.columns() == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "empty array".to_string());
        }

        let num_rows = data.rows();
        let num_cols = data.columns();

        let mut accumulator = if has_initial {
            let init = self.evaluate_node_in_context(&args[0], cell);
            if init.is_error() {
                return init;
            }
            init
        } else {
            data.at(0, 0)
        };

        let mut result = vec![vec![ArrayNode::Empty; num_cols]; num_rows];

        // When no initial value, the first element seeds the accumulator and its
        // slot in the output is that seed value itself (not the result of calling lambda).
        let mut start_col = 0;
        if !has_initial {
            let seed = self.lambda_result_value(accumulator.clone(), cell);
            result[0][0] = calc_result_to_array_node(seed);
            start_col = 1;
        }

        for (i, result_row) in result.iter_mut().enumerate() {
            let col_start = if i == 0 { start_col } else { 0 };
            for (j, slot) in result_row.iter_mut().enumerate().skip(col_start) {
                accumulator = self.call_lambda_with_values(
                    &lambda_result,
                    &[accumulator, data.at(i, j)],
                    cell,
                );
                if accumulator.is_error() {
                    return accumulator;
                }
                let value = self.lambda_result_value(accumulator.clone(), cell);
                *slot = calc_result_to_array_node(value);
            }
        }

        CalcResult::Array(result)
    }
}
