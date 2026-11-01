//! The error of an array argument's error element, told from the error of a whole argument.
//!
//! An error element is the result (docs/excel-semantics/statistics.md "An error element of an array
//! argument is that error"), but another argument's error, or one that does not coerce, wins over it
//! in the lifted functions: [`is_array_element_error`] lets them tell the two apart. The tag lives
//! only inside the call that read the element: `Model::call_by_reference_elements` drops it from
//! every built-in call's value (`CalcResult::untagged`), so an outer call sees a plain error
//! (T-3080).

use crate::calc_result::{CalcResult, ARRAY_ELEMENT_ERROR};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;

/// `error`, the error of an array argument's error element, as the result of the call at `cell`.
pub(crate) fn array_element_error(error: Error, cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(error, cell, ARRAY_ELEMENT_ERROR.to_string())
}

/// Whether `result` is the error of an array argument's error element.
pub(crate) fn is_array_element_error(result: &CalcResult) -> bool {
    matches!(result, CalcResult::Error { message, .. } if message == ARRAY_ELEMENT_ERROR)
}
