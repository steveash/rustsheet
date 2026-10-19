use crate::functions::logical::same_variable;
use crate::functions::Function;

use super::reference_result::yields_cells;
use super::{broadcast_static_result, run_static_analysis_on_node, Node, StaticResult};

/// AGGREGATE spills over an array `function_num`, options or fourth argument, broadcast against
/// each other (`Model::fn_aggregate`). A range `function_num` or options lifts too
/// (probes/aggregate-range-args); a range fourth argument is read, not lifted. A one-cell range
/// there counts as 1x1, like any broadcast argument: Excel shows the legacy
/// `=AGGREGATE(@CHOOSE(1,{9}),0,E1:E6)` (probes/legacy-one-cell-shape-intersection, T-1265).
pub(super) fn aggregate_static_result(args: &[Node]) -> StaticResult {
    let lifted = args
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < 2 || (*index == 3 && args.len() == 4))
        .filter(|(index, arg)| {
            *index < 2 || !matches!(run_static_analysis_on_node(arg), StaticResult::Range(..))
        })
        .map(|(_, arg)| arg);
    broadcast_static_result(lifted)
}

/// SUBTOTAL spills over an array or range `function_num` (`Model::fn_subtotal`, probes/
/// subtotal-array-function-num, T-1868), and when an argument after it evaluates to an array (each
/// element is `#VALUE!` or its own error, `Model::subtotal_reference_args`, T-1163). A written
/// reference is read, not lifted; anything else counts at its shape, a call even when that is
/// a range, since CHOOSE gives a range's shape to its array arms (`CHOOSE(1,{1,2})`). Not
/// `is_reference_operand`: that one accepts calls, which can evaluate to arrays.
pub(super) fn subtotal_static_result(args: &[Node]) -> StaticResult {
    broadcast_static_result(
        args.iter()
            .take(1)
            .chain(args.iter().skip(1).filter(|arg| !is_read_reference(arg))),
    )
}

/// AREAS spills the same way over its one argument (`AREAS(IF(TRUE,{1,2}))` is two `#VALUE!`,
/// probes/subtotal-reference-operands-b H4, T-1175).
pub(super) fn areas_static_result(args: &[Node]) -> StaticResult {
    broadcast_static_result(args.iter().filter(|arg| !is_read_reference(arg)))
}

/// An argument that SUBTOTAL and AREAS read as a reference instead of lifting over: a written
/// reference, or XLOOKUP, TAKE, DROP and TRIMRANGE, which evaluate to the cells they pick
/// (`Model::evaluate_picked_reference_arg`, probes/subtotal-reference-operands-b H12, H15:H17).
fn is_read_reference(arg: &Node) -> bool {
    // A written group is its child: `AREAS((A1,B1))` reads the union (T-1342).
    match arg.ungrouped() {
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::OpRangeKind { .. }
        | Node::OpIntersectKind { .. }
        | Node::OpUnionKind { .. }
        | Node::SheetSpanKind { .. } => true,
        // Over a literal or computed array they pick values, not cells, and spill like any array
        // (`SUBTOTAL(9,TAKE({1,2;3,4},1))`, probes/subtotal-reference-operands-c H1, T-1206;
        // `SUBTOTAL(9,TAKE(SEQUENCE(4),2))` is two `#VALUE!`, probes/subtotal-reference-operands-d
        // J4:J5, T-1211).
        Node::FunctionKind { kind, args } if kind.picks_reference() => {
            let picked = kind
                .picked_reference_arg()
                .and_then(|index| args.get(index));
            if *kind == Function::Xlookup {
                !matches!(picked, Some(Node::ArrayKind(_)))
            } else {
                // A CHOOSE with an array index lifts to values even over range arms
                // (`SUBTOTAL(9,TAKE(CHOOSE({1,2},H1:H2,I1:I2),2))` is a 2x2 of `#VALUE!`,
                // probes/lift-arrays-choose-arms A69:B70, T-1882). The picking function's own
                // shape is `Unknown` and never analyses its argument, so this is the one analysis
                // of it and nesting stays linear.
                picked.is_some_and(|first| {
                    yields_cells(first, &mut Vec::new()) && !lifts_to_values(first, &mut Vec::new())
                })
            }
        }
        _ => false,
    }
}

