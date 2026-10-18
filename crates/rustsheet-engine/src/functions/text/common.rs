use super::text_arg::{broadcast_text, TextArg};
use super::util::substitute_element;
use crate::cast::number_text_too_long;
use crate::number_format::number_to_text;
use crate::{
    arithmetic::{array_of_errors, own_error},
    calc_result::CalcResult,
    cast::{array_node_to_string, array_node_to_text, calc_result_to_array_node, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    formatter::{
        exceeds_complexity_limit,
        format::{format_text, format_text_value_exact, FormatContext, Formatted},
    },
    functions::{
        text::position::find_element,
        util::{array_node_to_calc_result, near_integer_count},
    },
    model::Model,
    number_format::to_precision,
    text::{Char, Text},
};

/// Computes a single LEFT result from one element of each argument.
// Excel: LEFT and RIGHT count characters (code points), unlike LEN, MID, FIND and REPLACE, which
// count UTF-16 code units: `LEFT("a😀b",3)` is "a😀b" (docs/excel-semantics/text.md "Text
// positions count UTF-16 code units", probes/text-surrogate-positions A3, B1-B5).
fn left_element(text: &ArrayNode, num: Result<f64, Error>) -> Result<ArrayNode, Error> {
    let s = array_node_to_text(text)?;
    let num = num?;
    // Excel: any non-negative count works, however large (`LEFT("abc",1E+100)` is "abc"), and
    // -1E-300 is already #VALUE! (probes/text-count-argument-bounds F14, N18).
    if num < 0.0 {
        return Err(Error::VALUE);
    }
    let num_chars = near_integer_count(num) as usize;
    Ok(ArrayNode::String(s.left_chars(num_chars)))
}

/// Computes a single RIGHT result from one element of each argument.
fn right_element(text: &ArrayNode, num: Result<f64, Error>) -> Result<ArrayNode, Error> {
    let s = array_node_to_text(text)?;
    let num = num?;
    if num < 0.0 {
        return Err(Error::VALUE);
    }
    let num_chars = near_integer_count(num) as usize;
    Ok(ArrayNode::String(s.right_chars(num_chars)))
}

/// MID's `start_num` and `num_chars` must be below 2^31 (they are read as 32-bit integers).
const MID_ARGUMENT_LIMIT: f64 = 2_147_483_648.0;

/// Computes a single MID result from one element of each argument.
fn mid_element(
    text: &ArrayNode,
    start: Result<f64, Error>,
    length: Result<f64, Error>,
) -> Result<ArrayNode, Error> {
    let s = array_node_to_text(text)?;
    let start = start?;
    let length = length?;
    // Excel: MID truncates both numbers exactly (`MID(t,1,1-1E-15)` is "") and either one
    // from 2^31 up is #VALUE!, while 2^31-1 is fine (probes/text-count-argument-bounds C, D, H, I).
    if !(1.0..MID_ARGUMENT_LIMIT).contains(&start.floor()) {
        return Err(Error::VALUE);
    }
    if !(0.0..MID_ARGUMENT_LIMIT).contains(&length.floor()) {
        return Err(Error::VALUE);
    }
    // Excel: MID positions count UTF-16 code units, so `MID("a😀b",2,2)` is the emoji
    // (probes/text-surrogate-positions A9-A13).
    let first = (start.floor() as usize).saturating_sub(1);
    let end = first.saturating_add(length.floor() as usize);
    Ok(ArrayNode::String(s.slice_utf16(first, end)))
}

/// Computes a single LEN result.
fn len_element(text: &ArrayNode) -> Result<ArrayNode, Error> {
    let s = array_node_to_string(text)?;
    // Excel: LEN counts UTF-16 code units, so an emoji counts two (probes/text-surrogate-positions A1).
    Ok(ArrayNode::Number(s.encode_utf16().count() as f64))
}

/// Computes a single REPT result from one element of each argument.
fn rept_element(text: &ArrayNode, number_times: Result<f64, Error>) -> Result<ArrayNode, Error> {
    let text = array_node_to_text(text)?;
    let number_times = number_times?.floor();
    // Excel's cell text limit, counted in UTF-16 code units (probes/concatenate-limit-spill E12);
    // it also bounds the allocation (CLAUDE.md directive 5).
    let units = text.len_utf16() as f64;
    // A NaN or infinite count is #VALUE! on its own, not by way of the limit below.
    if !number_times.is_finite()
        || number_times < 0.0
        || units * number_times > MAX_TEXT_UNITS as f64
    {
        return Err(Error::VALUE);
    }
    // Repeats the exact units, so a lone surrogate survives (probes/text-lone-surrogates B1-B6).
    Ok(ArrayNode::String(text.repeat(number_times as usize)))
}

/// Computes a single EXACT result from one element of each argument.
fn exact_element(text1: &ArrayNode, text2: &ArrayNode) -> Result<ArrayNode, Error> {
    if let (ArrayNode::Number(number1), ArrayNode::Number(number2)) = (text1, text2) {
        // In Excel two numbers are the same if they are the same up to 15 digits.
        return Ok(ArrayNode::Boolean(
            to_precision(*number1, 15) == to_precision(*number2, 15),
        ));
    }
    let string1 = array_node_to_text(text1)?;
    let string2 = array_node_to_text(text2)?;
    // `Text` equality is exact: a lone surrogate never equals U+FFFD (probes/text-lone-surrogates A10-A13).
    Ok(ArrayNode::Boolean(string1 == string2))
}

/// A formatted TEXT result as an array element: its text, or `#VALUE!` for a bad format.
fn formatted_node(d: Formatted) -> ArrayNode {
    d.error.map_or(ArrayNode::String(d.text.into()), |_| {
        ArrayNode::Error(Error::VALUE)
    })
}

/// The TEXT array element for a number (`None`: a blank).
fn text_node(value: Option<f64>, format: &str, ctx: &FormatContext) -> ArrayNode {
    formatted_node(format_text(value, format, ctx))
}

/// The TEXT array element for a text or boolean value: the format's text section applied to its
/// text ("TRUE"/"FALSE" for a boolean).
fn text_value_node(value: &ArrayNode, format: &str) -> ArrayNode {
    match array_node_to_text(value) {
        // The exact text, so a lone surrogate lands where `@` is (probes/text-lone-surrogates).
        Ok(text) => format_text_value_exact(&text, format)
            .map_or(ArrayNode::Error(Error::VALUE), ArrayNode::String),
        Err(error) => ArrayNode::Error(error),
    }
}

/// The TEXT array element for one value: numbers and blanks by the number sections, text and
/// booleans by the text section, an error element stays as it is.
fn text_element(node: &ArrayNode, format: &str, ctx: &FormatContext) -> ArrayNode {
    match node {
        ArrayNode::Number(f) => text_node(Some(*f), format, ctx),
        ArrayNode::Empty => text_node(None, format, ctx),
        text @ (ArrayNode::Boolean(_) | ArrayNode::String(_)) => text_value_node(text, format),
        e @ ArrayNode::Error(_) => e.clone(),
    }
}

/// TEXT of a scalar text or boolean value.
fn text_value_result(value: &ArrayNode, format: &str, cell: CellReferenceIndex) -> CalcResult {
    match text_value_node(value, format) {
        ArrayNode::String(s) => CalcResult::String(s),
        _ => CalcResult::Error {
            error: Error::VALUE,
            origin: cell,
            message: "Invalid format code".to_string(),
        },
    }
}

/// TEXT's `#VALUE!` for a format code over Excel's complexity limit (T-1393).
fn too_complex(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "Format code too complex".to_string())
}

