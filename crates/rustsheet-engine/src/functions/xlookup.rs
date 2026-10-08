use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::{
    binary_search::{
        binary_search_descending_or_greater, binary_search_descending_or_smaller,
        binary_search_or_greater, binary_search_or_smaller,
    },
    lookup_and_reference::xmatch::{linear_scan, MatchMode as XmatchMode},
    util::{array_node_to_calc_result, compare_values},
};

#[derive(PartialEq)]
enum SearchMode {
    StartAtFirstItem = 1,
    StartAtLastItem = -1,
    BinarySearchDescending = -2,
    BinarySearchAscending = 2,
}

#[derive(PartialEq)]
enum MatchMode {
    ExactMatchSmaller = -1,
    ExactMatch = 0,
    ExactMatchLarger = 1,
    WildcardMatch = 2,
}

// lookup_value in array, match_mode search_mode
fn linear_search(
    lookup_value: &CalcResult,
    array: &[CalcResult],
    search_mode: SearchMode,
    match_mode: MatchMode,
) -> Option<usize> {
    let length = array.len();
    let mode = match match_mode {
        MatchMode::ExactMatch => XmatchMode::Exact,
        MatchMode::ExactMatchSmaller => XmatchMode::ExactMatchSmaller,
        MatchMode::ExactMatchLarger => XmatchMode::ExactMatchLarger,
        MatchMode::WildcardMatch => XmatchMode::Wildcard,
    };
    let get = |index: usize| &array[index];
    if search_mode == SearchMode::StartAtFirstItem {
        linear_scan(lookup_value, 0..length, get, mode)
    } else {
        linear_scan(lookup_value, (0..length).rev(), get, mode)
    }
}

impl Model {
    /// The XLOOKUP result at (`row_offset`, `column_offset`) of the return range. A multi-column
    /// return range (only accepted for a column lookup vector) yields the whole matched row as a
    /// 1xN array; a single column keeps the scalar result.
    fn xlookup_pick(
        &mut self,
        result_left: CellReferenceIndex,
        result_right: CellReferenceIndex,
        column_lookup: bool,
        row_offset: i32,
        column_offset: i32,
        as_reference: bool,
    ) -> CalcResult {
        let row = result_left.row + row_offset;
        if !column_lookup || result_right.column == result_left.column {
            let picked = CellReferenceIndex {
                sheet: result_left.sheet,
                row,
                column: result_left.column + column_offset,
            };
            if as_reference {
                return CalcResult::Range {
                    left: picked,
                    right: picked,
                };
            }
            return self.evaluate_cell(picked);
        }
        if as_reference {
            return CalcResult::Range {
                left: CellReferenceIndex { row, ..result_left },
                right: CellReferenceIndex {
                    row,
                    ..result_right
                },
            };
        }
        let values = (result_left.column..=result_right.column)
            .map(|column| {
                let value = self.evaluate_cell(CellReferenceIndex {
                    sheet: result_left.sheet,
                    row,
                    column,
                });
                crate::cast::calc_result_to_array_node(value)
            })
            .collect();
        CalcResult::Array(vec![values])
    }

    /// XLOOKUP over an array `lookup_array` (`XLOOKUP(2,{1;2;3},B1:B3)`, probes/isref-reference-lookups
    /// H10) with a range `return_array`, linear search modes only: the match position picks the
    /// return range's cell, as a reference when `as_reference`. A non-range return_array is `#N/A`.
    fn xlookup_array_lookup(
        &mut self,
        lookup_value: &CalcResult,
        array: &[Vec<ArrayNode>],
        return_node: &Node,
        (search_mode, match_mode, if_not_found): (SearchMode, MatchMode, CalcResult),
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        let rows = array.len();
        let columns = array.first().map_or(0, Vec::len);
        let vertical = columns == 1;
        if rows != 1 && !vertical {
            return CalcResult::new_error(
                Error::ERROR,
                cell,
                "Second argument must be a vector".to_string(),
            );
        }
        let values: Vec<CalcResult> = array
            .iter()
            .flatten()
            .map(|node| array_node_to_calc_result(node, cell))
            .collect();
        let CalcResult::Range {
            left: result_left,
            right: result_right,
        } = self.evaluate_node_in_context(return_node, cell)
        else {
            return CalcResult::new_error(Error::NA, cell, "Range expected".to_string());
        };
        match linear_search(lookup_value, &values, search_mode, match_mode) {
            Some(index) => {
                let (row_offset, column_offset) = if vertical {
                    (index as i32, 0)
                } else {
                    (0, index as i32)
                };
                self.xlookup_pick(
                    result_left,
                    result_right,
                    vertical,
                    row_offset,
                    column_offset,
                    as_reference,
                )
            }
            None => if_not_found,
        }
    }

