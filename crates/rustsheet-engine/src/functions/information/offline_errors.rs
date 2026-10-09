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
/// than `links.rs`, which also accepts ftp and mailto). Which
/// strings Excel counts as a URL is not probed beyond `https://example.invalid/none.png`.
fn is_url(text: &str) -> bool {
    let has_prefix = |prefix: &str| {
        text.get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
    };
    has_prefix("http://") || has_prefix("https://")
}

/// The error IMAGE gives for one `source` value: an error stays, a URL is `#BLOCKED!` (no fetch)
/// and everything else (empty, number, boolean, non-URL text) is `#VALUE!`
/// (probes/rich-errors-arg-kinds).
fn image_error(source: &ArrayNode) -> Error {
    match source {
        ArrayNode::Error(error) => error.clone(),
        ArrayNode::String(text) if is_url(text) => Error::BLOCKED,
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
    /// probes/rich-errors-args).
    fn offline_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Lifted, CalcResult> {
        let mut first = Lifted::Scalar(CalcResult::EmptyArg);
        for (i, arg) in args.iter().enumerate() {
            if i == 0 {
                first = self.lifted_arg(arg, cell)?;
                if let Lifted::Scalar(value) = &first {
                    if value.is_error() {
                        return Err(value.clone());
                    }
                }
            } else {
                let value = self.offline_arg(arg, cell);
                if value.is_error() {
                    return Err(value);
                }
            }
        }
        Ok(first)
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
            Ok(Lifted::Array(values)) => {
                CalcResult::Array(map_elements(&values, |v| ArrayNode::Error(image_error(v))))
            }
            Ok(Lifted::Scalar(source)) => {
                let error = image_error(&calc_result_to_array_node(source));
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
            Ok(Lifted::Array(values)) => error_array(&values, &Error::CONNECT),
            Ok(Lifted::Scalar(_)) => CalcResult::new_error(
                Error::CONNECT,
                cell,
                "Stock data is never fetched".to_string(),
            ),
        }
    }

    /// FIELDVALUE(value, field_name) is `#FIELD!`: values are never linked data types
    /// (decision 0027). An error in a scalar `field_name` wins; an error in `value` does not. An
    /// array or range in either argument gives an array of `#FIELD!`, the larger of the two
    /// shapes (probes/rich-errors-args, probes/rich-errors-arg-kinds).
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
        CalcResult::Array(vec![vec![ArrayNode::Error(Error::FIELD); cols]; rows])
    }
}
