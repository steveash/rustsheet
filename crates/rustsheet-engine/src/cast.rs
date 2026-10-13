use crate::arithmetic::{bcast_idx, flush_subnormal};
use crate::formatter::date_text::CurrentYear;
use crate::number_format::{number_to_text, parse_excel_number};
use crate::reference_operators::{ExternalArea, Ranges, RefArg};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::{Error, OpUnary},
        types::CellReferenceIndex,
    },
    formatter::format::parse_formatted_number,
    functions::math_and_trigonometry::array_size::{check_array_size, check_range_size},
    model::Model,
};

pub(crate) enum NumberOrArray {
    Number(f64),
    Array(Vec<Vec<ArrayNode>>),
}

impl NumberOrArray {
    /// Rows and columns; a number is 1x1.
    pub(crate) fn dims(&self) -> (usize, usize) {
        match self {
            NumberOrArray::Number(_) => (1, 1),
            NumberOrArray::Array(a) => (a.len(), a.first().map_or(0, Vec::len)),
        }
    }
}

/// A scalar `CalcResult` or a 2-D array, used for element-wise comparison.
pub(crate) enum ValueOrArray {
    Value(CalcResult),
    Array(Vec<Vec<ArrayNode>>),
}

/// A scalar string or a 2-D array, used for element-wise text operations.
pub(crate) enum StringOrArray {
    String(String),
    Array(Vec<Vec<ArrayNode>>),
}

/// Collapses a scalar [`CalcResult`] into a single array element.
/// Ranges, arrays and lambdas cannot be a single element, so they become `#VALUE!`.
pub(crate) fn calc_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Number(n) => ArrayNode::Number(n),
        CalcResult::Boolean(b) => ArrayNode::Boolean(b),
        CalcResult::String(s) => ArrayNode::String(s),
        CalcResult::Error { error, .. } => ArrayNode::Error(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
        _ => ArrayNode::Error(Error::VALUE),
    }
}

/// Longest text a cell can hold, in UTF-16 code units: an emoji (a surrogate pair) counts two
/// (docs/excel-semantics/coercion.md "Text longer than 32,767 characters"). Also bounds the
/// allocation a concatenation can make (CLAUDE.md directive 5).
pub(crate) const MAX_TEXT_UNITS: usize = 32767;

/// Truncates `text` to [`MAX_TEXT_UNITS`] UTF-16 code units, as Excel does for CONCATENATE and
/// `&` (probes/resource-limits C9, C10; probes/concatenate-limit-spill, excel-graph). A character
/// that would straddle the limit is dropped whole: Excel keeps its lone high surrogate, which a
/// Rust `String` cannot hold; TODO(T-1483) keeps it.
pub(crate) fn truncate_text(text: &mut String) {
    // UTF-8 never takes fewer bytes than UTF-16 takes code units, so short text skips the scan.
    if text.len() <= MAX_TEXT_UNITS {
        return;
    }
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        units += ch.len_utf16();
        if units > MAX_TEXT_UNITS {
            text.truncate(byte);
            return;
        }
    }
}

/// Concatenates two strings, truncating the result to the cell text limit.
pub(crate) fn concat_text(a: &str, b: &str) -> String {
    let mut joined = String::with_capacity(a.len() + b.len());
    joined.push_str(a);
    joined.push_str(b);
    truncate_text(&mut joined);
    joined
}

/// Converts a single array element to a string using the same rules as
/// [`Model::cast_to_string`]. Errors propagate.
pub(crate) fn array_node_to_string(node: &ArrayNode) -> Result<String, Error> {
    match node {
        ArrayNode::Number(f) => Ok(number_to_text(*f)),
        ArrayNode::String(s) => Ok(s.clone()),
        ArrayNode::Boolean(b) => Ok(if *b {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }),
        ArrayNode::Empty => Ok("".to_string()),
        ArrayNode::Error(e) => Err(e.clone()),
    }
}

/// Longest text Excel reads as a number or as clock text, counted in characters with any
/// surrounding spaces: at 255 it parses, at 256 it is #VALUE! (docs/excel-semantics/numbers.md,
/// probes/text-number-long-zero, probes/clock-text-length-limit).
pub(crate) const MAX_NUMBER_TEXT_LEN: usize = 255;

