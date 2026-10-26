//! Renaming a defined name in a parsed formula.

use crate::expressions::parser::{for_each_chain_operand_mut, Node};

pub(crate) fn rename_defined_name_in_node(
    node: &mut Node,
    name: &str,
    scope: Option<u32>,
    new_name: &str,
) {
    match node {
        Node::DefinedNameKind((n, s, _)) => {
            let prefix_len = n.len() - crate::expressions::utils::bare_defined_name(n).len();
            if name.to_lowercase() == n[prefix_len..].to_lowercase() && *s == scope {
                n.replace_range(prefix_len.., new_name); // `Sheet!` stays as typed
            }
        }
        // Go next level
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            rename_defined_name_in_node(left, name, scope, new_name);
            rename_defined_name_in_node(right, name, scope, new_name);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                rename_defined_name_in_node(operand, name, scope, new_name);
            });
        }
        Node::FunctionKind { kind: _, args } => {
            for arg in args {
                rename_defined_name_in_node(arg, name, scope, new_name);
            }
        }
        Node::NamedFunctionKind {
            name: _,
            args,
            id: _,
        } => {
            for arg in args {
                rename_defined_name_in_node(arg, name, scope, new_name);
            }
        }
        Node::UnaryKind { kind: _, right } => {
            rename_defined_name_in_node(right, name, scope, new_name);
        }
        Node::ImplicitIntersection { child, .. } => {
            rename_defined_name_in_node(child, name, scope, new_name);
        }
        Node::SpillRangeOperator { child }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child } => {
            rename_defined_name_in_node(child, name, scope, new_name);
        }
        // Do nothing
        Node::BooleanKind(_) => {}
        Node::NumberKind(_) => {}
        Node::StringKind(_) => {}
        Node::ErrorKind(_) => {}
        Node::ParseErrorKind { .. } => {}
        Node::ArrayKind(_) => {}
        Node::EmptyArgKind => {}
        Node::ReferenceKind { .. } => {}
        Node::RangeKind { .. } => {}
        Node::SheetSpanKind { .. } => {}
        Node::WrongReferenceKind { .. } => {}
        Node::WrongRangeKind { .. } => {}
        Node::StructuredReferenceKind { .. } => {}
        Node::ThisRowKind { .. } => {}
        Node::NamedVariableKind { .. } => {}
        Node::LambdaDefKind {
            parameters: _,
            body,
        } => {
            rename_defined_name_in_node(body, name, scope, new_name);
        }
        Node::LambdaCallKind { lambda, args } => {
            rename_defined_name_in_node(lambda, name, scope, new_name);
            for arg in args {
                rename_defined_name_in_node(arg, name, scope, new_name);
            }
        }
    }
}
