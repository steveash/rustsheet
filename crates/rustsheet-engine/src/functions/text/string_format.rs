// DOLLAR, FIXED, NUMBERVALUE, PROPER, REPLACE, ARRAYTOTEXT

use crate::{
    calc_result::CalcResult,
    cast::array_node_to_text,
    cast::number_text_too_long,
    cast::MAX_TEXT_UNITS,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
    number_format::parse_excel_number,
};

use super::text_arg::broadcast_text;
use crate::functions::util::near_integer_count;

fn format_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Formats a non-negative f64 with the given decimal places and optional thousands separator.
/// Returns the formatted string without sign.
fn format_abs(abs_value: f64, decimals: i32, use_thousands: bool) -> String {
    if decimals >= 0 {
        let factor = 10f64.powi(decimals);
        let rounded = (abs_value * factor).round() / factor;
        let int_part = rounded.floor() as u64;
        let int_str = if use_thousands {
            format_thousands(int_part)
        } else {
            int_part.to_string()
        };
        if decimals == 0 {
            int_str
        } else {
            let frac = rounded - rounded.floor();
            let frac_digits = (frac * factor).round() as u64;
            format!(
                "{}.{:0>width$}",
                int_str,
                frac_digits,
                width = decimals as usize
            )
        }
    } else {
        // Negative decimals: round to 10^(-decimals) places
        let factor = 10f64.powi(-decimals);
        let rounded = ((abs_value / factor).round() * factor) as u64;
        if use_thousands {
            format_thousands(rounded)
        } else {
            rounded.to_string()
        }
    }
}