/// True when `text`, counted with its surrounding spaces, is too long to read as a number.
pub(crate) fn number_text_too_long(text: &str) -> bool {
    text.len() > MAX_NUMBER_TEXT_LEN && text.chars().count() > MAX_NUMBER_TEXT_LEN
}

impl Model {
    /// Coerces a single array element to a number the way a scalar number argument is: booleans
    /// are 1 and 0, text goes through `cast_number` (`Data!D7:D7/1` with D7 = " 2 " is 2,
    /// gen-m1/1ce5b2467b3b19fd A14), empties are 0 and errors propagate.
    pub(crate) fn array_node_to_number(&self, node: &ArrayNode) -> Result<f64, Error> {
        match node {
            ArrayNode::Number(v) => Ok(*v),
            ArrayNode::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            ArrayNode::Empty => Ok(0.0),
            ArrayNode::String(s) => self.cast_number(s).ok_or(Error::VALUE),
            ArrayNode::Error(e) => Err(e.clone()),
        }
    }

    pub(crate) fn cast_number(&self, s: &str) -> Option<f64> {
        self.cast_number_in(s, &|| self.current_year())
    }

    /// [`Model::cast_number`] with year-less date text read in `current_year`.
    pub(crate) fn cast_number_in(&self, s: &str, current_year: CurrentYear) -> Option<f64> {
        if number_text_too_long(s) {
            return None;
        }
        match parse_excel_number(s.trim()) {
            Some(f) => Some(f),
            _ => {
                let currency = &self.locale.currency.symbol;
                let mut currencies = vec!["$", "€"];
                if !currencies.iter().any(|e| *e == currency) {
                    currencies.push(currency);
                }
                self.parse_number_text(s, &currencies, current_year)
            }
        }
    }

    /// Text as number coercion and VALUE read it, after their own trimming: a formatted number
    /// (dates, currencies in `currencies`, percentages), else time and date-time text
    /// (`0+"10:11:12.5"`; probes/timevalue-*). Number text below the smallest normal double
    /// reads as 0 like a literal, keeping `"1e-308"+0` at the Graph 0 of probes/text-number-words H6, H7,
    /// K1, K4, K5 (UNVERIFIED: Graph cannot tell a flush from a kept subnormal; T-1143).
    pub(crate) fn parse_number_text(
        &self,
        text: &str,
        currencies: &[&str],
        year: CurrentYear,
    ) -> Option<f64> {
        if let Ok((v, _number_format)) = parse_formatted_number(
            text,
            currencies,
            self.locale,
            year,
            self.workbook.settings.date_system,
        ) {
            return Some(crate::arithmetic::flush_subnormal(v));
        }
        crate::functions::date_and_time::time_text::parse_time_serial(
            text,
            self.locale,
            year,
            self.workbook.settings.date_system,
        )
    }
    /// Lifts the range `left:right` into a 2-D array of its cell values, for a function or
    /// operator that works on it element by element (or a formula that spills it).
    ///
    /// The size is checked before any cell is read: an untrusted formula such as
    /// `DATE(11:1899,12,1)` (31M cells) must not allocate, or record a read, per cell of an
    /// arbitrarily large range (CLAUDE.md directive 5). Past the bound it is `#ERROR!`
    /// (DEVIATION, docs/excel-semantics/arrays.md "Array results above 1,048,576 cells").
    /// A whole-row or whole-column range is read only within the sheet's used area; the
    /// rest is empty and goes on record as read empty in one rectangle (`clip_to_used_area`).
    pub(crate) fn evaluate_range(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        if let Some((error, message)) = check_range_size(&left, &right) {
            return Err(CalcResult::new_error(error, cell, message));
        }
        let (last_row, last_column) = self
            .clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
            .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
        let width = usize::try_from(right.column - left.column + 1).unwrap_or(0);
        let mut result = Vec::with_capacity(usize::try_from(right.row - left.row + 1).unwrap_or(0));
        for row in left.row..=right.row {
            let mut row_result = Vec::with_capacity(width);
            for column in left.column..=right.column {
                if row > last_row || column > last_column {
                    row_result.push(ArrayNode::Empty);
                    continue;
                }
                let value = match self.evaluate_cell(CellReferenceIndex {
                    sheet: left.sheet,
                    row,
                    column,
                }) {
                    CalcResult::Number(n) => ArrayNode::Number(n),
                    CalcResult::Boolean(b) => ArrayNode::Boolean(b),
                    CalcResult::String(s) => ArrayNode::String(s),
                    CalcResult::Error { error, .. } => ArrayNode::Error(error),
                    CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
                    CalcResult::Range { .. } | CalcResult::Array(_) | CalcResult::Lambda(_) => {
                        // A cell's value is never a range, array or lambda.
                        debug_assert!(false, "Unexpected array result in non-array formula");
                        ArrayNode::Error(Error::NIMPL)
                    }
                };
                row_result.push(value);
            }
            result.push(row_result);
        }
        Ok(result)
    }

