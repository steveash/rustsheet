//! The element-wise text functions on the SPEC-0011 driver (T-1775): LEFT, MID, FIND, REPT,
//! SUBSTITUTE, TEXT, CONCATENATE and the rest lift through `Model::lift_call`, which reads each
//! argument once, bounds the output and the text it holds, and calls the function's element once
//! per position. An element reads the borrowed values of its position
//! (`cast::calc_result_to_text`, `Model::calc_result_to_number`).

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
};

/// One element of a text function: its value from the values of one position, or the error of
/// the first argument that fails.
pub(crate) type TextElement<'k> = dyn FnMut(&Model, &[CalcResult]) -> Result<ArrayNode, Error> + 'k;

impl Model {
    /// Runs `element` once (no argument lifts) or once per position through the lifting driver,
    /// with `kind`'s table entry. Arguments are coerced left to right and the first failure
    /// decides (probes/error-precedence-text; docs/excel-semantics/arrays.md "Errors broadcast
    /// through `&`, comparisons, POWER and the element-wise functions"): see [`text_result`].
    pub(crate) fn lift_text(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        element: &mut TextElement<'_>,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |model, values| {
            let result = element(model, values);
            text_result(values, result, cell)
        })
    }
}

/// An element's result as a value. An error argument is returned as it came only when it is the
/// element's first failure; "first" is judged by comparing error kinds, which fits the probed
/// cases (T-1022). Other errors carry no message, so a failing element allocates nothing.
pub(crate) fn text_result(
    values: &[CalcResult],
    result: Result<ArrayNode, Error>,
    cell: CellReferenceIndex,
) -> CalcResult {
    let failure = match &result {
        Ok(ArrayNode::Error(e)) | Err(e) => Some(e),
        Ok(_) => None,
    };
    if let Some(error @ CalcResult::Error { error: arg, .. }) = values
        .iter()
        .find(|value| matches!(value, CalcResult::Error { .. }))
    {
        if failure.is_none_or(|e| e == arg) {
            return error.clone();
        }
    }
    match result {
        Ok(ArrayNode::String(s)) => CalcResult::String(s),
        Ok(ArrayNode::Number(n)) => CalcResult::Number(n),
        Ok(ArrayNode::Boolean(b)) => CalcResult::Boolean(b),
        Ok(ArrayNode::Empty) => CalcResult::String(String::new().into()),
        Ok(ArrayNode::Error(e)) | Err(e) => CalcResult::new_error(e, cell, String::new()),
    }
}
