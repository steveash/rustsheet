use super::lambda::LambdaCaller;
use crate::functions::util::array_node_to_calc_result;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    limits::{check_array_size, check_range_size},
    model::Model,
    reference_operators::{Ranges, RefArg},
};

/// What BYROW and BYCOL walk: a range on one sheet, whose rows or columns reach the LAMBDA as
/// references, or the values of anything else.
enum Source {
    Range(Range),
    Values(Vec<Vec<ArrayNode>>),
}

/// The elements MAP, SCAN and REDUCE hand to their LAMBDA: the cells of a range on one sheet
/// (as one-cell references, built when read) or the values of anything else.
pub(super) enum LambdaGrid {
    Cells {
        origin: CellReferenceIndex,
        rows: usize,
        columns: usize,
    },
    Values(Vec<Vec<CalcResult>>),
}

impl LambdaGrid {
    pub(super) fn rows(&self) -> usize {
        match self {
            LambdaGrid::Cells { rows, .. } => *rows,
            LambdaGrid::Values(data) => data.len(),
        }
    }

    pub(super) fn columns(&self) -> usize {
        match self {
            LambdaGrid::Cells { columns, .. } => *columns,
            LambdaGrid::Values(data) => data.first().map_or(0, Vec::len),
        }
    }

    /// The element at 0-based `(row, column)`, which must be inside the grid (callers loop over
    /// [`LambdaGrid::rows`] and [`LambdaGrid::columns`], so the conversions below cannot fail:
    /// a grid is at most one sheet).
    pub(super) fn at(&self, row: usize, column: usize) -> CalcResult {
        match self {
            LambdaGrid::Cells { origin, .. } => {
                let mut position = *origin;
                position.row += i32::try_from(row).unwrap_or(0);
                position.column += i32::try_from(column).unwrap_or(0);
                CalcResult::Range {
                    left: position,
                    right: position,
                }
            }
            LambdaGrid::Values(data) => data[row][column].clone(),
        }
    }
}

fn calc_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Number(n) => ArrayNode::Number(n),
        CalcResult::Boolean(b) => ArrayNode::Boolean(b),
        CalcResult::String(s) => ArrayNode::String(s),
        CalcResult::Error { error, .. } => ArrayNode::Error(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
        // A 1x1 array is its one value (probes/byrow-bycol-array-results, excel-graph 2026-10-16).
        CalcResult::Array(mut rows) if rows.len() == 1 && rows[0].len() == 1 => {
            rows.remove(0).remove(0)
        }
        _ => ArrayNode::Error(Error::VALUE),
    }
}

/// Whether a LAMBDA's result makes BYROW or BYCOL give one `#CALC!` cell: several values, but not
/// a 1x1 array, which counts as its one value (T-1670).
fn makes_calc_error(result: &CalcResult) -> bool {
    result.is_multi_value()
        && !matches!(result, CalcResult::Array(a) if a.len() == 1 && a[0].len() == 1)
}

impl Model {
    /// `=BYCOL(array, lambda)`
    ///
    /// Applies a LAMBDA to each column of `array`, returning a 1-row array of results.
    /// The lambda receives each column of a range as a reference, of an array as an N×1 array.
    pub(crate) fn fn_bycol(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (source, mut caller) = match self.by_slices_args(args, cell) {
            Ok(inputs) => inputs,
            Err(e) => return e,
        };
        let results = match source {
            Source::Range(range) => (range.left.column..=range.right.column)
                .map(|column| {
                    let mut slice = range.clone();
                    slice.left.column = column;
                    slice.right.column = column;
                    self.call_lambda_on_slice(&mut caller, slice, cell)
                })
                .collect::<Option<Vec<_>>>(),
            Source::Values(data) => (0..data[0].len())
                .map(|j| {
                    let column = data.iter().map(|row| vec![row[j].clone()]).collect();
                    let result = self.call_lambda_caller_with_nodes(
                        &mut caller,
                        &[Node::ArrayKind(column)],
                        cell,
                    );
                    (!makes_calc_error(&result)).then(|| calc_result_to_array_node(result))
                })
                .collect::<Option<Vec<_>>>(),
        };

        match results {
            Some(results) => CalcResult::Array(vec![results]),
            None => Self::by_slices_calc_error(cell),
        }
    }

    /// `=BYROW(array, lambda)`
    ///
    /// Applies a LAMBDA to each row of `array`, returning a 1-column array of results.
    /// The lambda receives each row of a range as a reference, of an array as a 1×M array.
    pub(crate) fn fn_byrow(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (source, mut caller) = match self.by_slices_args(args, cell) {
            Ok(inputs) => inputs,
            Err(e) => return e,
        };
        let results = match source {
            Source::Range(range) => (range.left.row..=range.right.row)
                .map(|row| {
                    let mut slice = range.clone();
                    slice.left.row = row;
                    slice.right.row = row;
                    self.call_lambda_on_slice(&mut caller, slice, cell)
                        .map(|node| vec![node])
                })
                .collect::<Option<Vec<_>>>(),
            Source::Values(data) => data
                .into_iter()
                .map(|row| {
                    let result = self.call_lambda_caller_with_nodes(
                        &mut caller,
                        &[Node::ArrayKind(vec![row])],
                        cell,
                    );
                    (!makes_calc_error(&result)).then(|| vec![calc_result_to_array_node(result)])
                })
                .collect::<Option<Vec<_>>>(),
        };

        match results {
            Some(results) => CalcResult::Array(results),
            None => Self::by_slices_calc_error(cell),
        }
    }

