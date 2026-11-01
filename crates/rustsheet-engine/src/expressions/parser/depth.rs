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
use crate::expressions::token::{self, TokenType};

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
/// into several runs (`(1+((...)))`) count together (unprobed), and so do levels of `-(`, which
/// Excel also nests to 256 and refuses at 257 (probes/t3266-nested-paren-*, T-3338). A run, signs
/// between its `(`s included, is parsed in a loop ([`Parser::parse_parenthesis_run`]), so this
/// bound is Excel's, not the stack's.
pub const MAX_PARENTHESES: usize = 256;

/// Most unary signs (`-`, `+`) open at an operand plus the operand's cost: every sign on its path
/// not yet applied, across nested runs (`=-(-(...))`), counts. A number literal written directly
/// after the signs costs 0; a reference, `TRUE`, a string or a number after a space costs 1; a
/// parenthesis (a run of them once) or a function call costs 2 ([`SIGNED_GROUP_COST`]). So Excel
/// loads 1,024 minus or plus signs before `1`, 1,023 before `C1`, `TRUE`, `"a"` or `- 1`, and
/// 1,022 before `(1)`, `((1))` or `ABS(1)`, and refuses one more of each, stored and typed;
/// 200 levels of `-(` around 824 minus signs and `1` load and 825 are refused
/// (probes/t3172-minus-*, -plus-*, -alt-*, probes/t3266-*; T-3172, T-3266, T-3338). A sign adds
/// no level of [`MAX_FORMULA_DEPTH`]: the evaluator walks a run in a loop
/// (`Model::apply_unary_chain`), so this bound is Excel's; it also keeps the walks that recurse
/// per sign (analysis, dropping) on the stack.
pub const MAX_SIGNS: usize = 1024;

/// Message of the parse error for a formula past [`MAX_SIGNS`]. Not a resource limit
/// ([`is_limit_error`]) but Excel's syntax: typed, Excel refuses the formula as it refuses one
/// that does not parse (Graph PATCH 400 `InvalidArgument`, the cell left blank;
/// probes/t3266-minus-paren-1023-typed and the like), and so does the API
/// (`TYPED_FORMULA_DOES_NOT_PARSE`, T-3338).
pub const TOO_MANY_SIGNS_MESSAGE: &str = "Formula has too many unary signs";

/// What a parenthesis or a function call after a run of signs costs toward [`MAX_SIGNS`].
pub(super) const SIGNED_GROUP_COST: usize = 2;

/// The run and the unary signs before an operand ([`Parser::parse_signs`]).
#[derive(Clone, Default)]
pub(super) struct Prefix {
    /// The run written ahead of the first sign.
    pub(super) before: Run,
    /// The signs kept in the tree, in written order: whether each is a minus, and the run
    /// after it.
    pub(super) signs: Vec<(bool, Run)>,
    /// Every sign written, a folded plus included: what counts toward [`MAX_SIGNS`].
    pub(super) count: usize,
    /// What the operand after the signs costs toward [`MAX_SIGNS`], as far as its first token
    /// tells it.
    pub(super) cost: usize,
}

/// Levels an operand adds over its subexpressions, at most: a call wrapped in
/// `@` or `#` on import (`_xlfn.SINGLE`), or `LAMBDA(...)(...)`.
pub(super) const PRIMARY_DEPTH: usize = 2;

/// [`Parser::parse_primary`]'s bound on its own recursion counts half levels, so that an operand
/// can cost one and a half: the most half levels open at once.
pub(super) const MAX_NESTING: usize = 2 * MAX_FORMULA_DEPTH;

/// Half levels of nesting an operand reserves before descending: [`PRIMARY_DEPTH`].
pub(super) const OPERAND_NESTING: usize = 2 * PRIMARY_DEPTH;

