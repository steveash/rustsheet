//! Scalar-argument lifting (SPEC-0011): running a function once per element when an argument is
//! an array or a multi-cell range, broadcast as the element-wise operators do.
//!
//! - `kinds.rs`: the argument-kind table, [`Function::arg_kinds`], which static analysis reads
//!   too (`expressions/parser/static_analysis/lifted.rs`).
//! - `call.rs`: `lift_call` and its read phase, for the functions in the table.
//! - `driver.rs`: the broadcasting core, `lift_values`, which the *IF(S), AGGREGATE, IFS, SWITCH
//!   and lookup lifts call directly (moved from `if_ifs.rs` by T-1784).
//!
//! [`Function::arg_kinds`]: crate::functions::Function::arg_kinds

mod arg_buf;
mod call;
mod driver;
mod kinds;
#[cfg(test)]
mod tests;

pub(crate) use driver::{grid_dims, LiftCost, LiftOptions, LiftedArg};
pub(crate) use kinds::{lifting_table_functions, ArgKind, ArgKinds};

use crate::{
    calc_result::CalcResult, expressions::token::Error, expressions::types::CellReferenceIndex,
};

/// The `#NUM!` of a lifted kernel's element: its message is empty, so a failing element
/// allocates nothing (SPEC-0011).
pub(crate) fn num_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, String::new())
}

/// The `#VALUE!` of a lifted kernel's element, likewise without a message.
pub(crate) fn value_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, String::new())
}
