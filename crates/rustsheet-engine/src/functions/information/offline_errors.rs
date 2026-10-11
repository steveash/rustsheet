//! IMAGE, STOCKHISTORY and FIELDVALUE without I/O (SPEC-0003 "Functions with no I/O",
//! docs/excel-semantics/information.md). Excel fetches an image or quotes and reads linked data
//! types; RustSheet does neither (CLAUDE.md directive 5, decision 0027), so each evaluates to the
//! error Excel gives when the fetch or the data type is not there.

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

use super::{map_elements, Lifted};
use crate::functions::math_and_trigonometry::array_size::check_array_size;

/// Whether IMAGE `source` text is URL-shaped and so reaches the (never performed) fetch (narrower
/// than `links.rs`, which also accepts ftp and mailto): after leading spaces, tabs and
/// no-break spaces, an `http:` or `https:` scheme in any case followed by something other than
/// slashes. `HTTPS://X/A.PNG`, `https:example.invalid/a.png` and ` https://...` are `#BLOCKED!`;
/// `https://`, ftp, `www.`, `data:` and `file:` sources are `#VALUE!`
/// (probes/rich-errors-shapes-urls A25:A35, T-1189; a leading tab or no-break space is skipped like
/// a space, probes/rich-errors-image-sizing, T-1345).
fn is_url(text: &str) -> bool {
    let text = text.trim_start_matches([' ', '\t', '\u{a0}']);
    let rest_after = |scheme: &str| {
        text.get(..scheme.len())
            .filter(|head| head.eq_ignore_ascii_case(scheme))
            .and_then(|_| text.get(scheme.len()..))
    };
    rest_after("http:")
        .or_else(|| rest_after("https:"))
        .is_some_and(|rest| !rest.trim_start_matches('/').is_empty())
}

/// The number IMAGE reads from a `sizing`, `height` or `width` argument: a number, or a boolean
/// (TRUE is 1: `IMAGE(url,"alt",TRUE)` is not `#VALUE!`), or an empty cell as 0 (UNVERIFIED).
/// Text and an omitted argument have none.
fn image_number(value: &CalcResult) -> Option<f64> {
    match value {
        CalcResult::Number(n) => Some(*n),
        CalcResult::Boolean(b) => Some(f64::from(u8::from(*b))),
        CalcResult::EmptyCell => Some(0.0),
        _ => None,
    }
}

/// Whether IMAGE's `[alt_text]`-onward options (`options` is `sizing`, `height`, `width`) are
/// valid; an invalid set makes IMAGE `#VALUE!` even for a URL source. Excel (probes/rich-errors-
/// image-sizing, excel-graph, 2026-10-11, T-1345): `sizing` is 0 to 3 (a fraction truncates;
/// -1, 4, 9, text, "" and an omitted `IMAGE(url,"alt",)` are `#VALUE!`); sizing 3 needs a
/// `height`, `height` and `width` are not negative, and not both 0; sizing 0 and 1 refuse
/// a `height`. Array-valued options are not validated (UNVERIFIED lifting).
fn image_options_valid(options: &[CalcResult]) -> bool {
    let Some(sizing) = options.first() else {
        return true;
    };
    if matches!(sizing, CalcResult::Array(_)) {
        return true;
    }
    let Some(sizing) = image_number(sizing).filter(|s| (0.0..4.0).contains(s)) else {
        return false;
    };
    let size = |value: Option<&CalcResult>| match value {
        None | Some(CalcResult::EmptyArg) => Ok(None),
        Some(CalcResult::Array(_)) => Ok(Some(f64::NAN)),
        Some(other) => image_number(other)
            .filter(|n| *n >= 0.0)
            .map(Some)
            .ok_or(()),
    };
    let (Ok(height), Ok(width)) = (size(options.get(1)), size(options.get(2))) else {
        return false;
    };
    match sizing.trunc() as u8 {
        3 => height.is_some() && (height != Some(0.0) || width.is_some_and(|w| w != 0.0)),
        0 | 1 => height.is_none() && width.is_none(),
        _ => true,
    }
}

/// The error IMAGE gives for one `source` value: an error stays, a URL is `#BLOCKED!` (no fetch)
/// unless the options are invalid (`valid`), and everything else (empty, number, boolean,
/// non-URL text) is `#VALUE!` (probes/rich-errors-arg-kinds).
fn image_error(source: &ArrayNode, valid: bool) -> Error {
    match source {
        ArrayNode::Error(error) => error.clone(),
        ArrayNode::String(text) if valid && is_url(text) => Error::BLOCKED,
        _ => Error::VALUE,
    }
}

/// An array of `error` with the shape of `values` (an element that is already an error stays).
fn error_array(values: &[Vec<ArrayNode>], error: &Error) -> CalcResult {
    CalcResult::Array(map_elements(values, |v| match v {
        ArrayNode::Error(_) => v.clone(),
        _ => ArrayNode::Error(error.clone()),
    }))
}