    /// The one `#CALC!` cell BYROW and BYCOL give when the LAMBDA returned an array or a
    /// multi-cell reference for any slice (probes/eta-reduced-names E49 and E101, T-1664).
    fn by_slices_calc_error(cell: CellReferenceIndex) -> CalcResult {
        CalcResult::new_error(Error::CALC, cell, "a LAMBDA returned an array".to_string())
    }

    /// The LAMBDA argument of BYROW or BYCOL. Anything else, an error or an unknown name too, is
    /// `#VALUE!`: `BYCOL(A1:B3,NA())`, `BYCOL(A1:B3,1)` and a bare `BYCOL(A1:B3,COUNTA)` read from
    /// a file (probes/eta-reduced-names E5:E21, E81, excel-graph 2026-10-14, T-1466).
    fn by_slices_lambda(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        match self.evaluate_node_in_context(arg, cell) {
            lambda @ CalcResult::Lambda(_) => Ok(lambda),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected a LAMBDA".to_string(),
            )),
        }
    }

    /// The array and the one-argument LAMBDA caller of a BYROW or BYCOL call, in that order of
    /// errors (the argument count first).
    fn by_slices_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Source, LambdaCaller), CalcResult> {
        if args.len() != 2 {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let source = self.by_slices_source(&args[0], cell)?;
        let lambda_result = self.by_slices_lambda(&args[1], cell)?;
        Ok((source, self.lambda_caller(&lambda_result, 1)))
    }

    /// One BYROW or BYCOL call of `lambda` on the row or column `slice`, bound as a reference.
    fn call_lambda_on_slice(
        &mut self,
        caller: &mut LambdaCaller,
        slice: Range,
        cell: CellReferenceIndex,
    ) -> Option<ArrayNode> {
        let reference = CalcResult::Range {
            left: slice.left,
            right: slice.right,
        };
        let result = self.call_lambda_caller(caller, [reference], cell);
        (!makes_calc_error(&result)).then(|| calc_result_to_array_node(result))
    }

    /// The array argument of BYROW or BYCOL. A range on one sheet (also through TAKE, DROP,
    /// INDEX and the like) stays a reference, so the LAMBDA gets each row or column as a
    /// reference: COUNTA of a column skips its blank cells and `ISREF`, `ROW` work on it
    /// (probes/lambda-helper-range-slices, excel-graph 2026-10-12, T-1395). The range keeps its
    /// full size, as a range read as an array does (docs/excel-semantics/arrays.md "Array
    /// results above 1,048,576 cells"), under the same size bound. Anything else is read as
    /// values.
    fn by_slices_source(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Source, CalcResult> {
        let data = match self.evaluate_picked_reference_arg(arg, cell) {
            RefArg::Ranges(Ranges::One(range)) if range.left.sheet == range.right.sheet => {
                if let Some((error, message)) = check_range_size(&range.left, &range.right) {
                    return Err(CalcResult::new_error(error, cell, message));
                }
                return Ok(Source::Range(range));
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

    /// The array argument of MAP, SCAN and REDUCE as a grid of LAMBDA arguments: over a range on
    /// one sheet each cell is a one-cell reference (`ISREF(v)` is TRUE, `COUNTA` and `ROW` see
    /// the cell), over anything else its value (probes/lambda-helper-range-slices, excel-graph
    /// 2026-10-12, T-1442). Over a range the references are built on demand ([`LambdaGrid::at`]),
    /// not stored per cell (T-1465).
    pub(super) fn lambda_cells(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LambdaGrid, CalcResult> {
        Ok(match self.by_slices_source(arg, cell)? {
            Source::Range(range) => LambdaGrid::Cells {
                origin: range.left,
                rows: usize::try_from(range.right.row - range.left.row + 1).unwrap_or(0),
                columns: usize::try_from(range.right.column - range.left.column + 1).unwrap_or(0),
            },
            Source::Values(data) => LambdaGrid::Values(
                data.iter()
                    .map(|row| {
                        row.iter()
                            .map(|node| array_node_to_calc_result(node, cell))
                            .collect()
                    })
                    .collect(),
            ),
        })
    }

    /// A LAMBDA's result as a value: a reference it returns (`LAMBDA(v,v)` over a range cell)
    /// is read, so MAP, SCAN and REDUCE never carry one into their output. A one-cell reference
    /// is read directly, without building a one-element array.
    pub(super) fn lambda_result_value(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match result {
            CalcResult::Range { left, right } if left == right => {
                match self.evaluate_cell(left) {
                    // Read as `evaluate_range` reads a cell: an error is re-originated here and
                    // an empty argument is an empty cell.
                    CalcResult::Error { error, .. } => {
                        array_node_to_calc_result(&ArrayNode::Error(error), cell)
                    }
                    CalcResult::EmptyArg => CalcResult::EmptyCell,
                    value => value,
                }
            }
            CalcResult::Range { .. } => match self.result_to_array(result, cell) {
                Ok(data) if data.len() == 1 && data[0].len() == 1 => {
                    array_node_to_calc_result(&data[0][0], cell)
                }
                Ok(data) => CalcResult::Array(data),
                Err(e) => e,
            },
            other => other,
        }
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

        let mut caller = self.lambda_caller(&lambda_result, 2);
        let mut result = Vec::with_capacity(rows);
        for r in 1..=rows {
            let mut row = Vec::with_capacity(cols);
            for c in 1..=cols {
                let cell_result = self.call_lambda_caller(
                    &mut caller,
                    [CalcResult::Number(r as f64), CalcResult::Number(c as f64)],
                    cell,
                );
                row.push(calc_result_to_array_node(cell_result));
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }
}
