//! INDEX: one cell, row, column or area of a reference or an array, lifted element-wise over
//! array indices (docs/excel-semantics/lookup.md).

use crate::cast::{calc_result_to_array_node, NumberOrArray};
use crate::functions::util::array_node_to_calc_result;
use crate::reference_operators::{three_d_needs_one_sheet, RefArg};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    expressions::types::CellReferenceIndex,
    model::Model,
};

/// INDEX's validated numeric arguments: row_num and column_num floored (`0` = the whole row or
/// column), area_num floored and at least 1.
#[derive(Clone, Copy)]
struct IndexNumbers {
    row: usize,
    col: usize,
    area: f64,
}

/// Validates INDEX's numbers, each cast already tried, in Excel's order: area_num's cast error,
/// then area_num below 1 (`#VALUE!`; past the last area, or NaN, is `#REF!` later, SPEC-0004), then
/// row_num's cast error and a negative row_num (`#VALUE!`), then the same for column_num.
fn index_numbers<E>(
    area: Result<f64, E>,
    row: Result<f64, E>,
    col: Result<f64, E>,
    out_of_range: impl Fn() -> E,
) -> Result<IndexNumbers, E> {
    let area = area?.floor();
    if area < 1.0 {
        return Err(out_of_range());
    }
    let row = row?;
    if row < 0.0 {
        return Err(out_of_range());
    }
    let col = col?;
    if col < 0.0 {
        return Err(out_of_range());
    }
    // A huge index saturates and is out of range later.
    Ok(IndexNumbers {
        row: row.floor() as usize,
        col: col.floor() as usize,
        area,
    })
}

/// Whether an evaluated index makes INDEX lift: a multi-cell range or a multi-element array.
/// A single cell or a 1x1 array stays a scalar index.
fn lifts(index: &CalcResult) -> bool {
    match index {
        CalcResult::Range { left, right } => left != right,
        CalcResult::Array(a) => a.len() > 1 || a.first().is_some_and(|r| r.len() > 1),
        _ => false,
    }
}

/// Resolves the effective (row, column) indices for INDEX, where `0` means "the
/// whole row/column" (used to spill an entire row, column or array).
///
/// In the two-argument form `INDEX(array, num)` the single index is interpreted
/// based on the shape of the source: for a single-row source it is a column
/// index (all rows), otherwise it is a row index (all columns — i.e. a whole row
/// for a 2-D source, or a single element for a column vector).
fn index_effective_indices(
    num_rows: usize,
    row_num: usize,
    col_num: usize,
    two_arg: bool,
) -> (usize, usize) {
    if two_arg {
        if num_rows == 1 {
            (0, row_num)
        } else {
            (row_num, 0)
        }
    } else {
        (row_num, col_num)
    }
}

/// Extracts the requested cell, row, column or whole array from `arr`.
/// `row_eff`/`col_eff` of `0` select every row/column respectively. A selection
/// that resolves to a single cell is returned as a scalar; otherwise an array.
fn index_from_array(
    arr: &[Vec<ArrayNode>],
    row_eff: usize,
    col_eff: usize,
    cell: CellReferenceIndex,
) -> CalcResult {
    let num_rows = arr.len();
    let num_cols = arr[0].len();
    if row_eff > num_rows || col_eff > num_cols {
        return CalcResult::Error {
            error: Error::REF,
            origin: cell,
            message: "Wrong reference".to_string(),
        };
    }
    let (row_start, row_end) = if row_eff == 0 {
        (1, num_rows)
    } else {
        (row_eff, row_eff)
    };
    let (col_start, col_end) = if col_eff == 0 {
        (1, num_cols)
    } else {
        (col_eff, col_eff)
    };
    if row_start == row_end && col_start == col_end {
        return array_node_to_calc_result(&arr[row_start - 1][col_start - 1], cell);
    }
    let mut result = Vec::with_capacity(row_end - row_start + 1);
    for r in row_start..=row_end {
        let mut data_row = Vec::with_capacity(col_end - col_start + 1);
        for c in col_start..=col_end {
            data_row.push(arr[r - 1][c - 1].clone());
        }
        result.push(data_row);
    }
    CalcResult::Array(result)
}

