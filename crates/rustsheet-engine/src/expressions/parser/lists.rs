//! Comma-separated lists inside an operand: function arguments and the rows
//! of an array constant.

use crate::expressions::parser::{ArrayNode, ExpectedTokens, Node, Parser};
use crate::expressions::token::{OpSum, TokenType};

impl Parser {
    /// An expression inside parentheses or an argument list, where the comma is not the bare
    /// union operator.
    pub(super) fn parse_enclosed_expr(&mut self) -> Node {
        let top_level = std::mem::take(&mut self.top_level);
        let t = self.parse_expr();
        self.top_level = top_level;
        t
    }

    /// One row of an array constant `{1,2;3,4}`. Items may only be numbers,
    /// strings, booleans or errors (a number may carry a minus sign);
    /// anything else is a syntax error.
    pub(super) fn parse_array_row(&mut self) -> Result<Vec<ArrayNode>, Node> {
        let column_separator_token = self.get_argument_separator_token();
        let mut row = vec![self.parse_array_item()?];
        while self.lexer.peek_token() == column_separator_token {
            self.lexer.advance_token();
            row.push(self.parse_array_item()?);
        }
        Ok(row)
    }

    fn parse_array_item(&mut self) -> Result<ArrayNode, Node> {
        // Excel: one sign, directly attached to a number literal, and nothing else
        // (docs/excel-semantics/syntax.md "Array constants take a single optional sign").
        if let TokenType::Addition(op) = self.lexer.peek_token() {
            self.lexer.advance_token();
            if self.lexer.whitespace_follows() {
                return Err(self.invalid_array_value());
            }
            return match self.lexer.next_token() {
                TokenType::Number(n) if op == OpSum::Minus => Ok(ArrayNode::Number(-n)),
                TokenType::Number(n) => Ok(ArrayNode::Number(n)),
                _ => Err(self.invalid_array_value()),
            };
        }
        match self.parse_enclosed_expr() {
            Node::BooleanKind(s) => Ok(ArrayNode::Boolean(s)),
            Node::NumberKind(s) => Ok(ArrayNode::Number(s)),
            Node::StringKind(s) => Ok(ArrayNode::String(s.into())),
            Node::ErrorKind(kind) => Ok(ArrayNode::Error(kind)),
            error @ Node::ParseErrorKind { .. } => Err(error),
            _ => Err(self.invalid_array_value()),
        }
    }

    fn invalid_array_value(&self) -> Node {
        Node::ParseErrorKind {
            formula: self.lexer.get_formula(),
            expecting: vec![ExpectedTokens::Other],
            message: "Invalid value in array".to_string(),
            position: self.lexer.get_position() as usize,
        }
    }

    /// The arguments of a call to `fn_name`, after its `(`; empty arguments
    /// (`IF(A1,,2)`) are `EmptyArgKind`.
    pub(super) fn parse_function_args(&mut self, fn_name: &str) -> Result<Vec<Node>, Node> {
        let arg_separator_token = &self.get_argument_separator_token();
        let mut args: Vec<Node> = Vec::new();
        let mut next_token = self.lexer.peek_token();
        if next_token == TokenType::RightParenthesis {
            return Ok(args);
        }
        // The cursor is currently in the first argument of `fn_name`.
        self.set_argument_hint(fn_name, 1);
        if &self.lexer.peek_token() == arg_separator_token {
            args.push(Node::EmptyArgKind);
        } else {
            let t = self.parse_enclosed_expr();
            if let Node::ParseErrorKind { .. } = t {
                return Err(t);
            }
            args.push(t);
        }
        next_token = self.lexer.peek_token();
        let mut arg_index = 1;
        while &next_token == arg_separator_token {
            self.lexer.advance_token();
            arg_index += 1;
            self.set_argument_hint(fn_name, arg_index);
            if &self.lexer.peek_token() == arg_separator_token {
                args.push(Node::EmptyArgKind);
                next_token = arg_separator_token.clone();
            } else if self.lexer.peek_token() == TokenType::RightParenthesis {
                args.push(Node::EmptyArgKind);
                return Ok(args);
            } else {
                let p = self.parse_enclosed_expr();
                if let Node::ParseErrorKind { .. } = p {
                    return Err(p);
                }
                next_token = self.lexer.peek_token();
                args.push(p);
            }
        }
        Ok(args)
    }

    /// Records that the position currently being parsed *starts* argument
    /// `index` (1-based) of `fn_name`: a fresh operand goes here, so a `Range`
    /// is a valid completion. Reported if an EOF lands on this empty slot (e.g.
    /// `SUM(` or `SUM(A1,`). Once the argument is complete the unclosed-call
    /// path in `parse_primary` reports the argument context without `Range`.
    pub(super) fn set_argument_hint(&mut self, fn_name: &str, index: u32) {
        self.expecting_here = vec![
            ExpectedTokens::Argument(fn_name.to_string(), index),
            ExpectedTokens::Range,
        ];
    }
}
