//! The spelling of a name called like a function that is none, `=foo(1)`, and of a LET or LAMBDA
//! name's uses.
//!
//! A use of a LET or LAMBDA name, called or not, prints in the spelling of the innermost
//! declaration that binds it: `LET(x,5,X*2)` reads `LET(x,5,x*2)`, and `LET(x,1,LET(X,5,x))`
//! reads `LET(x,1,LET(X,5,X))` (probes/let-name-use-spelling, T-1720).
//!
//! Excel keeps one spelling per such name, called or used as a value (T-1760), for the whole
//! workbook, the first it meets in load order and then in edit order, case-insensitively: after
//! `=foo(1)`, a typed `=Foo(1)` or `=FOO(1)` prints `=foo(1)` (probes/unknown-function-name-spelling,
//! docs/excel-semantics/references.md "Unknown function call names", T-1719).

use std::collections::HashMap;

use super::stringify::bound_variables::{walk_names, NameSite};
use super::Node;
use crate::types::DefinedName;

/// The registry key of a name: upper-cased, except that dotless ı, long ſ and the micro sign µ keep
/// their own key. Excel folds ß/SS, ς/σ/Σ, ﬁ/FI and ǆ/Ǆ but not ı/I, ſ/S or µ/μ/Μ
/// (probes/unknown-function-name-folds, T-1818).
fn spelling_key(name: &str) -> String {
    name.chars()
        .flat_map(|c| {
            let keep = matches!(c, 'ı' | 'ſ' | '\u{b5}');
            let (own, upper) = (keep.then_some(c), (!keep).then(|| c.to_uppercase()));
            own.into_iter().chain(upper.into_iter().flatten())
        })
        .collect()
}

/// Upper-cased name (ß folds with SS, T-1762; see `spelling_key`) → the spelling every call or value use of it prints with.
#[derive(Clone, Default)]
pub(crate) struct CallNameSpellings {
    first: HashMap<String, String>,
    /// Upper-cased defined name → its spelling, refreshed with every reparse of the workbook's
    /// formulas (every defined-name edit reparses them), so a lookup costs one allocation.
    defined: HashMap<String, String>,
}

impl CallNameSpellings {
    /// Registers the spellings of the workbook's defined names; the first of equal names wins.
    pub(crate) fn set_defined_names(&mut self, defined_names: &[DefinedName]) {
        self.defined.clear();
        for d in defined_names {
            self.defined
                .entry(spelling_key(&d.name))
                .or_insert_with(|| d.name.clone());
        }
    }

    /// Gives every callee and value-used name of `node` that no LET or LAMBDA binds the workbook's spelling, which a
    /// name met for the first time becomes. A bound name, called or used, takes the spelling of
    /// its declaration. A callee that is a defined name takes the defined name's spelling, whatever
    /// the first typed one (probes/unknown-function-name-reach, T-1763). Returns whether a name
    /// changed.
    pub(crate) fn apply(&mut self, node: &mut Node) -> bool {
        let mut changed = false;
        walk_names(node, &mut Vec::new(), &mut |site, name, declared| {
            let spelling = match (declared, site) {
                (Some(declared), _) => declared.to_string(),
                (None, NameSite::Declaration) => return,
                (None, NameSite::Callee | NameSite::Use) => {
                    let key = spelling_key(name);
                    match self.defined.get(&key) {
                        Some(defined) if site == NameSite::Callee => defined.clone(),
                        _ => self
                            .first
                            .entry(key)
                            .or_insert_with(|| name.clone())
                            .clone(),
                    }
                }
            };
            if *name != spelling {
                *name = spelling;
                changed = true;
            }
        });
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::spelling_key;

    #[test]
    fn folds_match_excel() {
        for (a, b) in [
            ("ςw", "σw"),
            ("ςu", "Σu"),
            ("ﬁq", "FIq"),
            ("ǆd", "Ǆd"),
            ("straße", "STRASSE"),
        ] {
            assert_eq!(spelling_key(a), spelling_key(b), "{a} {b}");
        }
        for (a, b) in [
            ("ıx", "Ix"),
            ("ſy", "Sy"),
            ("\u{b5}z", "μz"),
            ("\u{b5}v", "Μv"),
        ] {
            assert_ne!(spelling_key(a), spelling_key(b), "{a} {b}");
        }
    }
}
