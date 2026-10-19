//! Scalar-argument lifting (SPEC-0011): running a function once per element when an argument is
//! an array or a multi-cell range, broadcast as the element-wise operators do.
//!
//! - `kinds/`: the argument-kind table, split by family, [`Function::arg_kinds`], which static
//!   analysis reads too (`expressions/parser/static_analysis/lifted.rs`); `table_functions.rs`:
//!   its functions as a `match` pattern.
//! - `call.rs`: `lift_call` and its read phase, for the functions in the table.
//! - `driver.rs`: the broadcasting core, `lift_values`, which the *IF(S), AGGREGATE, IFS, SWITCH
//!   and lookup lifts call directly (moved from `if_ifs.rs` by T-1784).
//!
//! [`Function::arg_kinds`]: crate::functions::Function::arg_kinds

mod arg_buf;
mod call;
mod driver;
mod kinds;
mod table_functions;
#[cfg(test)]
mod tests;

pub(crate) use driver::{grid_dims, LiftCost, LiftOptions, LiftedArg};
pub(crate) use kinds::{ArgKind, ArgKinds, CallResult};
pub(crate) use table_functions::lifting_table_functions;

use crate::{
    calc_result::CalcResult, expressions::token::Error, expressions::types::CellReferenceIndex,
    model::Model,
};

impl Model {
    /// The two numbers of EFFECT, NOMINAL, DOLLARDE, DOLLARFR and QUOTIENT, in order (the first
    /// one's error wins); a boolean is `#VALUE!`.
    pub(crate) fn two_numbers(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        let [first, second] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        Ok((
            self.cast_number_no_bools_ref(first, cell)?,
            self.cast_number_no_bools_ref(second, cell)?,
        ))
    }
}

impl Model {
    /// A number argument of a lifted kernel, read as `cast_number_ref` reads it (a boolean is 0 or
    /// 1, a blank 0), except that text which is not a number is [`value_error`]: a failing
    /// element allocates nothing. A LAMBDA is `#VALUE!`, as the math functions read it before
    /// they moved onto the driver (T-1773).
    pub(crate) fn kernel_number(
        &self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        match value {
            CalcResult::String(text) => self.cast_number(text).ok_or_else(|| value_error(cell)),
            CalcResult::Lambda(_) => Err(Self::lambda_as_number_error(cell)),
            other => self.cast_number_ref(other, cell),
        }
    }

    /// [`Model::kernel_number`] for an optional argument: `default` when the call has no such
    /// argument. A written empty argument is there, and reads as 0.
    pub(crate) fn kernel_number_or(
        &self,
        value: Option<&CalcResult>,
        default: f64,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        value.map_or(Ok(default), |value| self.kernel_number(value, cell))
    }
}

/// A kernel's numeric result: an error carries no message, so a failing element allocates
/// nothing.
pub(crate) fn number_result(result: Result<f64, Error>, cell: CellReferenceIndex) -> CalcResult {
    match result {
        Ok(number) => CalcResult::Number(number),
        Err(error) => CalcResult::new_error(error, cell, String::new()),
    }
}

/// The `#NUM!` of a lifted kernel's element: its message is empty, so a failing element
/// allocates nothing (SPEC-0011).
pub(crate) fn num_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, String::new())
}

/// The `#VALUE!` of a lifted kernel's element, likewise without a message.
pub(crate) fn value_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, String::new())
}

/// The `#DIV/0!` of a lifted kernel's element, likewise without a message.
pub(crate) fn div_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::DIV, cell, String::new())
}

/// A lifted kernel's result as a cell value: the number, or the error (SPEC-0011 kernels return
/// `Result<f64, CalcResult>` so their reads chain with `?`).
pub(crate) fn number_or_error(result: Result<f64, CalcResult>) -> CalcResult {
    match result {
        Ok(number) => CalcResult::Number(number),
        Err(error) => error,
    }
}
