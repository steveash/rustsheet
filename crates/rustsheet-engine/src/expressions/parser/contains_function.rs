//! Walking a formula tree: the children of a node, and searching for calls of given functions.

use super::{chain, Node};
use crate::functions::Function;

impl Node {
    /// Whether the tree holds a call of one of `kinds` at any depth. SUBTOTAL and AGGREGATE skip
    /// every cell whose formula does (docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE
    /// over hidden and filtered rows", probes/subtotal-hidden-filtered Nest sheet). An explicit
    /// stack keeps the walk off the call stack: a formula's tree is untrusted input.
    pub(crate) fn contains_function(&self, kinds: &[Function]) -> bool {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if let Node::FunctionKind { kind, .. } = node {
                if kinds.contains(kind) {
                    return true;
                }
            }
            node.push_children(&mut pending);
        }
        false
    }

    /// Whether the tree reads, at any depth, a variable (an id patched in by LET or LAMBDA) for
    /// which `is_target` holds, as a value or as a called function. Walked with an explicit stack
    /// like [`Node::contains_function`].
    pub(crate) fn reads_variable(&self, is_target: impl Fn(usize) -> bool) -> bool {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if let Node::NamedVariableKind { id: Some(id), .. }
            | Node::NamedFunctionKind { id: Some(id), .. } = node
            {
                if is_target(*id as usize) {
                    return true;
                }
            }
            node.push_children(&mut pending);
        }
        false
    }

    /// Pushes the nodes directly under this one, the operands of every operator and the arguments
    /// of every call, LAMBDA bodies included, for the explicit-stack walks over a formula tree.
    pub(crate) fn push_children<'a>(&'a self, pending: &mut Vec<&'a Node>) {
        match self {
            Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
                pending.extend(args);
            }
            Node::SheetSpanKind {
                reference: child, ..
            }
            | Node::ImplicitIntersection { child, .. }
            | Node::SpillRangeOperator { child }
            | Node::Group { child, .. }
            | Node::UnaryKind { right: child, .. }
            | Node::LambdaDefKind { body: child, .. } => pending.push(child),
            Node::OpRangeKind { left, right }
            | Node::OpIntersectKind { left, right, .. }
            | Node::OpUnionKind { left, right, .. } => pending.extend([&**left, &**right]),
            Node::LambdaCallKind { lambda, args } => {
                pending.push(lambda);
                pending.extend(args);
            }
            _ => {
                if let Some((left, right)) = chain::value_operands(self) {
                    pending.extend([left, right]);
                }
            }
        }
    }
}
