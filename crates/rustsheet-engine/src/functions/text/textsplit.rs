use std::borrow::Cow;

use memchr::memmem::Finder;

use super::{
    casefold::{char_offsets, fold_for_split},
    lifted::SEARCHED_BYTES_PER_VISIT,
    util::{Case, Delimiters},
};
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::util::array_node_to_calc_result,
    limits::check_array_cells,
    model::Model,
};

impl Model {
    /// `=TEXTSPLIT(text, col_delimiter, [row_delimiter], [ignore_empty], [match_mode], [pad_with])`
    ///
    /// Splits text into a 2-D array using column and/or row delimiters.
    /// Each delimiter argument can be a single string or an array of strings — any
    /// matching delimiter triggers a split.
    ///   * text          – text to split (required)
    ///   * `col_delimiter` – one or more delimiters that split into columns (required)
    ///   * `row_delimiter` – one or more delimiters that split into rows (default: no row split)
    ///   * `ignore_empty`  – TRUE to skip empty segments (default FALSE)
    ///   * `match_mode`    – 0 = case-sensitive (default), 1 = case-insensitive
    ///   * `pad_with`      – value to pad short rows (default: #N/A error)
    pub(crate) fn fn_textsplit(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 6 {
            return CalcResult::new_args_number_error(cell);
        }
        // The delimiters are lists taken whole; the text and the scalar arguments lift over a
        // range or an array (probes/m7-argument-modes F13, F14).
        let (operands, lists) = self.lift_text_operands(args, &[1, 2], cell);
        // pad_with does not lift: an array of any size, a 1x1 one included, is #VALUE! (probes/
        // textsplit-lift-guesses A35-A38, A65).
        if let Some(CalcResult::Array(_)) = operands.get(5) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "pad_with must be a single value".to_string(),
            );
        }
        let mut lists = lists.into_iter();
        let col_delimiters = lists.next().map(|list| self.split_delimiters(list, cell));
        // A bare empty-arg (omitted) row delimiter → no row splitting, which is the empty list.
        let row_delimiters = lists.next().map(|list| self.split_delimiters(list, cell));
        let arg_count = args.len();
        let scanned: Vec<usize> = [&col_delimiters, &row_delimiters]
            .into_iter()
            .flatten()
            .flatten()
            .flat_map(|d| d.list.exact().iter().map(String::len))
            .collect();
        self.lift_text_call(
            operands,
            cell,
            &scanned,
            SEARCHED_BYTES_PER_VISIT,
            |model, values, first_only| {
                model.textsplit_element(
                    values,
                    arg_count,
                    col_delimiters.as_ref(),
                    row_delimiters.as_ref(),
                    first_only,
                    cell,
                )
            },
        )
    }

    /// A TEXTSPLIT delimiter list, folded once per call.
    fn split_delimiters(
        &mut self,
        list: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<SplitList, CalcResult> {
        let exact = self.delimiter_list(list, cell)?;
        Ok(SplitList::new(Delimiters::folded_with(
            exact,
            fold_for_split,
        )))
    }

    /// One TEXTSPLIT call on the values of the lifted arguments; `first_only` returns only the
    /// first value of its array (a lifted element).
    fn textsplit_element(
        &mut self,
        values: &[CalcResult],
        arg_count: usize,
        col_delimiters: Option<&Result<SplitList, CalcResult>>,
        row_delimiters: Option<&Result<SplitList, CalcResult>>,
        first_only: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let text = match Self::cast_string_ref(&values[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let no_delimiters = SplitList::new(Delimiters::new(vec![]));
        let col_delims = match col_delimiters {
            Some(Ok(d)) => d,
            Some(Err(e)) => return e.clone(),
            None => &no_delimiters,
        };

        let row_delims = match row_delimiters {
            Some(Ok(d)) => d,
            Some(Err(e)) => return e.clone(),
            None => &no_delimiters,
        };

        // An empty delimiter (a "" string, a blank cell or an empty element of a list) is
        // `#VALUE!`, in either list; an omitted delimiter is the empty list and is allowed when
        // the other one is given (probes/textsplit-empty-delimiter A1-A12).
        let lists = [&col_delims.list, &row_delims.list];
        if lists
            .iter()
            .any(|list| list.exact().iter().any(String::is_empty))
        {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "a delimiter must not be empty".to_string(),
            );
        }

        if col_delims.list.exact().is_empty() && row_delims.list.exact().is_empty() {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "TEXTSPLIT requires at least one delimiter".to_string(),
            );
        }

        let ignore_empty = if arg_count >= 4 {
            match Self::cast_bool_ref(&values[3], cell) {
                Ok(b) => b,
                Err(e) => return e,
            }
        } else {
            false
        };

        let case_insensitive = if arg_count >= 5 {
            match self.cast_number_ref(&values[4], cell) {
                Ok(n) => n != 0.0,
                Err(e) => return e,
            }
        } else {
            false
        };

        let pad_with: Option<ArrayNode> = if arg_count >= 6 {
            match &values[5] {
                CalcResult::Number(n) => Some(ArrayNode::Number(*n)),
                CalcResult::Boolean(b) => Some(ArrayNode::Boolean(*b)),
                CalcResult::String(s) => Some(ArrayNode::String(s.clone())),
                CalcResult::Error { error, .. } => Some(ArrayNode::Error(error.clone())),
                _ => None,
            }
        } else {
            None
        };

        // Excel: empty text (a literal, a blank cell or an empty string) is #VALUE!, not an empty
        // result (docs/excel-semantics/text.md "TEXTSPLIT of empty text is #VALUE!").
        if text.is_empty() {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "TEXTSPLIT of empty text".to_string(),
            );
        }

        let na_node = ArrayNode::Error(Error::NA);
        let pad_node = pad_with.as_ref().unwrap_or(&na_node);

        // Normalize to comparison strings for optional case-insensitive matching.
        // `fold_for_split` maps every char to exactly one char (TEXTSPLIT's folding, probes/
        // textsplit-casefold-length), so char positions in `cmp_text` are char positions in
        // `text`; `split_by_any` converts them to byte offsets of the original (T-1794).
        let (cmp_text, case) = if case_insensitive {
            (Cow::Owned(fold_for_split(&text)), Case::Insensitive)
        } else {
            (Cow::Borrowed(text.as_str()), Case::Sensitive)
        };
        let cmp_col_delims = col_delims.for_case(case);
        let cmp_row_delims = row_delims.for_case(case);

        let rows_of_cols: Vec<Vec<&str>> = if !cmp_row_delims.is_empty() {
            // Split into rows first, then each row into columns.
            let row_parts = split_by_any(&text, &cmp_text, cmp_row_delims);
            let cmp_row_parts = split_by_any(&cmp_text, &cmp_text, cmp_row_delims);

            row_parts
                .into_iter()
                .zip(cmp_row_parts)
                .filter_map(|(orig_row, cmp_row_part)| {
                    if ignore_empty && orig_row.is_empty() {
                        return None;
                    }
                    let cols = split_by_any(orig_row, cmp_row_part, cmp_col_delims);
                    let cols = if ignore_empty {
                        cols.into_iter().filter(|s| !s.is_empty()).collect()
                    } else {
                        cols
                    };
                    Some(cols)
                })
                .collect()
        } else {
            // No row delimiter — single row.
            let cols = split_by_any(&text, &cmp_text, cmp_col_delims);
            let cols = if ignore_empty {
                cols.into_iter().filter(|s| !s.is_empty()).collect()
            } else {
                cols
            };
            vec![cols]
        };

        if rows_of_cols.is_empty() || rows_of_cols.iter().all(|r| r.is_empty()) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "TEXTSPLIT produced no output".to_string(),
            );
        }

        let max_cols = rows_of_cols.iter().map(|r| r.len()).max().unwrap_or(0);

        // The padded shape is rows x widest row: a 32,767-char text with the column delimiters
        // first and the row delimiters after pads ~2^28 cells. Refuse one above the array limit
        // before any row is padded (DEVIATION, arrays.md "Array results above 1,048,576 cells";
        // T-1781). Cell count only: Excel builds a row wider than the grid (probe
        // textsplit-wide A1: 16,385 columns), so the past-the-grid test does not apply.
        if let Some((error, message)) = check_array_cells(rows_of_cols.len(), max_cols) {
            return CalcResult::new_error(error, cell, message);
        }
        if first_only {
            // A lifted element keeps its first value: the first piece, or the padding of an
            // empty first row; the padded array is not built (T-1272 performance review).
            return match rows_of_cols
                .into_iter()
                .next()
                .and_then(|row| row.into_iter().next())
            {
                Some(piece) => CalcResult::String(piece.to_string().into()),
                None => array_node_to_calc_result(pad_node, cell),
            };
        }

        let result: Vec<Vec<ArrayNode>> = rows_of_cols
            .into_iter()
            .map(|row| {
                let mut arr: Vec<ArrayNode> = row
                    .into_iter()
                    .map(|s| ArrayNode::String(s.to_string().into()))
                    .collect();
                while arr.len() < max_cols {
                    arr.push(pad_node.clone());
                }
                arr
            })
            .collect();

        CalcResult::Array(result)
    }
}

