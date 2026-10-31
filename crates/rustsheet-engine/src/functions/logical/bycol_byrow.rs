use super::lambda::LambdaCaller;
use crate::cast::calc_result_to_element;
use crate::functions::spill_functions::reject_empty_value;
use crate::functions::util::{array_element_value, array_node_to_calc_result};
use crate::functions::Function;
use crate::nested_array::NestedKind;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    limits::{check_built_array_size, check_range_size, SIZE_LIMIT_EXCLUSIVE},
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

/// `Ok` when an evaluated function argument is a LAMBDA (an eta name evaluates to one), else `Err`
/// with what it evaluated to: the check BYROW, BYCOL, GROUPBY and PIVOTBY share, each mapping
/// anything else to its own error.
pub(crate) fn as_lambda(value: CalcResult) -> Result<CalcResult, CalcResult> {
    match value {
        lambda @ CalcResult::Lambda(_) => Ok(lambda),
        other => Err(other),
    }
}

/// A LAMBDA result as one BYROW/BYCOL/MAKEARRAY cell: a 1x1 array is its one value (probes/
/// byrow-bycol-array-results, excel-graph 2026-10-16), a LAMBDA value stays a LAMBDA element, which
/// a cell shows as `#CALC!` (`BYCOL(Data!A11:D11,LAMBDA(x,LAMBDA(v,v)))` is four `#CALC!`) and `&`
/// reads as `#VALUE!` (`TEXTSPLIT(..&MAP(Data!Z1,LAMBDA(acc,LAMBDA(v,v))),..)` is `#VALUE!`; gen-m7
/// 23b5367c18059af2 A2578, T-2406) and MAP can call (`MAP(MAKEARRAY(1,2,LAMBDA(r,c,LAMBDA(x,x+c))),
/// LAMBDA(g,g(10)))` is 11, 12; probes/lambda-arrays B22, T-2039); otherwise the shared element
/// mapping.
fn lambda_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Array(mut rows) if rows.len() == 1 && rows[0].len() == 1 => {
            rows.remove(0).remove(0)
        }
        other => calc_result_to_element(other),
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
                    (!makes_calc_error(&result)).then(|| lambda_result_to_array_node(result))
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
                    (!makes_calc_error(&result)).then(|| vec![lambda_result_to_array_node(result)])
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
    pub(super) fn by_slices_calc_error(cell: CellReferenceIndex) -> CalcResult {
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
        as_lambda(self.evaluate_node_in_context(arg, cell))
            .map_err(|_| CalcResult::new_error(Error::VALUE, cell, "Expected a LAMBDA".to_string()))
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
        let source = self.by_slices_source(&args[0], cell, false, false)?;
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
        (!makes_calc_error(&result)).then(|| lambda_result_to_array_node(result))
    }

    /// The array argument of BYROW or BYCOL. A range on one sheet (also through TAKE, DROP,
    /// INDEX and the like) stays a reference, so the LAMBDA gets each row or column as a
    /// reference: COUNTA of a column skips its blank cells and `ISREF`, `ROW` work on it
    /// (probes/lambda-helper-range-slices, excel-graph 2026-10-12, T-1395). The range keeps its
    /// full size, as a range read as an array does (docs/excel-semantics/arrays.md "Array
    /// results above 1,048,576 cells"), under the same size bound. Anything else is read as
    /// values.
    ///
    /// With `refuse_scalar` a scalar value (no reference, no array) is `#VALUE!`, as MAP reads its
    /// second and later arrays: `MAP(1,{2},LAMBDA(x,y,x+y))` is 3 but `MAP({1},2,..)` and
    /// `MAP(1,2,..)` are `#VALUE!` (probes/lambda-value-arguments A28, B1-B10; arrays.md "MAP over
    /// single values").
    /// With `refuse_empty_value` the empty value is `#VALUE!` ([`reject_empty_value`]), as MAP reads
    /// its arrays (`MAP(CELL("contents",<blank>),LAMBDA(v,v))`; probes/t3030-cell-contents-blank-shapers
    /// A351, T-3053); SCAN, REDUCE, BYROW and BYCOL of it are unprobed.
    fn by_slices_source(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        refuse_scalar: bool,
        refuse_empty_value: bool,
    ) -> Result<Source, CalcResult> {
        let data = match self.evaluate_picked_reference_arg(arg, cell) {
            RefArg::Ranges(Ranges::One(range)) if range.left.sheet == range.right.sheet => {
                if let Some((error, message)) = check_range_size(&range.left, &range.right) {
                    return Err(CalcResult::new_error(error, cell, message));
                }
                return Ok(Source::Range(range));
            }
            other => {
                let mut value = other.into_single_area(cell);
                if refuse_empty_value {
                    value = reject_empty_value(arg, value, cell)?;
                }
                if refuse_scalar && is_scalar_value(&value) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Expecting an array".to_string(),
                    ));
                }
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
    /// `refuse_scalar` and `refuse_empty_value` as in [`Model::by_slices_source`].
    pub(super) fn lambda_cells(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        refuse_scalar: bool,
        refuse_empty_value: bool,
    ) -> Result<LambdaGrid, CalcResult> {
        Ok(
            match self.by_slices_source(arg, cell, refuse_scalar, refuse_empty_value)? {
                Source::Range(range) => LambdaGrid::Cells {
                    origin: range.left,
                    rows: usize::try_from(range.right.row - range.left.row + 1).unwrap_or(0),
                    columns: usize::try_from(range.right.column - range.left.column + 1)
                        .unwrap_or(0),
                },
                // A blank element of an array value reaches the LAMBDA as the empty value IFERROR gives
                // a blank cell: ISBLANK TRUE, yet COUNT and AVERAGE count it as 0
                // (`MAP(HSTACK(Z1:Z2,{1;2}),_xleta.COUNT)` is 1,1,1,1, probes/t2413-blank-in-stacked-arrays
                // A7:A11, T-2413). A range's blank cell stays a blank cell (A43:A47).
                // A nested cell of a GROUPBY or PIVOTBY result reaches the LAMBDA whole, as its array
                // (`MAP(GROUPBY(..,LAMBDA(v,v)),LAMBDA(x,ROWS(x)))` gives 2 for a two-record group,
                // `REDUCE(0,..,LAMBDA(a,x,a+ROWS(x)))` 19; SPEC-0018 rule 5, probes/t2592-nested-groupby-cells
                // A100:A105, A15). Its cells are charged to the walk budget (`NestedArray::value`), and
                // past it the whole call is `#ERROR!` (DEVIATION, decision 0014).
                Source::Values(data) => {
                    let mut grid = Vec::with_capacity(data.len());
                    for row in &data {
                        let mut values = Vec::with_capacity(row.len());
                        for node in row {
                            values.push(match node {
                                ArrayNode::Empty => CalcResult::EmptyArg,
                                ArrayNode::Nested(nested) if nested.kind() == NestedKind::Array => {
                                    match nested.value(&mut self.nested_budget, cell) {
                                        array @ CalcResult::Array(_) => array,
                                        error => return Err(error),
                                    }
                                }
                                node => array_element_value(node, cell),
                            });
                        }
                        grid.push(values);
                    }
                    LambdaGrid::Values(grid)
                }
            },
        )
    }

    /// A LAMBDA's result as a value: a reference it returns (`LAMBDA(v,v)` over a range cell)
    /// is read, so MAP and SCAN never carry one into their output. A one-cell reference is read
    /// directly, without building a one-element array. A multi-cell reference is no element:
    /// Excel gives the whole MAP or SCAN one `#CALC!` (probes/map-scan-range-result, excel-graph
    /// 2026-10-18, T-1792), so that is the `Err` the callers return as their result, and only its
    /// existing cells are read, for what reading does (a cycle through the range, the reads a
    /// spill checks), and no array is built.
    /// Building one made every call cost the range's full size (T-1769 fuzz timeout).
    pub(super) fn lambda_result_value(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        Ok(match result {
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
            // A one-cell range on two sheets reads as its first sheet's cell.
            CalcResult::Range { left, right }
                if left.row == right.row && left.column == right.column =>
            {
                self.lambda_result_value(CalcResult::Range { left, right: left }, cell)?
            }
            CalcResult::Range { left, right } => {
                // Read as `evaluate_range` reads it, its size limit and then its first sheet,
                // but over the cells that exist only, as a union walks an area: the blank rest is
                // recorded as read empty in one piece. Every value it could hold gives the same
                // element, and a dense walk made each call cost the range's height.
                if let Some((error, message)) = check_range_size(&left, &right) {
                    return Err(CalcResult::new_error(error, cell, message));
                }
                let area = Range {
                    left,
                    right: CellReferenceIndex {
                        sheet: left.sheet,
                        ..right
                    },
                };
                let read = self.for_each_position_in_area(&area, true, cell, &mut |model, at| {
                    model.evaluate_cell(at);
                    Ok(())
                });
                match read {
                    Ok(()) => return Err(Self::by_slices_calc_error(cell)),
                    Err(error) => return Err(error),
                }
            }
            // A multi-cell array is no element either, the same `#CALC!` as a multi-cell range
            // (docs/excel-semantics/arrays.md "MAP and SCAN with a LAMBDA that returns a multi-cell
            // range", T-2309).
            CalcResult::Array(rows)
                if rows.len() > 1 || rows.first().is_some_and(|r| r.len() > 1) =>
            {
                return Err(Self::by_slices_calc_error(cell));
            }
            // A 1x1 array is its one value, as BYROW and BYCOL take it (T-2407).
            CalcResult::Array(rows) if rows.len() == 1 && rows[0].len() == 1 => {
                array_element_value(&rows[0][0], cell)
            }
            other => other,
        })
    }

    /// `=MAKEARRAY(rows, cols, lambda)`
    ///
    /// Returns an array of the given size by calling `lambda(row, col)` (1-based) for each cell.
    pub(crate) fn fn_makearray(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // rows and cols lift over an array and a range, one MAKEARRAY per element whose value is
        // that call's top-left; the LAMBDA is read whole (SPEC-0011 "Array-valued functions",
        // T-1834).
        self.lift_call(Function::Makearray, args, cell, &mut |model, values| {
            model.makearray_kernel(values, cell)
        })
    }

    /// One MAKEARRAY call over argument values: `[rows, cols, lambda]`.
    fn makearray_kernel(&mut self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [rows, cols, lambda_result] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        // Both sizes are read, errors first in argument order, before either is validated:
        // `MAKEARRAY(1.5,NA(),..)` is `#N/A` (probes/expand-makearray-past-grid D9-D16).
        let sizes = self
            .cast_number_ref(rows, cell)
            .and_then(|rows| Ok((rows, self.cast_number_ref(cols, cell)?)));
        let (rows, cols) = match sizes {
            Ok(sizes) => sizes,
            Err(error) => return error,
        };
        let extent = |x: f64, name: &str| -> Result<usize, CalcResult> {
            // Unlike SEQUENCE, a fractional size is #VALUE!, not truncated (arrays.md
            // "MAKEARRAY rows and columns must be whole numbers").
            if x.fract() != 0.0 || x < 1.0 {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    format!("{name} must be a whole number >= 1"),
                ));
            }
            let n = x as i64;
            Ok(usize::try_from(n).unwrap_or(usize::MAX))
        };
        let rows = match extent(rows, "rows") {
            Ok(rows) => rows,
            Err(error) => return error,
        };
        let cols = match extent(cols, "cols") {
            Ok(cols) => cols,
            Err(error) => return error,
        };

        // Rows and columns past the grid are built: `COLUMNS(MAKEARRAY(1,16385,..))` is 16385
        // (probes/expand-makearray-past-grid D1), `ROWS(MAKEARRAY(1048577,1,..))` 1048577
        // (probes/t2579-array-value-edges B23, above our cell bound), and so is
        // `COLUMNS(MAKEARRAY(1,1048577,..))`, 1048577 (probes/t2452-makearray-cols-1048577): unlike
        // EXPAND's, MAKEARRAY's columns have no cap below 2^31 that Excel is known to check.
        if let Some((error, message)) = check_built_array_size(rows, cols, SIZE_LIMIT_EXCLUSIVE) {
            return CalcResult::new_error(error, cell, message);
        }

        if lambda_result.is_error() {
            return lambda_result.clone();
        }

        let mut caller = self.lambda_caller(lambda_result, 2);
        let mut result = Vec::with_capacity(rows);
        for r in 1..=rows {
            let mut row = Vec::with_capacity(cols);
            for c in 1..=cols {
                let cell_result = self.call_lambda_caller(
                    &mut caller,
                    [CalcResult::Number(r as f64), CalcResult::Number(c as f64)],
                    cell,
                );
                if makes_calc_error(&cell_result) {
                    return Self::by_slices_calc_error(cell);
                }
                // A one-cell reference result is its value (gen-m7 a470dc9e165bdb20 A465:
                // `TRIMRANGE(Data!D11:E11,2,2)` trims to one cell).
                row.push(lambda_result_to_array_node(
                    self.collapse_single_cell(cell_result),
                ));
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }
}

/// A number, text, boolean, blank or error value: a value that is neither an array nor a
/// reference. An error value as a later array of MAP is `#VALUE!` whatever the error
/// (probes/map-later-array-error-blank A1:A3, A12:A13, T-1946).
fn is_scalar_value(value: &CalcResult) -> bool {
    matches!(
        value,
        CalcResult::Error { .. }
            | CalcResult::Number(_)
            | CalcResult::String(_)
            | CalcResult::Boolean(_)
            | CalcResult::EmptyCell
            | CalcResult::EmptyArg
    )
}

#[cfg(test)]
mod tests {
    use super::as_lambda;
    use crate::calc_result::CalcResult;

    #[test]
    fn as_lambda_passes_anything_else_through_unchanged() {
        assert!(matches!(
            as_lambda(CalcResult::Lambda(7)),
            Ok(CalcResult::Lambda(7))
        ));
        assert!(matches!(
            as_lambda(CalcResult::Number(5.0)),
            Err(CalcResult::Number(n)) if n == 5.0
        ));
        assert!(matches!(
            as_lambda(CalcResult::Array(Vec::new())),
            Err(CalcResult::Array(rows)) if rows.is_empty()
        ));
    }
}
