//! The reference operators, which bind tighter than every other operator: the union (a
//! comma inside parentheses, `(A1,B2)`), the intersection (a space, `A1:B2 B1:B3`), the range
//! (`:`), `@` and the spill operator `#`. Also the check that a formula has no input left over
//! after its expression.
//!
//! Excel evidence: docs/excel-semantics/references.md "Intersection operator (space)" (gen-m1,
//! T-0105) and "Union (comma) reference in a value context is #VALUE!" (gen-m1, T-0134).

use crate::expressions::parser::{ExpectedTokens, Node, Parser};
use crate::expressions::token::{Error, TokenType};
use crate::functions::Function;
use crate::locale::Locale;

use super::stringify::argument_separator;

impl Parser {
    /// `'(' expr ')'`, after the `(`. Directly inside it, and nowhere else, the argument
    /// separator is the union operator: `(A1,B2)` is one union, `SUM(A1,B2)` two arguments.
    pub(super) fn parse_parenthesized(&mut self) -> Node {
        self.union_allowed = true;
        let t = self.parse_expr();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: err.position,
                message: err.message,
            };
        }
        t
    }

    /// `union => intersect (',' intersect)*`, left associative, only directly inside a
    /// parenthesis. It binds looser than the intersection (`(A1,B1 C1)` is `(A1,(B1 C1))`)
    /// and tighter than every other operator, as in Excel's precedence table, so `(A1+B1,C1)`
    /// is `A1+(B1,C1)`. Operands must be references, as for the intersection: `(1,A1)` is
    /// refused. Both are UNVERIFIED (docs/excel-semantics/references.md, T-0134).
    pub(super) fn parse_union(&mut self) -> Node {
        let mut t = self.parse_intersection();
        if !self.union_allowed {
            return t;
        }
        let separator = self.get_argument_separator_token();
        let mut depth = self.depth;
        while !matches!(t, Node::ParseErrorKind { .. }) && self.lexer.peek_token() == separator {
            self.lexer.advance_token();
            let p = self.parse_intersection();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            if !is_reference_operand(&t) || !is_reference_operand(&p) {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Range],
                    position: self.lexer.get_position() as usize,
                    message: "Union needs references".to_string(),
                };
            }
            if let Err(error) = self.grow(depth, self.depth) {
                return error;
            }
            depth = self.depth;
            t = Node::OpUnionKind {
                left: Box::new(t),
                right: Box::new(p),
            };
        }
        t
    }

    /// `intersect => range (' ' range)*`, left associative.
    ///
    /// The lexer skips whitespace, so a space is the operator only where the parser sees one
    /// between a complete operand that may be a reference and a token that may start one
    /// (`A1:B2 B1:B3`, `(A1) B1`, `A1 OFFSET(...)`). Everywhere else it keeps its meaning
    /// of nothing at all: `= 1 + 2`, `SUM( A1 , B1 )`, `A1 -B1`, `{ 1 , 2 }`.
    pub(super) fn parse_intersection(&mut self) -> Node {
        let start = self.lexer.get_position_index();
        let mut t = self.parse_range();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut depth = self.depth;
        let mut as_name = false;
        while !is_constant(&t)
            && self.lexer.whitespace_follows()
            && starts_reference(&self.lexer.peek_token())
        {
            // Excel refuses the formula (load-time `FileCorruptTryRepair`, so no oracle value)
            // on CR, TAB, NBSP and the like as separator, and on a value as an operand.
            let separator = self.lexer.separator();
            // U+3000 starts a name for Excel, and no such name is defined: `#NAME?`
            // (probes/intersection-whitespace H7).
            let name_error = separator.contains(&'\u{3000}');
            let separator_ok = separator
                .iter()
                .all(|c| matches!(c, ' ' | '\n' | '\u{3000}'));
            let p = self.parse_range();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            if !separator_ok
                || !(name_error || is_reference_operand(&t))
                || !is_reference_operand(&p)
            {
                return self.intersection_error();
            }
            as_name |= name_error;
            if let Err(error) = self.grow(depth, self.depth) {
                return error;
            }
            depth = self.depth;
            t = Node::OpIntersectKind {
                left: Box::new(t),
                right: Box::new(p),
            };
        }
        if as_name {
            // A synthetic undefined name holding the source text: evaluates to `#NAME?` and
            // prints as written, so FORMULATEXT and export keep the formula. Built once, after
            // the chain, which the depth cap bounds like any intersection.
            return Node::NamedVariableKind {
                name: self.lexer.source_since(start),
                id: None,
            };
        }
        t
    }

    fn intersection_error(&self) -> Node {
        Node::ParseErrorKind {
            formula: self.lexer.get_formula(),
            expecting: vec![ExpectedTokens::Range],
            position: self.lexer.get_position() as usize,
            message: "Intersection needs references separated by spaces or line feeds".to_string(),
        }
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

/// Whether `node` is a reference expression, the only kind of operand Excel accepts for an
/// intersection (docs/excel-semantics/references.md "Intersection operands must be
/// references"): values, value operators, error literals other than `#REF!` and value
/// functions make Excel refuse the formula. The accepted calls are the reference-returning
/// functions below (probes/intersection-function-operands; T-0161); value functions such as
/// SUM, NA, ABS, ROW, ROWS, MATCH, VLOOKUP, HLOOKUP, LOOKUP, IFERROR, IFNA, CHOOSEROWS, CHOOSECOLS, FILTER,
/// SORT, UNIQUE and VSTACK are refused, even those that pass a range through; functions not
/// probed are refused by inference.
fn is_reference_operand(node: &Node) -> bool {
    match node {
        Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::BooleanKind(_)
        | Node::ArrayKind(_)
        | Node::OpConcatenateKind { .. }
        | Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. }
        | Node::UnaryKind { .. } => false,
        Node::ErrorKind(error) => *error == Error::REF,
        Node::FunctionKind { kind, .. } => matches!(
            kind,
            Function::Index
                | Function::Offset
                | Function::If
                | Function::Indirect
                | Function::Choose
                | Function::Xlookup
                | Function::Let
                | Function::Switch
                | Function::Ifs
                | Function::Take
                | Function::Drop
                | Function::Trimrange
        ),
        _ => true,
    }
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
        Node::OpUnionKind { .. }
        | Node::ReferenceKind { .. }
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

/// `(left,right)`, the printed union of `left_text` and `right_text` (the printed `left` and
/// `right`). It always carries its parentheses: without them the comma would be an argument
/// separator (`SUM((A1,B2))`) or left over input. A left operand that is itself a union is
/// printed without its own, `(A1,B2,C3)`, which parses back to the same left-associative tree.
pub(crate) fn union_text(
    left_text: &str,
    left: &Node,
    right_text: &str,
    locale: &Locale,
) -> String {
    let left_text = match left {
        Node::OpUnionKind { .. } => left_text
            .strip_prefix('(')
            .and_then(|text| text.strip_suffix(')'))
            .unwrap_or(left_text),
        _ => left_text,
    };
    format!("({left_text}{}{right_text})", argument_separator(locale))
}
