//! The export side of the legacy `@` rewrite: which `@`s [`add_implicit_intersection`] would add
//! again on import, so the export drops them instead of writing `_xlfn.SINGLE`.

#[cfg(doc)]
use super::add_implicit_intersection;
use super::legacy::{
    leaves_call_whole, legacy_array_context, legacy_context_intersects, remove_array_at,
    remove_in_array_context,
};
use super::{
    add_legacy_intersection, for_each_chain_operand_mut, get_function_args_signature,
    is_array_context, Node,
};

/// Inverse of [`add_implicit_intersection`], used when exporting to Excel.
///
/// The internal (RC) representation stores every implicit intersection as `@`,
/// losing the `automatic` flag. When exporting we must decide, for each `@`,
/// whether it should be written as `_xlfn.SINGLE(...)` (because it is meaningful)
/// or dropped (because `add_implicit_intersection` would re-insert it on import).
///
/// That decision depends on the *context* of the operator, exactly like
/// `add_implicit_intersection`: an `@` is redundant only in a position that
/// expects a scalar (`add == true`). Crucially, an `@` sitting in a `Vector`
/// argument of a function such as `SUM` is **not** redundant: on import no
/// automatic intersection is added there, so dropping it would change the result
/// (e.g. `SUM(A1,B1,@J:J)` would become `SUM(A1,B1,J:J)`, scanning the whole
/// column). This pass removes only the redundant operators, leaving the rest to
/// be stringified as `_xlfn.SINGLE`.
///
/// `add` mirrors the flag in `add_implicit_intersection` and must start `true`
/// (a formula is evaluated in a scalar context).
pub fn remove_redundant_implicit_intersection(node: &mut Node, add: bool) {
    remove_redundant_legacy_intersection(node, add, add);
}

/// [`remove_redundant_implicit_intersection`] with the `arrays` flag of
/// [`add_legacy_intersection`].
pub(super) fn remove_redundant_legacy_intersection(node: &mut Node, add: bool, arrays: bool) {
    match node {
        // Leaves and nodes that never contain a nested implicit intersection.
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ErrorKind(_)
        | Node::EmptyArgKind
        | Node::ParseErrorKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::NamedFunctionKind { .. }
        | Node::ArrayKind(_)
        | Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::OpRangeKind { .. }
        | Node::OpIntersectKind { .. }
        | Node::OpUnionKind { .. }
        | Node::DefinedNameKind(_)
        | Node::NamedVariableKind { .. }
        | Node::LambdaDefKind { .. }
        | Node::LambdaCallKind { .. } => {}
        Node::ImplicitIntersection { child, .. } => {
            if add || arrays {
                // Would `add_implicit_intersection` re-insert this operator on
                // import? Probe the child in the same context to find out.
                let mut probe = child.as_ref().clone();
                add_legacy_intersection(&mut probe, add, arrays);
                if matches!(probe, Node::ImplicitIntersection { .. }) {
                    // Redundant: drop the operator and keep cleaning the child.
                    // The child is still in the same (scalar) context, so recurse
                    // with `add` to also remove any nested redundant operators.
                    let mut inner = child.as_ref().clone();
                    remove_redundant_legacy_intersection(&mut inner, add, arrays);
                    *node = inner;
                    return;
                }
            }
            // Meaningful operator: keep it, but still clean any nested ones.
            remove_redundant_legacy_intersection(child, false, false);
        }
        Node::SpillRangeOperator { child } => {
            remove_redundant_legacy_intersection(child, add, false);
        }
        Node::Group { child, .. } => remove_redundant_legacy_intersection(child, add, arrays),
        Node::UnaryKind { right, .. } => remove_redundant_legacy_intersection(right, add, arrays),
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                remove_redundant_legacy_intersection(operand, add, arrays);
            });
        }
        Node::FunctionKind { kind, args } if is_array_context(kind) => {
            remove_in_array_context(args);
        }
        Node::FunctionKind { kind, args } => {
            let signature = get_function_args_signature(kind, args.len());
            for (index, arg) in args.iter_mut().enumerate() {
                let arrays = legacy_array_context(kind, signature.get(index), index, arg, arrays);
                remove_array_at(kind, signature.get(index), index, arg, arrays);
                // Scalar arguments are an intersecting context; vector arguments
                // (ranges/arrays) are not (`legacy_context_intersects`).
                let child_add = legacy_context_intersects(kind, signature.get(index), index, arg)
                    && !leaves_call_whole(kind, arg);
                remove_redundant_legacy_intersection(arg, child_add, arrays);
            }
        }
    };
}
