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

/// Lowercase name → the spelling every call or value use of it prints with.
#[derive(Clone, Default)]
pub(crate) struct CallNameSpellings {
    first: HashMap<String, String>,
}

impl CallNameSpellings {
    /// Gives every callee and value-used name of `node` that no LET or LAMBDA binds the workbook's spelling, which a
    /// name met for the first time becomes. A bound name, called or used, takes the spelling of
    /// its declaration. Returns whether a name changed.
    pub(crate) fn apply(&mut self, node: &mut Node) -> bool {
        let mut changed = false;
        walk_names(node, &mut Vec::new(), &mut |site, name, declared| {
            let spelling = match (declared, site) {
                (Some(declared), _) => declared.to_string(),
                (None, NameSite::Callee | NameSite::Use) => self
                    .first
                    .entry(name.to_lowercase())
                    .or_insert_with(|| name.clone())
                    .clone(),
                (None, NameSite::Declaration) => return,
            };
            if *name != spelling {
                *name = spelling;
                changed = true;
            }
        });
        changed
    }
}
