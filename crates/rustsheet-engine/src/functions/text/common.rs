use super::text_kernel::text_result;
use super::util::substitute_element;
use crate::cast::number_text_too_long;
use crate::number_format::number_to_text;
use crate::{
    calc_result::CalcResult,
    cast::{calc_result_to_string, calc_result_to_text, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    formatter::{
        format::{format_text, format_text_value_exact, FormatContext, Formatted},
        rejects_format_code,
    },
    functions::{text::position::find_element, util::near_integer_count, Function},
    model::Model,
    number_format::to_precision,
    text::{Char, Text},
};

/// Excel: UPPER, LOWER, PROPER, TRIM, CLEAN and REPLACE of text over 32,767
/// UTF-16 units are `#VALUE!`, though LEN, LEFT, MID and the REGEX* functions take it
/// (probes/regexreplace-long-result-d, -e, text-long-input-a, -b, -c).
pub(super) fn reject_over_limit(text: &Text) -> Result<(), Error> {
    if text.len_utf16() > MAX_TEXT_UNITS {
        return Err(Error::VALUE);
    }
    Ok(())
}

/// Computes a single LEFT result from one element of each argument.
// Excel: LEFT and RIGHT count characters (code points), unlike LEN, MID, FIND and REPLACE, which
// count UTF-16 code units: `LEFT("a😀b",3)` is "a😀b" (docs/excel-semantics/text.md "Text
// positions count UTF-16 code units", probes/text-surrogate-positions A3, B1-B5).
fn left_element(text: &CalcResult, num: Result<f64, Error>) -> Result<ArrayNode, Error> {
    let s = calc_result_to_text(text)?;
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
fn right_element(text: &CalcResult, num: Result<f64, Error>) -> Result<ArrayNode, Error> {
    let s = calc_result_to_text(text)?;
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
    text: &CalcResult,
    start: Result<f64, Error>,
    length: Result<f64, Error>,
) -> Result<ArrayNode, Error> {
    let s = calc_result_to_text(text)?;
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
fn len_element(text: &CalcResult) -> Result<ArrayNode, Error> {
    let s = calc_result_to_string(text)?;
    // Excel: LEN counts UTF-16 code units, so an emoji counts two (probes/text-surrogate-positions A1).
    Ok(ArrayNode::Number(s.encode_utf16().count() as f64))
}

/// Computes a single REPT result from one element of each argument.
fn rept_element(text: &CalcResult, number_times: Result<f64, Error>) -> Result<ArrayNode, Error> {
    let text = calc_result_to_text(text)?;
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
fn exact_element(text1: &CalcResult, text2: &CalcResult) -> Result<ArrayNode, Error> {
    if let (CalcResult::Number(number1), CalcResult::Number(number2)) = (text1, text2) {
        // In Excel two numbers are the same if they are the same up to 15 digits.
        return Ok(ArrayNode::Boolean(
            to_precision(*number1, 15) == to_precision(*number2, 15),
        ));
    }
    let string1 = calc_result_to_text(text1)?;
    let string2 = calc_result_to_text(text2)?;
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
fn text_value_node(value: &CalcResult, format: &str) -> ArrayNode {
    match calc_result_to_text(value) {
        // The exact text, so a lone surrogate lands where `@` is (probes/text-lone-surrogates).
        Ok(text) => format_text_value_exact(&text, format)
            .map_or(ArrayNode::Error(Error::VALUE), ArrayNode::String),
        Err(error) => ArrayNode::Error(error),
    }
}

/// The TEXT element for one value: numbers and blanks by the number sections, text and booleans
/// by the text section. An omitted value is 0, not a blank (`TEXT(,"General")`).
fn text_element(value: &CalcResult, format: &str, ctx: &FormatContext) -> ArrayNode {
    match value {
        CalcResult::Number(f) => text_node(Some(*f), format, ctx),
        CalcResult::EmptyArg => text_node(Some(0.0), format, ctx),
        CalcResult::EmptyCell => text_node(None, format, ctx),
        text @ (CalcResult::Boolean(_) | CalcResult::String(_)) => text_value_node(text, format),
        CalcResult::Error { error, .. } => ArrayNode::Error(error.clone()),
        CalcResult::Range { .. } | CalcResult::Array(_) | CalcResult::Lambda(_) => {
            ArrayNode::Error(Error::VALUE)
        }
    }
}

impl Model {
    /// TEXT(value, format_text) lifts over a range or an array in either argument, one result
    /// per element; each format has its own complexity check (probes/text-format-length-limit-f
    /// A54:B54). The value's error wins over a bad or too complex format (number-formats.md), and
    /// the format's error over a text value (error_broadcast.xlsx Sheet2!A1, `TEXT("x",NA())`).
    pub(crate) fn fn_text(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let ctx = self.format_context();
        self.lift_call(Function::Text, args, cell, &mut |_, values| {
            let [value, format] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            if let error @ CalcResult::Error { .. } = value {
                return error.clone();
            }
            let format_code = match format {
                error @ CalcResult::Error { .. } => return error.clone(),
                format => match calc_result_to_text(format) {
                    Ok(code) => code,
                    Err(error) => return CalcResult::new_error(error, cell, String::new()),
                },
            };
            if rejects_format_code(&format_code) {
                return CalcResult::new_error(Error::VALUE, cell, String::new());
            }
            text_result(&[], Ok(text_element(value, &format_code, &ctx)), cell)
        })
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
        let kind = if is_search {
            Function::Search
        } else {
            Function::Find
        };
        self.lift_text(kind, args, cell, &mut |model, values| {
            let [find_text, within_text, start_num @ ..] = values else {
                return Err(Error::ERROR);
            };
            find_element(
                find_text,
                within_text,
                model.optional_number(start_num.first(), 1.0),
                is_search,
            )
        })
    }

    /// A lifted text element's optional number argument: `default` when the call has none.
    /// A written empty argument is there, and reads as 0 (`LEFT("abc",)` is "").
    fn optional_number(&self, value: Option<&CalcResult>, default: f64) -> Result<f64, Error> {
        value.map_or(Ok(default), |value| self.calc_result_to_number(value))
    }

    // LEN, LEFT, RIGHT, MID, LOWER, UPPER, TRIM
    pub(crate) fn fn_len(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Len, args, cell, &mut |_, values| match values {
            [text] => len_element(text),
            _ => Err(Error::ERROR),
        })
    }

    pub(crate) fn fn_trim(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Excel TRIM operates on the ASCII space (0x20) only: it removes
        // leading and trailing spaces and collapses each internal run of two
        // or more spaces to a single space. It must NOT touch tab (0x09),
        // non-breaking space (0xA0), or any other Unicode whitespace, so we
        // deliberately split on ' ' rather than using `str::trim`.
        self.try_apply_text_unary_text(args, cell, |text| {
            reject_over_limit(text)?;
            Ok(text.trim_spaces())
        })
    }

    pub(crate) fn fn_lower(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.try_apply_text_unary_text(args, cell, |text| {
            reject_over_limit(text)?;
            Ok(text.map_runs(super::string_format::lower_case))
        })
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
        self.try_apply_text_unary_text(args, cell, |text| Ok(f(text)))
    }

    /// [`Model::apply_text_unary_text`] for a transform that can fail with an error.
    pub(crate) fn try_apply_text_unary_text(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        f: impl Fn(&Text) -> Result<Text, Error>,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.get_string_or_array(&args[0], cell) {
            Ok(crate::cast::StringOrArray::String(s)) => match f(&s) {
                Ok(t) => CalcResult::String(t),
                Err(e) => CalcResult::new_error(e, cell, "Text too long".to_string()),
            },
            Ok(crate::cast::StringOrArray::Array(arr)) => CalcResult::Array(
                arr.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| match crate::cast::array_node_to_text(n) {
                                Ok(s) => match f(&s) {
                                    Ok(t) => crate::expressions::parser::ArrayNode::String(t),
                                    Err(e) => crate::expressions::parser::ArrayNode::Error(e),
                                },
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
        self.try_apply_text_unary_text(args, cell, |text| {
            reject_over_limit(text)?;
            Ok(text.map_runs(super::string_format::upper_case))
        })
    }

    pub(crate) fn fn_left(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Left, args, cell, &mut |model, values| {
            let [text, num @ ..] = values else {
                return Err(Error::ERROR);
            };
            left_element(text, model.optional_number(num.first(), 1.0))
        })
    }

    pub(crate) fn fn_right(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Right, args, cell, &mut |model, values| {
            let [text, num @ ..] = values else {
                return Err(Error::ERROR);
            };
            right_element(text, model.optional_number(num.first(), 1.0))
        })
    }

    pub(crate) fn fn_mid(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Mid, args, cell, &mut |model, values| {
            let [text, start, length] = values else {
                return Err(Error::ERROR);
            };
            mid_element(
                text,
                model.calc_result_to_number(start),
                model.calc_result_to_number(length),
            )
        })
    }

    // REPT(text, number_times)
    pub(crate) fn fn_rept(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Rept, args, cell, &mut |model, values| {
            let [text, number_times] = values else {
                return Err(Error::ERROR);
            };
            rept_element(text, model.calc_result_to_number(number_times))
        })
    }

    // SUBSTITUTE(text, old_text, new_text, [instance_num])
    // Without an instance_num every instance is replaced.
    pub(crate) fn fn_substitute(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Substitute, args, cell, &mut |model, values| {
            let [text, old_text, new_text, instance_num @ ..] = values else {
                return Err(Error::ERROR);
            };
            substitute_element(
                text,
                old_text,
                new_text,
                instance_num.first().map(|n| model.calc_result_to_number(n)),
            )
        })
    }

    // EXACT(text1, text2)
    // If any of the arguments is an array it compares element by element.
    pub(crate) fn fn_exact(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_text(Function::Exact, args, cell, &mut |_, values| match values {
            [text1, text2] => exact_element(text1, text2),
            _ => Err(Error::ERROR),
        })
    }

    // VALUE(text)
    pub(crate) fn fn_value(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Lifts over a range or array, each element read as a scalar argument: a blank is 0, a
        // boolean or "" #VALUE! and an error stays (probes/value-argument-lift C1:C8, D1:D6;
        // corpus-bench/42181 B14, T-1791).
        self.lift_text(Function::Value, args, cell, &mut |model, values| {
            let [text] = values else {
                return Err(Error::ERROR);
            };
            match text {
                CalcResult::String(text) if number_text_too_long(text) => Err(Error::VALUE),
                CalcResult::String(text) => {
                    // VALUE trims spaces around date and clock text, unlike number coercion, but
                    // only the space character: `VALUE("1/2/2020\u{a0}")` is #VALUE!
                    // (probes/date-text-grammar C7, value-clock-text B14, B44).
                    let text = text.trim_matches(' ');
                    // Clock text and date text with a clock suffix read as in number coercion,
                    // with surrounding spaces trimmed (probes/value-clock-text column B against
                    // C; date-text-grammar C30, C31, C53, C55, C63).
                    model
                        .parse_number_text(text, &["$", "€"], &|| model.current_year())
                        .map(ArrayNode::Number)
                        .ok_or(Error::VALUE)
                }
                CalcResult::Number(f) => Ok(ArrayNode::Number(*f)),
                CalcResult::Error { error, .. } => Err(error.clone()),
                CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(ArrayNode::Number(0.0)),
                CalcResult::Boolean(_)
                | CalcResult::Range { .. }
                | CalcResult::Array(_)
                | CalcResult::Lambda(_) => Err(Error::VALUE),
            }
        })
    }

    // VALUETOTEXT(value, [format]) lifts over a range or array like LEN, the format too
    // (probes/text-argument-modes F17:F19); an error value is spelled out, so it is not the
    // result as an error argument of the other text functions is (probes/valuetotext-format C4).
    pub(crate) fn fn_valuetotext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Valuetotext, args, cell, &mut |model, values| {
            let [value, format @ ..] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let text = format
                .first()
                .map_or(Ok(false), |format| model.valuetotext_format(format))
                .and_then(|strict| value_to_text(value, strict));
            text_result(&[], text.map(|text| ArrayNode::String(text.into())), cell)
        })
    }

    /// Whether VALUETOTEXT's `format` element selects the strict form (1, text quoted). Excel
    /// accepts a number in `[0, 2)` truncated (`1.9` is 1, `1E-20` is 0, `-0.5` and `2.5` are
    /// `#VALUE!`), a boolean, a number text and a blank (0); other text is `#VALUE!`
    /// (probes/valuetotext-format C9:C23, T-1229).
    fn valuetotext_format(&self, format: &CalcResult) -> Result<bool, Error> {
        let f = self.calc_result_to_number(format)?;
        if (0.0..2.0).contains(&f) {
            Ok(f >= 1.0)
        } else {
            Err(Error::VALUE)
        }
    }
}

/// The text VALUETOTEXT gives one element: an error spells out its code, and `strict` (format 1)
/// quotes text with its embedded quotes doubled, `"a""b"` (probes/valuetotext-format C1:C7).
fn value_to_text(value: &CalcResult, strict: bool) -> Result<String, Error> {
    match value {
        CalcResult::Error { error, .. } => Ok(error.to_string()),
        CalcResult::String(s) if strict => Ok(format!("\"{}\"", s.replace('"', "\"\""))),
        other => calc_result_to_string(other),
    }
}

#[cfg(test)]
mod rept_tests {
    use super::*;

    #[test]
    fn rept_element_rejects_non_finite_counts() {
        let text = CalcResult::String("a".into());
        for count in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(rept_element(&text, Ok(count)), Err(Error::VALUE));
        }
        assert_eq!(
            rept_element(&text, Ok(2.0)),
            Ok(ArrayNode::String("aa".into()))
        );
    }
}
