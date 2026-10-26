//! Which LAMBDA parameters a body reads exactly once per evaluation, so a call can move the bound
//! value into the one read instead of copying it ([`read_once`]).

use crate::{expressions::parser::Node, functions::Function};

/// One read count of [`read_once`]: more than one read, or reads it cannot count.
const MANY: u8 = 2;

/// Whether the function `kind` evaluates its argument `index` exactly once per evaluation of the
/// call (each audited): VSTACK and HSTACK ([`crate::model::Model::eval_to_array`]), LET (each value bound once,
/// `bind_argument`, then the body), IF (the condition, then one branch, or with an array condition
/// both: [`crate::variable_stack::VariableStack::copy_read_once`]), and the array of SORT, TAKE and DROP.
fn evaluates_once(kind: &Function, index: usize) -> bool {
    match kind {
        Function::Vstack | Function::Hstack | Function::Let | Function::If => true,
        Function::Sort | Function::Take | Function::Drop => index == 0,
        _ => false,
    }
}

/// For each of `ids`, whether `body` reads that variable exactly once per evaluation. A read
/// counts once when it is the body itself or reached from it only through groups and arguments
/// that [`evaluates_once`]; anywhere else a function might evaluate its argument twice, or a nested
/// LAMBDA might read it on each call, so the read counts as [`MANY`]. Reads add up, except that the
/// branches of an IF are exclusive, so they count as the most any one of them reads:
/// `IF(x>0,VSTACK(a,x),a)` reads `a` once. One walk serves every parameter, and it is not
/// recursive: nodes are listed parents first, then counted children first.
pub(super) fn read_once(body: &Node, ids: &[usize]) -> Vec<bool> {
    /// A node, the index of its parent, whether it is a branch of an IF parent, and whether it is
    /// evaluated once per evaluation of the body.
    struct Visit<'a> {
        node: &'a Node,
        parent: usize,
        branch: bool,
        once: bool,
    }
    let position = |read: u32| ids.iter().position(|&id| id == read as usize);
    let mut visits = vec![Visit {
        node: body,
        parent: 0,
        branch: false,
        once: true,
    }];
    let mut children = Vec::new();
    let mut next = 0;
    while let Some(visit) = visits.get(next) {
        let (node, once) = (visit.node, visit.once);
        node.push_children(&mut children);
        for (index, child) in children.drain(..).enumerate() {
            let (passes_once, branch) = match node {
                Node::Group { .. } | Node::UnaryPlus { .. } => (true, false),
                Node::FunctionKind { kind, .. } => (
                    evaluates_once(kind, index),
                    *kind == Function::If && index > 0,
                ),
                _ => (false, false),
            };
            visits.push(Visit {
                node: child,
                parent: next,
                branch,
                once: once && passes_once,
            });
        }
        next += 1;
    }
    // Per node and parameter: the reads of its own subtree outside IF branches, and the most
    // reads of any one of its IF branches.
    let width = ids.len();
    let mut sums = vec![0u8; visits.len() * width];
    let mut branches = vec![0u8; visits.len() * width];
    for (i, visit) in visits.iter().enumerate().rev() {
        let read = match visit.node {
            Node::NamedVariableKind { id: Some(read), .. } => {
                position(*read).map(|p| (p, if visit.once { 1 } else { MANY }))
            }
            Node::NamedFunctionKind { id: Some(read), .. } => position(*read).map(|p| (p, MANY)),
            _ => None,
        };
        if let Some((p, count)) = read {
            sums[i * width + p] = sums[i * width + p].saturating_add(count).min(MANY);
        }
        if i == 0 {
            break;
        }
        for p in 0..width {
            let total = sums[i * width + p]
                .saturating_add(branches[i * width + p])
                .min(MANY);
            let parent = visit.parent * width + p;
            if visit.branch {
                branches[parent] = branches[parent].max(total);
            } else {
                sums[parent] = sums[parent].saturating_add(total).min(MANY);
            }
        }
    }
    (0..width)
        .map(|p| sums[p].saturating_add(branches[p]) == 1)
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests")]

    use std::collections::HashMap;

    use super::read_once;
    use crate::expressions::{
        parser::{new_parser_english, Node},
        types::CellReferenceRC,
    };
    use crate::functions::logical::r#let::assign_variable_ids;

    /// `read_once` for the parameters `a` (id 1) and `x` (id 2) of `LAMBDA(a,x,body)`.
    fn read_once_of(body: &str) -> Vec<bool> {
        let mut parser = new_parser_english(vec!["Sheet1".to_string()], vec![], HashMap::new());
        let context = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };
        let Node::LambdaDefKind { mut body, .. } =
            parser.parse(&format!("LAMBDA(a,x,{body})"), &context)
        else {
            panic!("not a LAMBDA: {body}");
        };
        assign_variable_ids(&mut body, "a", 1);
        assign_variable_ids(&mut body, "x", 2);
        read_once(&body, &[1, 2])
    }

    #[test]
    fn reads_through_once_positions_count_once() {
        // T-1588's positions: the body itself and VSTACK/HSTACK arguments, through groups.
        assert_eq!(read_once_of("a"), [true, false]);
        assert_eq!(read_once_of("(VSTACK(HSTACK((a),1),x))"), [true, true]);
        // T-1593: the branches of IF are exclusive, and its condition is read once.
        assert_eq!(read_once_of("IF(x>0,VSTACK(a,x),a)"), [true, false]);
        assert_eq!(read_once_of("IF(a,1,IF(x,2,3))"), [true, true]);
        // LET's values and body.
        assert_eq!(read_once_of("LET(y,x*2,VSTACK(a,y))"), [true, false]);
        assert_eq!(read_once_of("LET(b,a,c,x,VSTACK(b,c))"), [true, true]);
        // The array of SORT, TAKE and DROP.
        assert_eq!(read_once_of("SORT(VSTACK(a,x))"), [true, true]);
        assert_eq!(read_once_of("TAKE(a,1)"), [true, false]);
        assert_eq!(read_once_of("DROP(a,x)"), [true, false]);
    }

    #[test]
    fn reads_more_than_once_or_elsewhere_do_not_count_once() {
        // An IF condition and a branch both read; a branch and a read outside the IF.
        assert_eq!(read_once_of("IF(SUM(a)>0,a,0)"), [false, false]);
        assert_eq!(read_once_of("IF(a,a,0)"), [false, false]);
        assert_eq!(read_once_of("VSTACK(IF(x,a,0),a)"), [false, true]);
        // One branch reading twice.
        assert_eq!(read_once_of("IF(x,VSTACK(a,a),a)"), [false, true]);
        // LET reading in a value and the body.
        assert_eq!(read_once_of("LET(b,a,VSTACK(a,b))"), [false, false]);
        // SORT, TAKE and DROP read their other arguments in ways not audited.
        assert_eq!(read_once_of("SORT(a,COLUMNS(a))"), [false, false]);
        assert_eq!(read_once_of("TAKE(x,a)"), [false, true]);
        assert_eq!(read_once_of("DROP(a,ROWS(a)-1)"), [false, false]);
        // A function outside the audited ones, and a nested LAMBDA.
        assert_eq!(read_once_of("SUM(a)"), [false, false]);
        assert_eq!(read_once_of("IF(x,MAP(x,LAMBDA(y,a)),0)"), [false, false]);
    }
}
