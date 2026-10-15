//! INDEX: one cell, row, column or area of a reference or an array, lifted element-wise over
//! array indices (docs/excel-semantics/lookup.md).

use crate::cast::{calc_result_to_array_node, NumberOrArray};
use crate::external_reference::ExternalArea;
use crate::functions::util::{
    array_node_to_calc_result, near_integer_count, NEAR_INTEGER_TOLERANCE,
};
use crate::limits::check_array_size;
use crate::reference_operators::{three_d_needs_one_sheet, Picked, RefArg};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    expressions::types::CellReferenceIndex,
    model::Model,
};

/// INDEX's validated numeric arguments: `row_num` and `column_num` read by [`index_number`] (`0` = the
/// whole row or column), `area_num` read like them ([`near_integer_count`]) and at least 1.
#[derive(Clone, Copy)]
struct IndexNumbers {
    row: usize,
    col: usize,
    area: f64,
}

/// Validates INDEX's numbers, each cast already tried, in Excel's order: `area_num`'s cast error,
/// then `area_num` below 1 (`#VALUE!`; past the last area, or NaN, is `#REF!` later, SPEC-0004), then
/// `row_num`'s cast error and a negative `row_num` (`#VALUE!`, see [`index_number`]), then the same
/// for `column_num`.
fn index_numbers<E>(
    area: Result<f64, E>,
    row: Result<f64, E>,
    col: Result<f64, E>,
    out_of_range: impl Fn() -> E,
) -> Result<IndexNumbers, E> {
    // area_num rounds up near an integer like row_num (probes/index-area-num-snap J1:J6; T-1424).
    let area = near_integer_count(area?);
    if area < 1.0 {
        return Err(out_of_range());
    }
    let row = index_number(row?).ok_or_else(&out_of_range)?;
    let col = index_number(col?).ok_or_else(&out_of_range)?;
    Ok(IndexNumbers { row, col, area })
}

/// A row or column index as INDEX reads it, `None` for a negative one (`#VALUE!`). A fraction
/// within [`NEAR_INTEGER_TOLERANCE`] below an integer counts as that integer
/// (`INDEX(A1:C3,3,1-2.3853E-7)` is column 1), and a negative index no further below 0 than half
/// that bound counts as 0 (`INDEX(A1:C3,3,-1E-7)` is the whole row 3; `-1.193E-7` is `#VALUE!`).
/// Excel: docs/excel-semantics/lookup.md "INDEX rounds an index up near an integer"
/// (probes/index-tiny-negative, probes/index-snap-tolerance).
fn index_number(x: f64) -> Option<usize> {
    if x < -NEAR_INTEGER_TOLERANCE / 2.0 {
        return None;
    }
    // A huge index saturates and is out of range later; NaN is 0, as before.
    Some(near_integer_count(x.max(0.0)) as usize)
}

