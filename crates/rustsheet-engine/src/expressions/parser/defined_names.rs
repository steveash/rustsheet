//! Defined-name lookups while parsing: one probe of `Parser::name_index` per lookup (T-3195).

use std::collections::HashMap;

use super::{DefinedNameS, Parser};

/// The positions of `names` by scope and lower-cased name, the first of names that fold alike
/// (`Parser::name_index`).
pub(super) fn index_names(names: &[DefinedNameS]) -> HashMap<(Option<u32>, String), usize> {
    let mut index = HashMap::with_capacity(names.len());
    for (position, (name, scope, _)) in names.iter().enumerate() {
        index
            .entry((*scope, name.to_lowercase()))
            .or_insert(position);
    }
    index
}

impl Parser {
    // Returns:
    //  * None: If there is no defined name by that name
    //  * Some((Some(index), formula)): If there is a defined name local to that sheet
    //  * Some(None): If there is a global defined name
    pub(super) fn get_defined_name(&self, name: &str, sheet: u32) -> Option<(Option<u32>, String)> {
        let lower = name.to_lowercase();
        if let Some(formula) = self.defined_name_formula(Some(sheet), &lower) {
            return Some((Some(sheet), formula.to_owned()));
        }
        self.defined_name_formula(None, &lower)
            .map(|formula| (None, formula.to_owned()))
    }

    /// The body of the name `lower` (already lower-cased) in `scope`, the first of the names that
    /// fold to it; `None` when there is none.
    pub(super) fn defined_name_formula(&self, scope: Option<u32>, lower: &str) -> Option<&str> {
        let index = *self.name_index.get(&(scope, lower.to_owned()))?;
        self.defined_names
            .get(index)
            .map(|(_, _, formula)| formula.as_str())
    }
}
