//! The `_xlpm.` form of LAMBDA/LET bound names: Excel's, on export, and the R1C1 storage form's.

use super::super::{for_each_chain_operand_mut, Node};
use crate::expressions::{types::CellReferenceRC, utils::reads_as_r1c1_cell};
use crate::functions::logical::same_variable;
use crate::functions::Function;

/// Excel stores LAMBDA parameters and LET variables with an `_xlpm.` prefix,
/// both at the declaration and at every use site:
/// `LET(x,1,x*2)` is written as `_xlfn.LET(_xlpm.x,1,_xlpm.x*2)`.
/// Internally IronCalc keeps the bare names, so before exporting we walk the
/// tree tracking which names are bound by an enclosing LAMBDA/LET and rename
/// those occurrences. Unbound names (e.g. a defined name that only resolves at
/// evaluation time) are left alone. Matching is case-insensitive, mirroring
/// evaluation (see `same_variable`).
pub(super) fn prefix_bound_variables(node: &mut Node, bound: &mut Vec<String>) {
    match node {
        Node::NamedVariableKind { name, .. } => {
            if bound.iter().any(|n| same_variable(n, name)) {
                *name = format!("_xlpm.{name}");
            }
        }
        // A bound lambda used as a function: `LET(f,LAMBDA(a,a*a),f(2))`
        Node::NamedFunctionKind { name, args, .. } => {
            if bound.iter().any(|n| same_variable(n, name)) {
                *name = format!("_xlpm.{name}");
            }
            for arg in args {
                prefix_bound_variables(arg, bound);
            }
        }
        Node::LambdaDefKind { parameters, body } => {
            let depth = bound.len();
            for parameter in parameters.iter() {
                bound.push(parameter.name.clone());
            }
            prefix_bound_variables(body, bound);
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
                    *name = format!("_xlpm.{name}");
                }
                prefix_bound_variables(&mut args[2 * i + 1], bound);
            }
            prefix_bound_variables(&mut args[2 * pair_count], bound);
            bound.truncate(depth);
        }
        Node::FunctionKind { args, .. } => {
            for arg in args {
                prefix_bound_variables(arg, bound);
            }
        }
        Node::LambdaCallKind { lambda, args } => {
            prefix_bound_variables(lambda, bound);
            for arg in args {
                prefix_bound_variables(arg, bound);
            }
        }
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            prefix_bound_variables(left, bound);
            prefix_bound_variables(right, bound);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| prefix_bound_variables(operand, bound));
        }
        Node::UnaryKind { right, .. } => prefix_bound_variables(right, bound),
        Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::Group { child, .. } => prefix_bound_variables(child, bound),
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
