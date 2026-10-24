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
        let (mut operands, lists) = self.lift_text_operands(args, &[1, 2], cell);
        // pad_with does not lift: an array of any size, a 1x1 one included, is #VALUE! (probes/
        // textsplit-lift-guesses A35-A38, A65), but per element, after an error argument and
        // beside lifted ones (gen-m7 f7e94a817e5385ac A2186, 2d9b48c6363e7a16 A3261; text.md
        // "TEXTSPLIT: argument errors first, an all-empty result"), so it is taken out of the lift.
        let pad_is_array = matches!(operands.get(5), Some(CalcResult::Array(_)));
        if pad_is_array {
            operands[5] = CalcResult::EmptyArg;
        }
        let mut lists = lists.into_iter();
        // A delimiter that is itself an error is an argument error, the first after the text's;
        // an error element of a list comes after ignore_empty and match_mode (probes/
        // textsplit-error-order A1-A4, A13, A14).
        let mut delimiter_error = None;
        let mut split = |model: &mut Model, list: CalcResult| {
            if delimiter_error.is_none() && matches!(list, CalcResult::Error { .. }) {
                delimiter_error = Some(list.clone());
            }
            model.split_delimiters(list, cell)
        };
        let col_delimiters = lists.next().map(|list| split(self, list));
        // A bare empty-arg (omitted) row delimiter → no row splitting, which is the empty list.
        let row_delimiters = lists.next().map(|list| split(self, list));
        let scanned: Vec<usize> = [&col_delimiters, &row_delimiters]
            .into_iter()
            .flatten()
            .flatten()
            .flat_map(|d| d.list.exact().iter().map(String::len))
            .collect();
        let call = SplitCall {
            arg_count: args.len(),
            delimiter_error,
            pad_is_array,
            col_delimiters,
            row_delimiters,
        };
        self.lift_text_call(
            operands,
            cell,
            &scanned,
            SEARCHED_BYTES_PER_VISIT,
            |model, values, first_only| model.textsplit_element(values, &call, first_only, cell),
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
        call: &SplitCall,
        first_only: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = call.arg_count;
        let text = match Self::cast_string_ref(&values[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        if let Some(error) = &call.delimiter_error {
            return error.clone();
        }

        // ignore_empty, then match_mode, each an error or not a boolean or number, is the result
        // before an array pad_with's #VALUE!, an error element of a delimiter list and
        // TEXTSPLIT's own #VALUE! for an empty delimiter, no delimiter or empty text (gen-m7
        // 90de8755a2e565e6 A1643, ac67253061bba16a A184; probes/textsplit-error-order A5, A6,
        // A8, A18; text.md "TEXTSPLIT: argument errors first, an all-empty result").
        let ignore_empty = if arg_count >= 4 {
            match Self::cast_bool_ref(&values[3], cell) {
                Ok(b) => b,
                Err(e) => return e,
            }
        } else {
            false
        };

        let match_mode = if arg_count >= 5 {
            match self.cast_number_ref(&values[4], cell) {
                Ok(n) => n.trunc(),
                Err(e) => return e,
            }
        } else {
            0.0
        };

        if call.pad_is_array {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "pad_with must be a single value".to_string(),
            );
        }

        let no_delimiters = SplitList::new(Delimiters::new(vec![]));
        let col_delims = match &call.col_delimiters {
            Some(Ok(d)) => d,
            Some(Err(e)) => return e.clone(),
            None => &no_delimiters,
        };

        let row_delims = match &call.row_delimiters {
            Some(Ok(d)) => d,
            Some(Err(e)) => return e.clone(),
            None => &no_delimiters,
        };

        // match_mode is cut toward zero and must then be 0 or 1, as in TEXTBEFORE, but checked
        // only after an error element of a delimiter list (probes/t2581-textsplit-textbefore
        // A17-A23, A29; text.md "TEXTSPLIT: column delimiters first, match_mode 0 or 1").
        let case_insensitive = match match_mode {
            0.0 => false,
            1.0 => true,
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "match_mode must be 0 or 1".to_string(),
                )
            }
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
        // `text`; `split_grid` converts them to byte offsets of the original (T-1794).
        let (cmp_text, case) = if case_insensitive {
            (Cow::Owned(fold_for_split(&text)), Case::Insensitive)
        } else {
            (Cow::Borrowed(text.as_str()), Case::Sensitive)
        };
        let cmp_col_delims = col_delims.for_case(case);
        let cmp_row_delims = row_delims.for_case(case);

        // One scan for both lists: a row delimiter that starts before a column delimiter ends
        // the row there, even inside what a rows-first split would have kept whole (probes/
        // t2581-textsplit-textbefore A6, A7). ignore_empty drops an empty piece, and an empty
        // row: one with no text between its row delimiters.
        let rows_of_cols: Vec<Vec<&str>> =
            split_grid(&text, &cmp_text, cmp_col_delims, cmp_row_delims)
                .into_iter()
                .filter(|row| !(ignore_empty && matches!(row.as_slice(), [""])))
                .map(|row| {
                    if ignore_empty {
                        row.into_iter().filter(|s| !s.is_empty()).collect()
                    } else {
                        row
                    }
                })
                .collect();

        // ignore_empty dropped every piece: #CALC!, the empty-array error (probes/
        // textsplit-error-order A11, A12; gen-m7 07d27aa0add0247f A768).
        if rows_of_cols.is_empty() || rows_of_cols.iter().all(|r| r.is_empty()) {
            return CalcResult::new_error(
                Error::CALC,
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

/// What every element of one TEXTSPLIT call shares: the argument count, the delimiter lists
/// (read once), the first delimiter that is itself an error, and whether `pad_with` is an array
/// (`#VALUE!`, but per element; it does not lift).
struct SplitCall {
    arg_count: usize,
    delimiter_error: Option<CalcResult>,
    pad_is_array: bool,
    col_delimiters: Option<Result<SplitList, CalcResult>>,
    row_delimiters: Option<Result<SplitList, CalcResult>>,
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

/// Split `original` into rows of pieces at the first (earliest) occurrence of any delimiter of
/// `cols` or `rows`, using `cmp` (a parallel, char-for-char comparison string, e.g. case-folded)
/// to locate match positions. At one position a column delimiter beats a row delimiter, whatever
/// their lengths (probes/t2581-textsplit-textbefore A1, A4, A5, A13), and the longest delimiter
/// of one list wins (probes/textsplit-lift-guesses A4, A6).
///
/// `cmp` and `original` have the same number of chars but not necessarily the same byte layout
/// (U+212A is 3 bytes, its fold `k` is 1), so a match is converted to a char index in `cmp` and
/// then to a byte offset in `original`. If the two ever disagree the original is returned whole
/// rather than sliced out of range.
fn split_grid<'a>(
    original: &'a str,
    cmp: &str,
    cols: &[Finder<'_>],
    rows: &[Finder<'_>],
) -> Vec<Vec<&'a str>> {
    if cols.is_empty() && rows.is_empty() {
        return vec![vec![original]];
    }

    // ASCII on both sides (the usual case) has the same byte layout: no offsets to map.
    let same_layout = original.len() == cmp.len() && original.is_ascii() && cmp.is_ascii();
    let (cmp_offsets, orig_offsets) = if same_layout {
        (vec![], vec![])
    } else {
        (char_offsets(cmp), char_offsets(original))
    };
    if cmp_offsets.len() != orig_offsets.len() {
        return vec![vec![original]];
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

    // Each delimiter with whether it ends a row; column delimiters first.
    let delims: Vec<(&Finder<'_>, bool)> = cols
        .iter()
        .map(|d| (d, false))
        .chain(rows.iter().map(|d| (d, true)))
        .collect();
    let mut result = Vec::new();
    let mut row = Vec::new();
    let mut pos = 0usize; // byte offset in `cmp`
    let mut orig_pos = 0usize; // byte offset in `original`

    // The next match of each delimiter at or after `pos`, searched again only once `pos` has
    // passed it, so each delimiter's searches move forward through `cmp` instead of rescanning
    // the rest of it for every piece (T-1272 performance review).
    let mut next: Vec<Option<usize>> = delims.iter().map(|(d, _)| d.find(cmp.as_bytes())).collect();

    loop {
        for (found, (d, _)) in next.iter_mut().zip(&delims) {
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
            .zip(&delims)
            .filter_map(|(found, (d, ends_row))| {
                found.map(|start| (start, *ends_row, start + d.needle().len()))
            })
            .min_by_key(|(start, ends_row, end)| (*start, *ends_row, std::cmp::Reverse(*end)));

        let matched = best.and_then(|(start, ends_row, end)| {
            Some((end, ends_row, to_original(start)?, to_original(end)?))
        });
        match matched {
            None => {
                row.push(original.get(orig_pos..).unwrap_or_default());
                result.push(row);
                break;
            }
            Some((cmp_end, ends_row, orig_start, orig_end)) => {
                row.push(original.get(orig_pos..orig_start).unwrap_or_default());
                if ends_row {
                    result.push(std::mem::take(&mut row));
                }
                pos = cmp_end;
                orig_pos = orig_end;
            }
        }
    }

    result
}
