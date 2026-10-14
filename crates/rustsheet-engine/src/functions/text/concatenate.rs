use super::text_arg::{broadcast_text, TextArg};
use crate::calc_result::Range;
use crate::functions::util::array_node_to_calc_result;
use crate::number_format::number_to_text;
use crate::{
    calc_result::CalcResult,
    cast::{array_node_to_string, truncate_text, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// Excel: CONCAT and TEXTJOIN text over 32767 characters is #CALC!, not truncated
/// (probes/text-result-limit C8, C9).
fn text_limit_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::CALC,
        cell,
        "Result is longer than 32767 characters".to_string(),
    )
}

/// Running UTF-16 size of a TEXTJOIN result, kept incrementally so each check is O(new values)
/// and an oversized join stops before it is built (CLAUDE.md directive 5).
struct JoinSize {
    delimiter_units: usize,
    counted: usize,
    units: usize,
}

impl JoinSize {
    fn new(delimiter: &str) -> Self {
        JoinSize {
            delimiter_units: delimiter.encode_utf16().count(),
            counted: 0,
            units: 0,
        }
    }

    fn exceeds(&mut self, values: &[String]) -> bool {
        for value in values.get(self.counted..).unwrap_or_default() {
            let separator = if self.counted == 0 {
                0
            } else {
                self.delimiter_units
            };
            self.units = self
                .units
                .saturating_add(separator + value.encode_utf16().count());
            self.counted += 1;
        }
        self.units > MAX_TEXT_UNITS
    }

    /// Whether the join is over the limit once the delimiter before the next value is counted.
    /// A cell error read from a reference is `#CALC!` when the text so far plus this delimiter is
    /// over the limit (probes/text-result-limit-errors C2-C3, C7-C9).
    fn exceeds_with_separator(&mut self, values: &[String]) -> bool {
        let over = self.exceeds(values);
        let separator = if self.counted == 0 {
            0
        } else {
            self.delimiter_units
        };
        over || self.units.saturating_add(separator) > MAX_TEXT_UNITS
    }

