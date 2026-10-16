use crate::expressions::parser::{ExpectedTokens, NamedVariable, Node, Parser};
use crate::expressions::token::TokenType;
use crate::functions::logical::has_duplicate_parameter;

use super::reference_operators::is_reference_operand;

impl Parser {
    // Called after `LAMBDA` and the opening `(` have been consumed.
    // Parses:  (param | '[' param ']')*, body ')' ['(' call_args ')']
    // Returns LambdaDefKind, or LambdaCallKind if immediately invoked.
    pub(crate) fn parse_lambda(&mut self) -> Node {
        let arg_separator = self.get_argument_separator_token();

        if self.lexer.peek_token() == TokenType::RightParenthesis {
            self.lexer.advance_token();
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: self.lexer.get_position() as usize,
                message: "LAMBDA requires at least one argument (the body)".to_string(),
            };
        }

        let mut parameters = Vec::new();
        // The parameters are in scope up to the LAMBDA's `)`.
        let scope = self.dotted_bound_names.len();
        let def = self.parse_lambda_items(&arg_separator, &mut parameters);
        self.dotted_bound_names.truncate(scope);
        let body = match def {
            Ok(body) => body,
            Err(error) => return error,
        };

        if has_duplicate_parameter(&parameters) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: self.lexer.get_position() as usize,
                message: "LAMBDA parameter names must be distinct".to_string(),
            };
        }

        let def = Node::LambdaDefKind {
            parameters,
            body: Box::new(body),
        };

        // Immediate invocation: LAMBDA(params, body)(call_args)
        if self.lexer.peek_token() == TokenType::LeftParenthesis {
            return self.parse_call(def);
        }

        def
    }

    /// `(callee)(args)`, after the first `(`: a parenthesized operand written directly before
    /// a `(`, with no space, is called like `LAMBDA(x,x+1)(2)`. Excel evaluates
    /// `(LAMBDA(x,x+1))(2)` to 3 and `(IF(TRUE,LAMBDA(x,x*2)))(3)` to 6, and gives `#REF!` for a
    /// called reference (`(A1)(A2)`), but refuses a called value (`(1)(2)`); with a space,
    /// `(A1) (A2)` is an intersection (docs/excel-semantics/references.md "Calling a
    /// parenthesized operand", probes/paren-call-*, T-0149). So the callee must be a reference
    /// operand, the same set the intersection accepts, which also covers LAMBDA and names.
    pub(super) fn parse_parenthesized_call(&mut self) -> Node {
        let callee = self.parse_parenthesized();
        if matches!(callee, Node::ParseErrorKind { .. })
            || self.lexer.whitespace_follows()
            || self.lexer.peek_token() != TokenType::LeftParenthesis
        {
            return callee;
        }
        if !is_reference_operand(&callee) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: self.lexer.get_position() as usize,
                message: "Only a reference or a LAMBDA can be called".to_string(),
            };
        }
        self.parse_call(callee)
    }

    /// `callee(args)`, at the `(` that opens the arguments: a call of whatever `callee`
    /// evaluates to. Its depth is covered by the operand's own ([`Parser::parse_primary`]).
    fn parse_call(&mut self, callee: Node) -> Node {
        self.lexer.advance_token(); // consume '('
        let call_args = match self.parse_function_args("LAMBDA") {
            Ok(args) => args,
            Err(e) => return e,
        };
        if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: err.position,
                message: err.message,
            };
        }
        Node::LambdaCallKind {
            lambda: Box::new(callee),
            args: call_args,
        }
    }

    /// The parameters (into `parameters`) and the body of a LAMBDA, up to and including its
    /// `)`; a parse error as `Err`.
    fn parse_lambda_items(
        &mut self,
        arg_separator: &TokenType,
        parameters: &mut Vec<NamedVariable>,
    ) -> Result<Node, Node> {
        // Parse items one by one. Items followed by the separator are parameters (must
        // be plain names or [name]). The final item before ')' is the body expression.
        let mut arg_index = 1;
        loop {
            // The cursor is currently in item `arg_index` of LAMBDA, so an EOF
            // here reports the argument context for the Formula Helper. LAMBDA
            // is parsed specially (not through `parse_function_args`), so this
            // hint has to be stamped explicitly.
            self.set_argument_hint("LAMBDA", arg_index);

            // Optional parameter syntax: [name]
            let bracket_optional = self.lexer.peek_token() == TokenType::LeftBracket;
            if bracket_optional {
                // consume '['
                self.lexer.advance_token();
            }

            let token = self.lexer.peek_token();
            let mut expr = self.parse_enclosed_expr();
            if let Node::ParseErrorKind { .. } = expr {
                return Err(expr);
            }

            if bracket_optional {
                if let Err(err) = self.lexer.expect(TokenType::RightBracket) {
                    return Err(Node::ParseErrorKind {
                        formula: self.lexer.get_formula(),
                        expecting: vec![ExpectedTokens::Other],
                        position: err.position,
                        message: "Expected ']' after optional parameter name".to_string(),
                    });
                }
            }

            let next = self.lexer.peek_token();
            if next == *arg_separator {
                // This item must be a parameter name.
                self.lexer.advance_token();
                arg_index += 1;
                expr = self.bind_dotted_name(&token, expr);
                match expr {
                    Node::NamedVariableKind { name, id } => {
                        // xlop: Excel Optional Parameter.
                        let (clean, is_optional) = if let Some(s) = name.strip_prefix("_xlop.") {
                            (s.to_string(), true)
                        } else {
                            (name, bracket_optional)
                        };
                        parameters.push(NamedVariable {
                            name: clean,
                            id,
                            is_optional,
                        });
                    }
                    _ => {
                        return Err(Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: "LAMBDA parameter must be a name".to_string(),
                        });
                    }
                }
            } else if next == TokenType::RightParenthesis {
                self.lexer.advance_token(); // consume ')'
                return Ok(expr);
            } else {
                return Err(Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: self.lexer.get_position() as usize,
                    message: "Expected ',' or ')' in LAMBDA".to_string(),
                });
            }
        }
    }
}
