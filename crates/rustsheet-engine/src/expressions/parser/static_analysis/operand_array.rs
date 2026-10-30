//! The array constants a value is made of through operators, parentheses and signs, which a
//! legacy formula intersects one by one (`=N(@{2;3}+0)`, `=@{2;3}+1`; docs/excel-semantics/
//! references.md "Legacy formulas put `@` on array-valued calls and array constants").

use super::{for_each_chain_operand_mut, Node};

/// Puts an `@` on each array constant `node` is made of through operators, parentheses and signs
/// ([`for_each_value_operand`]): `{2;3}+0` becomes `@{2;3}+0`.
pub(super) fn add_operand_array_at(node: &mut Node) {
    for_each_value_operand(node, &mut |operand| {
        if matches!(operand, Node::ArrayKind(_)) {
            *operand = Node::automatic_at(operand.clone());
        }
    });
}

/// Drops the `@`s [`add_operand_array_at`] adds again on import.
pub(super) fn remove_operand_array_at(node: &mut Node) {
    for_each_value_operand(node, &mut |operand| {
        if let Node::ImplicitIntersection { child, .. } = operand {
            if matches!(child.as_ref(), Node::ArrayKind(_)) {
                *operand = child.as_ref().clone();
            }
        }
    });
}

/// Calls `f` on `node`, or on each operand of the operators, parentheses and signs it is made
/// of, at any depth (bounded by the parser's depth cap).
pub(super) fn for_each_value_operand(node: &mut Node, f: &mut impl FnMut(&mut Node)) {
    match node {
        Node::UnaryKind { right: child, .. }
        | Node::Group { child, .. }
        | Node::UnaryPlus { child }
        | Node::Spaced { child, .. } => {
            for_each_value_operand(child, f);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| for_each_value_operand(operand, f));
        }
        _ => f(node),
    }
}