/// Half levels of nesting the first operand of a group's content reserves, unless it is a group
/// itself (T-3197).
const GROUP_HEAD_NESTING: usize = 3;

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
/// called LAMBDA alike, and for `SINGLE(` and `ANCHORARRAY(` spelled as calls (as a file stores
/// `@` and `#`); parentheses and operators, the `@` and `#` operators included, add no level
/// (docs/excel-semantics/syntax.md "Formula nesting depth cap", probes/t3097-nesting-*,
/// probes/t3143-*, T-3097, T-3143). Unlike
/// [`MAX_FORMULA_DEPTH`] this is not a parse error: the API refuses a typed formula past it
/// ([`Parser::typed_refusal`]) and the xlsx importer refuses the workbook, for a cell formula or a
/// defined-name body.
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
    /// the signs read ahead with it, if any ([`Parser::parse_parenthesis_run`]).
    pub(super) fn parse_prefix(&mut self) -> Prefix {
        if self.pending_operand.is_some() {
            return self.pending_prefix.take().unwrap_or_default();
        }
        let before = self.run_ahead();
        self.parse_signs(before)
    }

    /// The unary signs before an operand, in written order, after the run `before` them: whether
    /// each is a minus, and the run after it. Every sign is kept in its written order (`--x`
    /// coerces `x` to a number, `+-5` prints back as written), but a plus written directly before
    /// a number literal folds into it: `=+5` is `=5`, `=-+5` is `=-5`, `=+ 5` stays (syntax.md,
    /// T-2723). The run before a folded plus stays where it was: `=  +5` prints `=  5` (T-2776).
    /// Every sign, folded or not, counts toward [`MAX_SIGNS`], and so does the operand's cost as
    /// far as the next token tells it: 0 for a number literal written directly after the last
    /// sign, [`SIGNED_GROUP_COST`] for a parenthesis, 1 otherwise ([`Parser::parse_power`] checks
    /// a call once it is read).
    #[inline(never)]
    pub(super) fn parse_signs(&mut self, before: Run) -> Prefix {
        let mut signs = Vec::new();
        let mut count = 0;
        let mut before_number = false;
        while let TokenType::Addition(op) = self.lexer.peek() {
            let is_minus = *op == token::OpSum::Minus;
            self.lexer.advance_token();
            count += 1;
            before_number = !self.lexer.whitespace_follows()
                && matches!(self.lexer.peek(), TokenType::Number(_));
            if is_minus || !before_number {
                signs.push((is_minus, self.run_ahead()));
            }
        }
        let cost = if before_number {
            0
        } else if *self.lexer.peek() == TokenType::LeftParenthesis {
            SIGNED_GROUP_COST
        } else {
            1
        };
        Prefix {
            before,
            signs,
            count,
            cost,
        }
    }

    /// The [`TOO_MANY_SIGNS_MESSAGE`] error if `count` more signs open on the current path,
    /// before an operand costing `cost`, pass [`MAX_SIGNS`] (no error when there are none).
    pub(super) fn check_signs(&self, count: usize, cost: usize) -> Result<(), Node> {
        if count > 0 && self.open_signs + count + cost > MAX_SIGNS {
            return Err(self.limit_error(TOO_MANY_SIGNS_MESSAGE));
        }
        Ok(())
    }

    /// The operand of [`Parser::parse_power`] after its `prefix` of signs, which stay open while
    /// it is parsed. The signs and the operand's cost must fit [`MAX_SIGNS`]: a parenthesis is
    /// known from its first token ([`Parser::parse_signs`]), a call once it is read
    /// ([`is_call_operand`]).
    pub(super) fn parse_signed_operand(&mut self, prefix: &Prefix) -> Node {
        if let Err(error) = self.check_signs(prefix.count, prefix.cost) {
            return error;
        }
        self.open_signs += prefix.count;
        let t = self.parse_bare_union();
        self.open_signs -= prefix.count;
        if prefix.cost < SIGNED_GROUP_COST && is_call_operand(&t) {
            if let Err(error) = self.check_signs(prefix.count, SIGNED_GROUP_COST) {
                return error;
            }
        }
        t
    }

    /// Parses an operand. Its subexpressions (parenthesised, function
    /// arguments, array items, LAMBDA bodies) recurse into `parse_expr`, so
    /// this is where the parser's recursion is bounded: every open operand
    /// reserves its share of [`MAX_FORMULA_DEPTH`] before descending.
    ///
    /// The first operand of a group's content reserves one and a half levels of nesting, not
    /// [`PRIMARY_DEPTH`], unless it is a group itself, and there a plain function call adds one
    /// level to the tree: so a level of `(ABS(` costs three and a half on the way down and three
    /// on the way up, not four, and Excel's 65 nested calls each wrapped in parentheses
    /// (probes/t3097-nesting-paren-abs-65) fit, with 72 levels the most (T-3197). Groups directly
    /// inside groups (`( (`; an unbroken run is one operand, [`Parser::parse_parenthesis_run`])
    /// still cost two levels each. The at-limit tests measure the stack this takes.
    pub(super) fn parse_primary(&mut self) -> Node {
        let group_head = std::mem::take(&mut self.group_head);
        if let Some((node, depth)) = self.pending_operand.take() {
            self.depth = depth;
            return node;
        }
        let cost = if group_head && *self.lexer.peek() != TokenType::LeftParenthesis {
            GROUP_HEAD_NESTING
        } else {
            OPERAND_NESTING
        };
        if self.nesting + cost > MAX_NESTING {
            return self.too_deep();
        }
        let outer_depth = std::mem::replace(&mut self.inner_depth, 0);
        self.nesting += cost;
        let node = self.parse_operand();
        self.nesting -= cost;
        let inner_depth = std::mem::replace(&mut self.inner_depth, outer_depth);
        if let Node::ParseErrorKind { .. } = node {
            return node;
        }
        // A call is one node over its arguments; anything else may be two (see PRIMARY_DEPTH).
        let levels = if cost == GROUP_HEAD_NESTING && matches!(node, Node::FunctionKind { .. }) {
            1
        } else {
            PRIMARY_DEPTH
        };
        self.depth = inner_depth + levels;
        if self.depth > MAX_FORMULA_DEPTH {
            return self.too_deep();
        }
        node
    }
}

/// Whether the leftmost operand of `node`, a [`Parser::parse_bare_union`] result, is a function
/// call, which costs [`SIGNED_GROUP_COST`] after a run of signs as a parenthesis does
/// (probes/t3266-minus-abs-1022, -1023). Unprobed for a LAMBDA, a name or a group called.
fn is_call_operand(node: &Node) -> bool {
    let mut node = node;
    loop {
        match node {
            Node::OpRangeKind { left, .. }
            | Node::OpIntersectKind { left, .. }
            | Node::OpUnionKind { left, .. } => node = left,
            Node::Spaced { child, .. } => node = child,
            Node::FunctionKind { .. }
            | Node::LambdaDefKind { .. }
            | Node::LambdaCallKind { .. }
            | Node::NamedFunctionKind { .. } => return true,
            _ => return false,
        }
    }
}
