//! The arms of a legacy IF or IFS that are references, which the formula passes on whole instead of
//! intersecting (docs/excel-semantics/references.md "Legacy IF passes a range arm whole").

use super::Node;
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
            Node::RangeKind { .. } | Node::OpRangeKind { .. } | Node::DefinedNameKind(_) => {
                return true
            }
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
                if matches!(args[0].ungrouped(), Node::ReferenceKind { .. }) {
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

/// Whether `arm` is a range arm that makes the IF itself a value to intersect: not `INDEX(range,row)`
/// with a literal row of 1 or more, one cell, for which Excel shows no `@` at all (`=IF(TRUE,INDEX(B1:B6,3))`,
/// probes/legacy-if-arm-other-references M12, T-2326). The INDEX arm still counts as a reference arm
/// ([`is_range_arm`]), so it gets no `@` of its own either.
fn is_whole_range_arm(arm: &Node) -> bool {
    is_range_arm(arm) && !is_single_cell_index(arm)
}

/// Whether `arm` is `INDEX(range,row)` over a one-column range with a literal row of 1 or more,
/// possibly in parentheses. Only that shape is evidenced (M12); a 2-D range gives a whole row.
fn is_single_cell_index(arm: &Node) -> bool {
    matches!(
        arm.ungrouped(),
        Node::FunctionKind { kind: Function::Index, args }
            if args.len() == 2
                && matches!(args[0].ungrouped(), Node::RangeKind { column1, column2, .. } if column1 == column2)
                && matches!(args[1].ungrouped(), Node::NumberKind(n) if *n >= 1.0)
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

/// Whether `kind(args)` is an IF whose value arms are all single-cell `INDEX` calls
/// ([`is_single_cell_index`]): one cell whatever the condition, so no `@` on the call. IFS is not
/// covered: no oracle evidence for it.
pub(super) fn returns_single_cell_index(kind: &Function, args: &[Node]) -> bool {
    match kind {
        Function::If => args.len() > 1 && args[1..].iter().all(is_single_cell_index),
        _ => false,
    }
}