impl Model {
    /// The `decimals` argument of DOLLAR and FIXED (default 2): truncated toward zero, and
    /// `#VALUE!` from 128 up (docs/excel-semantics/text.md#dollar-and-fixed-decimals).
    fn decimals_arg(
        &mut self,
        arg: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<i32, CalcResult> {
        let Some(node) = arg else { return Ok(2) };
        let d = self.get_number(node, cell)?.trunc();
        if d >= 128.0 {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "decimals too large".to_string(),
            ));
        }
        // The clamp only keeps the cast in range: every large negative gives "0" (FIXED(1,-400)).
        Ok(d.max(-1000.0) as i32)
    }

    /// DOLLAR(number, [decimals]) — Formats a number as a dollar currency string.
    /// Negative numbers use parentheses: ($1,234.57)
    pub(crate) fn fn_dollar(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let decimals = match self.decimals_arg(args.get(1), cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let formatted = format_abs(value.abs(), decimals, true);
        let result = if value < 0.0 {
            format!("(${formatted})")
        } else {
            format!("${formatted}")
        };
        CalcResult::String(result.into())
    }

    /// FIXED(number, [decimals], [`no_commas`]) — Formats a number as text with fixed decimal places.
    pub(crate) fn fn_fixed(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let decimals = match self.decimals_arg(args.get(1), cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let no_commas = if args.len() == 3 {
            match self.get_boolean(&args[2], cell) {
                Ok(b) => b,
                Err(e) => return e,
            }
        } else {
            false
        };
        let use_thousands = !no_commas;
        let formatted = format_abs(value.abs(), decimals, use_thousands);
        // Excel drops the sign when the value rounds to zero: look at the rounded text, not the
        // value (docs/excel-semantics/text.md "FIXED drops the sign of a value that rounds to
        // zero"; gen-m6/f94b5b8236742618 A24, probes/dollar-fixed-unichar-edges A49, A50).
        let rounds_to_zero = !formatted.bytes().any(|b| matches!(b, b'1'..=b'9'));
        let result = if value < 0.0 && !rounds_to_zero {
            format!("-{formatted}")
        } else {
            formatted
        };
        CalcResult::String(result.into())
    }

    /// NUMBERVALUE(text, [`decimal_separator`], [`group_separator`])
    /// Converts text to a number using the given separators.
    pub(crate) fn fn_numbervalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let decimal_sep = if args.len() >= 2 {
            match self.get_string(&args[1], cell) {
                Ok(s) => s,
                Err(e) => return e,
            }
        } else {
            ".".to_string()
        };
        let group_sep = if args.len() == 3 {
            match self.get_string(&args[2], cell) {
                Ok(s) => s,
                Err(e) => return e,
            }
        } else {
            ",".to_string()
        };

        if decimal_sep.is_empty() || decimal_sep == group_sep {
            return CalcResult::new_error(Error::VALUE, cell, "Invalid separators".to_string());
        }

        // Excel counts the text with its spaces: 255 characters parse, 256 are #VALUE!
        // (probes/text-number-long-paths A13, A14).
        if number_text_too_long(&text) {
            return CalcResult::new_error(Error::VALUE, cell, "Text too long".to_string());
        }
        let trimmed = text.trim();

        // Handle percentage suffix
        let (trimmed, pct_count) = {
            let mut t = trimmed;
            let mut count = 0u32;
            while t.ends_with('%') {
                t = &t[..t.len() - 1];
                count += 1;
            }
            (t.trim_end(), count)
        };

        if trimmed.is_empty() {
            return CalcResult::new_error(Error::VALUE, cell, "Empty string".to_string());
        }

        // Remove group separators, replace decimal separator with '.'
        let cleaned = trimmed.replace(&group_sep, "");
        let cleaned = cleaned.replace(&decimal_sep, ".");

        // Reject if more than one '.' after replacement
        if cleaned.matches('.').count() > 1 {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Multiple decimal separators".to_string(),
            );
        }

        match parse_excel_number(&cleaned) {
            Some(mut v) => {
                for _ in 0..pct_count {
                    v /= 100.0;
                }
                CalcResult::Number(v)
            }
            None => CalcResult::new_error(Error::VALUE, cell, "Cannot parse number".to_string()),
        }
    }

    /// PROPER(text) — Converts text to title case (first letter of each word uppercase).
    pub(crate) fn fn_proper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, proper_case)
    }

    /// `REPLACE(old_text`, `start_num`, `num_chars`, `new_text`)
    /// Replaces `num_chars` characters starting at `start_num` (1-indexed) with `new_text`.
    /// If any of the arguments is an array it works element by element.
    pub(crate) fn fn_replace(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let old_text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let start_num = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let num_chars = match self.text_arg(&args[2], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let new_text = match self.text_arg(&args[3], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        broadcast_text(
            cell,
            &[&old_text, &start_num, &num_chars, &new_text],
            |i, j| {
                replace_element(
                    old_text.elem(i, j),
                    self.array_node_to_number(start_num.elem(i, j)),
                    self.array_node_to_number(num_chars.elem(i, j)),
                    new_text.elem(i, j),
                )
            },
        )
    }
}

/// Computes a single REPLACE result from one element of each argument.
fn replace_element(
    old_text: &ArrayNode,
    start_num: Result<f64, Error>,
    num_chars: Result<f64, Error>,
    new_text: &ArrayNode,
) -> Result<ArrayNode, Error> {
    // Every argument is coerced before the range checks: an error argument beats a start or length
    // failure (docs/excel-semantics/arrays.md, probes/error-arg-vs-range-check, T-1056).
    let old_text = array_node_to_text(old_text)?;
    let start_num = start_num?;
    let num_chars = num_chars?;
    let new_text = array_node_to_text(new_text)?;
    // Excel: both numbers count as whole like LEFT's (near an integer they round up, so
    // `REPLACE(t,1-2E-7,1,"z")` starts at 1) and either one above 32,767 is #VALUE!, even
    // past the end of a short text (probes/text-count-argument-bounds J, K, U, F22, F24).
    let start_num = near_integer_count(start_num);
    if num_chars < 0.0 || !(1.0..=MAX_TEXT_UNITS as f64).contains(&start_num) {
        return Err(Error::VALUE);
    }
    let num_chars = near_integer_count(num_chars);
    if num_chars > MAX_TEXT_UNITS as f64 {
        return Err(Error::VALUE);
    }

    // Excel: positions count UTF-16 code units, so `REPLACE("a😀b",2,2,"x")` is "axb"
    // (probes/text-surrogate-positions A27-A31).
    let len = old_text.len_utf16();
    // start_num is 1-indexed; clamp to valid range
    let start = (start_num as usize).saturating_sub(1).min(len);
    let end = start.saturating_add(num_chars as usize).min(len);
    // The lone half of a pair either bound cuts is kept (probes/text-surrogate-positions A29).
    let mut result = old_text.slice_utf16(0, start);
    result.push(&new_text);
    result.push(&old_text.slice_utf16(end, len));
    Ok(ArrayNode::String(result))
}

fn proper_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for c in s.chars() {
        if c.is_alphabetic() {
            if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        } else {
            result.push(c);
            capitalize_next = true;
        }
    }
    result
}
