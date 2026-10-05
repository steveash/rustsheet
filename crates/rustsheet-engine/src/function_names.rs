//! The list of implemented worksheet functions, for coverage reports (T-0020).

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
