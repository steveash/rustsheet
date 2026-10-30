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

/// One whitespace run between two tokens, in Excel's stored shape: a count of line feeds, then a
/// count of the spaces after the last of them (SPEC-0023).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(crate) struct Run {
    line_feeds: u8,
    spaces: u8,
}

/// `count` as Excel's one-byte run count: it wraps at 256, so 256 prints as none
/// (probes/t2782-whitespace-run-wrap).
fn byte_count(count: usize) -> u8 {
    u8::try_from(count % 256).unwrap_or_default()
}

impl Run {
    /// The run Excel keeps for the whitespace `text`: its line feeds, then only the spaces after
    /// the last line feed, each count wrapping at 256 on its own (`=1+ ` LF ` 2` prints `=1+` LF
    /// ` 2`; 300 spaces print 44, 300 line feeds print 44, and 3 spaces before 256 line feeds
    /// print nothing; docs/excel-semantics/syntax.md, probes/t2776-formula-text-whitespace-shapes
    /// and probes/t2782-whitespace-run-wrap). Whitespace holding anything but spaces and line
    /// feeds is not kept: the other whitespace characters print nothing today.
    pub(super) fn of(text: &[char]) -> Run {
        if !text.iter().all(|c| matches!(c, ' ' | '\n')) {
            return Run::default();
        }
        let line_feeds = text.iter().filter(|c| **c == '\n').count();
        let spaces = text.iter().rev().take_while(|c| **c == ' ').count();
        Run {
            line_feeds: byte_count(line_feeds),
            spaces: byte_count(spaces),
        }
    }

    fn is_empty(self) -> bool {
        self.line_feeds == 0 && self.spaces == 0
    }

    /// The two runs typed one after the other, as one run: the line feeds add up, and the spaces
    /// of `self` survive only when `next` has no line feed to end them. A `next` of exactly 256 line
    /// feeds has wrapped to none and keeps them; Excel's join of two such runs is unprobed.
    fn then(self, next: Run) -> Run {
        let spaces = if next.line_feeds == 0 {
            self.spaces.wrapping_add(next.spaces)
        } else {
            next.spaces
        };
        Run {
            line_feeds: self.line_feeds.wrapping_add(next.line_feeds),
            spaces,
        }
    }

    fn push_to(self, text: &mut String) {
        text.extend(std::iter::repeat_n('\n', usize::from(self.line_feeds)));
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
    ///
    /// A `Spaced` costs no level of [`super::MAX_FORMULA_DEPTH`]: it only ever wraps a node that
    /// costs one, and never another `Spaced`, so the tree is at most twice as deep as counted, and
    /// only the printers and edits walk it (evaluation reads the tree with no whitespace,
    /// decision 0127). A formula nests as deep with spaces as without, as Excel's does; the
    /// nesting tests print and edit the spaced shapes at the limit on a 2 MiB stack.
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

    /// The operator printed before `self` as the right operand of an intersection: a line feed in
    /// its run is the operator itself, so no space is printed beside it
    /// (probes/t2949-typed-4).
    pub(crate) fn intersection_operator(&self) -> &'static str {
        match self {
            Node::Spaced { space, .. } if space.before.line_feeds > 0 => "",
            _ => " ",
        }
    }

    /// The node under any [`Node::Spaced`] wrapper, for the printers that decide on its shape.
    pub(crate) fn unwrap_spaced(&self) -> &Node {
        let mut node = self;
        while let Node::Spaced { child, .. } = node {
            node = child;
        }
        node
    }

    /// `self`, a call, with the run written between the parentheses of an empty call: `=NOW( )`
    /// prints back as typed (probes/t2786-whitespace-structured-refs-runs I1, I11, T-2966). The
    /// run is held as one spaced empty argument, which [`Node::strip_spaces`] removes, so
    /// evaluation and the argument counts still see a call with none ([`is_empty_call_run`]).
    pub(super) fn with_empty_call_run(mut self, run: Run) -> Node {
        if let Node::FunctionKind { args, .. } = &mut self {
            if args.is_empty() && !run.is_empty() {
                args.push(Node::EmptyArgKind.spaced(run, Run::default()));
            }
        }
        self
    }

    /// [`Node::unwrap_spaced`] for a mutable borrow, for a walker that rewrites the node it
    /// decides on (one function cannot serve both borrows).
    pub(crate) fn unwrap_spaced_mut(&mut self) -> &mut Node {
        let mut node = self;
        while let Node::Spaced { child, .. } = node {
            node = child;
        }
        node
    }

    /// [`Node::spaced`] with the runs of `space`.
    pub(super) fn respaced(self, space: Spacing) -> Node {
        self.spaced(space.before, space.after)
    }

    /// Applies `rewrite`, a rewrite that decides on the shape of operands, as it applies to the
    /// tree with no whitespace, the tree evaluation reads (decision 0127). On a tree written with
    /// whitespace it runs on the bare tree and on the written one, and the written result is kept
    /// only when it is the bare result once its whitespace is removed; otherwise the tree becomes
    /// the bare result. A shape site that does not look past a [`Node::Spaced`] then costs the
    /// formula its whitespace, never a value: `N( IF(1,{2;3}) )` read as a legacy formula takes
    /// the `@`s `N(IF(1,{2;3}))` takes (T-2948).
    pub(crate) fn rewrite_as_unspaced(&mut self, rewrite: impl Fn(&mut Node)) {
        if !self.has_spaces() {
            rewrite(self);
            return;
        }
        let mut bare = self.clone();
        bare.strip_spaces();
        rewrite(&mut bare);
        rewrite(self);
        let mut unspaced = self.clone();
        unspaced.strip_spaces();
        if unspaced != bare {
            *self = bare;
        }
    }
}

/// Whether `args` holds only the run of an empty call ([`Node::with_empty_call_run`]).
pub(super) fn is_empty_call_run(args: &[Node]) -> bool {
    matches!(args, [Node::Spaced { child, .. }] if matches!(**child, Node::EmptyArgKind))
}

impl Parser {
    /// The run between a call's `(`, just read, and its `)` when no argument comes between them
    /// (`=NOW( )`); none otherwise.
    pub(super) fn empty_call_run(&mut self) -> Run {
        if self.lexer.peek_token() == TokenType::RightParenthesis {
            self.run_ahead()
        } else {
            Run::default()
        }
    }

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
}
