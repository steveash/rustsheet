//! The wrappers printed around one operand (unary signs, written parentheses, whitespace),
//! printed in a loop so a long run of them takes one frame (T-3193).

use super::{binding, Binding};
use crate::expressions::parser::{Node, Spacing};
use crate::expressions::token::OpUnary;

/// What [`wrapped_text`] prints after the operand of a wrapper: the `)`s of written parentheses,
/// or the whitespace written behind it.
enum Closer {
    Parentheses(u16),
    Space(Spacing),
}

/// The text of `node`, a run of the wrappers printed around one operand, unary signs, written
/// parentheses ([`Node::Group`]) and whitespace ([`Node::Spaced`]), and of that operand. Printed
/// in a loop, not a frame per wrapper: a sign run holds up to [`crate::expressions::parser::MAX_SIGNS`] signs, and
/// `-(-(...))` nests as deep as `((...))` (T-3193).
pub(super) fn wrapped_text(node: &Node, sub: impl Fn(&Node) -> String) -> String {
    let mut text = String::new();
    let mut closers = Vec::new();
    let mut node = node;
    loop {
        match node {
            Node::UnaryKind {
                kind: OpUnary::Minus,
                right: child,
            }
            | Node::UnaryPlus { child } => {
                text.push(if matches!(node, Node::UnaryPlus { .. }) {
                    '+'
                } else {
                    '-'
                });
                node = child;
                if binding(child) < Binding::Minus {
                    text.push('(');
                    closers.push(Closer::Parentheses(1));
                    break;
                }
            }
            Node::Group { levels, child } => {
                text.extend(std::iter::repeat_n('(', usize::from(*levels)));
                closers.push(Closer::Parentheses(*levels));
                node = child;
            }
            Node::Spaced { space, child } => {
                space.push_before(&mut text);
                closers.push(Closer::Space(*space));
                node = child;
            }
            _ => break,
        }
    }
    text.push_str(&sub(node));
    for closer in closers.iter().rev() {
        match closer {
            Closer::Parentheses(levels) => {
                text.extend(std::iter::repeat_n(')', usize::from(*levels)));
            }
            Closer::Space(space) => space.push_after(&mut text),
        }
    }
    text
}
