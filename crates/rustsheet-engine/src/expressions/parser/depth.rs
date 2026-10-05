//! The bound on formula depth (T-0931). Every consumer of a parsed formula
//! (evaluation, printing, static analysis, dropping the tree) recurses over
//! it, so the parser refuses trees deeper than [`MAX_FORMULA_DEPTH`]: it
//! checks on the way down, before each operand, so its own recursion stays
//! bounded, and on the way up, as each node is built over its children.

use crate::expressions::parser::{ExpectedTokens, Node, Parser};

/// Deepest formula tree the parser builds; deeper formulas are a parse error
/// ([`TOO_DEEP_MESSAGE`]), which keeps untrusted formulas from overflowing
/// the stack (CLAUDE.md directive 5).
/// An operand costs [`PRIMARY_DEPTH`] levels and an operator one, so up to 127
/// nested parentheses or calls fit (Excel allows 64 nested functions) and up
/// to 255 terms in one operator chain. Excel accepts longer chains
/// (`=A1+A2+...` up to its 8,192 character limit): a DEVIATION, see
/// docs/excel-semantics/syntax.md. Sized for optimized builds on a 1 MiB
/// stack (about 2 KB of parser stack per nesting level).
pub const MAX_FORMULA_DEPTH: usize = 256;

/// Levels an operand adds over its subexpressions, at most: a call wrapped in
/// `@` or `#` on import (`_xlfn.SINGLE`), or `LAMBDA(...)(...)`.
const PRIMARY_DEPTH: usize = 2;

/// Message of the parse error for a formula deeper than [`MAX_FORMULA_DEPTH`].
pub const TOO_DEEP_MESSAGE: &str = "Formula is nested too deeply";

impl Parser {
    /// The parse error for a formula deeper than [`MAX_FORMULA_DEPTH`].
    pub(super) fn too_deep(&self) -> Node {
        Node::ParseErrorKind {
            formula: self.lexer.get_formula(),
            expecting: vec![ExpectedTokens::Other],
            message: TOO_DEEP_MESSAGE.to_string(),
            position: self.lexer.get_position() as usize,
        }
    }

    /// Records that a node was built over children of depths `left` and
    /// `right`, or returns the too-deep error if it exceeds the limit.
    pub(super) fn grow(&mut self, left: usize, right: usize) -> Result<(), Node> {
        self.depth = left.max(right) + 1;
        if self.depth > MAX_FORMULA_DEPTH {
            return Err(self.too_deep());
        }
        Ok(())
    }

    /// Parses an operand. Its subexpressions (parenthesised, function
    /// arguments, array items, LAMBDA bodies) recurse into `parse_expr`, so
    /// this is where the parser's recursion is bounded: every open operand
    /// reserves its share of [`MAX_FORMULA_DEPTH`] before descending.
    pub(super) fn parse_primary(&mut self) -> Node {
        if self.nesting + PRIMARY_DEPTH > MAX_FORMULA_DEPTH {
            return self.too_deep();
        }
        let outer_depth = std::mem::replace(&mut self.inner_depth, 0);
        self.nesting += PRIMARY_DEPTH;
        let node = self.parse_operand();
        self.nesting -= PRIMARY_DEPTH;
        let inner_depth = std::mem::replace(&mut self.inner_depth, outer_depth);
        if let Node::ParseErrorKind { .. } = node {
            return node;
        }
        self.depth = inner_depth + PRIMARY_DEPTH;
        if self.depth > MAX_FORMULA_DEPTH {
            return self.too_deep();
        }
        node
    }
}
