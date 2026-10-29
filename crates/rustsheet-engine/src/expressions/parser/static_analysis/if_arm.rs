//! The arms of a legacy IF or IFS that are references, which the formula passes on whole instead of
//! intersecting (docs/excel-semantics/references.md "Legacy IF passes a range arm whole").

use super::{run_static_analysis_on_node, Node, StaticResult};
use crate::functions::Function;

/// Whether `arg` is a value arm of IF or IFS that is a reference ([`is_range_arm`]), which a legacy
/// formula passes on whole as the call's result: the `@` goes on the call where its result is a
/// value (`=IF(TRUE,B1:B6)` shows as `=@IF(TRUE,B1:B6)`, `=ABS(IF(TRUE,B1:B6))` as
/// `=ABS(@IF(TRUE,B1:B6))`), and none where it is a range argument (`=SUM(IF(TRUE,B1:B6))` is 12
/// and `=MATCH(3,IF(B1:B6>0,B1:B6),0)` 3 on and off the range's rows, the condition intersected;
/// probes/legacy-if-arm-range-argument, legacy-match-lookup-array-intersection J2, excel-graph,
/// T-1935). An operation over the range is still a value (`=MATCH(3,IF(TRUE,@B1:B6+0),0)`), and
/// so is IFERROR's value (`=MATCH(3,IFERROR(@B1:B6,0),0)`). An explicit `@` on the range is
/// looked through, so the export keeps it (`SUM(IF(TRUE,_xlfn.SINGLE(B1:B6)))`).
pub(super) fn is_reference_arm(kind: &Function, index: usize, arg: &Node) -> bool {
    is_value_arm(kind, index) && is_range_arm(arg)
}

/// Whether argument `index` of `kind` is an arm whose value IF or IFS returns: IF's two arms, IFS's
/// value arguments (the odd positions; the odd one out is a condition).
fn is_value_arm(kind: &Function, index: usize) -> bool {
    match kind {
        Function::If => index > 0,
        Function::Ifs => index % 2 == 1,
        _ => false,
    }
}

/// Whether `arm` is a range, possibly in parentheses or under an explicit `@`, a `:` range operation,
/// an OFFSET or INDEX of one, a defined name, or an IF or IFS that
/// returns one ([`returns_range_arm`]): `=MATCH(3,IF(TRUE,IF(TRUE,B1:B6)),0)` shows no `@` and
/// is 3 off the range's rows (probes/legacy-if-arm-range-argument T). An explicit stack keeps the
/// walk off the call stack: a formula's tree is untrusted input. Unlike `is_plain_reference`
/// (legacy.rs, N and T's reads) this accepts OFFSET with any arguments and any INDEX of a range
/// (probes/legacy-if-arm-other-references E, F, T-1969).
fn is_range_arm(arm: &Node) -> bool {
    let mut pending = vec![arm];
    while let Some(node) = pending.pop() {
        let node = match node.ungrouped() {
            Node::ImplicitIntersection { child, .. } => child.ungrouped(),
            other => other,
        };
        match node {
            // A defined name is an arm like a range: `=SUM(IF(TRUE,Rng))` shows no `@` and
            // `=IF(TRUE,Rng)` shows `=@IF(TRUE,Rng)` (probes/legacy-if-arm-other-references D,
            // R), although the name takes its own `@` elsewhere (T-2440).
            Node::RangeKind { .. }
            | Node::ThisRowKind { cell: false, .. }
            | Node::OpRangeKind { .. }
            | Node::DefinedNameKind(_) => return true,
            Node::FunctionKind {
                kind: Function::If,
                args,
            } if args.len() > 1 => pending.extend(&args[1..]),
            Node::FunctionKind {
                kind: Function::Ifs,
                args,
            } => pending.extend(args.iter().skip(1).step_by(2)),
            // Only whether the arm counts as a range argument: a resized OFFSET of one cell does, while at the
            // root Excel shows a one-cell OFFSET(B1,0,0) arm with no `@` (probes/legacy-if-arm-other-references S).
            Node::FunctionKind {
                kind: Function::Offset,
                args,
            } if !args.is_empty() => {
                if matches!(*args[0].ungrouped_static(), Node::ReferenceKind { .. }) {
                    return true;
                }
                pending.push(&args[0]);
            }
            Node::FunctionKind {
                kind: Function::Index,
                args,
            } if !args.is_empty() => pending.push(&args[0]),
            _ => {}
        }
    }
    false
}