    pub(crate) fn get_number_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<NumberOrArray, CalcResult> {
        let value = self.evaluate_node_in_context(node, cell);
        self.number_or_array(value, cell)
    }

    /// Applies `op` to `N` number arguments, element-wise when any is an array or range.
    /// Arguments broadcast like the arithmetic operators (a size-1 dimension repeats, positions
    /// past a shorter array are `#N/A`) and an element that is an error stays one. An argument
    /// that is an error is the result (the leftmost one), unless another is an array: then it
    /// is broadcast and every element is that error (as for DATE). `message` goes with a
    /// scalar error of `op`.
    pub(crate) fn lift_numbers<const N: usize>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        message: &str,
        op: impl Fn([f64; N]) -> Result<f64, Error>,
    ) -> CalcResult {
        debug_assert!(args.len() >= N, "callers check the argument count");
        let operands: Vec<_> = args
            .iter()
            .take(N)
            .map(|arg| self.get_number_or_array(arg, cell))
            .collect();
        self.lift_number_operands(operands, cell, message, op)
    }

    /// [`Model::lift_numbers`] of already evaluated operands (`N` of them), for a caller that
    /// reads some text its own way first (DATEDIF's date text in the workbook's date system).
    pub(crate) fn lift_number_operands<const N: usize>(
        &mut self,
        operands: Vec<Result<NumberOrArray, CalcResult>>,
        cell: CellReferenceIndex,
        message: &str,
        op: impl Fn([f64; N]) -> Result<f64, Error>,
    ) -> CalcResult {
        debug_assert!(operands.len() == N, "one operand per number");
        let any_array = operands
            .iter()
            .any(|na| matches!(na, Ok(NumberOrArray::Array(_))));
        let mut lifted = Vec::with_capacity(N);
        for na in operands {
            match na {
                Err(CalcResult::Error { error, .. }) if any_array => {
                    lifted.push(NumberOrArray::Array(vec![vec![ArrayNode::Error(error)]]));
                }
                Ok(na) => lifted.push(na),
                Err(e) => return e,
            }
        }
        if !any_array {
            // Without an array every operand is a `Number` (an error returned above).
            let mut numbers = [0.0; N];
            for (slot, na) in numbers.iter_mut().zip(&lifted) {
                if let NumberOrArray::Number(n) = na {
                    *slot = *n;
                }
            }
            return match op(numbers) {
                Ok(n) => CalcResult::Number(n),
                Err(e) => CalcResult::new_error(e, cell, message.to_string()),
            };
        }
        let rows = lifted.iter().map(|na| na.dims().0).max().unwrap_or(0);
        let cols = lifted.iter().map(|na| na.dims().1).max().unwrap_or(0);
        // Bounds the broadcast result (`ROUND(A:A, 1:1)`): DEVIATION, arrays.md "Array results
        // above 1,048,576 cells".
        if let Some((error, message)) = check_array_size(rows, cols) {
            return CalcResult::new_error(error, cell, message);
        }
        let result = (0..rows)
            .map(|i| {
                (0..cols)
                    .map(|j| {
                        let mut numbers = [0.0; N];
                        let mut value = Ok(());
                        for (slot, na) in numbers.iter_mut().zip(&lifted) {
                            match self.number_at(na, i, j) {
                                Ok(n) => *slot = n,
                                Err(e) => {
                                    value = Err(e);
                                    break;
                                }
                            }
                        }
                        match value.and_then(|()| op(numbers)) {
                            Ok(n) => ArrayNode::Number(n),
                            Err(e) => ArrayNode::Error(e),
                        }
                    })
                    .collect()
            })
            .collect();
        CalcResult::Array(result)
    }

    /// The number at (`i`, `j`) of `na`, a size-1 dimension repeating; text is cast like a scalar
    /// argument, a position past a shorter array is `#N/A`.
    pub(crate) fn number_at(&self, na: &NumberOrArray, i: usize, j: usize) -> Result<f64, Error> {
        match na {
            NumberOrArray::Number(f) => Ok(*f),
            NumberOrArray::Array(a) => {
                let (rows, cols) = na.dims();
                let node = bcast_idx(rows, i)
                    .and_then(|ri| a.get(ri))
                    .and_then(|r| bcast_idx(cols, j).and_then(|cj| r.get(cj)));
                match node {
                    None => Err(Error::NA),
                    Some(node) => self.array_node_to_number(node),
                }
            }
        }
    }

    /// [`Model::get_number_or_array`] of an already evaluated `value`.
    pub(crate) fn number_or_array(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<NumberOrArray, CalcResult> {
        match value {
            CalcResult::Number(f) => Ok(NumberOrArray::Number(f)),
            CalcResult::String(s) => match self.cast_number(&s) {
                Some(f) => Ok(NumberOrArray::Number(f)),
                None => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expecting number".to_string(),
                )),
            },
            CalcResult::Boolean(f) => {
                if f {
                    Ok(NumberOrArray::Number(1.0))
                } else {
                    Ok(NumberOrArray::Number(0.0))
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(NumberOrArray::Number(0.0)),
            CalcResult::Range { left, right } => {
                let sheet = left.sheet;
                if sheet != right.sheet {
                    return Err(CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "3D ranges are not allowed".to_string(),
                    });
                }
                Ok(NumberOrArray::Array(
                    self.evaluate_range(left, right, cell)?,
                ))
            }
            CalcResult::Array(s) => Ok(NumberOrArray::Array(s)),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "LAMBDA cannot be used as a number".to_string(),
            )),
        }
    }

    /// Like `get_number_or_array` but preserves the full `CalcResult` type (no coercion to f64).
    /// Used for element-wise comparison operators.
    pub(crate) fn get_value_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<ValueOrArray, CalcResult> {
        let value = self.evaluate_node_in_context(node, cell);
        self.value_or_array(value, cell)
    }

    /// [`Model::get_value_or_array`] of an already evaluated `value`.
    pub(crate) fn value_or_array(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<ValueOrArray, CalcResult> {
        match value {
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => {
                Ok(ValueOrArray::Array(self.evaluate_range(left, right, cell)?))
            }
            CalcResult::Array(arr) => Ok(ValueOrArray::Array(arr)),
            other => Ok(ValueOrArray::Value(other)),
        }
    }

    /// Like `get_number_or_array` but for text: scalars are coerced to a `String`,
    /// ranges and arrays are returned as a 2-D array (to be processed element-wise).
    pub(crate) fn get_string_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<StringOrArray, CalcResult> {
        let value = self.evaluate_node_in_context(node, cell);
        self.string_or_array(value, cell)
    }

    /// [`Model::get_string_or_array`] of an already evaluated `value`.
    pub(crate) fn string_or_array(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<StringOrArray, CalcResult> {
        match value {
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => Ok(StringOrArray::Array(
                self.evaluate_range(left, right, cell)?,
            )),
            CalcResult::Array(arr) => Ok(StringOrArray::Array(arr)),
            other => Ok(StringOrArray::String(self.cast_to_string(other, cell)?)),
        }
    }

    /// Evaluates a unary `-` or `%` node. A chain such as `B:B%%%` or `---B:B` is applied
    /// in one element-wise pass, so a whole column is lifted once rather than once per
    /// operator (the `formula_eval` fuzz timeout, T-1078). The operators commute (negation is
    /// exact and the subnormal flush is sign-symmetric), so only the minus parity and the
    /// `%` count matter, and the walk allocates nothing.
    pub(crate) fn apply_unary_chain(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut negate = false;
        let mut percents = 0_usize;
        let mut operand = node;
        while let Node::UnaryKind { kind, right } = operand {
            match kind {
                OpUnary::Minus => negate = !negate,
                OpUnary::Percentage => percents += 1,
            }
            operand = right;
        }
        self.apply_number_unary(operand, cell, |mut f| {
            // A subnormal quotient is 0 like for `/` (numbers.md "Subnormal results
            // become 0", probes/subnormal-function-results A1, A21); 0 stays 0, so blank
            // cells stop early.
            for _ in 0..percents {
                if f == 0.0 {
                    break;
                }
                f = flush_subnormal(f / 100.0);
            }
            Ok(if negate { -f } else { f })
        })
    }

    /// Applies a scalar numeric transform element-wise over a node that may evaluate
    /// to a number, a range, or an array. Mirrors the `single_number_fn!` /
    /// `date_part_fn!` macros but accepts a closure, so callers can capture extra
    /// scalar arguments (e.g. WEEKDAY's `return_type`) or use non-arithmetic
    /// transforms (e.g. date helpers).
    ///
    /// Element coercion matches the rest of the array machinery: booleans → 0/1,
    /// empty → 0, strings are parsed with `cast_number` (`#VALUE!` if they don't
    /// parse), and errors propagate.
    pub(crate) fn apply_number_unary(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        op: impl Fn(f64) -> Result<f64, Error>,
    ) -> CalcResult {
        match self.get_number_or_array(node, cell) {
            Ok(NumberOrArray::Number(f)) => match op(f) {
                Ok(x) => CalcResult::Number(x),
                Err(e) => CalcResult::new_error(e, cell, String::new()),
            },
            // Rewritten in place: a whole column is a million one-element rows, and
            // reallocating each of them per operator dominated the cost (T-1078).
            Ok(NumberOrArray::Array(mut array)) => {
                for value in array.iter_mut().flatten() {
                    let n = match value {
                        ArrayNode::Number(n) => Some(*n),
                        ArrayNode::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
                        ArrayNode::Empty => Some(0.0),
                        ArrayNode::String(s) => self.cast_number(s),
                        ArrayNode::Error(_) => continue,
                    };
                    *value = match n {
                        Some(f) => match op(f) {
                            Ok(x) => ArrayNode::Number(x),
                            Err(e) => ArrayNode::Error(e),
                        },
                        None => ArrayNode::Error(Error::VALUE),
                    };
                }
                CalcResult::Array(array)
            }
            Err(e) => e,
        }
    }

    pub(crate) fn get_number(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.cast_to_number(result, cell)
    }

    /// A reference where a value is expected (typically returned by INDIRECT,
    /// OFFSET or INDEX inside a scalar argument) is dereferenced: a single cell
    /// gives its value, a larger range is intersected with the row or column of
    /// `cell`, as Excel does in a scalar context. Literal ranges never get here:
    /// static analysis wraps them in `@` at parse time. ISEVEN/ISODD call it too.
    ///
    /// A 1x1 array (the result of arithmetic on such a reference, for instance
    /// `INDIRECT("C3")*1`) is a scalar wrapped in an array and unwraps likewise.
    pub(crate) fn dereference(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match result {
            CalcResult::Range { .. } => self.implicit_intersection_to_value(result, cell),
            CalcResult::Array(ref array) if array.len() == 1 && array[0].len() == 1 => {
                match &array[0][0] {
                    ArrayNode::Number(n) => CalcResult::Number(*n),
                    ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
                    ArrayNode::String(s) => CalcResult::String(s.clone()),
                    ArrayNode::Error(error) => CalcResult::new_error(
                        error.clone(),
                        cell,
                        error.to_localized_error_string(self.language),
                    ),
                    ArrayNode::Empty => CalcResult::EmptyCell,
                }
            }
            other => other,
        }
    }

    /// The `#N/A` an aggregate gives for a direct text argument of more than 255 characters,
    /// numeric or not (`None` for shorter text), where every other path gives `#VALUE!`
    /// (probes/sum-long-text-args, probes/long-text-args-min-criteria, excel-graph;
    /// docs/excel-semantics/numbers.md).
    pub(crate) fn aggregate_text_too_long(
        text: &str,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        number_text_too_long(text)
            .then(|| CalcResult::new_error(Error::NA, cell, "Text too long".to_string()))
    }

    /// [`Model::cast_to_number`] for a direct argument of the aggregates (SUM, PRODUCT, MIN, MAX,
    /// AVERAGE, SUMSQ, ...; AVERAGEA, MAXA and MINA via [`Model::aggregate_text_too_long`]).
    pub(crate) fn cast_aggregate_arg_to_number(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        if let CalcResult::String(text) = &result {
            if let Some(error) = Self::aggregate_text_too_long(text, cell) {
                return Err(error);
            }
        }
        self.cast_to_number(result, cell)
    }

    pub(crate) fn cast_to_number(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = self.dereference(result, cell);
        match result {
            CalcResult::Number(f) => Ok(f),
            CalcResult::String(s) => match self.cast_number(&s) {
                Some(f) => Ok(f),
                None => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expecting number".to_string(),
                )),
            },
            CalcResult::Boolean(f) => {
                if f {
                    Ok(1.0)
                } else {
                    Ok(0.0)
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(0.0),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::Array(_) | CalcResult::Lambda(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
        }
    }

    pub(crate) fn get_number_no_bools(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        let result = self.dereference(result, cell);
        if matches!(result, CalcResult::Boolean(_)) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expecting number".to_string(),
            ));
        }
        self.cast_to_number(result, cell)
    }

    pub(crate) fn get_string(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<String, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.cast_to_string(result, cell)
    }

    pub(crate) fn cast_to_string(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<String, CalcResult> {
        let result = self.dereference(result, cell);
        match result {
            CalcResult::Number(f) => Ok(number_to_text(f)),
            CalcResult::String(s) => Ok(s),
            CalcResult::Boolean(f) => {
                if f {
                    Ok("TRUE".to_string())
                } else {
                    Ok("FALSE".to_string())
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok("".to_string()),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::Array(_) | CalcResult::Lambda(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
        }
    }

    pub(crate) fn get_boolean(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.cast_to_bool(result, cell)
    }

    pub(crate) fn cast_to_bool(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        let result = self.dereference(result, cell);
        match result {
            CalcResult::Number(f) => {
                if f == 0.0 {
                    return Ok(false);
                }
                Ok(true)
            }
            CalcResult::String(s) => {
                if s.to_lowercase() == *"true" {
                    return Ok(true);
                } else if s.to_lowercase() == *"false" {
                    return Ok(false);
                }
                Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Expected boolean".to_string(),
                })
            }
            CalcResult::Boolean(b) => Ok(b),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(false),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::Array(_) | CalcResult::Lambda(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
        }
    }

    // tries to return a reference. That is either a reference or a formula that evaluates to a range/reference
    pub(crate) fn get_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        // Reference context: references stay references and an `@` node yields
        // the intersected 1x1 reference (e.g. COLUMNS(@A1:C1) is 1).
        let value = self.evaluate_node_with_reference(node, cell);
        if value.is_error() {
            return Err(value);
        }
        if let CalcResult::Range { left, right } = value {
            Ok(Range { left, right })
        } else {
            Err(CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Expected reference".to_string(),
            })
        }
    }

    /// Rows and columns of the reference or array that ROWS/COLUMNS count. An array
    /// is counted, never read: `ROWS(NA()-{1;2;3})` is 3 although every element is
    /// `#N/A` (probes/error-operand-broadcast C1, C3, C6-C8, C10). A non-reference,
    /// non-array argument counts as 1x1 (probes/rows-columns-args C1-C5, C12).
    pub(crate) fn rows_columns_extent(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<(usize, usize), CalcResult> {
        if let Some(area) = self.one_external_area(node, cell) {
            return area.map(ExternalArea::extent);
        }
        // ROWS/COLUMNS of a reference of several areas is #REF!, not the #VALUE! of a value
        // context (probes/multiarea-count H2, H3; SPEC-0004 row 5).
        let result = if self.is_multi_area(node) {
            match self.evaluate_reference_arg(node, cell) {
                RefArg::Ranges(Ranges::Areas(_)) => {
                    return Err(CalcResult::new_error(
                        Error::REF,
                        cell,
                        "Reference of several areas".to_string(),
                    ))
                }
                // A 3-D span through a variable, IF or CHOOSE is `#VALUE!`, as the written one
                // is (probes/3d-ref-let-bound-span C7:C8, 3d-ref-let-values C5).
                other => other.into_single_sheet_area(cell),
            }
        } else {
            self.evaluate_node_with_reference(node, cell)
        };
        match result {
            CalcResult::Range { left, right } => Ok((
                (right.row - left.row + 1) as usize,
                (right.column - left.column + 1) as usize,
            )),
            CalcResult::Array(array) => Ok((array.len(), array.first().map_or(0, Vec::len))),
            error @ CalcResult::Error { .. } => Err(error),
            // Excel: a scalar is 1x1 (probes/rows-columns-args); other kinds stay unprobed.
            CalcResult::Number(_) | CalcResult::Boolean(_) | CalcResult::String(_) => Ok((1, 1)),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected reference or array".to_string(),
            )),
        }
    }
}
