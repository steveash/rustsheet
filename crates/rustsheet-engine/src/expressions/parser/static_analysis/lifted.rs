//! Static analysis of the functions in the lifting table (SPEC-0011 "The table wins"): their
//! argument signatures, their result shapes and the legacy `@` of their `LiftArrays` arguments
//! are derived from `Function::arg_kinds`, never written by hand.

use super::broadcast::broadcast_shapes;
use super::reference_parameter::reference_by_caller_shape;
use super::reference_result::yields_cells;
use super::{run_static_analysis_on_node, Node, Signature, StaticResult};
use crate::functions::lift::{ArgKind, ArgKinds};
use crate::functions::Function;

/// A `Lift` argument is `Signature::Scalar`, so a legacy formula intersects a range in it
/// (`=PMT(@C1:C60,10,100)`); every other kind is `Signature::Vector`, so a bare range keeps its
/// text (`=EDATE(A1:A60,1)`, probes/scalar-lift-shapes-c E, H). A bad count is all
/// `Signature::Error`.
pub(super) fn lifted_signature(kinds: &ArgKinds, arg_count: usize) -> Vec<Signature> {
    if !kinds.accepts(arg_count) {
        return vec![Signature::Error; arg_count];
    }
    (0..arg_count)
        .map(|index| match kinds.kind(index) {
            Some(ArgKind::Lift) => Signature::Scalar,
            _ => Signature::Vector,
        })
        .collect()
}

/// The shape of a call: its lifted arguments broadcast. A `Lift` argument adds the shape of an
/// array or a range; a `LiftArrays` or `LiftArraysRangeWhole` argument only that of an array,
/// since a range there is refused or read whole (`EDATE(A1:A3,1)` is a scalar,
/// `EDATE({1;2},A1:A3)` a 2x1 array). A range shape whose arms may be values is an array: CHOOSE
/// gives its array arms a range's shape (`IMABS(CHOOSE(1,{1,2,3}))` is 1x3,
/// probes/lift-arrays-choose-arms A13, T-1874). An `Unknown` lifted argument makes the call
/// `Unknown`; a `ReferenceByCaller` argument adds the shape of an array that is not a reference
/// (RANK's `ref`); a `Whole` argument adds nothing. A call with a bad count is a scalar (the count
/// error).
pub(super) fn lifted_static_result(kinds: &ArgKinds, args: &[Node]) -> StaticResult {
    if !kinds.accepts(args.len()) {
        return StaticResult::Scalar;
    }
    broadcast_shapes(
        args.iter()
            .enumerate()
            .map(|(index, arg)| match kinds.kind(index) {
                Some(ArgKind::Lift) => run_static_analysis_on_node(arg),
                Some(ArgKind::LiftArrays | ArgKind::LiftArraysRangeWhole) => {
                    match run_static_analysis_on_node(arg) {
                        StaticResult::Range(..) if yields_cells(arg, &mut Vec::new()) => {
                            StaticResult::Scalar
                        }
                        other => other,
                    }
                }
                Some(ArgKind::ReferenceByCaller) => reference_by_caller_shape(arg),
                Some(ArgKind::Whole | ArgKind::WholeByCaller) | None => StaticResult::Scalar,
            }),
    )
}

/// Whether argument `index` of `kind` is a `LiftArrays` argument: a legacy formula intersects
/// the ranges of a value operation there, as in an aggregate's argument (`=EDATE(A1:A60+0,1)`
/// shows `=EDATE(@A1:A60+0,1)`, probes/scalar-lift-shapes-c F, Q).
pub(super) fn lifts_arrays_at(kind: &Function, index: usize) -> bool {
    kind.arg_kinds()
        .and_then(|kinds| kinds.kind(index))
        .is_some_and(|kind| kind == ArgKind::LiftArrays)
}

#[cfg(test)]
mod tests {
    use super::super::{fallback_shape, fallback_signature};
    use crate::expressions::parser::Node;
    use crate::functions::lift::lifting_table_functions;
    use crate::functions::Function;

    /// The table wins: no function in the lifting table also has a hand-written signature or
    /// shape arm, so the two can never disagree (SPEC-0011 Test plan).
    #[test]
    fn no_function_in_the_table_has_a_fallback_arm() {
        let mut entered = 0;
        for function in Function::into_iter() {
            let Some(kinds) = function.arg_kinds() else {
                continue;
            };
            entered += 1;
            for count in 0..=kinds.max + 1 {
                assert!(
                    fallback_signature(&function, count).is_none(),
                    "{function:?}: a fallback signature for {count} arguments"
                );
                let args = vec![Node::NumberKind(1.0); count];
                assert!(
                    fallback_shape(&function, &args).is_none(),
                    "{function:?}: a fallback shape for {count} arguments"
                );
            }
        }
        assert!(
            entered >= 16,
            "the date, IS*, ERROR.TYPE and ENCODEURL functions are in the table"
        );
    }

    /// The static-analysis pattern lists exactly the functions of the table.
    #[test]
    fn the_pattern_macro_matches_the_table() {
        for function in Function::into_iter() {
            assert_eq!(
                matches!(function, lifting_table_functions!()),
                function.arg_kinds().is_some(),
                "{function:?}"
            );
        }
    }

    /// And the reverse: a function outside the table keeps its hand-written arms.
    #[test]
    fn a_function_outside_the_table_has_its_fallback_arms() {
        let args = [Node::NumberKind(2.0), Node::NumberKind(3.0)];
        assert!(Function::Power.arg_kinds().is_none());
        assert!(fallback_signature(&Function::Power, 2).is_some());
        assert!(fallback_shape(&Function::Power, &args).is_some());
    }
}