/// A TEXTSPLIT delimiter list with one substring searcher per non-empty delimiter and case,
/// built once per call: `str::find` builds a searcher per piece, which costs as much as the
/// delimiter is long each time (T-1272 performance review).
struct SplitList {
    list: Delimiters,
    exact: Vec<Finder<'static>>,
    folded: Vec<Finder<'static>>,
}

impl SplitList {
    fn new(list: Delimiters) -> Self {
        let finders = |case| {
            list.for_case(case)
                .iter()
                .filter(|d| !d.is_empty())
                .map(|d| Finder::new(d.as_bytes()).into_owned())
                .collect()
        };
        let exact = finders(Case::Sensitive);
        let folded = finders(Case::Insensitive);
        SplitList {
            list,
            exact,
            folded,
        }
    }

    /// The searchers a text is matched with under `case`, in list order.
    fn for_case(&self, case: Case) -> &[Finder<'static>] {
        match case {
            Case::Sensitive => &self.exact,
            Case::Insensitive => &self.folded,
        }
    }
}

/// Split `original` by the first (earliest) occurrence of any of the delimiters `delims`
/// searches for, using `cmp` (a parallel, char-for-char comparison string, e.g. case-folded)
/// to locate match positions.
///
/// `cmp` and `original` have the same number of chars but not necessarily the same byte layout
/// (U+212A is 3 bytes, its fold `k` is 1), so a match is converted to a char index in `cmp` and
/// then to a byte offset in `original`. If the two ever disagree the original is returned whole
/// rather than sliced out of range.
fn split_by_any<'a>(original: &'a str, cmp: &str, delims: &[Finder<'_>]) -> Vec<&'a str> {
    if delims.is_empty() {
        return vec![original];
    }

    // ASCII on both sides (the usual case) has the same byte layout: no offsets to map.
    let same_layout = original.len() == cmp.len() && original.is_ascii() && cmp.is_ascii();
    let (cmp_offsets, orig_offsets) = if same_layout {
        (vec![], vec![])
    } else {
        (char_offsets(cmp), char_offsets(original))
    };
    if cmp_offsets.len() != orig_offsets.len() {
        return vec![original];
    }
    // The byte offset in `original` of a byte offset in `cmp` that `find` returned (always a
    // char boundary).
    let to_original = |byte: usize| {
        if same_layout {
            return Some(byte);
        }
        let index = cmp_offsets.binary_search(&byte).ok()?;
        orig_offsets.get(index).copied()
    };

    let mut result = Vec::new();
    let mut pos = 0usize; // byte offset in `cmp`
    let mut orig_pos = 0usize; // byte offset in `original`

    // The next match of each delimiter at or after `pos`, searched again only once `pos` has
    // passed it, so each delimiter's searches move forward through `cmp` instead of rescanning
    // the rest of it for every piece (T-1272 performance review).
    let mut next: Vec<Option<usize>> = delims.iter().map(|d| d.find(cmp.as_bytes())).collect();

    loop {
        // Find the earliest match among all delimiters (the longest on a tie, probes/
        // textsplit-lift-guesses A4, A6).
        for (found, d) in next.iter_mut().zip(delims) {
            if found.is_some_and(|start| start < pos) {
                *found = cmp
                    .as_bytes()
                    .get(pos..)
                    .and_then(|rest| d.find(rest))
                    .map(|i| pos + i);
            }
        }
        let best = next
            .iter()
            .zip(delims)
            .filter_map(|(found, d)| found.map(|start| (start, start + d.needle().len())))
            .min_by_key(|(start, end)| (*start, std::cmp::Reverse(*end)));

        let matched = best
            .and_then(|(start, end)| Some((start, end, to_original(start)?, to_original(end)?)));
        match matched {
            None => {
                result.push(original.get(orig_pos..).unwrap_or_default());
                break;
            }
            Some((_, cmp_end, orig_start, orig_end)) => {
                result.push(original.get(orig_pos..orig_start).unwrap_or_default());
                pos = cmp_end;
                orig_pos = orig_end;
            }
        }
    }

    result
}
