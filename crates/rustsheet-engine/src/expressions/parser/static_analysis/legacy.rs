//! Where a legacy (pre-dynamic-array) formula implicitly intersects an argument that a dynamic
//! formula reads whole: the rules behind the automatic `@`s of `add_implicit_intersection` and
//! their mirror in `remove_redundant_implicit_intersection`. Excel:
//! docs/excel-semantics/references.md "Legacy formulas intersect the ranges in value arguments".

use super::{
    add_implicit_intersection, for_each_chain_operand_mut, remove_redundant_implicit_intersection,
    run_static_analysis_on_node, Node, Signature, StaticResult,
};
use crate::functions::Function;

/// Legacy formulas evaluate SUMPRODUCT's arguments as arrays, with no implicit intersection inside
/// (`SUMPRODUCT(SUMIFS(B1:B6,A1:A6,D1:D2))` lifts the criterion where `SUM` intersects it;
/// probes/ifs-legacy-sumproduct, excel-graph, T-0278), except for the conditions of
/// [`for_each_array_context_condition`].
pub(super) fn is_array_context(kind: &Function) -> bool {
    matches!(kind, Function::Sumproduct)
}

/// Adds the automatic `@`s inside the arguments of an array-context function
/// ([`is_array_context`]).
pub(super) fn add_in_array_context(args: &mut [Node]) {
    for arg in args {
        for_each_array_context_condition(arg, &mut |condition| {
            if !matches!(run_static_analysis_on_node(condition), StaticResult::Scalar) {
                add_implicit_intersection(condition, true);
            }
        });
    }
}

/// Drops the `@`s [`add_in_array_context`] would add again on import; any other `@` inside the
/// arguments of an array-context function is meaningful and kept.
pub(super) fn remove_in_array_context(args: &mut [Node]) {
    for arg in args {
        for_each_array_context_condition(arg, &mut |condition| {
            remove_redundant_implicit_intersection(condition, true);
        });
    }
}

/// Whether a legacy formula intersects `arg`, the argument of `kind` with `signature`: a scalar
/// argument that is not a scalar, `IFERROR(B1:B6*1,100)` and `IF(A1:A6="a",1,0)` included
/// (probes/legacy-value-argument-intersection I10, array-context-legacy-sumproduct R10), or an
/// argument [`legacy_context_intersects`] makes a value.
pub(super) fn legacy_intersects(
    kind: &Function,
    signature: Option<&Signature>,
    arg: &Node,
) -> bool {
    (is_scalar_context(kind, signature)
        && !matches!(run_static_analysis_on_node(arg), StaticResult::Scalar))
        || legacy_value_operand(kind, arg)
}

/// Whether the argument `arg` of `kind` with `signature` is read as a value, where an `@` is
/// redundant because [`legacy_intersects`] adds it again on import.
pub(super) fn legacy_context_intersects(
    kind: &Function,
    signature: Option<&Signature>,
    arg: &Node,
) -> bool {
    is_scalar_context(kind, signature) || legacy_value_operand(kind, arg)
}

/// A scalar argument, or any argument of a function whose arguments a legacy formula reads as
/// scalars although a dynamic one takes them whole: legacy `TYPE(B1:B6)` is 1 on the range's rows
/// and 16 (`#VALUE!`) off them, where a dynamic one is 64 (probes/legacy-type-intersection,
/// fixtures/external-link-cached-ranges Refs!B62; T-1526).
fn is_scalar_context(kind: &Function, signature: Option<&Signature>) -> bool {
    matches!(signature, Some(Signature::Scalar)) || matches!(kind, Function::Type)
}

