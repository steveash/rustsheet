use crate::expressions::parser::reference_operators::is_reference_operand;
use crate::functions::logical::same_variable;
use crate::functions::Function;

use super::reference_parameter::is_choose_with_array_index;
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

/// SUBTOTAL spills when an argument after the function number evaluates to an array (each
/// element is `#VALUE!` or its own error, `Model::subtotal_reference_args`, T-1163). A written
/// reference is read, not lifted; anything else counts at its shape, a call even when that is
/// a range, since CHOOSE gives a range's shape to its array arms (`CHOOSE(1,{1,2})`). Not
/// `is_reference_operand`: that one accepts calls, which can evaluate to arrays.
pub(super) fn subtotal_static_result(args: &[Node]) -> StaticResult {
    broadcast_static_result(args.iter().skip(1).filter(|arg| !is_read_reference(arg)))
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
                picked.is_some_and(|first| yields_cells(first, &mut Vec::new()))
            }
        }
        _ => false,
    }
}

/// Whether `node`, the first argument of TAKE, DROP or TRIMRANGE, evaluates to cells rather than
/// a computed array. IF and LET pass on what their branch or body is, so `TAKE(IF(TRUE,A1:A4),2)`
/// and `TAKE(LET(x,A1:A4,x),2)` pick cells while `TAKE(IF(TRUE,SEQUENCE(4)),2)` and
/// `TAKE(LET(x,SEQUENCE(4),x),2)` pick values and spill like any array
/// (probes/subtotal-reference-operands-e I2:J2, O2:P2, R2:U2, T-1365). `bound` holds the LET
/// variables in scope with their answer, computed once when bound so a chain of variables costs
/// linear time; a name not in it (a LAMBDA parameter) counts as cells, as before.
pub(super) fn yields_cells<'a>(node: &'a Node, bound: &mut Vec<(&'a str, bool)>) -> bool {
    match node {
        Node::FunctionKind {
            kind: Function::If,
            args,
        } => args
            .iter()
            .skip(1)
            .all(|branch| yields_cells(branch, bound)),
        // CHOOSE, SWITCH and IFS pass on the arm they select just as IF does, so a computed array
        // in any arm makes the call spill (`SUBTOTAL(9,TAKE(CHOOSE(1,SEQUENCE(4)),2))` is two
        // `#VALUE!`, probes/subtotal-reference-operands-f, T-1378). CHOOSE with an array index
        // lifts to an array of values even over range arms (probes/lift-arrays-choose-arms A21,
        // T-1874).
        Node::FunctionKind {
            kind: Function::Choose,
            args,
        } => {
            !is_choose_with_array_index(node)
                && args.iter().skip(1).all(|arm| yields_cells(arm, bound))
        }
        Node::FunctionKind {
            kind: Function::Switch,
            args,
        } => args
            .iter()
            .skip(1)
            .enumerate()
            // After the value, results sit at the odd positions of the (case, result) pairs; a
            // trailing unpaired argument is the default.
            .filter(|(index, _)| index % 2 == 1 || *index == args.len().saturating_sub(2))
            .all(|(_, arm)| yields_cells(arm, bound)),
        Node::FunctionKind {
            kind: Function::Ifs,
            args,
        } => args
            .iter()
            .skip(1)
            .step_by(2)
            .all(|arm| yields_cells(arm, bound)),
        Node::FunctionKind {
            kind: Function::Let,
            args,
        } if args.len() >= 3 && args.len() % 2 == 1 => {
            let scope = bound.len();
            for pair in args[..args.len() - 1].chunks_exact(2) {
                if let Node::NamedVariableKind { name, .. } = &pair[0] {
                    // A variable sees only the variables bound before it.
                    let cells = yields_cells(&pair[1], bound);
                    bound.push((name, cells));
                }
            }
            let result = args.last().is_some_and(|body| yields_cells(body, bound));
            bound.truncate(scope);
            result
        }
        Node::NamedVariableKind { name, .. } => bound
            .iter()
            .rfind(|(bound_name, _)| same_variable(bound_name, name))
            .is_none_or(|(_, cells)| *cells),
        _ => is_reference_operand(node),
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
