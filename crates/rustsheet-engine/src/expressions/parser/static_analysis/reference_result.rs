//! The result shapes of the reference-returning OFFSET, INDIRECT, CHOOSE and MATCH, and whether an
//! argument evaluates to cells ([`yields_cells`], shared by `aggregate` and `lifted`). Moved out
//! of `static_analysis.rs` (T-1770, T-1874, T-1882) to keep it under its size lock.

use super::broadcast::broadcast_shapes;
use super::{run_static_analysis_on_node, Node, StaticResult};
use crate::expressions::parser::reference_operators::is_reference_operand;
use crate::functions::logical::same_variable;
use crate::functions::Function;

pub(super) fn static_analysis_offset(args: &[Node]) -> StaticResult {
    // If first argument is a single cell reference and there are no4th and 5th argument,
    // or they are 1, then it is a scalar
    let arg_count = args.len();
    if arg_count < 3 {
        // Actually an error
        return StaticResult::Scalar;
    }
    // An array or range in rows, cols, height or width lifts the call into an array of
    // references in their broadcast shape, which spills its value view (SPEC-0014;
    // probes/reference-lift-elements K1, K4, K13).
    let lifted = broadcast_shapes(args[1..].iter().map(run_static_analysis_on_node));
    if !matches!(lifted, StaticResult::Scalar) {
        return lifted;
    }
    if !matches!(*args[0].lowered_static(), Node::ReferenceKind { .. }) {
        return StaticResult::Unknown;
    }
    if arg_count == 3 {
        return StaticResult::Scalar;
    }
    if !matches!(args[3], Node::NumberKind(f) if f == 1.0) {
        return StaticResult::Unknown;
    }
    if arg_count == 4 {
        return StaticResult::Scalar;
    }
    if !matches!(args[4], Node::NumberKind(f) if f == 1.0) {
        return StaticResult::Unknown;
    }
    // Both height and width are explicitly 1, so OFFSET will return a single cell
    StaticResult::Scalar
}

pub(super) fn static_analysis_indirect(args: &[Node]) -> StaticResult {
    // We will always insert the @, but we don't need to do that in every scenario`
    lifted_or_unknown(args)
}

/// TRIMRANGE: an array or range in `trim_rows` or `trim_cols` lifts the call into an array of
/// references (SPEC-0014; T-1957), otherwise the shape is not known.
pub(super) fn static_analysis_trimrange(args: &[Node]) -> StaticResult {
    lifted_or_unknown(args.get(1..).unwrap_or_default())
}

/// The broadcast shape of `args` when one of them lifts the call, else `Unknown`.
fn lifted_or_unknown(args: &[Node]) -> StaticResult {
    match broadcast_shapes(args.iter().map(run_static_analysis_on_node)) {
        StaticResult::Scalar => StaticResult::Unknown,
        lifted => lifted,
    }
}

/// CHOOSE is a reference function: with a range arm the call itself is reference-shaped, so
/// legacy import wraps the whole CALL in `@` (Excel upgrades `=CHOOSE(2,A1:A10,B1:B10)` to
/// `=@CHOOSE(...)`). Its arms' counts become the extents of that range shape. An index that is an
/// array or a multi-cell range lifts CHOOSE to an array of values, even over range arms, so the
/// call is an array: a `LiftArrays` argument lifts over it rather than refusing a range
/// (`IMABS(CHOOSE({1,2},H1:H2,I1:I2))` is 2x2, probes/lift-arrays-choose-arms A21, T-1874). Each
/// argument is analysed once (`shapes`, in argument order), so nested calls stay linear.
pub(super) fn choose_static_result(shapes: impl IntoIterator<Item = StaticResult>) -> StaticResult {
    let mut shapes = shapes.into_iter();
    let index = shapes.next().unwrap_or(StaticResult::Scalar);
    let scalar_index = matches!(index, StaticResult::Scalar | StaticResult::Range(0, 0));
    match broadcast_shapes(std::iter::once(index).chain(shapes)) {
        StaticResult::Array(n, m) if scalar_index => StaticResult::Range(n - 1, m - 1),
        other => other,
    }
}

/// Whether `args` are the arguments of a well-formed LET: name/value pairs and a body.
pub(super) fn is_let_call(args: &[Node]) -> bool {
    args.len() >= 3 && args.len() % 2 == 1
}

/// What `classify` says of the body of the LET call `args` (see [`is_let_call`]), with `bound`
/// holding the variables in scope and their answer, computed once when bound: a variable sees only
/// the variables bound before it. The one LET scope walk of `yields_cells`, `lifts_to_values` and
/// `let_returns_range`.
pub(super) fn let_result<'a>(
    args: &'a [Node],
    bound: &mut Vec<(&'a str, bool)>,
    classify: fn(&'a Node, &mut Vec<(&'a str, bool)>) -> bool,
) -> bool {
    let scope = bound.len();
    for pair in args[..args.len() - 1].chunks_exact(2) {
        if let Node::NamedVariableKind { name, .. } = &pair[0] {
            let answer = classify(&pair[1], bound);
            bound.push((name, answer));
        }
    }
    let result = args.last().is_some_and(|body| classify(body, bound));
    bound.truncate(scope);
    result
}

/// Whether `node`, the first argument of TAKE, DROP or TRIMRANGE, evaluates to cells rather than
/// a computed array. IF and LET pass on what their branch or body is, so `TAKE(IF(TRUE,A1:A4),2)`
/// and `TAKE(LET(x,A1:A4,x),2)` pick cells while `TAKE(IF(TRUE,SEQUENCE(4)),2)` and
/// `TAKE(LET(x,SEQUENCE(4),x),2)` pick values and spill like any array
/// (probes/subtotal-reference-operands-e I2:J2, O2:P2, R2:U2, T-1365). `bound` holds the LET
/// variables in scope with their answer, computed once when bound so a chain of variables costs
/// linear time; a name not in it (a LAMBDA parameter) counts as cells, as before. A written group
/// is its child: `IMABS((CHOOSE(1,{1,2,3})))` lifts like the bare call (T-1874).
pub(super) fn yields_cells<'a>(node: &'a Node, bound: &mut Vec<(&'a str, bool)>) -> bool {
    let node = node.ungrouped();
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
        // `#VALUE!`, probes/subtotal-reference-operands-f, T-1378).
        Node::FunctionKind {
            kind: Function::Choose,
            args,
        } => args.iter().skip(1).all(|arm| yields_cells(arm, bound)),
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
        } if is_let_call(args) => let_result(args, bound, yields_cells),
        Node::NamedVariableKind { name, .. } => bound
            .iter()
            .rfind(|(bound_name, _)| same_variable(bound_name, name))
            .is_none_or(|(_, cells)| *cells),
        _ => is_reference_operand(node),
    }
}

/// MATCH spills by `lookup_value` and by an array `match_type`; a range `match_type` reads its
/// top-left cell and never spills (probes/match-type-nested-passthrough, T-1917).
pub(super) fn match_static_result(args: &[Node]) -> StaticResult {
    let match_type = args.get(2).map(run_static_analysis_on_node);
    let match_type = match match_type {
        Some(StaticResult::Range(..)) | None => StaticResult::Scalar,
        Some(other) => other,
    };
    broadcast_shapes([
        args.first()
            .map_or(StaticResult::Scalar, run_static_analysis_on_node),
        match_type,
    ])
}
