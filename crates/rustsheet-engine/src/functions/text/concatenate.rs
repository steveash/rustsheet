use crate::arithmetic::lambda_text_error;
use crate::calc_result::Range;
use crate::functions::util::{array_node_to_calc_result, RangeItem};
use crate::number_format::number_to_text;
use crate::{
    calc_result::CalcResult,
    cast::{array_node_to_string, calc_result_to_text, StringOrArray, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    text::Text,
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
    /// UTF-16 size of each delimiter; the one before value `n` (n >= 1) is `delimiter_units[(n - 1) % len]`.
    delimiter_units: Vec<usize>,
    counted: usize,
    units: usize,
}

impl JoinSize {
    fn new(delimiters: &[String]) -> Self {
        JoinSize {
            delimiter_units: delimiters
                .iter()
                .map(|d| d.encode_utf16().count())
                .collect(),
            counted: 0,
            units: 0,
        }
    }

    /// Size of the delimiter before the value at index `counted`; delimiters cycle (T-1510).
    fn separator(&self) -> usize {
        match self.counted.checked_sub(1) {
            Some(gap) => self.delimiter_units[gap % self.delimiter_units.len()],
            None => 0,
        }
    }

    fn exceeds(&mut self, values: &[String]) -> bool {
        for value in values.get(self.counted..).unwrap_or_default() {
            let separator = self.separator();
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
        let separator = self.separator();
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
            // An empty string is skipped like an empty cell with ignore_empty
            // (probes/textjoin-array-limit-empty-strings A7-A10).
            CalcResult::String(text) if ignore_empty && text.is_empty() => {}
            CalcResult::String(text) => values.push(text.into_lossy_string()),
            CalcResult::Boolean(b) => values.push(if b { "TRUE" } else { "FALSE" }.to_string()),
            CalcResult::EmptyCell if !ignore_empty => values.push(String::new()),
            CalcResult::Error { .. } if *over_limit || self.exceeds_with_separator(values) => {
                *over_limit = true;
            }
            error @ CalcResult::Error { .. } => return Err(error),
            CalcResult::EmptyCell | CalcResult::EmptyArg | CalcResult::Range { .. } => {}
            // Excel: an uncalled LAMBDA joined as text is #VALUE! (probes/t3130-concat-lambda-
            // trimrange-cell-variable Q5-Q12; arrays.md "An uncalled LAMBDA is one value").
            CalcResult::Lambda(_) => return Err(lambda_text_error(cell)),
            CalcResult::Array(_) => {
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
        if self.delimiter_units.iter().all(|&units| units == 0) {
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
        // Array arguments lift element-wise (arrays.md "NOT and CONCATENATE lift over array
        // arguments"); with only scalars the result is the plain joined string.
        self.lift_text(Function::Concatenate, args, cell, &mut |_, values| {
            let mut joined = Text::default();
            // Once the limit has cut the text, the later arguments are still read (an error
            // argument wins) but cannot change it, so they are not appended and re-cut.
            let mut at_limit = false;
            for value in values {
                let piece = calc_result_to_text(value)?;
                if !at_limit {
                    joined.push(&piece);
                    at_limit = joined.truncate_utf16(MAX_TEXT_UNITS);
                }
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
                    // 3-D references are walked sheet by sheet by the shared range walk, which
                    // leaves out the blanks past the used area: a blank joins "" anyway.
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
                CalcResult::Lambda(_) => return lambda_text_error(cell),
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
        CalcResult::String(result.text.into())
    }

    /// TEXTJOIN's delimiter: a scalar, or the elements of an array or range in row-major order,
    /// which cycle between the joined values (probes/textjoin-delimiter-array A1-A6, T-1510).
    /// Never empty.
    fn textjoin_delimiters(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<String>, CalcResult> {
        let value = self.evaluate_node_in_context(node, cell);
        if matches!(value, CalcResult::Lambda(_)) {
            return Err(lambda_text_error(cell));
        }
        match self.string_or_array(value, cell)? {
            StringOrArray::String(text) => Ok(vec![text.into_lossy_string()]),
            StringOrArray::Array(array) => {
                let mut delimiters = Vec::new();
                for element in array.iter().flatten() {
                    delimiters.push(array_node_to_string(element).map_err(|error| {
                        CalcResult::new_error(error, cell, "Error in delimiter".to_string())
                    })?);
                }
                if delimiters.is_empty() {
                    delimiters.push(String::new());
                }
                Ok(delimiters)
            }
        }
    }

    /// TEXTJOIN(delimiter, `ignore_empty`, text1, [text2], ...): joins text with a delimiter; over 32767
    /// characters is #CALC! (probes/text-result-limit C9).
    ///
    /// `ignore_empty` lifts over an array and a range, one join per element, while the delimiter
    /// and the texts are read once and handed to every element whole (lifting table, T-2318):
    /// `TEXTJOIN(",",{TRUE;FALSE},"a","","b")` spills "a,b" and "a,,b", and a delimiter error or a
    /// join over the limit is every element's result (probes/textjoin-ignore-empty-lift;
    /// docs/excel-semantics/arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments").
    pub(crate) fn fn_textjoin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !Function::Textjoin
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let delimiters = self.textjoin_delimiters(&args[0], cell);
        // A delimiter error is the result, so the texts are not read then. A direct single-cell
        // reference is flagged: its error past the limit is dropped (`textjoin_walk`).
        let texts: Vec<(CalcResult, bool)> = if delimiters.is_ok() {
            args[2..]
                .iter()
                .map(|arg| {
                    let value = self
                        .evaluate_reference_arg(arg, cell)
                        .into_single_area(cell);
                    (
                        value,
                        matches!(*arg.ungrouped_at(cell.row), Node::ReferenceKind { .. }),
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        // An element reads only as TRUE or FALSE, so at most two joins are built, whatever the
        // number of elements; the driver's text budget bounds their copies.
        let mut joins: [Option<CalcResult>; 2] = [None, None];
        self.lift_call(Function::Textjoin, args, cell, &mut |model, values| {
            let delimiters = match &delimiters {
                Ok(delimiters) => delimiters,
                Err(error) => return error.clone(),
            };
            let ignore_empty = match values.get(1).map(|value| Self::cast_bool_ref(value, cell)) {
                Some(Ok(ignore_empty)) => ignore_empty,
                Some(Err(error)) => return error,
                None => false,
            };
            joins[usize::from(ignore_empty)]
                .get_or_insert_with(|| model.textjoin_walk(delimiters, ignore_empty, &texts, cell))
                .clone()
        })
    }

    /// Walks `range` for [`Self::textjoin_walk`], pushing each cell, and each run of blank
    /// cells with `ignore_empty` FALSE, onto the join state `(size, values, over_limit)`.
    fn textjoin_push_range(
        &mut self,
        range: &Range,
        ignore_empty: bool,
        (size, values, over_limit): (&mut JoinSize, &mut Vec<String>, &mut bool),
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.for_each_cell_or_error_in_whole_range(range, cell, |_, item| match item {
            RangeItem::Cell(value) => size.push_cell(value, ignore_empty, values, over_limit, cell),
            RangeItem::Blanks(count) => {
                if !ignore_empty && !*over_limit {
                    *over_limit =
                        size.push_empties(values, i64::try_from(count).unwrap_or(i64::MAX));
                }
                Ok(())
            }
        })
    }

    /// Walks a one-sheet `left:right` for [`Self::textjoin_walk`] with `ignore_empty` FALSE.
    ///
    /// Excel joins every cell of a whole column or row, so the empty cells the clip leaves off the
    /// used area still add a delimiter each: `TEXTJOIN(",",FALSE,D:D)` is #CALC! (over 32,767) and
    /// `TEXTJOIN(",",FALSE,10:10)` has 16,383 delimiters (probes/whole-column-empty-walkers F1, F8;
    /// text.md).
    fn textjoin_push_padded_range(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        (size, values, over_limit): (&mut JoinSize, &mut Vec<String>, &mut bool),
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        let (row1, column1) = (left.row, left.column);
        let (full_row2, full_column2) = (right.row, right.column);
        let (row2, column2) =
            match self.clip_to_used_area(left.sheet, row1, column1, full_row2, full_column2) {
                Ok(clipped) => clipped,
                Err(message) => {
                    return Err(CalcResult::new_error(Error::ERROR, cell, message));
                }
            };
        // `max` covers a clip that ends before the range starts (nothing walked).
        let skipped_columns = i64::from(full_column2) - i64::from(column2.max(column1 - 1));
        let skipped_rows = i64::from(full_row2) - i64::from(row2.max(row1 - 1));
        let width = i64::from(full_column2) - i64::from(column1) + 1;
        // One walk per row, so the join stays in row-major order (the first error cell reached, and
        // the limit, depend on it) when columns are clipped, and so the walk can stop at the limit:
        // Excel reads no cell past it, so a range that includes the TEXTJOIN cell itself is #CALC!,
        // not circular (probes/textjoin-whole-rows-self-reference{,-full-width} B5; text.md). Past
        // the limit nothing more changes the result (cell errors are dropped).
        for row in row1..=row2 {
            if column2 >= column1 {
                let row_range = Range {
                    left: CellReferenceIndex {
                        row,
                        column: column1,
                        ..left
                    },
                    right: CellReferenceIndex {
                        row,
                        column: column2,
                        ..right
                    },
                };
                self.textjoin_push_range(
                    &row_range,
                    false,
                    (&mut *size, &mut *values, &mut *over_limit),
                    cell,
                )?;
            }
            if !*over_limit {
                *over_limit = size.push_empties(values, skipped_columns);
            }
            if *over_limit {
                return Ok(());
            }
        }
        if !*over_limit {
            *over_limit = size.push_empties(values, skipped_rows.saturating_mul(width));
        }
        Ok(())
    }

    /// One TEXTJOIN join of `texts` (each an argument's value, and whether it is a direct
    /// single-cell reference) with `delimiters` cycling between the joined values.
    fn textjoin_walk(
        &mut self,
        delimiters: &[String],
        ignore_empty: bool,
        texts: &[(CalcResult, bool)],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut values: Vec<String> = Vec::new();
        let mut size = JoinSize::new(delimiters);
        // Once over the limit the values are dropped, but arguments are still evaluated so an error
        // argument after the limit wins; a cell error read past it is dropped and evaluation goes on,
        // so the result is #CALC! unless a later error argument wins
        // (probes/text-result-limit C18-C20, -errors C2-C4).
        let mut over_limit = false;
        for (text, direct_reference) in texts {
            over_limit = over_limit || size.exceeds(&values);
            if over_limit {
                values.clear();
            }
            // A union stays `#VALUE!` here: TEXTJOIN is not probed with one (SPEC-0004).
            match text {
                CalcResult::Number(value) => values.push(number_to_text(*value)),
                &CalcResult::Range { left, right } => {
                    // A reference is walked sheet by sheet (a 3-D one spans several); an error cell
                    // past the limit is dropped like a range cell's (probes/text-result-limit-errors-refs
                    // D10-D11). With ignore_empty FALSE a blank cell adds a delimiter, so the blanks
                    // inside the used area are walked too, a run of them in a mostly blank area as
                    // one count, so the walk costs the stored cells (T-2710, T-2713).
                    let state = (&mut size, &mut values, &mut over_limit);
                    let walked = if !ignore_empty && left.sheet == right.sheet {
                        self.textjoin_push_padded_range(left, right, state, cell)
                    } else {
                        self.textjoin_push_range(&Range { left, right }, ignore_empty, state, cell)
                    };
                    if let Err(error) = walked {
                        return error;
                    }
                }
                // A direct single-cell reference is read like a range cell (probes/text-result-limit-errors
                // C3).
                CalcResult::Error { .. }
                    if *direct_reference
                        && (over_limit || size.exceeds_with_separator(&values)) =>
                {
                    over_limit = true;
                }
                error @ CalcResult::Error { .. } => return error.clone(),
                CalcResult::String(value) if ignore_empty && value.is_empty() => {}
                CalcResult::String(value) => values.push(value.clone().into_lossy_string()),
                CalcResult::Boolean(value) => {
                    if *value {
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
                // ignore_empty (probes/textjoin-array-arguments A1-A28, A30-A35); an error element past the 32,767
                // limit is dropped like a range cell's (probes/textjoin-array-limit-empty-strings A1-A5).
                CalcResult::Array(array) => {
                    for element in array.iter().flatten() {
                        let value = array_node_to_calc_result(element, cell);
                        if let Err(error) =
                            size.push_cell(value, ignore_empty, &mut values, &mut over_limit, cell)
                        {
                            return error;
                        }
                    }
                }
                CalcResult::Lambda(_) => return lambda_text_error(cell),
            };
        }
        if over_limit || size.exceeds(&values) {
            return text_limit_error(cell);
        }
        let mut joined = String::new();
        for (index, value) in values.iter().enumerate() {
            // Same cycling rule as `JoinSize::separator`, which sizes these delimiters.
            if let Some(gap) = index.checked_sub(1) {
                joined.push_str(&delimiters[gap % delimiters.len()]);
            }
            joined.push_str(value);
        }
        CalcResult::String(joined.into())
    }
}
