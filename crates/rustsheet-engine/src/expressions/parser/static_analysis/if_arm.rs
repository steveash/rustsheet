//! The arms of a legacy IF or IFS that are references, which the formula passes on whole instead of
//! intersecting (docs/excel-semantics/references.md "Legacy IF passes a range arm whole").

use super::Node;
use crate::functions::Function;

/// Whether `arg` is an arm of IF that is a range, possibly in parentheses, which a legacy formula
/// passes on whole as IF's result: the `@` goes on the IF where its result is a value
/// (`=IF(TRUE,B1:B6)` shows as `=@IF(TRUE,B1:B6)`, `=ABS(IF(TRUE,B1:B6))` as
/// `=ABS(@IF(TRUE,B1:B6))`), and none where it is a range argument (`=SUM(IF(TRUE,B1:B6))` is 12
/// and `=MATCH(3,IF(B1:B6>0,B1:B6),0)` 3 on and off the range's rows, the condition intersected;
/// probes/legacy-if-arm-range-argument, legacy-match-lookup-array-intersection J2, excel-graph,
/// T-1935). An operation over the range is still a value (`=MATCH(3,IF(TRUE,@B1:B6+0),0)`), and
/// so is IFERROR's value (`=MATCH(3,IFERROR(@B1:B6,0),0)`); an OFFSET or INDEX of a range, a `B1:INDEX(..)` range operation and
/// IFS's value arms pass whole like a range (probes/legacy-if-arm-other-references, T-1969). An explicit `@` on the range is looked through, so the export
/// keeps it (`SUM(IF(TRUE,_xlfn.SINGLE(B1:B6)))`).
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
/// an OFFSET or INDEX of one (a defined name for a range is already a range), or an IF or IFS that
/// returns one ([`returns_range_arm`]): `=MATCH(3,IF(TRUE,IF(TRUE,B1:B6)),0)` shows no `@` and
/// is 3 off the range's rows (probes/legacy-if-arm-range-argument T). An explicit stack keeps the
/// walk off the call stack: a formula's tree is untrusted input.
fn is_range_arm(arm: &Node) -> bool {
    let mut pending = vec![arm];
    while let Some(node) = pending.pop() {
        let node = match node.ungrouped() {
            Node::ImplicitIntersection { child, .. } => child.ungrouped(),
            other => other,
        };
        match node {
            Node::RangeKind { .. } | Node::OpRangeKind { .. } => return true,
            Node::FunctionKind {
                kind: Function::If,
                args,
            } if args.len() > 1 => pending.extend(&args[1..]),
            Node::FunctionKind {
                kind: Function::Ifs,
                args,
            } => pending.extend(args.iter().skip(1).step_by(2)),
            // OFFSET resizes even a one-cell base into a range.
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

/// Whether `kind(args)` is an IF with an arm that is a range ([`is_range_arm`]), which a legacy
/// formula intersects as a whole wherever it reads the IF as a value, also under a range
/// argument: `=SUM(IF(TRUE,B1:B6)+0)` shows as `=SUM(@IF(TRUE,B1:B6)+0)`, 2 on the range's rows
/// and `#VALUE!` off them, and so do `-`, a comparison, `&` and `=MATCH(3,IF(TRUE,B1:B6)*1,0)`
/// (probes/legacy-if-arm-range-argument-b, excel-graph, T-1935).
pub(super) fn returns_range_arm(kind: &Function, args: &[Node]) -> bool {
    match kind {
        Function::If => args.len() > 1 && args[1..].iter().any(is_range_arm),
        Function::Ifs => args.iter().skip(1).step_by(2).any(is_range_arm),
        _ => false,
    }
}
