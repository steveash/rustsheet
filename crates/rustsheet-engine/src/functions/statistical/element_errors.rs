//! The error of an array argument's error element, told from the error of a whole argument.
//!
//! An error element is the result (docs/excel-semantics/statistics.md "An error element of an array
//! argument is that error"), but another argument's error, or one that does not coerce, wins over it
//! in the lifted functions: [`is_array_element_error`] lets them tell the two apart.

use crate::calc_result::CalcResult;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;

/// The message of an array element's error, which [`is_array_element_error`] looks for.
const ARRAY_ELEMENT_ERROR: &str = "Error in array argument";

/// `error`, the error of an array argument's error element, as the result of the call at `cell`.
pub(crate) fn array_element_error(error: Error, cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(error, cell, ARRAY_ELEMENT_ERROR.to_string())
}

/// Whether `result` is the error of an array argument's error element.
pub(crate) fn is_array_element_error(result: &CalcResult) -> bool {
    matches!(result, CalcResult::Error { message, .. } if message == ARRAY_ELEMENT_ERROR)
}