/// Calls `f` on each argument under an array context ([`is_array_context`]) that a legacy formula
/// still intersects: the condition of IF and the value of IFERROR and IFNA, at any depth through
/// the operators and functions that keep the array context. `SUMPRODUCT(IF(B1:B6>1,B1:B6,0))`,
/// `SUMPRODUCT(ABS(IF(B1:B6>1,B1:B6,0)))` and `SUMPRODUCT(IFERROR(B1:B6,100))` are `#VALUE!`,
/// `#VALUE!` and 100 off the range's rows, while IF's value branches stay arrays
/// (`SUMPRODUCT(IF(TRUE,B1:B6))` 12; probes/array-context-legacy-sumproduct F10, S10, U10,
/// legacy-value-argument-intersection J10, K10, L10, N10; T-0293, T-1526). Recursion is bounded
/// by the parser's depth cap.
fn for_each_array_context_condition(node: &mut Node, f: &mut impl FnMut(&mut Node)) {
    match node {
        Node::FunctionKind {
            kind: Function::If | Function::Iferror | Function::Ifna,
            args,
        } => {
            let mut args = args.iter_mut();
            if let Some(condition) = args.next() {
                f(condition);
            }
            for arg in args {
                for_each_array_context_condition(arg, f);
            }
        }
        Node::FunctionKind { args, .. } => {
            for arg in args {
                for_each_array_context_condition(arg, f);
            }
        }
        Node::UnaryKind { right: child, .. } | Node::Group { child, .. } => {
            for_each_array_context_condition(child, f);
        }
        Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                for_each_array_context_condition(operand, f);
            });
        }
        _ => {}
    }
}

/// Whether `arg` of `kind` is arithmetic or a comparison that a legacy formula evaluates as a
/// value, intersecting the ranges in it, although `kind` takes a range there: `SUM(B1:B6+0)`,
/// `SUM(-B1:B6)`, `MAX(B1:B6*1)` and `LARGE(B1:B6*1,1)` are `#VALUE!` off the range's rows and
/// `COUNT(B1:B6*1)` 0, while `SUM(B1:B6)` and `SUM((B1:B6))` read the range
/// (probes/legacy-value-argument-intersection; `[1]Uses!$A$2:$A$3+0` in
/// fixtures/external-link-cached-ranges Refs!B44; T-1526), as for the other aggregates in the match
/// below (probes/legacy-value-argument-intersection-b, T-1551, and -c, T-1556); MODE, MODE.MULT, MODE.SNGL, MMULT, INDEX and LOOKUP
/// evaluate such an argument as an array (that probe S; probes/array-context-legacy-sumproduct
/// N, P, Q). docs/excel-semantics/references.md has the result per function.
fn legacy_value_operand(kind: &Function, arg: &Node) -> bool {
    matches!(
        kind,
        Function::Sum
            | Function::Average
            | Function::Averagea
            | Function::Max
            | Function::Count
            | Function::Counta
            | Function::Large
            | Function::Median
            | Function::Min
            | Function::MinA
            | Function::MaxA
            | Function::Product
            | Function::Sumsq
            | Function::StDevS
            | Function::StDevP
            | Function::VarS
            | Function::VarP
            | Function::Devsq
            | Function::Avedev
            | Function::Geomean
            | Function::Harmean
            | Function::Small
            | Function::PercentileInc
            | Function::QuartileInc
            | Function::Kurt
            | Function::Skew
            | Function::Gcd
            | Function::Lcm
            | Function::Concat
            | Function::Textjoin
            | Function::Stdev
            | Function::StDevPCompat
            | Function::VarCompat
            | Function::VarPCompat
            | Function::Stdeva
            | Function::Stdevpa
            | Function::VarA
            | Function::VarpA
            | Function::PercentileCompat
            | Function::QuartileCompat
            | Function::PercentileExc
            | Function::QuartileExc
            | Function::PercentrankCompat
            | Function::PercentrankInc
    ) && is_value_operation(arg)
}

/// Whether `node` is arithmetic, concatenation or a comparison, possibly in parentheses:
/// `SUM((B1:B6*1))` intersects like `SUM(B1:B6*1)`, unlike `SUM((B1:B6))`
/// (probes/legacy-value-argument-intersection-b Z2, Z10; T-1551).
fn is_value_operation(node: &Node) -> bool {
    match node {
        Node::Group { child, .. } => is_value_operation(child),
        Node::UnaryKind { .. }
        | Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. } => true,
        _ => false,
    }
}