impl Model {
    // INDEX(array, row_num, [column_num])
    // INDEX(reference, row_num, [column_num], [area_num])
    pub(crate) fn fn_index(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.index_impl(args, cell, false)
    }

    /// INDEX; `as_reference` keeps a single selected cell as a 1x1 range, as an
    /// aggregate argument sees it (T-0216; docs/excel-semantics/references.md).
    ///
    /// A multi-cell range or array as row_num, column_num or area_num lifts INDEX: the indices
    /// broadcast like arithmetic operands and each element is the scalar INDEX of its indices,
    /// as a value (`INDEX(A1:A5,{1,3})` spills A1 and A3; `ROWS` of it is 1 and `ISREF` FALSE).
    /// An element that selects a whole row, column or array keeps only its top-left value
    /// (`INDEX(A1:A5,{0,2})` is A1, A2; probes/index-array-indices, T-1260).
    pub(crate) fn index_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let two_arg = args.len() == 2;
        // A missing or empty row_num or column_num is 0 ("the whole row/column"); area_num defaults to 1.
        let row = self.evaluate_node_in_context(&args[1], cell);
        let col = args.get(2).map(|a| self.evaluate_node_in_context(a, cell));
        let area = args.get(3).map(|a| self.evaluate_node_in_context(a, cell));
        if lifts(&row) || col.as_ref().is_some_and(lifts) || area.as_ref().is_some_and(lifts) {
            return self.index_lifted(&args[0], row, col, area, two_arg, cell);
        }
        let area = area.map_or(Ok(1.0), |a| self.cast_to_number(a, cell));
        let row = self.cast_to_number(row, cell);
        let col = col.map_or(Ok(0.0), |c| self.cast_to_number(c, cell));
        let numbers = match index_numbers(area, row, col, || {
            CalcResult::new_error(Error::VALUE, cell, "Index out of range".into())
        }) {
            Ok(n) => n,
            Err(e) => return e,
        };
        let arg = self.evaluate_reference_arg(&args[0], cell);
        self.index_select(&arg, numbers, two_arg, cell, as_reference)
    }

    /// The lifted INDEX: one scalar INDEX per broadcast element of the indices, the reference
    /// evaluated once. An index that is an error makes every element that error.
    fn index_lifted(
        &mut self,
        reference: &Node,
        row: CalcResult,
        col: Option<CalcResult>,
        area: Option<CalcResult>,
        two_arg: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut operand = |value: Option<CalcResult>, default: f64| match value {
            None => NumberOrArray::Number(default),
            Some(v) => match self.number_or_array(v, cell) {
                Ok(na) => na,
                Err(e) => NumberOrArray::Array(vec![vec![calc_result_to_array_node(e)]]),
            },
        };
        // Excel's order for an element: area_num's error, then row_num's, then column_num's.
        let operands = [
            operand(area, 1.0),
            operand(Some(row), 0.0),
            operand(col, 0.0),
        ];
        let rows = operands.iter().map(|na| na.dims().0).max().unwrap_or(0);
        let cols = operands.iter().map(|na| na.dims().1).max().unwrap_or(0);
        let arg = self.evaluate_reference_arg(reference, cell);
        let mut result = Vec::with_capacity(rows);
        for i in 0..rows {
            let mut data_row = Vec::with_capacity(cols);
            for j in 0..cols {
                let [area, row, col] = &operands;
                let numbers = index_numbers(
                    self.number_at(area, i, j),
                    self.number_at(row, i, j),
                    self.number_at(col, i, j),
                    || Error::VALUE,
                );
                data_row.push(match numbers {
                    Err(e) => ArrayNode::Error(e),
                    Ok(numbers) => {
                        let selected = self.index_select(&arg, numbers, two_arg, cell, false);
                        self.top_left_node(selected)
                    }
                });
            }
            result.push(data_row);
        }
        CalcResult::Array(result)
    }

    /// One element of a lifted INDEX: a selected row, column or array keeps its top-left value.
    fn top_left_node(&mut self, selected: CalcResult) -> ArrayNode {
        match selected {
            CalcResult::Range { left, .. } => calc_result_to_array_node(self.evaluate_cell(left)),
            CalcResult::Array(a) => a
                .into_iter()
                .next()
                .and_then(|r| r.into_iter().next())
                .unwrap_or(ArrayNode::Error(Error::REF)),
            other => calc_result_to_array_node(other),
        }
    }

    /// The scalar INDEX of validated `numbers` into the evaluated reference or array `arg`.
    fn index_select(
        &mut self,
        arg: &RefArg,
        numbers: IndexNumbers,
        two_arg: bool,
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        let IndexNumbers {
            row: row_num,
            col: col_num,
            area: area_num,
        } = numbers;
        let area_range;
        let reference = if let Some(areas) = arg.single_sheet_areas() {
            // NaN is out of range too; in range the conversion below is exact.
            if area_num.is_nan() || area_num > areas.len() as f64 {
                return CalcResult::new_error(Error::REF, cell, "Area out of range".into());
            }
            let Range { left, right } = areas[area_num as usize - 1];
            area_range = CalcResult::Range { left, right };
            &area_range
        } else {
            match arg {
                RefArg::Ranges(_) => return three_d_needs_one_sheet(cell),
                RefArg::Value(_) if area_num != 1.0 => {
                    return CalcResult::new_error(Error::REF, cell, "Area out of range".into());
                }
                RefArg::Value(value) => value,
            }
        };
        match reference {
            &CalcResult::Range { left, right } => {
                let num_rows = (right.row - left.row + 1) as usize;
                let (row_eff, col_eff) =
                    index_effective_indices(num_rows, row_num, col_num, two_arg);
                let num_cols = (right.column - left.column + 1) as usize;
                // Excel: one index into a 2-D range is #REF!, while an array
                // gives the row (docs/excel-semantics/lookup.md).
                if row_eff > num_rows
                    || col_eff > num_cols
                    || (two_arg && num_rows > 1 && num_cols > 1)
                {
                    return CalcResult::Error {
                        error: Error::REF,
                        origin: cell,
                        message: "Wrong reference".to_string(),
                    };
                }
                let (row_start, row_end) = if row_eff == 0 {
                    (left.row, right.row)
                } else {
                    let r = left.row + row_eff as i32 - 1;
                    (r, r)
                };
                let (col_start, col_end) = if col_eff == 0 {
                    (left.column, right.column)
                } else {
                    let c = left.column + col_eff as i32 - 1;
                    (c, c)
                };
                if !as_reference && row_start == row_end && col_start == col_end {
                    self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row: row_start,
                        column: col_start,
                    })
                } else {
                    CalcResult::Range {
                        left: CellReferenceIndex {
                            sheet: left.sheet,
                            row: row_start,
                            column: col_start,
                        },
                        right: CellReferenceIndex {
                            sheet: left.sheet,
                            row: row_end,
                            column: col_end,
                        },
                    }
                }
            }
            CalcResult::Array(arr) => {
                if arr.is_empty() || arr[0].is_empty() {
                    return CalcResult::Error {
                        error: Error::REF,
                        origin: cell,
                        message: "Empty array".to_string(),
                    };
                }
                let (row_eff, col_eff) =
                    index_effective_indices(arr.len(), row_num, col_num, two_arg);
                index_from_array(arr, row_eff, col_eff, cell)
            }
            error @ CalcResult::Error { .. } => error.clone(),
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Expecting a Range".to_string(),
            },
        }
    }
}
