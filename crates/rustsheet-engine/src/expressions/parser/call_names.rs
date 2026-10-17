//! The spelling of a name called like a function that is none, `=foo(1)`.
//!
//! Excel keeps one spelling per such name for the whole workbook, the first it meets in load
//! order and then in edit order, case-insensitively: after `=foo(1)`, a typed `=Foo(1)` or
//! `=FOO(1)` prints `=foo(1)` (probes/unknown-function-name-spelling,
//! docs/excel-semantics/references.md "Unknown function call names", T-1719).

use std::collections::HashMap;

use super::stringify::bound_variables::{walk_names, NameSite};
use super::Node;

/// Lowercase name → the spelling every call of it prints with.
#[derive(Clone, Default)]
pub(crate) struct CallNameSpellings {
    first: HashMap<String, String>,
}

impl CallNameSpellings {
    /// Gives every callee of `node` that no LET or LAMBDA binds the workbook's spelling, which a
    /// name met for the first time becomes. A bound callee keeps the lowercase it always printed
    /// with. Returns whether a name changed.
    pub(crate) fn apply(&mut self, node: &mut Node) -> bool {
        let mut changed = false;
        walk_names(node, &mut Vec::new(), &mut |site, name, bound| {
            if site != NameSite::Callee {
                return;
            }
            let lower = name.to_lowercase();
            let spelling = if bound {
                lower
            } else {
                self.first
                    .entry(lower)
                    .or_insert_with(|| name.clone())
                    .clone()
            };
            if *name != spelling {
                *name = spelling;
                changed = true;
            }
        });
        changed
    }
}