/// Whether `arm` is a range arm that makes the IF itself a value to intersect: not a single-cell
/// INDEX ([`is_single_cell_index`]), for which Excel shows no `@` at all (`=IF(TRUE,INDEX(B1:B6,3))`,
/// probes/legacy-if-arm-other-references M12, T-2326). The INDEX arm still counts as a reference arm
/// ([`is_range_arm`]), so it gets no `@` of its own either.
fn is_whole_range_arm(arm: &Node) -> bool {
    is_range_arm(arm) && !is_single_cell_index(arm)
}

/// Whether `arm` is `INDEX(range,row)` or `INDEX(range,row,column)` over a range of any shape whose
/// indexes are all literal whole numbers of 1 or more, possibly in parentheses: Excel shows no `@`
/// on a legacy IF over `INDEX(B1:C6,3,1)`, `INDEX(B1:F1,3)`, `INDEX(B1:B6,3,1)` or even the 2-D
/// `INDEX(B1:C6,3)` (`#REF!`), but does over `INDEX(B1:B6,3.5)`, `INDEX(B1:B6,"3")`,
/// `INDEX(B1:B6,1+2)`, `INDEX(B1:B6,A20)` with A20 = 3, `INDEX(B1:B6,0)` and `INDEX(B1:C6,3,0)`
/// (probes/t3134-legacy-if-index-arm-shapes I31-I49, excel-graph, T-3134).
fn is_single_cell_index(arm: &Node) -> bool {
    matches!(
        arm.ungrouped(),
        Node::FunctionKind { kind: Function::Index, args }
            if (args.len() == 2 || args.len() == 3)
                && matches!(&*args[0].ungrouped_static(), Node::RangeKind { .. })
                && args[1..].iter().all(|index| {
                    matches!(index.ungrouped(), Node::NumberKind(n) if *n >= 1.0 && n.fract() == 0.0)
                })
    )
}

/// Whether `kind(args)` is an IF with an arm that is a range ([`is_range_arm`]), which a legacy
/// formula intersects as a whole wherever it reads the IF as a value, also under a range
/// argument: `=SUM(IF(TRUE,B1:B6)+0)` shows as `=SUM(@IF(TRUE,B1:B6)+0)`, 2 on the range's rows
/// and `#VALUE!` off them, and so do `-`, a comparison, `&` and `=MATCH(3,IF(TRUE,B1:B6)*1,0)`
/// (probes/legacy-if-arm-range-argument-b, excel-graph, T-1935).
pub(super) fn returns_range_arm(kind: &Function, args: &[Node]) -> bool {
    match kind {
        Function::If => args.len() > 1 && args[1..].iter().any(is_whole_range_arm),
        Function::Ifs => args.iter().skip(1).step_by(2).any(is_range_arm),
        _ => false,
    }
}

/// Whether `kind(args)` is an IF or CHOOSE that is one value whatever its condition or index, so a
/// legacy formula shows no `@` on it or on a call over it: every value arm is a single-cell INDEX
/// ([`is_single_cell_index`]), a literal, or an IF or CHOOSE of the same kind, and every condition
/// and CHOOSE index is a scalar. Excel shows `=IF(TRUE,INDEX(B1:B6,3),5)`,
/// `=IF(FALSE,INDEX(B1:B6,3),INDEX(C1:C6,2))`, `=IF(TRUE,IF(TRUE,INDEX(B1:B6,3)))` and
/// `=CHOOSE(1,INDEX(B1:C6,3))` with no `@`, also under ABS, but `=@IF(TRUE,INDEX(B1:B6,3),B1:B6)`
/// and an `@` on every IFS, `=@IFS(TRUE,INDEX(B1:B6,3))` (probes/t3134-legacy-if-index-arm-shapes
/// I30-I49, K30-K49, excel-graph, T-3134). An explicit stack keeps the walk off the call stack: a
/// formula's tree is untrusted input.
pub(super) fn returns_single_cell_index(kind: &Function, args: &[Node]) -> bool {
    if !matches!(kind, Function::If | Function::Choose) || args.len() < 2 {
        return false;
    }
    let mut conditions = vec![&args[0]];
    let mut pending: Vec<&Node> = args[1..].iter().collect();
    while let Some(arm) = pending.pop() {
        match arm.ungrouped() {
            Node::BooleanKind(_) | Node::NumberKind(_) | Node::StringKind(_) => {}
            Node::FunctionKind {
                kind: Function::If | Function::Choose,
                args,
            } if args.len() > 1 => {
                conditions.push(&args[0]);
                pending.extend(&args[1..]);
            }
            other if is_single_cell_index(other) => {}
            _ => return false,
        }
    }
    conditions
        .iter()
        .all(|condition| matches!(run_static_analysis_on_node(condition), StaticResult::Scalar))
}