/// Whether an evaluated index makes INDEX lift: a multi-cell range or any array. A single cell
/// stays a scalar index, but a 1x1 array lifts too, so `INDEX(D1:D5,{2})` is a 1x1 array and not
/// a reference: ISREF FALSE, TYPE 64, `COUNTBLANK` and `AREAS` of it `#VALUE!`
/// (docs/excel-semantics/lookup.md "INDEX lifts over array and range indices";
/// probes/index-1x1-array-index, countblank-array-values H1).
fn lifts(index: &CalcResult) -> bool {
    match index {
        CalcResult::Range { left, right } => left != right,
        CalcResult::Array(_) => true,
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

/// The rows and columns INDEX selects in an area of `num_rows` by `num_cols` cells, 0-based and
/// inclusive: `(first_row, last_row, first_column, last_column)`. `None` is `#REF!`: an index past
/// the area, or one index into a 2-D area, where an array would give the row
/// (docs/excel-semantics/lookup.md).
fn index_window(
    num_rows: usize,
    num_cols: usize,
    numbers: IndexNumbers,
    two_arg: bool,
) -> Option<(usize, usize, usize, usize)> {
    let (row_eff, col_eff) = index_effective_indices(num_rows, numbers.row, numbers.col, two_arg);
    if row_eff > num_rows || col_eff > num_cols || (two_arg && num_rows > 1 && num_cols > 1) {
        return None;
    }
    let span = |eff: usize, count: usize| {
        if eff == 0 {
            (0, count.saturating_sub(1))
        } else {
            (eff - 1, eff - 1)
        }
    };
    let (first_row, last_row) = span(row_eff, num_rows);
    let (first_col, last_col) = span(col_eff, num_cols);
    Some((first_row, last_row, first_col, last_col))
}

/// Extracts the requested cell, row, column or whole array from `arr`.
/// `row_eff`/`col_eff` of `0` select every row/column respectively. A selection
/// that resolves to a single cell is returned as a scalar; otherwise an array, unless `top_left`
/// asks for its first value only (a lifted element, which must not copy the selection).
/// Whether `node` is an IF or CHOOSE call, which INDEX reads through to the branch it picks.
fn picks_a_branch(node: &Node) -> bool {
    matches!(
        node.ungrouped(),
        Node::FunctionKind {
            kind: crate::functions::Function::If | crate::functions::Function::Choose,
            ..
        }
    )
}

fn index_from_array(
    arr: &[Vec<ArrayNode>],
    row_eff: usize,
    col_eff: usize,
    top_left: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    let num_rows = arr.len();
    let num_cols = arr.first().map_or(0, Vec::len);
    if row_eff > num_rows || col_eff > num_cols {
        return wrong_reference(cell);
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
    if top_left || (row_start == row_end && col_start == col_end) {
        return match arr.get(row_start - 1).and_then(|r| r.get(col_start - 1)) {
            Some(node) => array_node_to_calc_result(node, cell),
            None => wrong_reference(cell),
        };
    }
    let mut result = Vec::with_capacity(row_end - row_start + 1);
    for row in &arr[row_start - 1..row_end] {
        result.push(row.get(col_start - 1..col_end).unwrap_or_default().to_vec());
    }
    CalcResult::Array(result)
}

fn wrong_reference(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "Wrong reference".into())
}

/// INDEX's indices: validated numbers, or the evaluated indices when one lifts INDEX.
enum IndexArgs {
    Numbers(IndexNumbers),
    Lifted {
        row: CalcResult,
        col: Option<CalcResult>,
        area: Option<CalcResult>,
    },
}

/// What INDEX returns for its selection.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndexForm {
    /// A single selected cell is its value; a row, column or area stays a reference.
    Value,
    /// A single selected cell stays a 1x1 reference, as an aggregate argument sees it.
    Reference,
    /// The value of the selection's top-left cell (an element of a lifted INDEX).
    TopLeft,
}

impl Model {
    // INDEX(array, row_num, [column_num])
    // INDEX(reference, row_num, [column_num], [area_num])
    pub(crate) fn fn_index(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.index_impl(args, cell, IndexForm::Value)
    }

    /// INDEX in reference context: a single selected cell stays a 1x1 range ([`IndexForm::Reference`]).
    /// The one definition for every caller that evaluates an INDEX node as a reference.
    pub(crate) fn index_reference(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.index_impl(args, cell, IndexForm::Reference)
    }

    /// INDEX; [`IndexForm::Reference`] keeps a single selected cell as a 1x1 range, as an
    /// aggregate argument sees it (T-0216; docs/excel-semantics/references.md).
    ///
    /// A multi-cell range or array as `row_num`, `column_num` or `area_num` lifts INDEX: the indices
    /// broadcast like arithmetic operands and each element is the scalar INDEX of its indices,
    /// as a value (`INDEX(A1:A5,{1,3})` spills A1 and A3; `ROWS` of it is 1 and `ISREF` FALSE).
    /// An element that selects a whole row, column or array keeps only its top-left value
    /// (`INDEX(A1:A5,{0,2})` is A1, A2; probes/index-array-indices, T-1260).
    pub(crate) fn index_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        form: IndexForm,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let two_arg = args.len() == 2;
        match self.external_index_args(args, cell) {
            None => {}
            Some(Err(error)) => return error,
            Some(Ok((areas, IndexArgs::Numbers(numbers)))) => {
                return match external_index_area(&areas, numbers, two_arg, cell) {
                    Ok(area) => self.external_picked_value(area, cell),
                    Err(error) => error,
                };
            }
            // Each element is the scalar INDEX's top-left cell, empty with no cache
            // (probes/external-ref-no-link-ifchoose H1, I1).
            Some(Ok((areas, IndexArgs::Lifted { row, col, area }))) => {
                return self.index_lifted(row, col, area, cell, None, |model, numbers| {
                    match external_index_area(&areas, numbers, two_arg, cell) {
                        Ok(picked) => model.external_picked_value(picked.top_left(), cell),
                        Err(error) => error,
                    }
                });
            }
        }
        let numbers = match self.index_numbers(args, cell) {
            Ok(IndexArgs::Numbers(numbers)) => numbers,
            Ok(IndexArgs::Lifted { row, col, area }) => {
                let arg = self.evaluate_reference_arg(&args[0], cell);
                // An error first argument is every element's result.
                let arg_error = match &arg {
                    RefArg::Value(CalcResult::Error { error, .. }) => Some(error.clone()),
                    _ => None,
                };
                return self.index_lifted(row, col, area, cell, arg_error, |model, numbers| {
                    model.index_select(&arg, numbers, two_arg, cell, IndexForm::TopLeft)
                });
            }
            Err(error) => return error,
        };
        // INDEX into an IF or CHOOSE that picks a reference into another workbook picks from
        // that reference (`ERROR.TYPE(INDEX(IF(TRUE,…!A1:C4),2,2))` is `#N/A`;
        // probes/external-ref-no-link-ifchoose A22, A23). The pick reads the condition once,
        // after the indices, as the reference is read otherwise.
        let arg = match self.pick_branch(&args[0], cell) {
            Picked::Node(reference) => {
                if let Some(areas) = self.external_reference(reference, cell) {
                    return match areas
                        .and_then(|areas| external_index_area(&areas, numbers, two_arg, cell))
                    {
                        Ok(area) => self.external_picked_value(area, cell),
                        Err(error) => error,
                    };
                }
                match self.evaluate_reference_arg(reference, cell) {
                    // A scalar picked by IF or CHOOSE is a one-element array: `INDEX(CHOOSE(1,
                    // @B1:B6*1),3)` is `#REF!` on the range's rows (probes/legacy-choose-arm-
                    // aggregate-and-array-constant-at H1, T-1643). A scalar anywhere else keeps
                    // being `#VALUE!` (unprobed).
                    RefArg::Value(
                        scalar @ (CalcResult::Number(_)
                        | CalcResult::String(_)
                        | CalcResult::Boolean(_)),
                    ) if picks_a_branch(&args[0]) => {
                        RefArg::Value(CalcResult::Array(vec![vec![calc_result_to_array_node(
                            scalar,
                        )]]))
                    }
                    other => other,
                }
            }
            Picked::Value(value) => RefArg::Value(value),
        };
        self.index_select(&arg, numbers, two_arg, cell, form)
    }

    /// INDEX's `row_num`, `column_num` and `area_num` (`args[1..]`), evaluated and validated, or
    /// the evaluated indices when one lifts. A missing or empty `row_num` or `column_num` is 0
    /// ("the whole row/column"); `area_num` defaults to 1.
    fn index_numbers(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<IndexArgs, CalcResult> {
        let row = args.get(1).map_or(CalcResult::EmptyArg, |a| {
            self.evaluate_node_in_context(a, cell)
        });
        let col = args.get(2).map(|a| self.evaluate_node_in_context(a, cell));
        let area = args.get(3).map(|a| self.evaluate_node_in_context(a, cell));
        if lifts(&row) || col.as_ref().is_some_and(lifts) || area.as_ref().is_some_and(lifts) {
            return Ok(IndexArgs::Lifted { row, col, area });
        }
        let area = area.map_or(Ok(1.0), |a| self.cast_to_number(a, cell));
        let row = self.cast_to_number(row, cell);
        let col = col.map_or(Ok(0.0), |c| self.cast_to_number(c, cell));
        index_numbers(area, row, col, || {
            CalcResult::new_error(Error::VALUE, cell, "Index out of range".into())
        })
        .map(IndexArgs::Numbers)
    }

    /// INDEX into a reference into another workbook ([`Model::external_reference`]): the area
    /// it picks, still a reference (`ROWS(INDEX(…!A1:C4,2,0))` is 1, `ISREF(INDEX(…,2,2))` TRUE,
    /// `ROWS(INDEX((…!A1:B2,…!C3:D5),1,1,2))` 1; probes/external-ref-no-link-shapes A3, A4,
    /// external-ref-no-link-index A21). `None` when `args[0]` is no such reference (nothing is
    /// evaluated then) or the argument count is wrong. An index array that would lift INDEX is
    /// `#BLOCKED!` here, as a reference (UNVERIFIED: not probed); INDEX itself lifts it ([`Self::index_impl`]).
    pub(crate) fn external_index(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<Result<ExternalArea, CalcResult>> {
        Some(match self.external_index_args(args, cell)? {
            Ok((areas, IndexArgs::Numbers(numbers))) => {
                external_index_area(&areas, numbers, args.len() == 2, cell)
            }
            Ok((_, IndexArgs::Lifted { .. })) => Err(CalcResult::new_error(
                Error::BLOCKED,
                cell,
                "External reference".into(),
            )),
            Err(error) => Err(error),
        })
    }

    /// The external areas `args[0]` is and INDEX's evaluated indices, for [`Self::external_index`]
    /// and the lifted INDEX of [`Self::index_impl`]. `None` as for `external_index`.
    fn external_index_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<Result<(Vec<ExternalArea>, IndexArgs), CalcResult>> {
        if !(2..=4).contains(&args.len()) {
            return None;
        }
        let areas = match self.external_reference(&args[0], cell)? {
            Ok(areas) => areas,
            Err(error) => return Some(Err(error)),
        };
        Some(
            self.index_numbers(args, cell)
                .map(|numbers| (areas, numbers)),
        )
    }

    /// The lifted INDEX: `select` is the scalar INDEX of one broadcast element's validated
    /// indices, the reference evaluated once by the caller. An index that is an error makes every
    /// element that error, and so does `arg_error`, the first argument's error, after it.
    fn index_lifted(
        &mut self,
        row: CalcResult,
        col: Option<CalcResult>,
        area: Option<CalcResult>,
        cell: CellReferenceIndex,
        arg_error: Option<Error>,
        mut select: impl FnMut(&mut Self, IndexNumbers) -> CalcResult,
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
        // The shared array bound (CLAUDE.md directive 5; DEVIATION, docs/excel-semantics/arrays.md).
        if let Some((error, message)) = check_array_size(rows, cols) {
            return CalcResult::new_error(error, cell, message);
        }
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
                data_row.push(match (numbers, &arg_error) {
                    (Err(e), _) => ArrayNode::Error(e),
                    (Ok(_), Some(error)) => ArrayNode::Error(error.clone()),
                    (Ok(numbers), None) => calc_result_to_array_node(select(self, numbers)),
                });
            }
            result.push(data_row);
        }
        CalcResult::Array(result)
    }

    /// The scalar INDEX of validated `numbers` into the evaluated reference or array `arg`.
    fn index_select(
        &mut self,
        arg: &RefArg,
        numbers: IndexNumbers,
        two_arg: bool,
        cell: CellReferenceIndex,
        form: IndexForm,
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
                let num_cols = (right.column - left.column + 1) as usize;
                let Some((first_row, last_row, first_col, last_col)) =
                    index_window(num_rows, num_cols, numbers, two_arg)
                else {
                    return wrong_reference(cell);
                };
                let row_start = left.row + first_row as i32;
                let row_end = left.row + last_row as i32;
                let col_start = left.column + first_col as i32;
                let col_end = left.column + last_col as i32;
                let single = row_start == row_end && col_start == col_end;
                if form == IndexForm::TopLeft || (form == IndexForm::Value && single) {
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
                index_from_array(arr, row_eff, col_eff, form == IndexForm::TopLeft, cell)
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

/// The area INDEX picks from `areas` of another workbook with `numbers` (`two_arg` when no
/// `column_num` is given), or the error it gives: `#REF!` past the last area or the area's edge.
fn external_index_area(
    areas: &[ExternalArea],
    numbers: IndexNumbers,
    two_arg: bool,
    cell: CellReferenceIndex,
) -> Result<ExternalArea, CalcResult> {
    // NaN is out of range too; in range the conversion below is exact.
    let area = if numbers.area.is_nan() || numbers.area > areas.len() as f64 {
        None
    } else {
        areas.get(numbers.area as usize - 1)
    };
    let Some(&area) = area else {
        return Err(CalcResult::new_error(
            Error::REF,
            cell,
            "Area out of range".into(),
        ));
    };
    let (num_rows, num_cols) = area.extent();
    let Some((first_row, last_row, first_col, last_col)) =
        index_window(num_rows, num_cols, numbers, two_arg)
    else {
        return Err(wrong_reference(cell));
    };
    // Offsets within an area on the grid, so they fit an i32.
    let offset = |first: i32, by: usize| first + by as i32;
    Ok(ExternalArea {
        first_row: offset(area.first_row, first_row),
        first_column: offset(area.first_column, first_col),
        last_row: offset(area.first_row, last_row),
        last_column: offset(area.first_column, last_col),
        cache: area.cache,
    })
}
