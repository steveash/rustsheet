//! Searching a formula tree for calls of given functions.

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
            match node {
                Node::FunctionKind { kind, args } => {
                    if kinds.contains(kind) {
                        return true;
                    }
                    pending.extend(args);
                }
                Node::SheetSpanKind {
                    reference: child, ..
                }
                | Node::ImplicitIntersection { child, .. }
                | Node::SpillRangeOperator { child }
                | Node::GroupedRootKind { child }
                | Node::UnaryKind { right: child, .. }
                | Node::LambdaDefKind { body: child, .. } => pending.push(child),
                Node::OpRangeKind { left, right }
                | Node::OpIntersectKind { left, right, .. }
                | Node::OpUnionKind { left, right, .. } => pending.extend([&**left, &**right]),
                Node::LambdaCallKind { lambda, args } => {
                    pending.push(lambda);
                    pending.extend(args);
                }
                Node::NamedFunctionKind { args, .. } => pending.extend(args),
                _ => {
                    if let Some((left, right)) = chain::value_operands(node) {
                        pending.extend([left, right]);
                    }
                }
            }
        }
        false
    }
}
