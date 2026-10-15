#[cfg(not(target_arch = "wasm32"))]
use regex::Regex;
#[cfg(target_arch = "wasm32")]
use regex_lite::Regex;

use crate::expressions::types::CellReferenceIndex;
use crate::number_format::number_to_text;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::lookup_table::SearchVector;
use crate::functions::{
    binary_search::binary_search, criteria::wildcard_text_matcher, util::compare_values,
};

#[derive(PartialEq)]
enum SearchMode {
    FirstToLast = 1,
    LastToFirst = -1,
    BinaryAscending = 2,
    BinaryDescending = -2,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum MatchMode {
    ExactMatchSmaller = -1,
    Exact = 0,
    ExactMatchLarger = 1,
    Wildcard = 2,
    Regex = 3,
}

impl MatchMode {
    /// The side a binary search may settle on when nothing is equal (`binary_search`).
    fn ordering(self) -> std::cmp::Ordering {
        match self {
            MatchMode::ExactMatchSmaller => std::cmp::Ordering::Less,
            MatchMode::ExactMatchLarger => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

fn is_blank(v: &CalcResult) -> bool {
    // An omitted lookup_value is a blank in every match and search mode
    // (probes/xlookup-omitted-lookup-value, docs/excel-semantics/lookup.md, T-1188).
    matches!(v, CalcResult::EmptyCell | CalcResult::EmptyArg)
}

/// Orders a candidate against the lookup value in an ordered match mode. A blank is larger than
/// every number, text and boolean, and equal only to another blank (probes/xmatch-xlookup-blanks
/// Col!M5-M12, T-1018; probes/xmatch-text-bool-equal-runs Txt!M7, M16, Blk!M6-M9, T-1153).
/// Unlike `compare_values`, which reads a blank as 0, `""` or FALSE.
fn compare_blank_last(candidate: &CalcResult, lookup_value: &CalcResult) -> i32 {
    match (is_blank(candidate), is_blank(lookup_value)) {
        (true, true) => 0,
        (true, false) => 1,
        (false, true) => -1,
        (false, false) => compare_values(candidate, lookup_value),
    }
}

/// The first (in `indices` order) candidate of `get` that matches `lookup_value` under
/// `match_mode`, shared by XMATCH and XLOOKUP. A blank candidate never matches a non-blank
/// lookup value in an exact or wildcard search, and matches a blank one (Col!M14, M30-M34), which
/// in turn matches nothing but a blank, not a 0, `""` or FALSE (Blk!M1-M5, M19-M22);
/// the regex mode is XMATCH's own and matches nothing here.
pub(crate) fn linear_scan<'a>(
    lookup_value: &CalcResult,
    indices: impl IntoIterator<Item = usize>,
    get: impl Fn(usize) -> &'a CalcResult,
    match_mode: MatchMode,
) -> Option<usize> {
    let blank_lookup = is_blank(lookup_value);
    match match_mode {
        MatchMode::Exact => indices.into_iter().find(|&idx| {
            let v = get(idx);
            if is_blank(v) || blank_lookup {
                is_blank(v) && blank_lookup
            } else {
                compare_values(v, lookup_value) == 0
            }
        }),
        MatchMode::ExactMatchSmaller | MatchMode::ExactMatchLarger => {
            let m = match_mode as i32;
            let mut best_idx = None;
            let mut best_val: Option<&CalcResult> = None;
            for idx in indices {
                let v = get(idx);
                let c = compare_blank_last(v, lookup_value);
                if c == 0 {
                    return Some(idx);
                } else if c == m && best_val.is_none_or(|p| compare_blank_last(p, v) == m) {
                    best_val = Some(v);
                    best_idx = Some(idx);
                }
            }
            best_idx
        }
        MatchMode::Wildcard => {
            let matches: Box<dyn Fn(&CalcResult) -> bool> =
                if let CalcResult::String(s) = lookup_value {
                    wildcard_text_matcher(s)
                } else {
                    Box::new(move |x| compare_values(x, lookup_value) == 0)
                };
            indices.into_iter().find(|&idx| {
                let v = get(idx);
                if is_blank(v) || blank_lookup {
                    is_blank(v) && blank_lookup
                } else {
                    matches(v)
                }
            })
        }
        MatchMode::Regex => None,
    }
}

/// Searches `array` for `lookup_value` and returns the 0-based index of the match,
/// or `None` if not found. Only the candidates `SearchVector` names are tested:
/// they cover the virtual empty tail with a single cell.
fn linear_search(
    lookup_value: &CalcResult,
    array: &SearchVector,
    search_mode: &SearchMode,
    match_mode: MatchMode,
) -> Option<usize> {
    let candidates = array.linear_candidates(*search_mode == SearchMode::FirstToLast);

    match match_mode {
        MatchMode::Regex => {
            let pattern = match lookup_value {
                CalcResult::String(s) => &**s,
                _ => return None,
            };
            let re = Regex::new(pattern).ok()?;
            candidates.into_iter().find(|&idx| {
                let text = match array.get(idx) {
                    CalcResult::String(s) => s.to_string(),
                    CalcResult::Number(n) => number_to_text(*n),
                    CalcResult::Boolean(b) => b.to_string(),
                    _ => return false,
                };
                re.is_match(&text)
            })
        }
        // Regex is handled above; the other modes share XLOOKUP's blank-aware scan.
        _ => linear_scan(lookup_value, candidates, |idx| array.get(idx), match_mode),
    }
}

impl Model {
    /// `=XMATCH(lookup_value, lookup_array, [match_mode], [search_mode])`
    ///
    /// Returns the relative position (1-based) of an item in a row or column array.
    ///
    /// `match_mode`:
    ///   *  0 – exact match (default)
    ///   * -1 – exact match or next smaller
    ///   *  1 – exact match or next larger
    ///   *  2 – wildcard match (* ? ~)
    ///
    /// `search_mode`:
    ///   *  1 – first to last (default)
    ///   * -1 – last to first
    ///   *  2 – binary search ascending
    ///   * -2 – binary search descending
    pub(crate) fn fn_xmatch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let lookup_value = self.lookup_value_arg(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }

        let match_mode = if args.len() >= 3 {
            match self.get_number(&args[2], cell) {
                Ok(n) => match n.floor() as i32 {
                    -1 => MatchMode::ExactMatchSmaller,
                    0 => MatchMode::Exact,
                    1 => MatchMode::ExactMatchLarger,
                    2 => MatchMode::Wildcard,
                    3 => MatchMode::Regex,
                    _ => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Invalid match_mode".to_string(),
                        )
                    }
                },
                Err(e) => return e,
            }
        } else {
            MatchMode::Exact
        };

