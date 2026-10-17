use std::cmp::Ordering;

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::{
    binary_search::{binary_search, Sequence},
    lift::{LiftCost, LiftOptions, LiftedArg},
    lookup_and_reference::lookup_table::SearchVector,
    lookup_and_reference::xmatch::{linear_scan, MatchMode as XmatchMode},
    util::array_node_to_calc_result,
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

/// The first match of `lookup_value` in `values` walking in `search_mode`'s direction. Only the
/// candidates `SearchVector` names are tested: its virtual empty tail is one cell.
fn linear_search(
    lookup_value: &CalcResult,
    values: &SearchVector,
    search_mode: &SearchMode,
    match_mode: &MatchMode,
) -> Option<usize> {
    let mode = match match_mode {
        MatchMode::ExactMatch => XmatchMode::Exact,
        MatchMode::ExactMatchSmaller => XmatchMode::ExactMatchSmaller,
        MatchMode::ExactMatchLarger => XmatchMode::ExactMatchLarger,
        MatchMode::WildcardMatch => XmatchMode::Wildcard,
    };
    let candidates = values.linear_candidates(*search_mode == SearchMode::StartAtFirstItem);
    linear_scan(lookup_value, candidates, |index| values.get(index), mode)
}

/// Where a binary-search XLOOKUP lands in `values`.
enum BinaryHit {
    Found(usize),
    NotFound,
    /// Wildcard match mode with a binary search, which is `#VALUE!`.
    Wildcard,
}

/// The binary-search XLOOKUP modes (2 ascending, -2 descending) over `values`: the position of
/// the match, or of the next smaller/larger item for match modes -1/1. Exact match mode only
/// accepts an equal item.
fn binary_hit<S: Sequence<CalcResult> + ?Sized>(
    lookup_value: &CalcResult,
    values: &S,
    search_mode: &SearchMode,
    match_mode: &MatchMode,
) -> BinaryHit {
    if *match_mode == MatchMode::WildcardMatch {
        return BinaryHit::Wildcard;
    }
    let side = match match_mode {
        MatchMode::ExactMatchSmaller => Ordering::Less,
        MatchMode::ExactMatchLarger => Ordering::Greater,
        _ => Ordering::Equal,
    };
    match binary_search(
        lookup_value,
        values,
        *search_mode == SearchMode::BinarySearchDescending,
        side,
    ) {
        Some(index) => BinaryHit::Found(index),
        None => BinaryHit::NotFound,
    }
}

/// XLOOKUP's `lookup_array` and `return_array`, evaluated and shape-checked once, so that a lifted
/// `lookup_value` runs only the search and the pick for each element (T-1264).
struct XlookupTable {
    /// The lookup vector. A range is evaluated only up to the sheet's used area.
    values: SearchVector,
    /// The lookup vector runs down a column.
    vertical: bool,
    returns: XlookupReturns,
}

/// An XLOOKUP `return_array`.
enum XlookupReturns {
    Range(CellReferenceIndex, CellReferenceIndex),
    Array(Vec<Vec<ArrayNode>>),
}

/// What one XLOOKUP search returns for its match.
#[derive(Clone, Copy)]
enum XlookupPick {
    /// The whole matched row (column lookup vector) or column (row lookup vector) of the return
    /// array, as a reference when `as_reference`.
    Whole { as_reference: bool },
    /// The first cell of the match only, as a value: the result of each element of a lifted
    /// `lookup_value`. `XLOOKUP({2;3},A1:A4,C1:E4)` is 12; 13 in one column
    /// (probes/xlookup-array-lookup-value A15, probes/xlookup-array-lookup-value-args A1, A33).
    FirstCell,
}

/// XLOOKUP's `match_mode` and `search_mode` from their values.
fn xlookup_modes(
    model: &mut Model,
    match_mode: &CalcResult,
    search_mode: &CalcResult,
    cell: CellReferenceIndex,
) -> Result<(MatchMode, SearchMode), CalcResult> {
    let unexpected = || CalcResult::new_error(Error::VALUE, cell, "Unexpected number".to_string());
    let match_mode = match model.cast_to_number(match_mode.clone(), cell)?.floor() as i32 {
        -1 => MatchMode::ExactMatchSmaller,
        1 => MatchMode::ExactMatchLarger,
        0 => MatchMode::ExactMatch,
        2 => MatchMode::WildcardMatch,
        _ => return Err(unexpected()),
    };
    // An unknown search_mode is #VALUE! (probes/xlookup-array-lookup-value-args A9, A67).
    let search_mode = match model.cast_to_number(search_mode.clone(), cell)?.floor() as i32 {
        1 => SearchMode::StartAtFirstItem,
        -1 => SearchMode::StartAtLastItem,
        -2 => SearchMode::BinarySearchDescending,
        2 => SearchMode::BinarySearchAscending,
        _ => return Err(unexpected()),
    };
    Ok((match_mode, search_mode))
}

/// The XLOOKUP result at `index` of an array `return_array`: the element, or, for the whole
/// pick, the matched row of a wider array (column lookup vector) as a 1xN array or the matched
/// column of a taller array (row lookup vector) as an Nx1 array, as for ranges
/// (probes/xlookup-row-column-picks E13, E28, E31).
fn xlookup_pick_array(
    returns: &[Vec<ArrayNode>],
    vertical: bool,
    index: usize,
    pick: XlookupPick,
    cell: CellReferenceIndex,
) -> CalcResult {
    let out_of_range =
        || CalcResult::new_error(Error::VALUE, cell, "Index out of range".to_string());
    let whole = matches!(pick, XlookupPick::Whole { .. });
    let node = if vertical {
        match returns.get(index) {
            Some(row) if row.len() > 1 && whole => return CalcResult::Array(vec![row.clone()]),
            Some(row) => row.first(),
            None => None,
        }
    } else if returns.len() > 1 && whole {
        let column: Option<Vec<Vec<ArrayNode>>> = returns
            .iter()
            .map(|row| row.get(index).map(|node| vec![node.clone()]))
            .collect();
        return column.map_or_else(out_of_range, CalcResult::Array);
    } else {
        returns.first().and_then(|row| row.get(index))
    };
    node.map_or_else(out_of_range, |node| array_node_to_calc_result(node, cell))
}

/// Rows and columns of a range or array result (1x1 for anything else).
fn shape(result: &CalcResult) -> (usize, usize) {
    match result {
        CalcResult::Range { left, right } => (
            usize::try_from(right.row - left.row + 1).unwrap_or(0),
            usize::try_from(right.column - left.column + 1).unwrap_or(0),
        ),
        CalcResult::Array(array) => (array.len(), array.first().map_or(0, Vec::len)),
        _ => (1, 1),
    }
}

/// Whether an XLOOKUP lookup vector of `rows` x `columns` runs down a column, given the return
/// array's row count: a column is vertical and a row is not; a single cell is vertical when the
/// return has one row (`XLOOKUP(1,A1:A1,B1:C1)` is the row) and a row otherwise
/// (`XLOOKUP("a",F1:F1,F2:F4)` is the column; probes/xlookup-row-column-picks E9, E23). `None`
/// for a 2-D lookup array, which is `#VALUE!` (probes/xlookup-array-lookup-value-args A68).
fn xlookup_orientation(rows: usize, columns: usize, return_rows: usize) -> Option<bool> {
    match (rows, columns) {
        (1, 1) => Some(return_rows == 1),
        (_, 1) => Some(true),
        (1, _) => Some(false),
        _ => None,
    }
}

impl Model {
    /// The XLOOKUP result at (`row_offset`, `column_offset`) of the return range. A return range
    /// wider than a column lookup vector yields the whole matched row as a 1xN array, and one
    /// taller than a row lookup vector the whole matched column as an Nx1 array
    /// (probes/xlookup-array-lookup-value-args C33); otherwise the result is the matched cell.
    fn xlookup_pick(
        &mut self,
        (result_left, result_right): (CellReferenceIndex, CellReferenceIndex),
        column_lookup: bool,
        (row_offset, column_offset): (i32, i32),
        pick: XlookupPick,
    ) -> CalcResult {
        let first = CellReferenceIndex {
            sheet: result_left.sheet,
            row: result_left.row + row_offset,
            column: result_left.column + column_offset,
        };
        let last = if column_lookup {
            CellReferenceIndex {
                column: result_right.column,
                ..first
            }
        } else {
            CellReferenceIndex {
                row: result_right.row,
                ..first
            }
        };
        let as_reference = match pick {
            XlookupPick::Whole { as_reference } => as_reference,
            XlookupPick::FirstCell => return self.evaluate_cell(first),
        };
        if as_reference {
            return CalcResult::Range {
                left: first,
                right: last,
            };
        }
        if first == last {
            return self.evaluate_cell(first);
        }
        let mut rows = Vec::new();
        for row in first.row..=last.row {
            let values = (first.column..=last.column)
                .map(|column| {
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: first.sheet,
                        row,
                        column,
                    });
                    crate::cast::calc_result_to_array_node(value)
                })
                .collect();
            rows.push(values);
        }
        CalcResult::Array(rows)
    }

    /// XLOOKUP's `lookup_array` and `return_array`. The lookup array, a range or an array, must be a
    /// row or a column ([`xlookup_orientation`]). The return array, a range or an array, needs
    /// the lookup column's row count (any width: the matched row is returned) or the lookup
    /// row's column count (any height: the matched column); otherwise it is `#VALUE!`
    /// (poi/xlookup.xlsx Sheet1!C2; probes/xlookup-array-return E1-E24,
    /// probes/xlookup-row-column-picks; docs/excel-semantics/lookup.md "Array `lookup_array`").
    fn xlookup_table(
        &mut self,
        lookup_node: &Node,
        return_node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<XlookupTable, CalcResult> {
        let value_error =
            |message: &str| CalcResult::new_error(Error::VALUE, cell, message.to_string());
        // A single-cell reference is a one-cell vector (probes/text-collation-longer A102).
        let lookup = match self.evaluate_node_with_reference(lookup_node, cell) {
            lookup @ (CalcResult::Range { .. } | CalcResult::Array(_)) => lookup,
            error @ CalcResult::Error { .. } => return Err(error),
            _ => {
                return Err(CalcResult::new_error(
                    Error::NA,
                    cell,
                    "Range expected".to_string(),
                ))
            }
        };
        let (rows, columns) = shape(&lookup);
        if xlookup_orientation(rows, columns, 1).is_none() {
            return Err(value_error("Second argument must be a vector"));
        }
        let returned = self.evaluate_node_with_reference(return_node, cell);
        let (return_rows, return_columns) = shape(&returned);
        let returns = match returned {
            CalcResult::Range { left, right } => XlookupReturns::Range(left, right),
            CalcResult::Array(returns) => XlookupReturns::Array(returns),
            error @ CalcResult::Error { .. } => return Err(error),
            _ => return Err(value_error("Range expected")),
        };
        let vertical = xlookup_orientation(rows, columns, return_rows) == Some(true);
        if (vertical && return_rows != rows) || (!vertical && return_columns != columns) {
            return Err(value_error("Arrays must be of the same size"));
        }
        let values = match lookup {
            // Both searches walk the whole vector, blanks past the used area included (they
            // are probed like any cell; T-1019), but only the used area is evaluated.
            CalcResult::Range { left, right } => self.range_search_vector(&left, &right, vertical),
            CalcResult::Array(array) => SearchVector::new(
                array
                    .iter()
                    .flatten()
                    .map(|node| array_node_to_calc_result(node, cell))
                    .collect(),
            ),
            _ => return Err(value_error("Range expected")),
        };
        Ok(XlookupTable {
            values,
            vertical,
            returns,
        })
    }

    /// One XLOOKUP search: `values` are the lookup value, `match_mode` and `search_mode`. A lookup
    /// value error comes first, then a mode error, then an error of the table; `if_not_found`
    /// only when nothing matches (probes/xlookup-array-lookup-value-args A18, A61, A64).
    fn xlookup_one(
        &mut self,
        values: &[CalcResult],
        table: &Result<XlookupTable, CalcResult>,
        if_not_found: &CalcResult,
        pick: XlookupPick,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let lookup_value = &values[0];
        if lookup_value.is_error() {
            return lookup_value.clone();
        }
        let (match_mode, search_mode) = match xlookup_modes(self, &values[1], &values[2], cell) {
            Ok(modes) => modes,
            Err(error) => return error,
        };
        let table = match table {
            Ok(table) => table,
            Err(error) => return error.clone(),
        };
        let found = match search_mode {
            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                linear_search(lookup_value, &table.values, &search_mode, &match_mode)
            }
            SearchMode::BinarySearchAscending | SearchMode::BinarySearchDescending => {
                match binary_hit(lookup_value, &table.values, &search_mode, &match_mode) {
                    BinaryHit::Found(index) => Some(index),
                    BinaryHit::NotFound => None,
                    BinaryHit::Wildcard => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Cannot use wildcard in binary search".to_string(),
                        )
                    }
                }
            }
        };
        let Some(index) = found else {
            return if_not_found.clone();
        };
        match &table.returns {
            XlookupReturns::Range(left, right) => {
                let offset = i32::try_from(index).unwrap_or(i32::MAX);
                let offsets = if table.vertical {
                    (offset, 0)
                } else {
                    (0, offset)
                };
                self.xlookup_pick((*left, *right), table.vertical, offsets, pick)
            }
            XlookupReturns::Array(returns) => {
                xlookup_pick_array(returns, table.vertical, index, pick, cell)
            }
        }
    }

    /// The XLOOKUP function searches a range or an array, and then returns the item corresponding
    /// to the first match it finds. If no match exists, then XLOOKUP can return the closest (approximate) match.
    /// =`XLOOKUP(lookup_value`, `lookup_array`, `return_array`, [`if_not_found`], [`match_mode`], [`search_mode`])
    ///
    /// `lookup_array` and `return_array` must be column or row arrays and of the same dimension.
    /// Otherwise #VALUE! is returned
    /// [`if_not_found`]
    /// Where a valid match is not found, return the [`if_not_found`] text you supply.
    /// If a valid match is not found, and [`if_not_found`] is missing, #N/A is returned.
    ///
    /// [`match_mode`]
    /// Specify the match type:
    ///   *  0 - Exact match. If none found, return #N/A. This is the default.
    ///   * -1 - Exact match. If none found, return the next smaller item.
    ///   *  1 - Exact match. If none found, return the next larger item.
    ///   *  2 - A wildcard match where *, ?, and ~ have special meaning.
    ///
    /// [`search_mode`]
    /// Specify the search mode to use:
    ///   *  1 - Perform a search starting at the first item. This is the default.
    ///   * -1 - Perform a reverse search starting at the last item.
    ///   *  2 - Perform a binary search that relies on `lookup_array` being sorted
    ///      in ascending order. If not sorted, invalid results will be returned.
    ///   * -2 - Perform a binary search that relies on `lookup_array` being sorted
    ///     in descending order. If not sorted, invalid results will be returned.
    pub(crate) fn fn_xlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.xlookup_impl(args, cell, false)
    }

    /// XLOOKUP; `as_reference` gives the picked cells of a return range as a reference, as an
    /// intersection operand sees them: `=A1:A3 XLOOKUP(2,A1:A3,A1:B3)` is 2
    /// (probes/intersection-function-operands H8; T-0165).
    ///
    /// An array or multi-cell `lookup_value`, `match_mode` or `search_mode` lifts: the result has
    /// their broadcast shape, each element looked up by the scalar rules, an error argument
    /// giving that error per element (probes/let-name-uncalled-picked-refs H21,
    /// probes/xlookup-array-lookup-value, probes/xlookup-array-lookup-value-args; T-1264). The
    /// table is evaluated once and each element costs one search of the lookup vector, bounded
    /// by the shared cell-visit cap (DEVIATION, docs/excel-semantics/lookup.md).
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
        let match_mode = match args.get(4) {
            Some(node) => self.evaluate_node_in_context(node, cell),
            None => CalcResult::Number(0.0),
        };
        let search_mode = match args.get(5) {
            Some(node) => self.evaluate_node_in_context(node, cell),
            None => CalcResult::Number(1.0),
        };
        let criteria = vec![lookup_value, match_mode, search_mode];
        let lifts = criteria.iter().any(LiftedArg::is_grid);
        let if_not_found = self.xlookup_if_not_found(args.get(3), lifts, as_reference, cell);
        let table = self.xlookup_table(&args[1], &args[2], cell);
        // A lifted element is a value, never a reference, and keeps only the first cell of its
        // match.
        let pick = if lifts {
            XlookupPick::FirstCell
        } else {
            XlookupPick::Whole { as_reference }
        };
        // Each element searches the evaluated cells of the lookup vector once, whatever the
        // three criteria are; the table is never walked again.
        let cost = table.as_ref().map_or(1, |table| {
            u64::try_from(table.values.search_cost())
                .unwrap_or(u64::MAX)
                .max(1)
        });
        self.lift_values(
            criteria,
            LiftCost::Cells(cost),
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |model, values| model.xlookup_one(values, &table, &if_not_found, pick, cell),
        )
    }

    /// XLOOKUP's `if_not_found` (`#N/A` when omitted or empty). A range stays a reference in the
    /// reference form: `ISREF(XLOOKUP(9,A1:A3,B1:B3,C1))` is TRUE
    /// (probes/intersection-lookup-operand-errors H10). Under a lifted `lookup_value` an array
    /// gives its first element and a reference, even a single cell, `#VALUE!`
    /// (probes/xlookup-array-lookup-value-args A27-A31, A58).
    fn xlookup_if_not_found(
        &mut self,
        node: Option<&Node>,
        lifts: bool,
        as_reference: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let value = match node {
            Some(node) if lifts || as_reference => self.evaluate_node_with_reference(node, cell),
            Some(node) => self.evaluate_node_in_context(node, cell),
            None => CalcResult::EmptyArg,
        };
        match value {
            CalcResult::EmptyArg => CalcResult::new_error(Error::NA, cell, "Not found".to_string()),
            CalcResult::Range { .. } if lifts => CalcResult::new_error(
                Error::VALUE,
                cell,
                "Reference if_not_found with an array lookup_value".to_string(),
            ),
            CalcResult::Array(rows) if lifts => {
                rows.first().and_then(|row| row.first()).map_or_else(
                    || CalcResult::new_error(Error::VALUE, cell, "Empty array".to_string()),
                    |node| array_node_to_calc_result(node, cell),
                )
            }
            other => other,
        }
    }
}