    /// The XLOOKUP function searches a range or an array, and then returns the item corresponding
    /// to the first match it finds. If no match exists, then XLOOKUP can return the closest (approximate) match.
    /// =XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])
    ///
    /// lookup_array and return_array must be column or row arrays and of the same dimension.
    /// Otherwise #VALUE! is returned
    /// [if_not_found]
    /// Where a valid match is not found, return the [if_not_found] text you supply.
    /// If a valid match is not found, and [if_not_found] is missing, #N/A is returned.
    ///
    /// [match_mode]
    /// Specify the match type:
    ///   *  0 - Exact match. If none found, return #N/A. This is the default.
    ///   * -1 - Exact match. If none found, return the next smaller item.
    ///   *  1 - Exact match. If none found, return the next larger item.
    ///   *  2 - A wildcard match where *, ?, and ~ have special meaning.
    ///
    /// [search_mode]
    /// Specify the search mode to use:
    ///   *  1 - Perform a search starting at the first item. This is the default.
    ///   * -1 - Perform a reverse search starting at the last item.
    ///   *  2 - Perform a binary search that relies on lookup_array being sorted
    ///      in ascending order. If not sorted, invalid results will be returned.
    ///   * -2 - Perform a binary search that relies on lookup_array being sorted
    ///     in descending order. If not sorted, invalid results will be returned.
    pub(crate) fn fn_xlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.xlookup_impl(args, cell, false)
    }

    /// XLOOKUP; `as_reference` gives the picked cells of a return range as a reference, as an
    /// intersection operand sees them: `=A1:A3 XLOOKUP(2,A1:A3,A1:B3)` is 2
    /// (probes/intersection-function-operands H8; T-0165).
    pub(crate) fn xlookup_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if args.len() < 3 || args.len() > 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        // Get optional arguments
        let if_not_found = if args.len() >= 4 {
            // A range `if_not_found` stays a reference in the reference form:
            // `ISREF(XLOOKUP(9,A1:A3,B1:B3,C1))` is TRUE (probes/intersection-lookup-operand-errors H10).
            let v = if as_reference {
                self.evaluate_node_with_reference(&args[3], cell)
            } else {
                self.evaluate_node_in_context(&args[3], cell)
            };
            match v {
                CalcResult::EmptyArg => CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Not found".to_string(),
                },
                _ => v,
            }
        } else {
            // default
            CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not found".to_string(),
            }
        };
        let match_mode = if args.len() >= 5 {
            match self.get_number(&args[4], cell) {
                Ok(c) => match c.floor() as i32 {
                    -1 => MatchMode::ExactMatchSmaller,
                    1 => MatchMode::ExactMatchLarger,
                    0 => MatchMode::ExactMatch,
                    2 => MatchMode::WildcardMatch,
                    _ => {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Unexpected number".to_string(),
                        };
                    }
                },
                Err(s) => return s,
            }
        } else {
            // default
            MatchMode::ExactMatch
        };
        let search_mode = if args.len() == 6 {
            match self.get_number(&args[5], cell) {
                Ok(c) => match c.floor() as i32 {
                    1 => SearchMode::StartAtFirstItem,
                    -1 => SearchMode::StartAtLastItem,
                    -2 => SearchMode::BinarySearchDescending,
                    2 => SearchMode::BinarySearchAscending,
                    _ => {
                        return CalcResult::Error {
                            error: Error::ERROR,
                            origin: cell,
                            message: "Unexpected number".to_string(),
                        };
                    }
                },
                Err(s) => return s,
            }
        } else {
            // default
            SearchMode::StartAtFirstItem
        };
        // lookup_array
        match self.evaluate_node_in_context(&args[1], cell) {
            CalcResult::Range { left, right } => {
                let is_row_vector = if left.row == right.row {
                    false
                } else if left.column == right.column {
                    true
                } else {
                    // second argument must be a vector
                    return CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    };
                };
                // return array
                match self.evaluate_node_in_context(&args[2], cell) {
                    CalcResult::Range {
                        left: result_left,
                        right: result_right,
                    } => {
                        // A column lookup vector accepts a return range with any number of columns
                        // (poi/xlookup.xlsx Sheet1!C2); a row vector keeps the same-shape rule.
                        let wide_columns_ok = is_row_vector;
                        if result_right.row - result_left.row != right.row - left.row
                            || (!wide_columns_ok
                                && result_right.column - result_left.column
                                    != right.column - left.column)
                        {
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Arrays must be of the same size".to_string(),
                            };
                        }
                        let mut row2 = right.row;
                        let row1 = left.row;
                        let mut column2 = right.column;
                        let column1 = left.column;

                        match self.clip_to_used_area(left.sheet, row1, column1, row2, column2) {
                            Ok((r, c)) => {
                                row2 = r;
                                column2 = c;
                            }
                            Err(message) => {
                                return CalcResult::new_error(Error::ERROR, cell, message)
                            }
                        }
                        let left = CellReferenceIndex {
                            sheet: left.sheet,
                            column: column1,
                            row: row1,
                        };
                        let right = CellReferenceIndex {
                            sheet: left.sheet,
                            column: column2,
                            row: row2,
                        };
                        match search_mode {
                            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                                let array = &self.prepare_array(&left, &right, is_row_vector);
                                match linear_search(&lookup_value, array, search_mode, match_mode) {
                                    Some(index) => {
                                        let row_index =
                                            if is_row_vector { index as i32 } else { 0 };
                                        let column_index =
                                            if is_row_vector { 0 } else { index as i32 };
                                        self.xlookup_pick(
                                            result_left,
                                            result_right,
                                            is_row_vector,
                                            row_index,
                                            column_index,
                                            as_reference,
                                        )
                                    }
                                    None => if_not_found,
                                }
                            }
                            SearchMode::BinarySearchAscending
                            | SearchMode::BinarySearchDescending => {
                                let index = if match_mode == MatchMode::ExactMatchLarger {
                                    if search_mode == SearchMode::BinarySearchAscending {
                                        binary_search_or_greater(
                                            &lookup_value,
                                            &self.prepare_array(&left, &right, is_row_vector),
                                        )
                                    } else {
                                        binary_search_descending_or_greater(
                                            &lookup_value,
                                            &self.prepare_array(&left, &right, is_row_vector),
                                        )
                                    }
                                } else if search_mode == SearchMode::BinarySearchAscending {
                                    binary_search_or_smaller(
                                        &lookup_value,
                                        &self.prepare_array(&left, &right, is_row_vector),
                                    )
                                } else {
                                    binary_search_descending_or_smaller(
                                        &lookup_value,
                                        &self.prepare_array(&left, &right, is_row_vector),
                                    )
                                };
                                match index {
                                    None => if_not_found,
                                    Some(l) => {
                                        let row =
                                            result_left.row + if is_row_vector { l } else { 0 };
                                        let column =
                                            result_left.column + if is_row_vector { 0 } else { l };
                                        if match_mode == MatchMode::ExactMatch {
                                            let value = self.evaluate_cell(CellReferenceIndex {
                                                sheet: left.sheet,
                                                row: left.row + if is_row_vector { l } else { 0 },
                                                column: left.column
                                                    + if is_row_vector { 0 } else { l },
                                            });
                                            if compare_values(&value, &lookup_value) == 0 {
                                                self.xlookup_pick(
                                                    result_left,
                                                    result_right,
                                                    is_row_vector,
                                                    row - result_left.row,
                                                    column - result_left.column,
                                                    as_reference,
                                                )
                                            } else {
                                                if_not_found
                                            }
                                        } else if match_mode == MatchMode::ExactMatchSmaller
                                            || match_mode == MatchMode::ExactMatchLarger
                                        {
                                            self.xlookup_pick(
                                                result_left,
                                                result_right,
                                                is_row_vector,
                                                row - result_left.row,
                                                column - result_left.column,
                                                as_reference,
                                            )
                                        } else {
                                            CalcResult::Error {
                                                error: Error::VALUE,
                                                origin: cell,
                                                message: "Cannot use wildcard in binary search"
                                                    .to_string(),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    error @ CalcResult::Error { .. } => error,
                    _ => CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Range expected".to_string(),
                    },
                }
            }
            CalcResult::Array(array)
                if matches!(
                    search_mode,
                    SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem
                ) =>
            {
                self.xlookup_array_lookup(
                    &lookup_value,
                    &array,
                    &args[2],
                    (search_mode, match_mode, if_not_found),
                    cell,
                    as_reference,
                )
            }
            error @ CalcResult::Error { .. } => error,
            _ => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Range expected".to_string(),
            },
        }
    }
}