        let search_mode = if args.len() == 4 {
            match self.get_number(&args[3], cell) {
                Ok(n) => match n.floor() as i32 {
                    1 => SearchMode::FirstToLast,
                    -1 => SearchMode::LastToFirst,
                    2 => SearchMode::BinaryAscending,
                    -2 => SearchMode::BinaryDescending,
                    _ => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Invalid search_mode".to_string(),
                        )
                    }
                },
                Err(e) => return e,
            }
        } else {
            SearchMode::FirstToLast
        };

        // Materialise `lookup_array` into a one-dimensional vector of values.
        // It may be a range reference or an in-formula array constant `{…}`;
        // either way it must be a single row or single column.
        let array: SearchVector = match self.evaluate_node_in_context(&args[1], cell) {
            CalcResult::Range { left, right } => {
                let is_col_vec = left.column == right.column;
                let is_row_vec = left.row == right.row;
                if !is_col_vec && !is_row_vec {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "lookup_array must be a single row or column".to_string(),
                    );
                }

                // Entire-column/row references evaluate only the used part;
                // the rest is a virtual run of empty cells (T-0127).
                self.range_search_vector(&left, &right, is_col_vec)
            }
            CalcResult::Array(rows) => {
                // An array constant must be a single row or single column.
                let n_rows = rows.len();
                let n_cols = rows.first().map(|r| r.len()).unwrap_or(0);
                let is_row_vec = n_rows == 1;
                let is_col_vec = n_cols == 1;
                if n_rows == 0 || n_cols == 0 || (!is_row_vec && !is_col_vec) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "lookup_array must be a single row or column".to_string(),
                    );
                }
                SearchVector::new(
                    rows.iter()
                        .flatten()
                        .map(|node| super::array_node_to_calc_result(node, cell))
                        .collect(),
                )
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "lookup_array must be a range or array".to_string(),
                )
            }
        };

        match search_mode {
            SearchMode::FirstToLast | SearchMode::LastToFirst => {
                if match_mode == MatchMode::Regex {
                    if let CalcResult::String(ref pat) = lookup_value {
                        if Regex::new(pat).is_err() {
                            return CalcResult::new_error(
                                Error::VALUE,
                                cell,
                                "Invalid regular expression".to_string(),
                            );
                        }
                    }
                }
                match linear_search(&lookup_value, &array, &search_mode, match_mode) {
                    Some(idx) => CalcResult::Number(idx as f64 + 1.0),
                    None => CalcResult::new_error(Error::NA, cell, "Not found".to_string()),
                }
            }
            SearchMode::BinaryAscending | SearchMode::BinaryDescending => {
                if match_mode == MatchMode::Wildcard || match_mode == MatchMode::Regex {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Wildcard/regex match cannot be used with binary search".to_string(),
                    );
                }
                match binary_search(
                    &lookup_value,
                    &array,
                    search_mode == SearchMode::BinaryDescending,
                    match_mode.ordering(),
                ) {
                    None => CalcResult::new_error(Error::NA, cell, "Not found".to_string()),
                    Some(l) => CalcResult::Number(l as f64 + 1.0),
                }
            }
        }
    }
}
