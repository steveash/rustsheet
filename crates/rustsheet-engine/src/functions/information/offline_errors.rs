//! IMAGE, STOCKHISTORY and FIELDVALUE without I/O (SPEC-0003 "Functions with no I/O",
//! docs/excel-semantics/information.md). Excel fetches an image or quotes and reads linked data
//! types; RustSheet does neither (CLAUDE.md directive 5, decision 0027), so each evaluates to the
//! error Excel gives when the fetch or the data type is not there.

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

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
    /// evaluated first argument otherwise (IMAGE and STOCKHISTORY: an argument error wins,
    /// probes/rich-errors-args).
    fn offline_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let mut first = CalcResult::EmptyArg;
        for (i, arg) in args.iter().enumerate() {
            let value = self.offline_arg(arg, cell);
            if value.is_error() {
                return Err(value);
            }
            if i == 0 {
                first = value;
            }
        }
        Ok(first)
    }

    /// IMAGE(source, [alt_text], [sizing], [height], [width]) is `#BLOCKED!`: no fetch happens.
    /// An empty or numeric `source` is `#VALUE!` (probes/rich-errors-args).
    pub(crate) fn fn_image(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 5 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.offline_args(args, cell) {
            Err(error) => error,
            Ok(CalcResult::Number(_) | CalcResult::EmptyCell | CalcResult::EmptyArg) => {
                CalcResult::new_error(Error::VALUE, cell, "Invalid image source".to_string())
            }
            Ok(_) => {
                CalcResult::new_error(Error::BLOCKED, cell, "Images are never fetched".to_string())
            }
        }
    }

    /// STOCKHISTORY(stock, start_date, [end_date], [interval], [headers], [property0..5]) is
    /// `#CONNECT!`: no quotes are fetched.
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
            Ok(_) => CalcResult::new_error(
                Error::CONNECT,
                cell,
                "Stock data is never fetched".to_string(),
            ),
        }
    }

    /// FIELDVALUE(value, field_name) is `#FIELD!`: values are never linked data types
    /// (decision 0027). An error in `field_name` wins; an error in `value` does not, and an array
    /// `value` gives an array of the same shape (probes/rich-errors-args).
    pub(crate) fn fn_fieldvalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let field_error = |origin| CalcResult::new_error(Error::FIELD, origin, String::new());
        let value = self.evaluate_node_in_context(&args[0], cell);
        let field_name = self.offline_arg(&args[1], cell);
        if field_name.is_error() {
            return field_name;
        }
        let shape = match value {
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(values) => Some(values),
                Err(error) => return error,
            },
            CalcResult::Array(values) => Some(values),
            _ => None,
        };
        // A 1x1 range or array stays a scalar `#FIELD!`; only the shape of a larger one is used.
        match shape {
            Some(values) if values.len() > 1 || values.first().is_some_and(|row| row.len() > 1) => {
                CalcResult::Array(
                    values
                        .iter()
                        .map(|row| row.iter().map(|_| ArrayNode::Error(Error::FIELD)).collect())
                        .collect(),
                )
            }
            _ => field_error(cell),
        }
    }
}