    /// Appends one range cell's text to a TEXTJOIN. Once the join is over the limit the values
    /// are dropped, and a cell error met there is dropped too (the result is #CALC! unless a later
    /// error argument wins); otherwise the error is returned (probes/text-result-limit C18-C20,
    /// -errors C2, C4, -errors-refs D10-D11).
    fn push_cell(
        &mut self,
        value: CalcResult,
        ignore_empty: bool,
        values: &mut Vec<String>,
        over_limit: &mut bool,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        *over_limit = *over_limit || self.exceeds(values);
        if *over_limit {
            values.clear();
        }
        match value {
            CalcResult::Number(n) => values.push(number_to_text(n)),
            CalcResult::String(text) => values.push(text),
            CalcResult::Boolean(b) => values.push(if b { "TRUE" } else { "FALSE" }.to_string()),
            CalcResult::EmptyCell if !ignore_empty => values.push(String::new()),
            CalcResult::Error { .. } if *over_limit || self.exceeds_with_separator(values) => {
                *over_limit = true;
            }
            error @ CalcResult::Error { .. } => return Err(error),
            CalcResult::EmptyCell | CalcResult::EmptyArg | CalcResult::Range { .. } => {}
            CalcResult::Array(_) | CalcResult::Lambda(_) => {
                return Err(CalcResult::new_error(
                    Error::NIMPL,
                    cell,
                    "Arrays not supported yet".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Adds `count` empty values, the empty cells a clipped range walk skipped, stopping once the
    /// join is over the limit (so at most about 32,767 are added). Returns whether it is over.
    /// With an empty delimiter they add nothing, so none are added.
    fn push_empties(&mut self, values: &mut Vec<String>, count: i64) -> bool {
        if self.delimiter_units == 0 {
            return false;
        }
        for _ in 0..count {
            values.push(String::new());
            if self.exceeds(values) {
                return true;
            }
        }
        false
    }
}

/// CONCAT's result under construction with its running UTF-16 size, so each append is O(piece)
/// however many cells a range has. Once over the cell limit it stops appending but keeps
/// counting (CLAUDE.md directive 5).
#[derive(Default)]
struct ConcatText {
    text: String,
    units: usize,
}

impl ConcatText {
    fn push(&mut self, piece: &str) {
        self.units = self.units.saturating_add(piece.encode_utf16().count());
        if self.units <= MAX_TEXT_UNITS {
            self.text.push_str(piece);
        }
    }

    /// Appends the text of a scalar cell value; an error value is returned as `Err`.
    fn push_value(&mut self, value: CalcResult) -> Result<(), CalcResult> {
        match value {
            CalcResult::String(value) => self.push(&value),
            CalcResult::Number(value) => self.push(&number_to_text(value)),
            CalcResult::Boolean(value) => self.push(if value { "TRUE" } else { "FALSE" }),
            error @ CalcResult::Error { .. } => return Err(error),
            _ => {}
        }
        Ok(())
    }
}

impl Model {
    /// CONCATENATE(text1, [text2], ...): joins text; array arguments lift element-wise.
    pub(crate) fn fn_concatenate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        // Array arguments lift element-wise (arrays.md "NOT and CONCATENATE lift over array
        // arguments"); with only scalars the result is the plain joined string.
        let mut operands = Vec::with_capacity(args.len());
        for arg in args {
            match self.text_arg(arg, cell) {
                Ok(operand) => operands.push(operand),
                Err(error) => return error,
            }
        }
        let operand_refs: Vec<&TextArg> = operands.iter().collect();
        broadcast_text(cell, &operand_refs, |i, j| {
            let mut joined = String::new();
            for operand in &operands {
                joined.push_str(&array_node_to_string(operand.elem(i, j))?);
                truncate_text(&mut joined);
            }
            Ok(ArrayNode::String(joined))
        })
    }

    /// CONCAT(text1, [text2], ...): joins text and ranges; over 32767 characters is #CALC!
    /// (probes/text-result-limit C8).
    pub(crate) fn fn_concat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = ConcatText::default();
        for arg in args {
            // A union stays `#VALUE!` here: CONCAT is not probed with one (SPEC-0004).
            let value = self
                .evaluate_reference_arg(arg, cell)
                .into_single_area(cell);
            match value {
                CalcResult::Range { left, right } => {
                    // 3-D references are walked sheet by sheet by the shared range walk.
                    if let Err(error) = self.for_each_cell_or_error_in_areas(
                        &[Range { left, right }],
                        cell,
                        |_, cell_value| {
                            if matches!(cell_value, CalcResult::Array(_) | CalcResult::Lambda(_)) {
                                return Err(CalcResult::new_error(
                                    Error::NIMPL,
                                    cell,
                                    "Arrays not supported yet".to_string(),
                                ));
                            }
                            match result.push_value(cell_value) {
                                // Excel: a cell error met once the text is over the limit is
                                // dropped and the result is #CALC! (probes/text-result-limit-errors
                                // C1 for a range cell; C10 for a single-cell reference, which
                                // `evaluate_reference_arg` hands over as a range); at exactly the
                                // limit it is returned (C5-C6).
                                Err(_) if result.units > MAX_TEXT_UNITS => Ok(()),
                                other => other,
                            }
                        },
                    ) {
                        return error;
                    }
                }
                // An array argument is joined element by element, row by row
                // (gen-m2/d0c29dd5c530e28c Calc!A246: CONCAT of an all-empty MID array is "").
                CalcResult::Array(rows) => {
                    for node in rows.iter().flatten() {
                        let element = array_node_to_calc_result(node, cell);
                        match result.push_value(element) {
                            Err(_) if result.units > MAX_TEXT_UNITS => break,
                            Err(error) => return error,
                            Ok(()) => {}
                        }
                    }
                }
                CalcResult::Lambda(_) => {
                    return CalcResult::new_error(
                        Error::NIMPL,
                        cell,
                        "Arrays not supported yet".to_string(),
                    );
                }
                scalar => {
                    if let Err(error) = result.push_value(scalar) {
                        return error;
                    }
                }
            }
        }
        // Excel: joined text over 32767 characters is #CALC!, not truncated (probes/text-result-limit
        // C8). Later arguments are still evaluated so an error argument wins (C18, -errors C4).
        if result.units > MAX_TEXT_UNITS {
            return text_limit_error(cell);
        }
        CalcResult::String(result.text)
    }

    /// TEXTJOIN(delimiter, `ignore_empty`, text1, [text2], ...): joins text with a delimiter; over 32767
    /// characters is #CALC! (probes/text-result-limit C9).
    pub(crate) fn fn_textjoin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let delimiter = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let ignore_empty = match self.get_boolean(&args[1], cell) {
            Ok(b) => b,
            Err(error) => return error,
        };
        let mut values: Vec<String> = Vec::new();
        let mut size = JoinSize::new(&delimiter);
        // Once over the limit the values are dropped, but arguments are still evaluated so an error
        // argument after the limit wins; a cell error read past it is dropped and evaluation goes on,
        // so the result is #CALC! unless a later error argument wins
        // (probes/text-result-limit C18-C20, -errors C2-C4).
        let mut over_limit = false;
        for arg in &args[2..] {
            over_limit = over_limit || size.exceeds(&values);
            if over_limit {
                values.clear();
            }
            // A union stays `#VALUE!` here: TEXTJOIN is not probed with one (SPEC-0004).
            match self
                .evaluate_reference_arg(arg, cell)
                .into_single_area(cell)
            {
                CalcResult::Number(value) => values.push(number_to_text(value)),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        // A 3-D reference is walked sheet by sheet; an error cell past the limit
                        // is dropped like a range cell's (probes/text-result-limit-errors-refs
                        // D10-D11).
                        if let Err(error) = self.for_each_cell_or_error_in_areas(
                            &[Range { left, right }],
                            cell,
                            |_, value| {
                                size.push_cell(
                                    value,
                                    ignore_empty,
                                    &mut values,
                                    &mut over_limit,
                                    cell,
                                )
                            },
                        ) {
                            return error;
                        }
                        continue;
                    }
                    let row1 = left.row;
                    let mut row2 = right.row;
                    let column1 = left.column;
                    let mut column2 = right.column;
                    let (full_row2, full_column2) = (row2, column2);
                    match self.clip_to_used_area(left.sheet, row1, column1, row2, column2) {
                        Ok((r, c)) => {
                            row2 = r;
                            column2 = c;
                        }
                        Err(message) => return CalcResult::new_error(Error::ERROR, cell, message),
                    }
                    // Excel joins every cell of a whole column or row, so the empty cells the clip
                    // skips still add a delimiter each with ignore_empty FALSE: `TEXTJOIN(",",
                    // FALSE,D:D)` is #CALC! (over 32,767) and `TEXTJOIN(",",FALSE,10:10)` has 16,383
                    // delimiters (probes/whole-column-empty-walkers F1, F8; text.md).
                    let pad = !ignore_empty;
                    // `max` covers a clip that ends before the range starts (nothing walked).
                    let skipped_columns =
                        i64::from(full_column2) - i64::from(column2.max(column1 - 1));
                    let skipped_rows = i64::from(full_row2) - i64::from(row2.max(row1 - 1));
                    let width = i64::from(full_column2) - i64::from(column1) + 1;
                    for row in row1..row2 + 1 {
                        for column in column1..(column2 + 1) {
                            let value = self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            });
                            if let Err(error) = size.push_cell(
                                value,
                                ignore_empty,
                                &mut values,
                                &mut over_limit,
                                cell,
                            ) {
                                return error;
                            }
                        }
                        if pad && !over_limit {
                            over_limit = size.push_empties(&mut values, skipped_columns);
                        }
                    }
                    if pad && !over_limit {
                        over_limit =
                            size.push_empties(&mut values, skipped_rows.saturating_mul(width));
                    }
                }
                // A direct single-cell reference is read like a range cell (probes/text-result-limit-errors
                // C3).
                CalcResult::Error { .. }
                    if matches!(arg.ungrouped(), Node::ReferenceKind { .. })
                        && (over_limit || size.exceeds_with_separator(&values)) =>
                {
                    over_limit = true;
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(value) => values.push(value),
                CalcResult::Boolean(value) => {
                    if value {
                        values.push("TRUE".to_string());
                    } else {
                        values.push("FALSE".to_string());
                    }
                }
                CalcResult::EmptyCell => {
                    if !ignore_empty {
                        values.push("".to_string());
                    }
                }
                CalcResult::EmptyArg => {}
                // Excel joins an array row by row; the first error element wins and an empty element
                // (a blank cell passed through SORT, UNIQUE, TRANSPOSE) is skipped only with
                // ignore_empty (probes/textjoin-array-arguments A1-A28).
                CalcResult::Array(array) => {
                    for element in array.iter().flatten() {
                        // An empty string element counts as empty too (A4, A19, A20).
                        if ignore_empty
                            && matches!(element, ArrayNode::String(text) if text.is_empty())
                        {
                            continue;
                        }
                        let value = array_node_to_calc_result(element, cell);
                        if let Err(error) =
                            size.push_cell(value, ignore_empty, &mut values, &mut over_limit, cell)
                        {
                            return error;
                        }
                    }
                }
                CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
        }
        if over_limit || size.exceeds(&values) {
            return text_limit_error(cell);
        }
        CalcResult::String(values.join(&delimiter))
    }
}
