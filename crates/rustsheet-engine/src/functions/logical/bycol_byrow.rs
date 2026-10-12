use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::math_and_trigonometry::array_size::check_array_size,
    model::Model,
    reference_operators::{Ranges, RefArg},
};

/// What BYROW and BYCOL walk: a range on one sheet, whose rows or columns reach the LAMBDA as
/// references, or the values of anything else.
enum Source {
    Range(Range),
    Values(Vec<Vec<ArrayNode>>),
}

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
    /// `=BYCOL(array, lambda)`
    ///
    /// Applies a LAMBDA to each column of `array`, returning a 1-row array of results.
    /// The lambda receives each column as an N×1 array.
    pub(crate) fn fn_bycol(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let source = match self.by_slices_source(&args[0], cell) {
            Ok(source) => source,
            Err(e) => return e,
        };

        let lambda_result = self.evaluate_node_in_context(&args[1], cell);
        if lambda_result.is_error() {
            return lambda_result;
        }

        let results = match source {
            Source::Range(range) => (range.left.column..=range.right.column)
                .map(|column| {
                    let mut left = range.left;
                    let mut right = range.right;
                    left.column = column;
                    right.column = column;
                    let slice = CalcResult::Range { left, right };
                    let result =
                        self.call_lambda_with_values(lambda_result.clone(), vec![slice], cell);
                    calc_result_to_array_node(result)
                })
                .collect(),
            Source::Values(data) => (0..data[0].len())
                .map(|j| {
                    let column = data.iter().map(|row| vec![row[j].clone()]).collect();
                    let result =
                        self.call_lambda(lambda_result.clone(), &[Node::ArrayKind(column)], cell);
                    calc_result_to_array_node(result)
                })
                .collect(),
        };

        CalcResult::Array(vec![results])
    }

    /// `=BYROW(array, lambda)`
    ///
    /// Applies a LAMBDA to each row of `array`, returning a 1-column array of results.
    /// The lambda receives each row as a 1×M array.
    pub(crate) fn fn_byrow(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let source = match self.by_slices_source(&args[0], cell) {
            Ok(source) => source,
            Err(e) => return e,
        };

        let lambda_result = self.evaluate_node_in_context(&args[1], cell);
        if lambda_result.is_error() {
            return lambda_result;
        }

        let results = match source {
            Source::Range(range) => (range.left.row..=range.right.row)
                .map(|row| {
                    let mut left = range.left;
                    let mut right = range.right;
                    left.row = row;
                    right.row = row;
                    let slice = CalcResult::Range { left, right };
                    let result =
                        self.call_lambda_with_values(lambda_result.clone(), vec![slice], cell);
                    vec![calc_result_to_array_node(result)]
                })
                .collect(),
            Source::Values(data) => data
                .into_iter()
                .map(|row| {
                    let result = self.call_lambda(
                        lambda_result.clone(),
                        &[Node::ArrayKind(vec![row])],
                        cell,
                    );
                    vec![calc_result_to_array_node(result)]
                })
                .collect(),
        };

        CalcResult::Array(results)
    }

    /// The array argument of BYROW or BYCOL. A range on one sheet (also through TAKE, DROP,
    /// INDEX and the like) stays a reference, clipped to the used area for whole rows and
    /// columns, so the LAMBDA gets each row or column as a reference: COUNTA of a column skips
    /// its blank cells and `ISREF`, `ROW` work on it (probes/lambda-helper-range-slices,
    /// excel-graph 2026-10-12, T-1395). Anything else is read as values.
    fn by_slices_source(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Source, CalcResult> {
        let data = match self.evaluate_picked_reference_arg(arg, cell) {
            RefArg::Ranges(Ranges::One(range)) if range.left.sheet == range.right.sheet => {
                let left = range.left;
                let (row, column) = self
                    .used_area_corner(
                        left.sheet,
                        left.row,
                        left.column,
                        range.right.row,
                        range.right.column,
                    )
                    .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
                if row < left.row || column < left.column {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "empty array".to_string(),
                    ));
                }
                let mut right = range.right;
                right.row = row;
                right.column = column;
                return Ok(Source::Range(Range { left, right }));
            }
            other => {
                let value = other.into_single_area(cell);
                self.result_to_array(value, cell)?
            }
        };
        if data.is_empty() || data[0].is_empty() {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "empty array".to_string(),
            ));
        }
        Ok(Source::Values(data))
    }

    /// `=MAKEARRAY(rows, cols, lambda)`
    ///
    /// Returns an array of the given size by calling `lambda(row, col)` (1-based) for each cell.
    pub(crate) fn fn_makearray(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let rows = match self.get_number(&args[0], cell) {
            Ok(f) => {
                let n = f.floor() as i64;
                if n < 1 {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "rows must be >= 1".to_string(),
                    );
                }
                n as usize
            }
            Err(e) => return e,
        };

        let cols = match self.get_number(&args[1], cell) {
            Ok(f) => {
                let n = f.floor() as i64;
                if n < 1 {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "cols must be >= 1".to_string(),
                    );
                }
                n as usize
            }
            Err(e) => return e,
        };

        if let Some((error, message)) = check_array_size(rows, cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let lambda_result = self.evaluate_node_in_context(&args[2], cell);
        if lambda_result.is_error() {
            return lambda_result;
        }

        let mut result = Vec::with_capacity(rows);
        for r in 1..=rows {
            let mut row = Vec::with_capacity(cols);
            for c in 1..=cols {
                let cell_result = self.call_lambda_with_values(
                    lambda_result.clone(),
                    vec![CalcResult::Number(r as f64), CalcResult::Number(c as f64)],
                    cell,
                );
                row.push(calc_result_to_array_node(cell_result));
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }
}