impl Model {
    pub(crate) fn fn_text(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value_result = self.evaluate_node_in_context(&args[0], cell);
        // Normalise a range to an array so both broadcast (spill) the same way.
        let value_result = match value_result {
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) => CalcResult::Array(array),
                Err(error) => return error,
            },
            other => other,
        };
        // The format is evaluated once: evaluating it again per path would make TEXT nested in
        // the format argument exponential in its depth.
        let format_result = match self.evaluate_node_in_context(&args[1], cell) {
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) => CalcResult::Array(array),
                Err(error) => return error,
            },
            other => other,
        };
        // An array or range of format codes gives one result per format; each format has its own
        // complexity check and an error element keeps its error (probes/text-format-length-limit-f
        // A54:B54).
        if let CalcResult::Array(formats) = format_result {
            let value = match value_result {
                CalcResult::Array(a) => TextArg::Array(a),
                // An error value lifts over the formats like a scalar.
                err @ CalcResult::Error { .. } => {
                    TextArg::Error(calc_result_to_array_node(err.clone()), err)
                }
                other => TextArg::Scalar(calc_result_to_array_node(other)),
            };
            let formats = TextArg::Array(formats);
            let ctx = self.format_context();
            return broadcast_text(cell, &[&value, &formats], |i, j| {
                // An error value wins over a bad or too complex format (number-formats.md).
                if let e @ ArrayNode::Error(_) = value.elem(i, j) {
                    return Ok(e.clone());
                }
                let format_code = array_node_to_string(formats.elem(i, j))?;
                if exceeds_complexity_limit(&format_code) {
                    return Err(Error::VALUE);
                }
                Ok(text_element(value.elem(i, j), &format_code, &ctx))
            });
        }
        let format_code = self.cast_to_string(format_result, cell);
        match value_result {
            CalcResult::Array(arr) => {
                let format_code = match format_code {
                    Ok(s) => s,
                    // An error in every element; error elements keep theirs.
                    Err(e) => return array_of_errors(&arr, e, own_error),
                };
                if exceeds_complexity_limit(&format_code) {
                    return array_of_errors(&arr, too_complex(cell), own_error);
                }
                let ctx = self.format_context();
                CalcResult::Array(
                    arr.iter()
                        .map(|row| {
                            row.iter()
                                .map(|node| text_element(node, &format_code, &ctx))
                                .collect()
                        })
                        .collect(),
                )
            }
            error @ CalcResult::Error { .. } => error,
            other => {
                // An error in the format wins, also for text (error_broadcast.xlsx Sheet2!A1).
                let format_code = match format_code {
                    Ok(s) => s,
                    Err(s) => return s,
                };
                if exceeds_complexity_limit(&format_code) {
                    return too_complex(cell);
                }
                let is_blank = matches!(other, CalcResult::EmptyCell);
                let value = match other {
                    CalcResult::Number(f) => f,
                    CalcResult::String(s) => {
                        return text_value_result(&ArrayNode::String(s), &format_code, cell)
                    }
                    CalcResult::Boolean(b) => {
                        return text_value_result(&ArrayNode::Boolean(b), &format_code, cell)
                    }
                    CalcResult::Range { .. } => {
                        return CalcResult::Error {
                            error: Error::NIMPL,
                            origin: cell,
                            message: "Implicit Intersection not implemented".to_string(),
                        };
                    }
                    CalcResult::EmptyCell | CalcResult::EmptyArg => 0.0,
                    error @ CalcResult::Error { .. } => return error,
                    CalcResult::Array(_) | CalcResult::Lambda(_) => unreachable!(),
                };
                let d = format_text(
                    (!is_blank).then_some(value),
                    &format_code,
                    &self.format_context(),
                );
                if let Some(_e) = d.error {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Invalid format code".to_string(),
                    };
                }
                CalcResult::String(d.text.into())
            }
        }
    }

    /// `FIND(find_text`, `within_text`, [`start_num`])
    ///  * FIND and FINDB are case sensitive and don't allow wildcard characters.
    ///  * If `find_text` is "" (empty text), FIND matches the first character in the search string (that is, the character numbered `start_num` or 1).
    ///  * `Find_text` cannot contain any wildcard characters.
    ///  * If `find_text` does not appear in `within_text`, FIND and FINDB return the #VALUE! error value.
    ///  * If `start_num` is not greater than zero, FIND and FINDB return the #VALUE! error value.
    ///  * If `start_num` is greater than the length of `within_text`, FIND and FINDB return the #VALUE! error value.
    ///    NB: FINDB is not implemented. It is the same as FIND function unless locale is a DBCS (Double Byte Character Set)
    ///  * If any of the arguments is an array it works element by element.
    pub(crate) fn fn_find(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.find_or_search(args, cell, false)
    }

    /// Same API as FIND but:
    ///  * Allows wildcards
    ///  * It is case insensitive
    ///    `SEARCH(find_text`, `within_text`, [`start_num`])
    pub(crate) fn fn_search(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.find_or_search(args, cell, true)
    }

    fn find_or_search(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        is_search: bool,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let find_text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let within_text = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let start_num = if args.len() == 3 {
            match self.text_arg(&args[2], cell) {
                Ok(o) => o,
                Err(e) => return e,
            }
        } else {
            TextArg::Scalar(ArrayNode::Number(1.0))
        };
        broadcast_text(cell, &[&find_text, &within_text, &start_num], |i, j| {
            find_element(
                find_text.elem(i, j),
                within_text.elem(i, j),
                self.array_node_to_number(start_num.elem(i, j)),
                is_search,
            )
        })
    }

    // LEN, LEFT, RIGHT, MID, LOWER, UPPER, TRIM
    pub(crate) fn fn_len(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text], |i, j| len_element(text.elem(i, j)))
    }

    pub(crate) fn fn_trim(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Excel TRIM operates on the ASCII space (0x20) only: it removes
        // leading and trailing spaces and collapses each internal run of two
        // or more spaces to a single space. It must NOT touch tab (0x09),
        // non-breaking space (0xA0), or any other Unicode whitespace, so we
        // deliberately split on ' ' rather than using `str::trim`.
        self.apply_text_unary_text(args, cell, Text::trim_spaces)
    }

    pub(crate) fn fn_lower(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, super::string_format::lower_case)
    }

    /// Applies a single-argument string transform element-wise.
    /// A scalar argument yields a single String; a range or array argument yields
    /// an array (which spills, or is consumed element-wise in array contexts such
    /// as SUMPRODUCT). Used by text functions like UPPER and LOWER.
    pub(crate) fn apply_text_unary(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        f: impl Fn(&str) -> String,
    ) -> CalcResult {
        self.apply_text_unary_text(args, cell, |text| text.map_runs(&f))
    }

    /// [`Model::apply_text_unary`] for a transform that reads the exact text, so it can keep a
    /// lone surrogate.
    pub(crate) fn apply_text_unary_text(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        f: impl Fn(&Text) -> Text,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.get_string_or_array(&args[0], cell) {
            Ok(crate::cast::StringOrArray::String(s)) => CalcResult::String(f(&s)),
            Ok(crate::cast::StringOrArray::Array(arr)) => CalcResult::Array(
                arr.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| match crate::cast::array_node_to_text(n) {
                                Ok(s) => crate::expressions::parser::ArrayNode::String(f(&s)),
                                Err(e) => crate::expressions::parser::ArrayNode::Error(e),
                            })
                            .collect()
                    })
                    .collect(),
            ),
            Err(e) => e,
        }
    }

    pub(crate) fn fn_unicode(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 1 {
            let s = match self.evaluate_node_in_context(&args[0], cell) {
                CalcResult::Number(v) => Text::from(number_to_text(v)),
                CalcResult::String(v) => v,
                CalcResult::Boolean(b) => Text::from(if b { "TRUE" } else { "FALSE" }),
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Range { .. } => {
                    // Implicit Intersection not implemented
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Implicit Intersection not implemented".to_string(),
                    };
                }
                CalcResult::EmptyCell | CalcResult::EmptyArg => {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Empty cell".to_string(),
                    }
                }
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };

            // Excel: a lone high surrogate is #VALUE!, a lone low one gives its code
            // (probes/text-lone-surrogates A3, A4; text-surrogate-positions A38).
            match s.chars_lossless().next() {
                Some(Char::Scalar(c)) => return CalcResult::Number(f64::from(c as u32)),
                Some(Char::Lone(u)) if !(0xD800..0xDC00).contains(&u) => {
                    return CalcResult::Number(f64::from(u));
                }
                Some(Char::Lone(_)) => {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Lone high surrogate".to_string(),
                    };
                }
                None => {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Empty cell".to_string(),
                    };
                }
            }
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_upper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, super::string_format::upper_case)
    }

    /// Evaluates an argument of a text function into a scalar value or a 2-D array of
    /// values. References to a single cell collapse to a scalar; multi-cell ranges
    /// and array literals/spills become arrays that the result is broadcast over.
    pub(crate) fn text_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<TextArg, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            err @ CalcResult::Error { .. } => {
                Ok(TextArg::Error(calc_result_to_array_node(err.clone()), err))
            }
            CalcResult::Range { left, right } => {
                Ok(TextArg::Array(self.evaluate_range(left, right, cell)?))
            }
            CalcResult::Array(a) => Ok(TextArg::Array(a)),
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expecting a value".to_string(),
            )),
            other => Ok(TextArg::Scalar(calc_result_to_array_node(other))),
        }
    }

    pub(crate) fn fn_left(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let num = if args.len() == 2 {
            match self.text_arg(&args[1], cell) {
                Ok(o) => o,
                Err(e) => return e,
            }
        } else {
            TextArg::Scalar(ArrayNode::Number(1.0))
        };
        broadcast_text(cell, &[&text, &num], |i, j| {
            left_element(text.elem(i, j), self.array_node_to_number(num.elem(i, j)))
        })
    }

    pub(crate) fn fn_right(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let num = if args.len() == 2 {
            match self.text_arg(&args[1], cell) {
                Ok(o) => o,
                Err(e) => return e,
            }
        } else {
            TextArg::Scalar(ArrayNode::Number(1.0))
        };
        broadcast_text(cell, &[&text, &num], |i, j| {
            right_element(text.elem(i, j), self.array_node_to_number(num.elem(i, j)))
        })
    }

    pub(crate) fn fn_mid(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let start = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let length = match self.text_arg(&args[2], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text, &start, &length], |i, j| {
            mid_element(
                text.elem(i, j),
                self.array_node_to_number(start.elem(i, j)),
                self.array_node_to_number(length.elem(i, j)),
            )
        })
    }

    // REPT(text, number_times)
    pub(crate) fn fn_rept(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let number_times = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text, &number_times], |i, j| {
            rept_element(
                text.elem(i, j),
                self.array_node_to_number(number_times.elem(i, j)),
            )
        })
    }

    // SUBSTITUTE(text, old_text, new_text, [instance_num])
    pub(crate) fn fn_substitute(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=4).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let old_text = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let new_text = match self.text_arg(&args[2], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        // Without it every instance is replaced
        let instance_num = if arg_count > 3 {
            match self.text_arg(&args[3], cell) {
                Ok(o) => Some(o),
                Err(e) => return e,
            }
        } else {
            None
        };
        let mut operands = vec![&text, &old_text, &new_text];
        if let Some(instance_num) = &instance_num {
            operands.push(instance_num);
        }
        broadcast_text(cell, &operands, |i, j| {
            substitute_element(
                text.elem(i, j),
                old_text.elem(i, j),
                new_text.elem(i, j),
                instance_num
                    .as_ref()
                    .map(|n| self.array_node_to_number(n.elem(i, j))),
            )
        })
    }
    // EXACT(text1, text2)
    // If any of the arguments is an array it compares element by element.
    pub(crate) fn fn_exact(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let text1 = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let text2 = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text1, &text2], |i, j| {
            exact_element(text1.elem(i, j), text2.elem(i, j))
        })
    }
    // VALUE(text)
    pub(crate) fn fn_value(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts over a range or array, each element read as a scalar argument: a blank is 0, a
        // boolean or "" #VALUE! and an error stays (probes/value-argument-lift C1:C8, D1:D6;
        // corpus-bench/42181 B14, T-1791).
        let text = match self.text_arg(&args[0], cell) {
            Ok(t) => t,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text], |i, j| match text.elem(i, j) {
            ArrayNode::String(text) if number_text_too_long(text) => Err(Error::VALUE),
            ArrayNode::String(text) => {
                // VALUE trims spaces around date and clock text, unlike number coercion, but
                // only the space character: `VALUE("1/2/2020\u{a0}")` is #VALUE!
                // (probes/date-text-grammar C7, value-clock-text B14, B44).
                let text = text.trim_matches(' ');
                // Clock text and date text with a clock suffix read as in number coercion,
                // with surrounding spaces trimmed (probes/value-clock-text column B against C;
                // date-text-grammar C30, C31, C53, C55, C63).
                self.parse_number_text(text, &["$", "€"], &|| self.current_year())
                    .map(ArrayNode::Number)
                    .ok_or(Error::VALUE)
            }
            ArrayNode::Number(f) => Ok(ArrayNode::Number(*f)),
            ArrayNode::Boolean(_) => Err(Error::VALUE),
            ArrayNode::Error(e) => Err(e.clone()),
            ArrayNode::Empty => Ok(ArrayNode::Number(0.0)),
        })
    }

    // VALUETOTEXT(value, [format]) lifts over a range or array like LEN
    // (probes/text-argument-modes F17:F19); an error element is spelled out.
    pub(crate) fn fn_valuetotext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let format = match args.get(1) {
            Some(node) => match self.text_arg(node, cell) {
                Ok(TextArg::Error(_, error)) => return error,
                Ok(f) => Some(f),
                Err(e) => return e,
            },
            None => None,
        };
        // The format is lifted over arrays like the text; a scalar error text still prints as
        // text (`C4`) unless the format is invalid.
        let operands: Vec<&TextArg> = std::iter::once(&text).chain(format.as_ref()).collect();
        let element = |i: usize, j: usize| {
            let strict = format
                .as_ref()
                .map_or(Ok(false), |f| self.valuetotext_format(f.elem(i, j)))?;
            Ok(ArrayNode::String(
                value_to_text(text.elem(i, j), strict).into(),
            ))
        };
        // A scalar error text prints as text, which `broadcast_text` would return as the error.
        let scalar_format = format.as_ref().is_none_or(|f| f.dims() == (1, 1));
        match &text {
            TextArg::Error(..) if scalar_format => match element(0, 0) {
                Ok(node) => array_node_to_calc_result(&node, cell),
                Err(e) => CalcResult::new_error(e, cell, String::new()),
            },
            _ => broadcast_text(cell, &operands, element),
        }
    }

    /// Whether VALUETOTEXT's `format` element selects the strict form (1, text quoted). Excel
    /// accepts a number in `[0, 2)` truncated (`1.9` is 1, `1E-20` is 0, `-0.5` and `2.5` are
    /// `#VALUE!`), a boolean, a number text and a blank (0); other text is `#VALUE!`
    /// (probes/valuetotext-format C9:C23, T-1229).
    fn valuetotext_format(&self, node: &ArrayNode) -> Result<bool, Error> {
        let f = self.array_node_to_number(node)?;
        if (0.0..2.0).contains(&f) {
            Ok(f >= 1.0)
        } else {
            Err(Error::VALUE)
        }
    }
}

/// The text VALUETOTEXT gives one element: an error spells out its code, and `strict` (format 1)
/// quotes text with its embedded quotes doubled, `"a""b"` (probes/valuetotext-format C1:C7).
fn value_to_text(node: &ArrayNode, strict: bool) -> String {
    match node {
        ArrayNode::Error(e) => e.to_string(),
        ArrayNode::String(s) if strict => format!("\"{}\"", s.replace('"', "\"\"")),
        other => array_node_to_string(other).unwrap_or_default(),
    }
}

#[cfg(test)]
mod rept_tests {
    use super::*;

    #[test]
    fn rept_element_rejects_non_finite_counts() {
        let text = ArrayNode::String("a".into());
        for count in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(rept_element(&text, Ok(count)), Err(Error::VALUE));
        }
        assert_eq!(
            rept_element(&text, Ok(2.0)),
            Ok(ArrayNode::String("aa".into()))
        );
    }
}
