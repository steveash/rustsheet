//! The `_xlpm.` form of LAMBDA/LET bound names: Excel's, on export, and the R1C1 storage form's.

use super::super::{for_each_chain_operand_mut, Node};
use crate::expressions::{types::CellReferenceRC, utils::reads_as_r1c1_cell};
use crate::functions::logical::same_variable;
use crate::functions::Function;
use crate::language::get_default_language;

/// Where [`walk_names`] meets a name.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum NameSite {
    /// A name read as a value: a LET or LAMBDA name, or a name that only resolves at evaluation.
    Use,
    /// A name called like a function, `f(2)`: a bound LAMBDA or a name that is no function.
    Callee,
    /// The name a LET pair declares (it is bound from its own value onwards).
    Declaration,
}

/// Calls `visit` with every name of `node`, where it is and whether an enclosing LAMBDA or LET
/// binds it, `bound` holding the names already in scope. Matching is case-insensitive, mirroring
/// evaluation (see `same_variable`). LAMBDA parameters themselves are not visited.
pub(crate) fn walk_names<F>(node: &mut Node, bound: &mut Vec<String>, visit: &mut F)
where
    F: FnMut(NameSite, &mut String, bool),
{
    let is_bound = |bound: &[String], name: &str| bound.iter().any(|n| same_variable(n, name));
    match node {
        Node::NamedVariableKind { name, .. } => {
            let b = is_bound(bound, name);
            visit(NameSite::Use, name, b);
        }
        Node::NamedFunctionKind { name, args, .. } => {
            let b = is_bound(bound, name);
            visit(NameSite::Callee, name, b);
            for arg in args {
                walk_names(arg, bound, visit);
            }
        }
        Node::LambdaDefKind { parameters, body } => {
            let depth = bound.len();
            for parameter in parameters.iter() {
                bound.push(parameter.name.clone());
            }
            walk_names(body, bound, visit);
            bound.truncate(depth);
        }
        Node::FunctionKind {
            kind: Function::Let,
            args,
        } if args.len() >= 3 && args.len() % 2 == 1 => {
            // LET(name1, value1, [name2, value2, ...], body): each name is in
            // scope from its own value expression onwards.
            let depth = bound.len();
            let pair_count = (args.len() - 1) / 2;
            for i in 0..pair_count {
                if let Node::NamedVariableKind { name, .. } = &mut args[2 * i] {
                    bound.push(name.clone());
                    visit(NameSite::Declaration, name, true);
                }
                walk_names(&mut args[2 * i + 1], bound, visit);
            }
            walk_names(&mut args[2 * pair_count], bound, visit);
            bound.truncate(depth);
        }
        Node::FunctionKind { args, .. } => {
            for arg in args {
                walk_names(arg, bound, visit);
            }
        }
        Node::LambdaCallKind { lambda, args } => {
            walk_names(lambda, bound, visit);
            for arg in args {
                walk_names(arg, bound, visit);
            }
        }
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            walk_names(left, bound, visit);
            walk_names(right, bound, visit);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| walk_names(operand, bound, visit));
        }
        Node::UnaryKind { right, .. } => walk_names(right, bound, visit),
        Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::Group { child, .. } => walk_names(child, bound, visit),
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::ArrayKind(_)
        | Node::DefinedNameKind(_)
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => {}
    }
}

/// Excel stores LAMBDA parameters and LET variables with an `_xlpm.` prefix,
/// both at the declaration and at every use site:
/// `LET(x,1,x*2)` is written as `_xlfn.LET(_xlpm.x,1,_xlpm.x*2)`.
/// Internally IronCalc keeps the bare names, so before exporting we walk the
/// tree tracking which names are bound by an enclosing LAMBDA/LET and rename
/// those occurrences, a bound lambda used as a function (`LET(f,LAMBDA(a,a*a),f(2))`)
/// included. Unbound names (e.g. a defined name that only resolves at
/// evaluation time) are left alone.
pub(super) fn prefix_bound_variables(node: &mut Node) {
    walk_names(node, &mut Vec::new(), &mut |_, name, bound| {
        if bound {
            *name = format!("_xlpm.{name}");
        }
    });
}

/// A LET or LAMBDA name (or any unresolved name). In the R1C1 storage form (no `context`), one
/// the R1C1 lexer would read back as a cell gets the `_xlpm.` prefix the parser drops, as on
/// xlsx import: Excel takes `LET(R1C1,5,R1C1*2)` (10, probes/r1c1-let-name-r1c1, T-1658).
pub(super) fn variable_text(name: &str, context: Option<&CellReferenceRC>) -> String {
    // No `context` is the R1C1 storage form (`to_rc_format`); a context is A1 output.
    if context.is_none() && reads_as_r1c1_cell(name) {
        format!("_xlpm.{name}")
    } else {
        name.to_string()
    }
}

/// A name called like a function that is none. In the R1C1 storage form (no `context`), one the
/// stored formula would read back as something else gets the `_xlpm.` prefix the parser drops:
/// a cell, as for [`variable_text`], or a function or boolean of the English the storage is read
/// in (a Spanish `=TRUE()` calls no function, and must not share the stored text of `TRUE()`).
pub(super) fn callee_text(name: &str, context: Option<&CellReferenceRC>) -> String {
    let reads_as_function = || {
        let english = get_default_language();
        english.functions.lookup_stored(name).is_some()
            || name.eq_ignore_ascii_case(&english.booleans.r#true)
            || name.eq_ignore_ascii_case(&english.booleans.r#false)
    };
    if context.is_none() && (reads_as_r1c1_cell(name) || reads_as_function()) {
        format!("_xlpm.{name}")
    } else {
        name.to_string()
    }
}