/// Whether `node`, whose arms all yield cells, still lifts to an array of values: an IF, CHOOSE,
/// SWITCH or IFS with an array or multi-cell range condition, index or value lifts even over range
/// arms (`SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2))`, the IFS twin and the CHOOSE one are
/// a 2x2 of `#VALUE!`, probes/subtotal-take-array-selector A30:B31, A40:B41, probes/
/// lift-arrays-choose-arms A69:B70, T-1882, T-1914), and so does one whose selected arm lifts
/// (`IF(TRUE,SWITCH({1,2},...))`). SWITCH and IFS have an `Unknown` shape of their own, so their
/// selectors are checked here. Each selector and each arm is analysed once, so nesting stays
/// linear. An `Unknown` selector counts as lifting: the cell is left to spill at run time rather
/// than taken for a scalar. LET passes on what its body is; `bound` holds the variables in scope
/// with their answer, computed once when bound, as in `yields_cells`. A variable selector (a LET
/// name or LAMBDA parameter, `LET(s,{1,2},SWITCH(s,...))`) counts as lifting too: its own shape
/// does not tell what it holds.
fn lifts_to_values<'a>(node: &'a Node, bound: &mut Vec<(&'a str, bool)>) -> bool {
    let not_scalar = |arg: &Node| {
        matches!(arg.ungrouped(), Node::NamedVariableKind { .. })
            || !matches!(
                run_static_analysis_on_node(arg),
                StaticResult::Scalar | StaticResult::Range(0, 0)
            )
    };
    let node = node.ungrouped();
    if let Node::NamedVariableKind { name, .. } = node {
        return bound
            .iter()
            .rfind(|(bound_name, _)| same_variable(bound_name, name))
            .is_some_and(|(_, lifts)| *lifts);
    }
    let Node::FunctionKind { kind, args } = node else {
        return matches!(run_static_analysis_on_node(node), StaticResult::Array(..));
    };
    match kind {
        Function::If | Function::Choose => {
            args.first().is_some_and(not_scalar)
                || args.iter().skip(1).any(|arm| lifts_to_values(arm, bound))
        }
        Function::Switch => {
            args.first().is_some_and(not_scalar)
                || args
                    .iter()
                    .skip(1)
                    .enumerate()
                    // As in `yields_cells`: results at the odd positions after the value, and a
                    // trailing unpaired default.
                    .filter(|(index, _)| index % 2 == 1 || *index == args.len().saturating_sub(2))
                    .any(|(_, arm)| lifts_to_values(arm, bound))
        }
        Function::Ifs => {
            args.iter().step_by(2).any(not_scalar)
                || args
                    .iter()
                    .skip(1)
                    .step_by(2)
                    .any(|arm| lifts_to_values(arm, bound))
        }
        Function::Let if args.len() >= 3 && args.len() % 2 == 1 => {
            let scope = bound.len();
            for pair in args[..args.len() - 1].chunks_exact(2) {
                if let Node::NamedVariableKind { name, .. } = &pair[0] {
                    let lifts = lifts_to_values(&pair[1], bound);
                    bound.push((name, lifts));
                }
            }
            let result = args.last().is_some_and(|body| lifts_to_values(body, bound));
            bound.truncate(scope);
            result
        }
        _ => matches!(run_static_analysis_on_node(node), StaticResult::Array(..)),
    }
}

/// VLOOKUP/HLOOKUP spill by `lookup_value`, the index and `range_lookup`, broadcast; the table is
/// consumed (probes/vlookup-array-index-range-lookup, T-1269).
pub(super) fn lookup_static_result(args: &[Node]) -> StaticResult {
    broadcast_static_result(
        args.iter()
            .enumerate()
            .filter(|(i, _)| *i != 1)
            .map(|(_, arg)| arg),
    )
}
