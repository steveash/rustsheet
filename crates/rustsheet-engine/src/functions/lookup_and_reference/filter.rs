use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::spill_functions::{one_cell_error_ref_as_array, wrap_one_cell_error_ref},
    model::Model,
};

fn array_node_is_truthy(node: &ArrayNode) -> bool {
    match node {
        ArrayNode::Boolean(b) => *b,
        ArrayNode::Number(n) => *n != 0.0,
        _ => false,
    }
}

/// Whether `include`, a mask for an array of `rows` x `cols`, filters its rows (`Some(true)`) or
/// its columns (`Some(false)`): a column or a row as long as the rows, else as long as the
/// columns; `None` when it is neither.
fn include_axis(include: &[Vec<ArrayNode>], rows: usize, cols: usize) -> Option<bool> {
    let height = include.len();
    let width = include.first().map_or(0, Vec::len);
    if (height == rows && width == 1) || (height == 1 && width == rows) {
        Some(true)
    } else if (height == 1 && width == cols) || (height == cols && width == 1) {
        Some(false)
    } else {
        None
    }
}

/// FILTER's `#VALUE!` for an include of the wrong shape.
fn include_shape_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "include array dimensions must match array rows or columns".to_string(),
    )
}

impl Model {
    /// `=FILTER(array, include, [if_empty])`
    ///
    /// Returns only the rows of array where the corresponding include value is truthy.
    ///   * include  – a column vector of booleans/numbers with the same height as array
    ///   * `if_empty` – value returned when no rows pass the filter (default: #CALC! error)
    pub(crate) fn fn_filter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        // A one-cell reference to an error cell is a 1x1 array holding it, for the array and the
        // include alike, so the shape and include are checked before the error is seen.
        let data = self.eval_array_arg(&args[0], cell);
        let data = match one_cell_error_ref_as_array(&args[0], data, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        let include = self.eval_to_array(&args[1], cell);
        let (include, wrapped) = wrap_one_cell_error_ref(&args[1], include, cell);
        let include = match include {
            Ok(d) => d,
            Err(e) => return e,
        };
        // A one-cell error reference as include is checked for its shape first:
        // `FILTER(Data!A1:D12,Data!G9,Data!G3)` is `#VALUE!` with G9 `#DIV/0!`
        // (gen-m7/d7a127bd9334c2b4 A2380, T-3506). Every other include keeps its element errors
        // first, a wrongly shaped multi-cell one too (ours, unprobed).
        let axis = include_axis(&include, num_rows, num_cols);
        if wrapped && axis.is_none() {
            return include_shape_error(cell);
        }

        // Strings and errors are invalid as include mask elements.
        // Strings → #VALUE!; errors → propagate the error.
        for row in &include {
            for node in row {
                match node {
                    ArrayNode::Boolean(_) | ArrayNode::Number(_) | ArrayNode::Empty => {}
                    ArrayNode::String(_) | ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "include array must contain numbers or booleans".to_string(),
                        );
                    }
                    ArrayNode::Error(e) => {
                        return CalcResult::new_error(
                            e.clone(),
                            cell,
                            "include array contains an error".to_string(),
                        );
                    }
                }
            }
        }

        let Some(filter_rows) = axis else {
            return include_shape_error(cell);
        };
        let include_flat: Vec<bool> = include.iter().flatten().map(array_node_is_truthy).collect();

        let filtered: Vec<Vec<ArrayNode>> = if filter_rows {
            data.into_iter()
                .enumerate()
                .filter_map(|(i, row)| if include_flat[i] { Some(row) } else { None })
                .collect()
        } else {
            let col_indices: Vec<usize> = include_flat
                .iter()
                .enumerate()
                .filter_map(|(i, &keep)| if keep { Some(i) } else { None })
                .collect();
            data.iter()
                .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
                .collect()
        };

        let is_empty_result = filtered.is_empty() || filtered.first().is_none_or(|r| r.is_empty());

        if is_empty_result {
            if args.len() >= 3 {
                let empty_val = self.evaluate_node_in_context(&args[2], cell);
                match empty_val {
                    CalcResult::EmptyArg => CalcResult::new_error(
                        Error::CALC,
                        cell,
                        "No data returned by FILTER".to_string(),
                    ),
                    // A range is its values: FILTER gives values, never a reference, found or not
                    // (Excel-cached: upstream ISREF.xlsx Sheet1!D11, D14, D17, D20; probes/isref-names-filter-if-empty).
                    CalcResult::Range { left, right } => {
                        match self.evaluate_range(left, right, cell) {
                            Ok(array) => CalcResult::Array(array),
                            Err(error) => error,
                        }
                    }
                    // An array if_empty spills as is (Excel: probes/filter-if-empty-array).
                    CalcResult::Array(a) => CalcResult::Array(a),
                    // A scalar stays a scalar, not a 1x1 array: `MAX(FILTER(.., .., ""))` with no
                    // match is #VALUE! because MAX gets the text "" as a value (upstream
                    // MAX_MIN.xlsx H39, H40, H43).
                    v => v,
                }
            } else {
                CalcResult::new_error(Error::CALC, cell, "No data returned by FILTER".to_string())
            }
        } else {
            CalcResult::Array(filtered)
        }
    }
}
