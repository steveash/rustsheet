//! Closing a group (SPEC-0013 section 1): its alternatives and items become one node, one level
//! deeper than its contents.

use super::super::ast::{Ast, Condition};
use super::{Frame, GroupKind, Invalid, NEST_LIMIT};

/// The items of one alternative as a node, and its depth.
pub(super) fn concat(mut items: Vec<(Ast, u32)>) -> (Ast, u32) {
    let depth = items.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let ast = match items.len() {
        0 => Ast::Empty,
        1 => items.pop().map_or(Ast::Empty, |(ast, _)| ast),
        _ => Ast::Concat(items.into_iter().map(|(ast, _)| ast).collect()),
    };
    (ast, depth)
}

/// An assertion condition's lookaround, the first item of `yes`, and the rest of `yes`.
fn leading_assertion(yes: Ast) -> Result<(Ast, Ast), Invalid> {
    match yes {
        Ast::LookAround { .. } => Ok((yes, Ast::Empty)),
        Ast::Concat(mut asts) if matches!(asts.first(), Some(Ast::LookAround { .. })) => {
            let assertion = asts.remove(0);
            let rest = match asts.len() {
                0 => Ast::Empty,
                1 => asts.pop().ok_or(Invalid)?,
                _ => Ast::Concat(asts),
            };
            Ok((assertion, rest))
        }
        _ => Err(Invalid),
    }
}

/// The alternatives of a group, the last one's `items` included, as a node and its depth.
pub(super) fn alternation(mut alternatives: Vec<(Ast, u32)>, items: Vec<(Ast, u32)>) -> (Ast, u32) {
    let last = concat(items);
    if alternatives.is_empty() {
        return last;
    }
    alternatives.push(last);
    let depth = alternatives.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let ast = Ast::Alternation(alternatives.into_iter().map(|(ast, _)| ast).collect());
    (ast, depth)
}

/// The node a closed group becomes, one level deeper than its contents.
pub(super) fn close(frame: Frame) -> Result<(Ast, u32), Invalid> {
    let alternatives = frame.alternatives.len() + 1;
    let (ast, depth) = alternation(frame.alternatives, frame.items);
    let depth = depth + 1;
    if depth > NEST_LIMIT {
        return Err(Invalid);
    }
    let ast = Box::new(ast);
    let ast = match frame.kind {
        GroupKind::Root => return Err(Invalid),
        GroupKind::Capture { index, name } => Ast::Capture { index, name, ast },
        GroupKind::NonCapture => *ast,
        GroupKind::LookAround(kind) => Ast::LookAround { kind, ast },
        GroupKind::Atomic => Ast::Atomic(ast),
        GroupKind::Conditional(condition) => {
            if alternatives > 2 {
                return Err(Invalid);
            }
            let (yes, no) = match *ast {
                Ast::Alternation(mut both) if alternatives == 2 => {
                    let no = both.pop().ok_or(Invalid)?;
                    let yes = both.pop().ok_or(Invalid)?;
                    (yes, no)
                }
                yes => (yes, Ast::Empty),
            };
            let (condition, yes) = match condition {
                Condition::Assert(_) => {
                    let (assertion, yes) = leading_assertion(yes)?;
                    (Condition::Assert(Box::new(assertion)), yes)
                }
                group @ Condition::Group(_) => (group, yes),
            };
            Ast::Conditional {
                condition,
                yes: Box::new(yes),
                no: Box::new(no),
            }
        }
    };
    Ok((ast, depth))
}
