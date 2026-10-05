//! The reference operators, which bind tighter than every other operator: the intersection
//! (a space, `A1:B2 B1:B3`), the range (`:`), `@` and the spill operator `#`. Also the check
//! that a formula has no input left over after its expression.
//!
//! Excel evidence for the intersection: docs/excel-semantics/references.md "Intersection
//! operator (space)" (gen-m1, T-0105).

use crate::expressions::parser::{ExpectedTokens, Node, Parser};
use crate::expressions::token::{Error, TokenType};

impl Parser {
    /// `intersect => range (' ' range)*`, left associative.
    ///
    /// The lexer skips whitespace, so a space is the operator only where the parser sees one
    /// between a complete operand that may be a reference and a token that may start one
    /// (`A1:B2 B1:B3`, `(A1) B1`, `A1 OFFSET(...)`). Everywhere else it keeps its meaning
    /// of nothing at all: `= 1 + 2`, `SUM( A1 , B1 )`, `A1 -B1`, `{ 1 , 2 }`.
    pub(super) fn parse_intersection(&mut self) -> Node {
        let mut t = self.parse_range();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut depth = self.depth;
        while !is_constant(&t)
            && self.lexer.whitespace_follows()
            && starts_reference(&self.lexer.peek_token())
        {
            let p = self.parse_range();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            if let Err(error) = self.grow(depth, self.depth) {
                return error;
            }
            depth = self.depth;
            t = Node::OpIntersectKind {
                left: Box::new(t),
                right: Box::new(p),
            };
        }
        t
    }

    pub(super) fn parse_range(&mut self) -> Node {
        let t = self.parse_implicit();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let depth = self.depth;
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::Colon {
            self.lexer.advance_token();
            let p = self.parse_primary();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            if let Err(error) = self.grow(depth, self.depth) {
                return error;
            }
            return Node::OpRangeKind {
                left: Box::new(t),
                right: Box::new(p),
            };
        }
        t
    }

    fn parse_implicit(&mut self) -> Node {
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::At {
            self.lexer.advance_token();
            let t = self.parse_primary();
            if let Node::ParseErrorKind { .. } = t {
                return t;
            }
            if let Err(error) = self.grow(self.depth, 0) {
                return error;
            }
            return Node::ImplicitIntersection {
                automatic: false,
                child: Box::new(t),
            };
        }
        let primary = self.parse_primary();
        if let Node::ParseErrorKind { .. } = primary {
            return primary;
        }
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::Spill {
            self.lexer.advance_token();
            if let Err(error) = self.grow(self.depth, 0) {
                return error;
            }
            return Node::SpillRangeOperator {
                child: Box::new(primary),
            };
        }
        primary
    }

    /// `node` if the whole formula was consumed, otherwise a parse error: input left over
    /// after a complete expression (`=1 2`, `=SUM(A1))`) is invalid, not silently dropped.
    pub(super) fn expect_end(&mut self, node: Node) -> Node {
        if matches!(node, Node::ParseErrorKind { .. }) || self.lexer.peek_token() == TokenType::EOF
        {
            return node;
        }
        Node::ParseErrorKind {
            formula: self.lexer.get_formula(),
            expecting: vec![ExpectedTokens::Other],
            position: self.lexer.get_position() as usize,
            message: "Unexpected input after the end of the formula".to_string(),
        }
    }
}

/// A literal can never be a reference, so a space after one is not an intersection (`1 A1`
/// is left over input).
fn is_constant(node: &Node) -> bool {
    matches!(
        node,
        Node::NumberKind(_) | Node::StringKind(_) | Node::BooleanKind(_) | Node::ArrayKind(_)
    )
}

/// Whether `token` can start the right operand of an intersection: a reference, a name or
/// call, a parenthesis, or the `#REF!` Excel writes in place of a deleted reference.
fn starts_reference(token: &TokenType) -> bool {
    matches!(
        token,
        TokenType::Reference { .. }
            | TokenType::Range { .. }
            | TokenType::Ident(_)
            | TokenType::StructuredReference { .. }
            | TokenType::LeftParenthesis
            | TokenType::Error(Error::REF)
    )
}

/// `text` (the printed `node`) as an operand of the intersection operator, parenthesized
/// where printing it bare would parse back differently: an operator that binds looser
/// (`A1 (B1+C1)`), a right operand that is itself an intersection (to keep the tree), and
/// anything the parser does not take as a reference operand (`A1 (1)`, `A1 (@B1)`).
pub(crate) fn intersection_operand(text: String, node: &Node, is_left: bool) -> String {
    let bare = match node {
        Node::OpIntersectKind { .. } => is_left,
        Node::ErrorKind(error) => *error == Error::REF,
        Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::OpRangeKind { .. }
        | Node::FunctionKind { .. }
        | Node::NamedFunctionKind { .. }
        | Node::LambdaCallKind { .. }
        | Node::DefinedNameKind(_)
        | Node::TableNameKind(_)
        | Node::NamedVariableKind { .. }
        | Node::SpillRangeOperator { .. } => true,
        _ => false,
    };
    if bare {
        text
    } else {
        format!("({text})")
    }
}

/// `text` (the printed `node`) as the operand of a reference operator that binds tighter
/// than the intersection (`:`, `@`, `#`): an intersection there needs parentheses,
/// `(A1 B1):C3`.
pub(crate) fn tight_operand(text: String, node: &Node) -> String {
    if matches!(node, Node::OpIntersectKind { .. }) {
        format!("({text})")
    } else {
        text
    }
}