impl Model {
    /// The value an argument evaluates to; a range is its top-left cell, read without
    /// evaluating the rest of the range.
    fn offline_arg(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, .. } => self.evaluate_cell(left),
            other => other,
        }
    }

    /// Evaluates every argument in order and returns the first error among them, with the
    /// lifted first argument otherwise (IMAGE and STOCKHISTORY: an argument error wins,
    /// probes/rich-errors-args). A later argument's error lifts over an array or range first
    /// argument: `IMAGE({url,url},1/0)` is a 1x2 array of `#DIV/0!` (probes/rich-errors-shapes-urls
    /// A12:B15, T-1189).
    fn offline_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Lifted, Vec<CalcResult>), CalcResult> {
        // The callers check arity first; an empty list is an argument-count error, not a panic.
        let Some((head, rest)) = args.split_first() else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let first = self.lifted_arg(head, cell)?;
        if let Lifted::Scalar(value) = &first {
            if value.is_error() {
                return Err(value.clone());
            }
        }
        let mut rest_values = Vec::with_capacity(rest.len());
        for arg in rest {
            let value = self.offline_arg(arg, cell);
            if let CalcResult::Error { error, .. } = &value {
                return Err(match &first {
                    Lifted::Array(values) => {
                        CalcResult::Array(map_elements(values, |_| ArrayNode::Error(error.clone())))
                    }
                    Lifted::Scalar(_) => value,
                });
            }
            rest_values.push(value);
        }
        Ok((first, rest_values))
    }

    /// IMAGE(source, [alt_text], [sizing], [height], [width]) is `#BLOCKED!` for a URL: no fetch
    /// happens. Any other `source` is `#VALUE!` (probes/rich-errors-arg-kinds), and an array or
    /// range `source` gives an array of the same shape.
    pub(crate) fn fn_image(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 5 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.offline_args(args, cell) {
            Err(error) => error,
            Ok((Lifted::Array(values), options)) => {
                let valid = image_options_valid(options.get(1..).unwrap_or_default());
                CalcResult::Array(map_elements(&values, |v| {
                    ArrayNode::Error(image_error(v, valid))
                }))
            }
            Ok((Lifted::Scalar(source), options)) => {
                let valid = image_options_valid(options.get(1..).unwrap_or_default());
                let error = image_error(&calc_result_to_array_node(source), valid);
                let message = if error == Error::BLOCKED {
                    "Images are never fetched"
                } else {
                    "Invalid image source"
                };
                CalcResult::new_error(error, cell, message.to_string())
            }
        }
    }

    /// STOCKHISTORY(stock, start_date, [end_date], [interval], [headers], [property0..5]) is
    /// `#CONNECT!`: no quotes are fetched. An array or range `stock` gives an array of the same
    /// shape (probes/rich-errors-arg-kinds).
    pub(crate) fn fn_stockhistory(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 11 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.offline_args(args, cell) {
            Err(error) => error,
            Ok((Lifted::Array(values), _)) => error_array(&values, &Error::CONNECT),
            Ok((Lifted::Scalar(_), _)) => CalcResult::new_error(
                Error::CONNECT,
                cell,
                "Stock data is never fetched".to_string(),
            ),
        }
    }

    /// FIELDVALUE(value, field_name) is `#FIELD!`: values are never linked data types
    /// (decision 0027). An error in a scalar `field_name` wins, and an error element of an array
    /// `field_name` stays in its place; an error in `value` does not. An array or range in either
    /// argument gives an array of `#FIELD!`, the larger of the two shapes (probes/rich-errors-args,
    /// probes/rich-errors-arg-kinds, probes/rich-errors-shapes-urls).
    pub(crate) fn fn_fieldvalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.lifted_arg(&args[0], cell) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let field_name = match self.lifted_arg(&args[1], cell) {
            Ok(Lifted::Scalar(scalar)) if scalar.is_error() => return scalar,
            Ok(field_name) => field_name,
            Err(error) => return error,
        };
        // A single value (or a 1x1 range) is a scalar `#FIELD!`.
        if matches!(
            (&value, &field_name),
            (Lifted::Scalar(_), Lifted::Scalar(_))
        ) {
            return CalcResult::new_error(Error::FIELD, cell, String::new());
        }
        let ((value_rows, value_cols), (field_rows, field_cols)) =
            (value.dims(), field_name.dims());
        let (rows, cols) = (value_rows.max(field_rows), value_cols.max(field_cols));
        if let Some((error, message)) = check_array_size(rows, cols) {
            return CalcResult::new_error(error, cell, message);
        }
        // An error element of an array `field_name` stays itself (`FIELDVALUE(5,{"a",#DIV/0!})` is
        // `#FIELD!`, `#DIV/0!`); an error in `value` does not (probes/rich-errors-shapes-urls A9:C10,
        // T-1189). A 1-wide `field_name` repeats along that axis, and a `value` shorter than the result is
        // not padded (A2:C7).
        let field_error = |row: usize, col: usize| match &field_name {
            Lifted::Array(values) => {
                let r = if field_rows == 1 { 0 } else { row };
                let c = if field_cols == 1 { 0 } else { col };
                match values.get(r).and_then(|line| line.get(c)) {
                    Some(ArrayNode::Error(error)) => error.clone(),
                    // A `field_name` shorter than the result along an axis wider than 1 is `#N/A`
                    // there: `FIELDVALUE({1,2,3},{"a","b"})` is `#FIELD!`, `#FIELD!`, `#N/A`
                    // (probes/rich-errors-image-sizing, T-1345).
                    None => Error::NA,
                    _ => Error::FIELD,
                }
            }
            Lifted::Scalar(_) => Error::FIELD,
        };
        CalcResult::Array(
            (0..rows)
                .map(|row| {
                    (0..cols)
                        .map(|col| ArrayNode::Error(field_error(row, col)))
                        .collect()
                })
                .collect(),
        )
    }
}
