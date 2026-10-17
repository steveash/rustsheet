use super::casefold::{char_offsets, fold_for_split};
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
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
        let mut lists = lists.into_iter();
        let col_delimiters = lists.next().map(|list| self.delimiter_list(list, cell));
        let row_delimiters = lists.next().map(|list| self.delimiter_list(list, cell));
        let arg_count = args.len();
        self.lift_text_call(operands, cell, |model, values| {
            model.textsplit_element(
                values,
                arg_count,
                col_delimiters.as_ref(),
                row_delimiters.as_ref(),
                cell,
            )
        })
    }

    /// One TEXTSPLIT call on the values of the lifted arguments.
    fn textsplit_element(
        &mut self,
        values: &[CalcResult],
        arg_count: usize,
        col_delimiters: Option<&Result<Vec<String>, CalcResult>>,
        row_delimiters: Option<&Result<Vec<String>, CalcResult>>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let text = match self.cast_to_string(values[0].clone(), cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let col_delims = match col_delimiters {
            Some(Ok(d)) => d.clone(),
            Some(Err(e)) => return e.clone(),
            None => vec![],
        };

        // An explicit empty string "" as a delimiter is invalid.
        // An omitted col_delimiter (empty vec) is allowed when row_delimiter is provided.
        if col_delims.iter().any(|d| d.is_empty()) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "col_delimiter must not be empty string".to_string(),
            );
        }

        // A bare empty-arg (omitted) row delimiter → no row splitting, which is the empty list.
        let row_delims: Vec<String> = match row_delimiters {
            Some(Ok(d)) => d.iter().filter(|s| !s.is_empty()).cloned().collect(),
            Some(Err(e)) => return e.clone(),
            None => vec![],
        };

        if col_delims.is_empty() && row_delims.is_empty() {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "TEXTSPLIT requires at least one delimiter".to_string(),
            );
        }

        let ignore_empty = if arg_count >= 4 {
            match self.cast_to_bool(values[3].clone(), cell) {
                Ok(b) => b,
                Err(e) => return e,
            }
        } else {
            false
        };

        let case_insensitive = if arg_count >= 5 {
            match self.cast_to_number(values[4].clone(), cell) {
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

        let na_node = ArrayNode::Error(Error::NA);
        let pad_node = pad_with.as_ref().unwrap_or(&na_node);

        // Normalize to comparison strings for optional case-insensitive matching.
        // `fold_for_split` maps every char to exactly one char (TEXTSPLIT's folding, probes/
        // textsplit-casefold-length), so char positions in `cmp_text` are char positions in
        // `text`; `split_by_any` converts them to byte offsets of the original (T-1794).
        let cmp_text: String;
        let cmp_col_delims: Vec<String>;
        let cmp_row_delims: Vec<String>;
        if case_insensitive {
            cmp_text = fold_for_split(&text);
            cmp_col_delims = col_delims.iter().map(|s| fold_for_split(s)).collect();
            cmp_row_delims = row_delims.iter().map(|s| fold_for_split(s)).collect();
        } else {
            cmp_text = text.clone();
            cmp_col_delims = col_delims.clone();
            cmp_row_delims = row_delims.clone();
        }

        let rows_of_cols: Vec<Vec<String>> = if !cmp_row_delims.is_empty() {
            // Split into rows first, then each row into columns.
            let row_parts = split_by_any(&text, &cmp_text, &cmp_row_delims);
            let cmp_row_parts = split_by_any(&cmp_text, &cmp_text, &cmp_row_delims);

            row_parts
                .into_iter()
                .zip(cmp_row_parts)
                .filter_map(|(orig_row, cmp_row_part)| {
                    if ignore_empty && orig_row.is_empty() {
                        return None;
                    }
                    let cols = split_by_any(&orig_row, &cmp_row_part, &cmp_col_delims);
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
            let cols = split_by_any(&text, &cmp_text, &cmp_col_delims);
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

        let result: Vec<Vec<ArrayNode>> = rows_of_cols
            .into_iter()
            .map(|row| {
                let mut arr: Vec<ArrayNode> = row
                    .into_iter()
                    .map(|s| ArrayNode::String(s.into()))
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

/// Split `original` by the first (earliest) occurrence of any non-empty delimiter
/// in `cmp_delims`, using `cmp` (a parallel, char-for-char comparison string, e.g. case-folded)
/// to locate match positions.
///
/// `cmp` and `original` have the same number of chars but not necessarily the same byte layout
/// (U+212A is 3 bytes, its fold `k` is 1), so a match is converted to a char index in `cmp` and
/// then to a byte offset in `original`. If the two ever disagree the original is returned whole
/// rather than sliced out of range.
fn split_by_any(original: &str, cmp: &str, cmp_delims: &[String]) -> Vec<String> {
    let delims: Vec<&str> = cmp_delims
        .iter()
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .collect();

    if delims.is_empty() {
        return vec![original.to_string()];
    }

    let cmp_offsets = char_offsets(cmp);
    let orig_offsets = char_offsets(original);
    if cmp_offsets.len() != orig_offsets.len() {
        return vec![original.to_string()];
    }
    // Char index in `cmp` of a byte offset that `find` returned (always a char boundary).
    let char_index = |byte: usize| cmp_offsets.binary_search(&byte).ok();

    let mut result = Vec::new();
    let mut pos = 0usize; // byte offset in `cmp`
    let mut orig_pos = 0usize; // byte offset in `original`

    loop {
        // Find the earliest match among all delimiters.
        let best = delims
            .iter()
            .filter_map(|d| {
                cmp.get(pos..)?
                    .find(d)
                    .map(|i| (pos + i, pos + i + d.len()))
            })
            .min_by_key(|(p, _)| *p);

        let matched = best.and_then(|(start, end)| {
            let (s, e) = (char_index(start)?, char_index(end)?);
            Some((start, end, *orig_offsets.get(s)?, *orig_offsets.get(e)?))
        });
        match matched {
            None => {
                result.push(original.get(orig_pos..).unwrap_or_default().to_string());
                break;
            }
            Some((_, cmp_end, orig_start, orig_end)) => {
                result.push(
                    original
                        .get(orig_pos..orig_start)
                        .unwrap_or_default()
                        .to_string(),
                );
                pos = cmp_end;
                orig_pos = orig_end;
            }
        }
    }

    result
}
