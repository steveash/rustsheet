//! The bound on formula depth (T-0931). Every consumer of a parsed formula
//! (evaluation, printing, static analysis, dropping the tree) recurses over
//! it, so the parser refuses trees deeper than [`MAX_FORMULA_DEPTH`]: it
//! checks on the way down, before each operand, so its own recursion stays
//! bounded, and on the way up, as each node is built over its children.
//! Operator chains (`=A1+A2+...`) are walked in a loop, not recursively
//! (T-0128, `chain.rs`), so their links add no depth; their length is bounded
//! by [`MAX_VALUE_OPERATORS`] instead.

use crate::expressions::lexer::STRING_LITERAL_TOO_LONG_MESSAGE;
use crate::expressions::parser::{value_operands, ExpectedTokens, Node, Parser, Run};

/// Deepest formula tree the parser builds; deeper formulas are a parse error
/// ([`TOO_DEEP_MESSAGE`]), which keeps untrusted formulas from overflowing
/// the stack (CLAUDE.md directive 5).
///
/// An operand costs [`PRIMARY_DEPTH`] levels and an operator one, so up to 127
/// nested parentheses or calls fit (Excel allows 64 nested functions); a link
/// of an operator chain adds no level (see [`Parser::grow_chain`]). See
/// docs/excel-semantics/syntax.md. Sized for optimized builds on a 1 MiB
/// stack (about 2 KB of parser stack per nesting level).
pub const MAX_FORMULA_DEPTH: usize = 256;

/// Most written groups a formula may nest (`((...(x)...))`): Excel loads 256 directly nested
/// parentheses and refuses 257, stored and typed (probes/t3172-paren-256, T-3172). Groups split
/// into several runs (`(1+((...)))`) count together (unprobed). A run is parsed in a loop
/// ([`Parser::parse_parenthesis_run`]), so this bound is Excel's, not the stack's.
pub const MAX_PARENTHESES: usize = 256;

/// Levels an operand adds over its subexpressions, at most: a call wrapped in
/// `@` or `#` on import (`_xlfn.SINGLE`), or `LAMBDA(...)(...)`.
pub(super) const PRIMARY_DEPTH: usize = 2;

/// Message of the parse error for a formula deeper than [`MAX_FORMULA_DEPTH`].
pub const TOO_DEEP_MESSAGE: &str = "Formula is nested too deeply";

/// Most value operators (`+ - * / ^ &` and comparisons) one formula may hold.
/// Chains are walked and copied in loops, but dropping the tree still recurses
/// down each chain (measured on a 1 MiB stack, optimized: about 13,000 links
/// fit), and so do the derived `PartialEq` and `Debug` that tests use, so the
/// length needs a bound. Excel's 8,192 character limit fits at most 4,096
/// operators (`=1+1+...`), so no formula Excel accepts reaches this one.
pub const MAX_VALUE_OPERATORS: usize = 4096;

/// Message of the parse error for a formula with more than
/// [`MAX_VALUE_OPERATORS`] value operators.
pub const TOO_MANY_OPERATORS_MESSAGE: &str = "Formula has too many operators";

/// Most function calls one formula may nest, the outermost included: Excel takes 65 and refuses
/// 66, stored (createSession 501) and typed (Graph PATCH 400), for built-in functions, LET and a
/// called LAMBDA alike; parentheses and operators add no level (docs/excel-semantics/syntax.md
/// "Formula nesting depth cap", probes/t3097-nesting-*, T-3097). Unlike
/// [`MAX_FORMULA_DEPTH`] this is not a parse error: the API refuses a typed formula past it
/// ([`Parser::typed_refusal`]) and the xlsx importer refuses the workbook.
pub const MAX_NESTED_CALLS: usize = 65;

/// Why a formula nesting more than [`MAX_NESTED_CALLS`] calls is refused.
pub const TOO_MANY_NESTED_CALLS_MESSAGE: &str = "Formula nests more than 65 function calls";

/// Whether a parse error `message` is a resource limit ([`TOO_DEEP_MESSAGE`] or
/// [`TOO_MANY_OPERATORS_MESSAGE`] or the lexer's string-literal limit, T-2762) rather than a syntax error: no edit of the
/// formula's ending makes it parse.
pub(crate) fn is_limit_error(message: &str) -> bool {
    message == TOO_DEEP_MESSAGE
        || message == TOO_MANY_OPERATORS_MESSAGE
        || message == STRING_LITERAL_TOO_LONG_MESSAGE
}

impl Parser {
    /// The parse error for a formula deeper than [`MAX_FORMULA_DEPTH`].
    pub(super) fn too_deep(&self) -> Node {
        self.limit_error(TOO_DEEP_MESSAGE)
    }

    fn limit_error(&self, message: &str) -> Node {
        Node::ParseErrorKind {
            formula: self.lexer.get_formula(),
            expecting: vec![ExpectedTokens::Other],
            message: message.to_string(),
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

    /// Records that a value operator node was built over `left`, of depth
    /// `left_depth`, and a right operand of depth `right_depth`. A `left` that
    /// is itself a value operator continues a chain, which consumers walk in a
    /// loop (`chain.rs`): the node is then as deep as its deepest right operand
    /// plus one, not one more than `left`.
    pub(super) fn grow_chain(
        &mut self,
        left: &Node,
        left_depth: usize,
        right_depth: usize,
    ) -> Result<(), Node> {
        self.value_operators += 1;
        if self.value_operators > MAX_VALUE_OPERATORS {
            return Err(self.limit_error(TOO_MANY_OPERATORS_MESSAGE));
        }
        if value_operands(left).is_some() {
            self.depth = left_depth.max(right_depth + 1);
            if self.depth > MAX_FORMULA_DEPTH {
                return Err(self.too_deep());
            }
            Ok(())
        } else {
            self.grow(left_depth, right_depth)
        }
    }

    /// Whether the formula last parsed nests more than [`MAX_NESTED_CALLS`] function calls.
    pub fn nests_too_many_calls(&self) -> bool {
        self.deepest_calls > MAX_NESTED_CALLS
    }

    /// Runs `parse` (the arguments of a call, or a LAMBDA's parameters and body) one function
    /// call deeper, recording the deepest nesting for [`Parser::nests_too_many_calls`].
    pub(super) fn in_call<T>(&mut self, parse: impl FnOnce(&mut Self) -> T) -> T {
        self.open_calls += 1;
        self.deepest_calls = self.deepest_calls.max(self.open_calls);
        let result = parse(self);
        self.open_calls -= 1;
        result
    }

    /// The run and the signs before an operand of [`Parser::parse_power`]. A pending operand has
    /// none of its own (what follows it is read after it, [`Parser::parse_parenthesis_run`]).
    pub(super) fn parse_prefix(&mut self) -> (Run, Vec<(bool, Run)>) {
        if self.pending_operand.is_some() {
            return (Run::default(), Vec::new());
        }
        (self.run_ahead(), self.parse_signs())
    }

    /// Parses an operand. Its subexpressions (parenthesised, function
    /// arguments, array items, LAMBDA bodies) recurse into `parse_expr`, so
    /// this is where the parser's recursion is bounded: every open operand
    /// reserves its share of [`MAX_FORMULA_DEPTH`] before descending.
    pub(super) fn parse_primary(&mut self) -> Node {
        if let Some((node, depth)) = self.pending_operand.take() {
            self.depth = depth;
            return node;
        }
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
