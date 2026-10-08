use crate::cast::{calc_result_to_array_node, NumberOrArray};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::array_node_to_calc_result;
use crate::functions::Function;
use crate::reference_operators::{three_d_needs_one_sheet, Ranges, RefArg};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::Node,
    expressions::token::Error,
    model::Model,
    utils::ParsedReference,
};

use super::criteria::wildcard_text_matcher;
use lookup_table::{LookupTable, SearchVector};

mod address_areas;
mod choosecols_chooserows;
mod drop_take;
mod expand;
mod hstack_vstack;
mod index_arg;
mod lookup;
pub(crate) mod lookup_table;
mod tocol_torow;
mod transpose;
mod trimrange;
mod wrapcols_wraprows;
pub(crate) mod xmatch;

/// Finds the 0-based index of `lookup_value` within `search_vector`.
/// When `is_sorted` is true Excel's approximate-match search is used (the
/// largest value of the lookup value's type smaller than or equal to it);
/// otherwise a linear search for an exact match (supporting wildcards for
/// text), shared by VLOOKUP/HLOOKUP FALSE and MATCH 0. Returns `None` when
/// the value is not found.
fn lookup_index(
    lookup_value: &CalcResult,
    search_vector: &SearchVector,
    is_sorted: bool,
) -> Option<usize> {
    if is_sorted {
        search_vector.approximate_search(lookup_value, false)
    } else if let CalcResult::String(s) = lookup_value {
        search_vector.position(wildcard_text_matcher(s))
    } else {
        search_vector.exact_position(lookup_value)
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
    /// A call to a function whose result is a reference when its source argument is one,
    /// evaluated as that reference: XLOOKUP, TAKE, DROP and TRIMRANGE, which are otherwise
    /// evaluated to their values (probes/intersection-function-operands H8, H11, H12, H14;
    /// T-0165). `None` for any other function.
    pub(crate) fn evaluate_reference_form(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        Some(match kind {
            Function::Xlookup => self.xlookup_impl(args, cell, true),
            Function::Take => self.take_drop_impl(args, cell, drop_take::Edge::Take, true),
            Function::Drop => self.take_drop_impl(args, cell, drop_take::Edge::Drop, true),
            Function::Trimrange => self.trimrange_impl(args, cell, true),
            _ => return None,
        })
    }

    // INDEX(array, row_num, [column_num])
    // INDEX(range, row_num, [column_num], [area_num])
    // At the moment IronCalc does not support references with multiple areas,
    // so area_num = 1 (or missing).
    pub(crate) fn fn_index(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.index_impl(args, cell, false)
    }

    /// INDEX; `as_reference` keeps a single selected cell as a 1x1 range, as an
    /// aggregate argument sees it (T-0216; docs/excel-semantics/references.md).
    pub(crate) fn index_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        // area_num (4th argument), read as a float and range-checked before any integer
        // conversion: below 1 is #VALUE!, past the last area (or NaN) is #REF! (SPEC-0004).
        let area_num = if args.len() == 4 {
            let n = match self.get_number(&args[3], cell) {
                Ok(f) => f.floor(),
                Err(s) => return s,
            };
            if n < 1.0 {
                return CalcResult::new_error(Error::VALUE, cell, "Argument must be >= 1".into());
            }
            n
        } else {
            1.0
        };
        let negative_index =
            || CalcResult::new_error(Error::VALUE, cell, "Argument must be >= 0".to_string());
        // A missing or empty argument evaluates to 0, which means "the whole
        // row/column". A negative index is an error.
        let row_num = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if row_num < 0.0 {
            return negative_index();
        }
        let two_arg = args.len() == 2;
        let col_num = if two_arg {
            0.0
        } else {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        };
        if col_num < 0.0 {
            return negative_index();
        }
        let row_num = row_num.floor() as usize;
        let col_num = col_num.floor() as usize;

        let arg = self.evaluate_reference_arg(&args[0], cell);
        let reference = if let Some(areas) = arg.single_sheet_areas() {
            // NaN is out of range too; in range the conversion below is exact.
            if area_num.is_nan() || area_num > areas.len() as f64 {
                return CalcResult::new_error(Error::REF, cell, "Area out of range".into());
            }
            let Range { left, right } = areas[area_num as usize - 1];
            CalcResult::Range { left, right }
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
            CalcResult::Range { left, right } => {
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
                index_from_array(&arr, row_eff, col_eff, cell)
            }
            error @ CalcResult::Error { .. } => error,
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Expecting a Range".to_string(),
            },
        }
    }

    //     MATCH(lookup_value, lookup_array, [match_type])
    // The MATCH function syntax has the following arguments:
    //   * lookup_value    Required. The value that you want to match in lookup_array.
    //                     The lookup_value argument can be a value (number, text, or logical value)
    //                     or a cell reference to a number, text, or logical value.
    //   * lookup_array    Required. The range of cells being searched.
    //   * match_type      Optional. The number -1, 0, or 1.
    //                     The match_type argument specifies how Excel matches lookup_value
    //                     with values in lookup_array. The default value for this argument is 1.
    // Excel: see docs/excel-semantics/lookup.md "Approximate-match lookups".
    pub(crate) fn fn_match(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 3 || args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let target = self.evaluate_node_in_context(&args[0], cell);
        if target.is_error() {
            return target;
        }
        let match_type = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(v) => v as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        let match_range = self.evaluate_single_sheet_arg(&args[1], cell);

        // MATCH operates on a vector (a single row or single column). We
        // materialize the second argument, be it a range reference or an array
        // literal, into a flat list of values and then run the search on it.
        let values = match self.as_vector(match_range, cell) {
            Ok(values) => values,
            Err(error) => return error,
        };

        let found = if match_type == 0 {
            lookup_index(&target, &values, false)
        } else {
            // Any negative match_type searches descending, any positive one
            // ascending (probes/lookup-approximate-search N20, N21).
            values.approximate_search(&target, match_type < 0)
        };
        match found {
            Some(l) => CalcResult::Number(l as f64 + 1.0),
            None => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not found".to_string(),
            },
        }
    }

    /// HLOOKUP(lookup_value, table_array, row_index, [is_sorted])
    /// We look for `lookup_value` in the first row of table array
    /// We return the value in row `row_index` of the same column in `table_array`
    /// `is_sorted` is true by default and assumes that values in first row are ordered
    pub(crate) fn fn_hlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 4 || args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        let row_index = match self.lookup_index_arg(&args[2], cell) {
            Ok(index) => index,
            Err(error) => return error,
        };
        let is_sorted = if args.len() == 4 {
            match self.get_boolean(&args[3], cell) {
                Ok(v) => v,
                Err(s) => return s,
            }
        } else {
            true
        };
        let range = self.evaluate_single_sheet_arg(&args[1], cell);
        let table = match range {
            CalcResult::Range { left, right } => LookupTable::Range { left, right },
            CalcResult::Array(array) => LookupTable::Array(array),
            error @ CalcResult::Error { .. } => return error,
            CalcResult::String(_) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
            _ => {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        // We look for `lookup_value` in the first row of the table
        let search_vector = table.first_row(self, cell);
        // Excel searches first: a value that is not found is #N/A whatever the index is.
        match lookup_index(&lookup_value, &search_vector, is_sorted) {
            Some(column) => match self.resolve_lookup_index(row_index, table.rows(), cell) {
                Ok(row_index) => table.get(self, row_index, column as i32, cell),
                Err(error) => error,
            },
            None => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not found".to_string(),
            },
        }
    }

    /// VLOOKUP(lookup_value, table_array, row_index, [is_sorted])
    /// We look for `lookup_value` in the first column of table array
    /// We return the value in column `column_index` of the same row in `table_array`
    /// `is_sorted` is true by default and assumes that values in first column are ordered
    pub(crate) fn fn_vlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 4 || args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        let column_index = match self.lookup_index_arg(&args[2], cell) {
            Ok(index) => index,
            Err(error) => return error,
        };
        let is_sorted = if args.len() == 4 {
            match self.get_boolean(&args[3], cell) {
                Ok(v) => v,
                Err(s) => return s,
            }
        } else {
            true
        };
        let range = self.evaluate_single_sheet_arg(&args[1], cell);
        let table = match range {
            CalcResult::Range { left, right } => LookupTable::Range { left, right },
            CalcResult::Array(array) => LookupTable::Array(array),
            error @ CalcResult::Error { .. } => return error,
            CalcResult::String(_) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
            _ => {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        // We look for `lookup_value` in the first column of the table
        let search_vector = table.first_column(self, cell);
        // Excel searches first: a value that is not found is #N/A whatever the index is.
        match lookup_index(&lookup_value, &search_vector, is_sorted) {
            Some(row) => match self.resolve_lookup_index(column_index, table.columns(), cell) {
                Ok(column_index) => table.get(self, row as i32, column_index, cell),
                Err(error) => error,
            },
            None => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not found".to_string(),
            },
        }
    }

    // HYPERLINK(link_location, [friendly_name])
    // Attaches a link to the cell and returns friendly_name (or link_location
    // if friendly_name is missing). A link_location starting with "#" is an
    // internal link ("#Sheet1!A1"), anything else is external.
    // The link itself is dynamic: it lives in `Model::links` and is rebuilt on
    // every evaluation, it is not part of the workbook.
    pub(crate) fn fn_hyperlink(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let location = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };

        let display = if args.len() == 2 {
            // The friendly name is the value displayed in the cell. It can be
            // a string, a number or a boolean.
            let result = self.evaluate_node_in_context(&args[1], cell);
            match result {
                CalcResult::Range { .. } | CalcResult::Array(_) => {
                    match self.cast_to_string(result, cell) {
                        Ok(s) => CalcResult::String(s),
                        Err(error) => error,
                    }
                }
                other => other,
            }
        } else {
            CalcResult::String(location.clone())
        };
        // A cell displaying an error is not clickable: no link is attached
        if matches!(display, CalcResult::Error { .. }) {
            return display;
        }

        let link = match location.strip_prefix('#') {
            Some(internal) => crate::types::Link::Internal {
                location: internal.to_string(),
                tooltip: None,
            },
            None => crate::types::Link::External {
                target: location,
                tooltip: None,
            },
        };
        self.links.insert((cell.sheet, cell.row, cell.column), link);
        display
    }

    // ROW([reference])
    // If reference is not present returns the row of the present cell.
    // Otherwise returns the row number of reference. A reference that spans
    // several rows gives a column with the number of each of them.
    pub(crate) fn fn_row(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.row_or_column(args, cell, true, false)
    }

    /// Shared body of ROW and COLUMN. With `first_only` a multi-cell reference yields just its
    /// first number, which is what implicit intersection (`@ROW(A:A)`) keeps, so the vector over
    /// up to a million rows is never built.
    fn row_or_column(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        by_row: bool,
        first_only: bool,
    ) -> CalcResult {
        if args.len() > 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if args.is_empty() {
            return CalcResult::Number(if by_row { cell.row } else { cell.column } as f64);
        }
        match self.reference_ranges(&args[0], cell) {
            // ROW and COLUMN of a union are #REF!, not the #VALUE! of a value context
            // (probes/multiarea-cellfn H3, multiarea-cellfn-types H9; SPEC-0004 row 18).
            Ok(Ranges::Areas(_)) => {
                CalcResult::new_error(Error::REF, cell, "Reference of several areas".to_string())
            }
            Ok(Ranges::One(range)) => {
                let (a, b) = if by_row {
                    (range.left.row, range.right.row)
                } else {
                    (range.left.column, range.right.column)
                };
                let (first, last) = (a.min(b), a.max(b));
                if first == last || first_only {
                    return CalcResult::Number(first as f64);
                }
                let numbers = (first..=last).map(|n| ArrayNode::Number(n as f64));
                CalcResult::Array(if by_row {
                    numbers.map(|n| vec![n]).collect()
                } else {
                    vec![numbers.collect()]
                })
            }
            Err(s) => s,
        }
    }

    /// Evaluates the operand of `@`. `@ROW(A:A)` and `@COLUMN(A1:Z1)` take a fast path that skips
    /// materialising the whole vector; everything else is evaluated normally.
    /// `@` of a span over several sheets is `#REF!`, not the `#VALUE!` of a 3-D reference used as
    /// a single-sheet one: `=@Data1:Data3!A1` and `=SUM(@Data1:Data3!A1:A2)`
    /// (probes/3d-ref-span-edges B1, B3; T-1170).
    pub(crate) fn at_of_span_error(&self, cell: CellReferenceIndex) -> CalcResult {
        CalcResult::new_error(
            Error::REF,
            cell,
            "Implicit intersection of a 3-D reference".to_string(),
        )
    }

    pub(crate) fn evaluate_implicit_intersection_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node {
            Node::SheetSpanKind {
                first_index,
                last_index,
                ..
            } if first_index != last_index => self.at_of_span_error(cell),
            Node::FunctionKind {
                kind: Function::Row,
                args,
            } => self.row_or_column(args, cell, true, true),
            Node::FunctionKind {
                kind: Function::Column,
                args,
            } => self.row_or_column(args, cell, false, true),
            _ => self.evaluate_node_with_reference(node, cell),
        }
    }

    // ROWS(range)
    // Returns the number of rows in range
    pub(crate) fn fn_rows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.rows_columns_extent(&args[0], cell) {
            Ok((rows, _)) => CalcResult::Number(rows as f64),
            Err(s) => s,
        }
    }

    // COLUMN([reference])
    // If reference is not present returns the column of the present cell.
    // Otherwise returns the column number of reference. A reference that spans
    // several columns gives a row with the number of each of them.
    pub(crate) fn fn_column(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.row_or_column(args, cell, false, false)
    }

    /// CHOOSE(index_num, value1, [value2], ...)
    /// Uses index_num to return a value from the list of value arguments.
    pub(crate) fn fn_choose(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.choose_impl(args, cell, false).into_single_area(cell)
    }

    /// CHOOSE; with `as_reference` a scalar index hands back the chosen argument in
    /// reference context, so a union there stays its areas: `SUM(CHOOSE(2,A1,(A1,B2)))` is 6
    /// (probes/multiarea-ifchoose H3).
    pub(crate) fn choose_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> RefArg {
        if args.len() < 2 {
            return RefArg::Value(CalcResult::new_args_number_error(cell));
        }

        // The index argument may be a single number or an array/range. In the array
        // case we broadcast: each index element selects the matching value argument,
        // collapsed to a single scalar (the result has the shape of the index array).
        match self.get_number_or_array(&args[0], cell) {
            Ok(NumberOrArray::Number(index_num)) => {
                let index_num = index_num as usize;
                if index_num < 1 || index_num > (args.len() - 1) {
                    let error = CalcResult::new_error(Error::VALUE, cell, "Invalid index".into());
                    return RefArg::Value(error);
                }
                if as_reference {
                    return self.evaluate_reference_arg(&args[index_num], cell);
                }
                self.evaluate_node_with_reference(&args[index_num], cell)
                    .into()
            }
            Ok(NumberOrArray::Array(index_array)) => {
                let result = index_array
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|node| self.choose_element(args, node, cell))
                            .collect()
                    })
                    .collect();
                RefArg::Value(CalcResult::Array(result))
            }
            Err(calc_err) => RefArg::Value(calc_err),
        }
    }

    /// Selects the CHOOSE value argument for a single index element and collapses
    /// it to one array cell. A chosen range is reduced by implicit intersection.
    fn choose_element(
        &mut self,
        args: &[Node],
        index_node: &ArrayNode,
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        let index_num = match index_node {
            ArrayNode::Number(n) => *n,
            ArrayNode::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            ArrayNode::Empty => 0.0,
            ArrayNode::String(s) => match self.cast_number(s) {
                Some(f) => f,
                None => return ArrayNode::Error(Error::VALUE),
            },
            ArrayNode::Error(e) => return ArrayNode::Error(e.clone()),
        } as usize;

        if index_num < 1 || index_num > (args.len() - 1) {
            return ArrayNode::Error(Error::VALUE);
        }

        let result = self.evaluate_node_with_reference(&args[index_num], cell);
        let value = self.implicit_intersection_to_value(result, cell);
        calc_result_to_array_node(value)
    }

    // COLUMNS(range)
    // Returns the number of columns in range
    pub(crate) fn fn_columns(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.rows_columns_extent(&args[0], cell) {
            Ok((_, columns)) => CalcResult::Number(columns as f64),
            Err(s) => s,
        }
    }

    // INDIRECT(ref_text, [a1])
    // Returns the reference specified by 'ref_text'. It is read in A1 notation
    // unless 'a1' is FALSE, in which case it is read in R1C1 notation.
    pub(crate) fn fn_indirect(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let ref_text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(v) => return v,
        };
        let a1 = if args.len() == 2 {
            match self.get_boolean(&args[1], cell) {
                Ok(b) => b,
                Err(v) => return v,
            }
        } else {
            true
        };

        let parsed_reference = if a1 {
            ParsedReference::parse_reference_formula(
                Some(cell.sheet),
                &ref_text,
                self.locale,
                |name| self.get_sheet_index_by_name(name),
            )
        } else {
            ParsedReference::parse_r1c1_reference(cell, &ref_text, |name| {
                self.get_sheet_index_by_name(name)
            })
        };

        match parsed_reference {
            Ok(ParsedReference::CellReference(reference)) => CalcResult::Range {
                left: reference,
                right: reference,
            },
            Ok(ParsedReference::Range(left, right)) => CalcResult::Range { left, right },
            Err(message) => CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message,
            },
        }
    }

    /// Height or width argument of OFFSET as the signed distance from the anchor cell to the far
    /// edge of the area; `default` when the argument is missing or empty, `None` for exactly 0.
    ///
    /// Excel: see docs/excel-semantics/references.md "OFFSET negative and fractional height/width".
    /// A positive size `h` reaches `floor(h - 1)` cells on, a negative one `ceil(h + 1)` cells (so
    /// -3 reaches 2 back), and the fractions in (-1, 1) reach one cell back and on respectively.
    fn offset_size(
        &mut self,
        arg: Option<&Node>,
        default: i32,
        cell: CellReferenceIndex,
    ) -> Result<Option<i32>, CalcResult> {
        match arg {
            None | Some(Node::EmptyArgKind) => Ok(Some(default)),
            Some(node) => {
                let c = self.get_number(node, cell)?;
                Ok(if c > 0.0 {
                    Some((c - 1.0).floor() as i32)
                } else if c < 0.0 {
                    Some((c + 1.0).ceil() as i32)
                } else {
                    None
                })
            }
        }
    }

    // OFFSET(reference, rows, cols, [height], [width])
    // Returns a reference to a range that is a specified number of rows and columns from a cell or range of cells.
    // The reference that is returned can be a single cell or a range of cells.
    // You can specify the number of rows and the number of columns to be returned.
    pub(crate) fn fn_offset(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let l = args.len();
        if !(3..=5).contains(&l) {
            return CalcResult::new_args_number_error(cell);
        }
        let reference = match self.get_reference(&args[0], cell) {
            Ok(c) => c,
            Err(s) => return s,
        };
        let rows = match self.get_number(&args[1], cell) {
            Ok(c) => {
                if c < 0.0 {
                    c.ceil() as i32
                } else {
                    c.floor() as i32
                }
            }
            Err(s) => return s,
        };
        let cols = match self.get_number(&args[2], cell) {
            Ok(c) => {
                if c < 0.0 {
                    c.ceil() as i32
                } else {
                    c.floor() as i32
                }
            }
            Err(s) => return s,
        };
        // `as i32` saturates, so sums saturate too (untrusted input must not overflow): any such
        // result is outside the sheet and falls through to #REF!.
        let row_start = reference.left.row.saturating_add(rows);
        let column_start = reference.left.column.saturating_add(cols);
        // A missing or empty height/width keeps the reference's size
        // (poi/FormulaEvalTestData_Copy.xlsx EverythingTests!I1008, J1008, M1008, N1008).
        let height =
            match self.offset_size(args.get(3), reference.right.row - reference.left.row, cell) {
                Ok(h) => h,
                Err(e) => return e,
            };
        let width = match self.offset_size(
            args.get(4),
            reference.right.column - reference.left.column,
            cell,
        ) {
            Ok(w) => w,
            Err(e) => return e,
        };
        let (Some(height), Some(width)) = (height, width) else {
            // A size of exactly 0 is #REF!, not an empty area.
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        };
        // A negative size extends the area up/left from the anchor cell.
        let column_end = column_start.saturating_add(width);
        let row_end = row_start.saturating_add(height);
        let (row_start, row_end) = (row_start.min(row_end), row_start.max(row_end));
        let (column_start, column_end) =
            (column_start.min(column_end), column_start.max(column_end));
        if row_start < 1 || row_end > LAST_ROW || column_start < 1 || column_end > LAST_COLUMN {
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        }
        let left = CellReferenceIndex {
            sheet: reference.left.sheet,
            row: row_start,
            column: column_start,
        };
        let right = CellReferenceIndex {
            sheet: reference.right.sheet,
            row: row_end,
            column: column_end,
        };
        CalcResult::Range { left, right }
    }

    // FORMULATEXT(reference)
    // Returns a formula as a string. Two differences with Excel:
    // - It returns the formula in English
    // - It formats the formula without spaces between elements
    pub(crate) fn fn_formulatext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if let CalcResult::Range { left, right } = self.evaluate_node_with_reference(&args[0], cell)
        {
            if left.sheet != right.sheet {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "3D ranges not supported".to_string(),
                };
            }
            if left.row != right.row || left.column != right.column {
                // FIXME: Implicit intersection or dynamic arrays
                return CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "argument must be a reference to a single cell".to_string(),
                };
            }
            if let Ok(Some(f)) = self.get_english_cell_formula(left.sheet, left.row, left.column) {
                CalcResult::String(f)
            } else {
                CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Reference does not have a formula".to_string(),
                }
            }
        } else {
            CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Argument must be a reference".to_string(),
            }
        }
    }
}
