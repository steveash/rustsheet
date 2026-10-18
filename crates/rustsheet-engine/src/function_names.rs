//! The list of implemented worksheet functions, for coverage reports (T-0020), and the
//! argument-kind table the harness checks the catalog against (T-1776).

use crate::functions::lift::ArgKind;
use crate::functions::Function;
use crate::language::get_default_language;

/// Every worksheet function the engine implements, by its en-US name (`SUM`, `ERROR.TYPE`), sorted.
#[must_use]
pub fn implemented_function_names() -> Vec<String> {
    let english = get_default_language();
    let mut names: Vec<String> = Function::into_iter()
        .map(|f| f.to_localized_name(english))
        .collect();
    names.sort_unstable();
    names
}

/// One function's row of the SPEC-0011 argument-kind table, for the harness's catalog check
/// (T-1776). Kinds are the `ArgKind` variant names (`Lift`, `LiftArrays`, `WholeByCaller`, ...).
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LiftKinds {
    /// The en-US name, e.g. `EDATE`.
    pub name: String,
    /// The fewest arguments a valid call takes.
    pub required: usize,
    /// The kind of each listed parameter, in order.
    pub params: Vec<&'static str>,
    /// The kind of every argument past `params`, when the last parameter repeats.
    pub repeat: Option<&'static str>,
    /// The most arguments a valid call takes.
    pub max: usize,
    /// `Constant`, or `Weighted(n)` for a value-dependent kernel.
    pub work: String,
}

/// Testing hook, not public API (no stability promise): the argument-kind table row of every
/// function the lift driver handles, sorted by name.
#[doc(hidden)]
#[must_use]
pub fn lift_kinds_table() -> Vec<LiftKinds> {
    let english = get_default_language();
    let mut rows: Vec<LiftKinds> = Function::into_iter()
        .filter_map(|function| {
            let kinds = function.arg_kinds()?;
            Some(LiftKinds {
                name: function.to_localized_name(english),
                required: kinds.required,
                params: kinds.params.iter().map(|kind| kind.name()).collect(),
                repeat: kinds.repeat.map(ArgKind::name),
                max: kinds.max,
                work: format!("{:?}", kinds.work),
            })
        })
        .collect();
    rows.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    rows
}
