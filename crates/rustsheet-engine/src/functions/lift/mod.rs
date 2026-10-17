//! Scalar-argument lifting (SPEC-0011): running a function once per element when an argument is
//! an array or a multi-cell range, broadcast as the element-wise operators do.
//!
//! T-1784 moved the *IF(S) broadcasting core here as [`Model::lift_values`]; T-1770 adds the
//! argument-kind table and `lift_call`.
//!
//! [`Model::lift_values`]: crate::model::Model::lift_values

mod driver;

pub(crate) use driver::{LiftCost, LiftOptions, LiftedArg};
