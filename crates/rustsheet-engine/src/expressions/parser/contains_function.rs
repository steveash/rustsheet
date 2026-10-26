//! Walking a formula tree: the children of a node, and searching for calls of given functions.

use super::{chain, Node};
use crate::functions::Function;

/// The pattern of the nodes with exactly one child, binding it to `$child`: one list for the
/// shared and the mutable walks below.
macro_rules! one_child {
    ($child:ident) => {
        Node::SheetSpanKind {
            reference: $child,
            ..
        } | Node::ImplicitIntersection { child: $child, .. }
            | Node::SpillRangeOperator { child: $child }
            | Node::Group { child: $child, .. }
            | Node::UnaryPlus { child: $child }
            | Node::Spaced { child: $child, .. }
            | Node::UnaryKind { right: $child, .. }
            | Node::LambdaDefKind { body: $child, .. }
    };
}

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

    /// Whether the tree holds a LAMBDA call, written out as `LAMBDA(x,x)(1)`.
    pub(crate) fn contains_lambda_call(&self) -> bool {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if matches!(node, Node::LambdaCallKind { .. }) {
                return true;
            }
            node.push_children(&mut pending);
        }
        false
    }

    /// Whether the tree holds a legacy N or T (SPEC-0012), the one fact of a legacy formula its
    /// display text does not show, so an edit that re-enters the text must keep it (T-1852).
    pub(crate) fn has_legacy_marker(&self) -> bool {
        self.contains_function(&Function::INTERNAL)
    }

    /// Whether the tree reads, at any depth, a variable (an id patched in by LET or LAMBDA) for
    /// which `is_target` holds, as a value or as a called function. Walked with an explicit stack
    /// like [`Node::contains_function`].
    pub(crate) fn reads_variable(&self, mut is_target: impl FnMut(usize) -> bool) -> bool {
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

    /// Whether the tree holds, at any depth, a `#This Row` reference to one of `tables` (lowercase
    /// names), whose printed area then depends on the row of the cell (T-1699).
    pub(crate) fn has_this_row_to(&self, tables: &[String]) -> bool {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if let Node::StructuredReferenceKind { reference, lowered } = node {
                if matches!(**lowered, Node::ThisRowKind { .. })
                    && tables.contains(&reference.table_name.to_lowercase())
                {
                    return true;
                }
            }
            node.push_children(&mut pending);
        }
        false
    }

    /// Replaces every `#This Row` structured reference in the tree by its area on `row` (the cell
    /// or range, or `#VALUE!` off the data rows), as the parse-time lowering did before SPEC-0016
    /// step 2 (T-1699). For a formula with no cell of its own, a defined name's body. An explicit
    /// stack keeps the walk off the call stack.
    pub(crate) fn lower_this_rows_at(&mut self, row: i32) {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            let area = match node {
                Node::StructuredReferenceKind { lowered, .. } => lowered.this_row_at(row),
                _ => None,
            };
            if let Some(area) = area {
                *node = area;
                continue;
            }
            node.push_children_mut(&mut pending);
        }
    }

    /// Whether the tree holds whitespace written in the formula ([`Node::Spaced`], SPEC-0023).
    pub(crate) fn has_spaces(&self) -> bool {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if matches!(node, Node::Spaced { .. }) {
                return true;
            }
            node.push_children(&mut pending);
        }
        false
    }

    /// The tree without the whitespace written in it: every [`Node::Spaced`] replaced by its
    /// child. An explicit stack keeps the walk off the call stack.
    pub(crate) fn strip_spaces(&mut self) {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            while let Node::Spaced { child, .. } = node {
                let child = std::mem::replace(&mut **child, Node::EmptyArgKind);
                *node = child;
            }
            node.push_children_mut(&mut pending);
        }
    }

    /// [`Node::push_children`] for the walks that rewrite the tree in place.
    fn push_children_mut<'a>(&'a mut self, pending: &mut Vec<&'a mut Node>) {
        match self {
            Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
                pending.extend(args.iter_mut());
            }
            one_child!(child) => pending.push(child),
            Node::OpRangeKind { left, right }
            | Node::OpIntersectKind { left, right, .. }
            | Node::OpUnionKind { left, right, .. } => {
                pending.extend([&mut **left, &mut **right]);
            }
            Node::LambdaCallKind { lambda, args } => {
                pending.push(lambda);
                pending.extend(args.iter_mut());
            }
            other => {
                if let Ok((left, right)) = chain::value_operands_mut(other) {
                    pending.extend([left, right]);
                }
            }
        }
    }

    /// Pushes the nodes directly under this one, the operands of every operator and the arguments
    /// of every call, LAMBDA bodies included, for the explicit-stack walks over a formula tree.
    pub(crate) fn push_children<'a>(&'a self, pending: &mut Vec<&'a Node>) {
        match self {
            Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
                pending.extend(args);
            }
            one_child!(child) => pending.push(child),
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
