//! Whitespace typed in a formula. Excel keeps most of it and prints it back in FORMULATEXT
//! (`=1 + 2`, `=SUM( 1 )`, `=- 5`; docs/excel-semantics/syntax.md "Formula text keeps typed
//! whitespace…"), so the parser keeps the runs Excel keeps as [`Node::Spaced`] wrappers around
//! the operand they touch, and drops the rest (SPEC-0023).
//!
//! A run is kept before an operand (after `=`, a binary operator, a unary sign, `(`, a comma or
//! `@`) and after one when a binary operator, `%` or `)` follows. A run before a comma, at the end
//! of the formula, around `:` or inside an array constant is dropped, as Excel drops it.

use super::super::token::TokenType;
use super::{Node, Parser};

/// One whitespace run between two tokens, in Excel's stored shape: a count of spaces (SPEC-0023).
/// The line-feed count is T-2782's.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(crate) struct Run {
    spaces: u8,
}

impl Run {
    /// The run Excel keeps for the whitespace `text`: its spaces, the count wrapping at 256 as
    /// Excel's one-byte count does (`=1+` + 300 spaces + `2` prints 44 spaces,
    /// probes/t2776-formula-text-whitespace-shapes). Whitespace holding anything but spaces is not
    /// kept: line feeds are T-2782's, and the other whitespace characters print nothing today.
    fn of(text: &[char]) -> Run {
        if !text.iter().all(|c| *c == ' ') {
            return Run::default();
        }
        Run {
            spaces: u8::try_from(text.len() % 256).unwrap_or_default(),
        }
    }

    fn is_empty(self) -> bool {
        self.spaces == 0
    }

    /// The two runs typed one after the other, as one run.
    fn then(self, next: Run) -> Run {
        Run {
            spaces: self.spaces.wrapping_add(next.spaces),
        }
    }

    fn push_to(self, text: &mut String) {
        text.extend(std::iter::repeat_n(' ', usize::from(self.spaces)));
    }
}

/// The runs written around an operand: `before` ahead of its first token, `after` behind its last.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Spacing {
    before: Run,
    after: Run,
}

impl Spacing {
    fn is_empty(self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }

    /// `text`, the printed operand, with the runs around it.
    pub(crate) fn around(self, text: &str) -> String {
        let mut spaced = String::with_capacity(text.len() + 4);
        self.before.push_to(&mut spaced);
        spaced.push_str(text);
        self.after.push_to(&mut spaced);
        spaced
    }
}

impl Node {
    /// `self` with `before` and `after` written around it: itself when both are empty, so a
    /// formula with no whitespace holds no [`Node::Spaced`]. Runs around an operand that already
    /// has some join them, so a `Spaced` never wraps another.
    pub(super) fn spaced(self, before: Run, after: Run) -> Node {
        let (space, child) = self.unspaced();
        let space = Spacing {
            before: before.then(space.before),
            after: space.after.then(after),
        };
        if space.is_empty() {
            return child;
        }
        Node::Spaced {
            space,
            child: Box::new(child),
        }
    }

    /// The runs written around `self` and the operand under them (no runs and `self` when it is
    /// not [`Node::Spaced`]). For the parser sites that read an operand's own shape, a LET name
    /// or a LAMBDA parameter, and put the runs back with [`Node::spaced`] where they print.
    pub(super) fn unspaced(self) -> (Spacing, Node) {
        match self {
            Node::Spaced { space, child } => (space, *child),
            node => (Spacing::default(), node),
        }
    }

    /// [`Node::spaced`] with the runs of `space`.
    pub(super) fn respaced(self, space: Spacing) -> Node {
        self.spaced(space.before, space.after)
    }
}

impl Parser {
    /// The whitespace run between the last token read and the next one.
    pub(super) fn run_ahead(&self) -> Run {
        Run::of(self.lexer.separator())
    }

    /// The run after an operand, kept when a binary operator or `)` follows it (`=1 + 2`,
    /// `=SUM(B1:B2 )`); before a comma or at the end of the formula Excel drops it.
    pub(super) fn run_after_operand(&mut self) -> Run {
        match self.lexer.peek_token() {
            TokenType::Compare(_)
            | TokenType::And
            | TokenType::Addition(_)
            | TokenType::Product(_)
            | TokenType::Power
            | TokenType::RightParenthesis => self.run_ahead(),
            _ => Run::default(),
        }
    }

    /// `node` with `before` and `after` around it, counted as one more level of depth when it
    /// gains a [`Node::Spaced`], so whitespace never lets a tree outgrow
    /// [`super::MAX_FORMULA_DEPTH`] (CLAUDE.md directive 5).
    pub(super) fn spaced_level(
        &mut self,
        node: Node,
        before: Run,
        after: Run,
    ) -> Result<Node, Node> {
        let was_spaced = matches!(node, Node::Spaced { .. });
        let node = node.spaced(before, after);
        if !was_spaced && matches!(node, Node::Spaced { .. }) {
            self.grow(self.depth, 0)?;
        }
        Ok(node)
    }
}
