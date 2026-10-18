//! Whether an argument evaluates to cells rather than a computed array, shared by the SUBTOTAL
//! and AREAS reference reading (`aggregate`) and the `LiftArrays` arguments (`lifted`).

use crate::expressions::parser::reference_operators::is_reference_operand;
use crate::functions::logical::same_variable;
use crate::functions::Function;

use super::Node;

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
