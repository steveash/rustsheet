//! Operator chains (T-0128). The parser builds `=A1+A2+...+A4000` as
//! left-nested binary nodes, so a chain of N terms is a tree N deep down its
//! left operands. Every consumer that recurses over the tree walks such a chain
//! with the helpers below, in a loop down the left operands, and recurses only
//! into the right operands and the first operand. A chain therefore costs one
//! stack frame however long it is, and the parser's depth bound
//! ([`super::MAX_FORMULA_DEPTH`]) does not count its links (see
//! `Parser::grow_chain`).
//!
//! The parser builds every chain in one loop, [`Parser::parse_chain`].
//!
//! The chained operators are the value operators: `+ - * / ^ &` and the
//! comparisons. The reference operators (`:`, space, `,`) are not chained:
//! their trees stay bounded by the depth limit.

use super::super::token::{OpCompare, OpProduct, OpSum, TokenType};
use super::{Node, Parser};

impl Clone for Node {
    /// Copies a chain of value operators in a loop down its left operands, so a long chain
    /// costs no stack (a derived `Clone` overflowed a 1 MiB stack at about 4,600 links).
    fn clone(&self) -> Self {
        let (links, first) = left_chain(self);
        let mut node = first.clone_operand();
        for link in links {
            node = relink(link, node);
        }
        node
    }
}

/// The `(left, right)` operands of `node` if it is a value operator.
pub(crate) fn value_operands(node: &Node) -> Option<(&Node, &Node)> {
    match node {
        Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::CompareKind { left, right, .. } => Some((left, right)),
        _ => None,
    }
}

/// The `(left, right)` operands of `node` if it is a value operator, else
/// `node` itself back.
fn value_operands_mut(node: &mut Node) -> Result<(&mut Node, &mut Node), &mut Node> {
    match node {
        Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::CompareKind { left, right, .. } => Ok((left, right)),
        other => Err(other),
    }
}

/// The chain of value operators from `node` down its left operands, innermost
/// (first applied) first, and the chain's first operand. A `node` that is not
/// a value operator is an empty chain whose first operand is `node`.
///
/// Folding the links in order over the first operand's result gives the same
/// result, in the same evaluation order, as recursing into each left operand.
pub(crate) fn left_chain(node: &Node) -> (Vec<&Node>, &Node) {
    let mut links = Vec::new();
    let mut first = node;
    while let Some((left, _)) = value_operands(first) {
        links.push(first);
        first = left;
    }
    links.reverse();
    (links, first)
}

/// Calls `f` on every operand of the chain rooted at `node` (see
/// [`left_chain`]): each link's right operand, outermost first, then the first
/// operand. For walkers whose visits do not depend on one another's order.
pub(crate) fn for_each_chain_operand_mut(node: &mut Node, mut f: impl FnMut(&mut Node)) {
    let mut current = node;
    loop {
        match value_operands_mut(current) {
            Ok((left, right)) => {
                f(right);
                current = left;
            }
            Err(first) => {
                f(first);
                return;
            }
        }
    }
}

/// A copy of value operator `link` over `left` in place of its left operand.
fn relink(link: &Node, left: Node) -> Node {
    let left = Box::new(left);
    match link {
        Node::OpSumKind { kind, right, .. } => Node::OpSumKind {
            kind: kind.clone(),
            left,
            right: right.clone(),
        },
        Node::OpProductKind { kind, right, .. } => Node::OpProductKind {
            kind: kind.clone(),
            left,
            right: right.clone(),
        },
        Node::OpPowerKind { right, .. } => Node::OpPowerKind {
            left,
            right: right.clone(),
        },
        Node::OpConcatenateKind { right, .. } => Node::OpConcatenateKind {
            left,
            right: right.clone(),
        },
        Node::CompareKind { kind, right, .. } => Node::CompareKind {
            kind: kind.clone(),
            left,
            right: right.clone(),
        },
        // `left_chain` links are value operators only.
        other => other.clone_operand(),
    }
}

impl Node {
    /// A copy of `self`, which is not a value operator (those are copied by `relink`). Its
    /// subexpressions are copied with `Node::clone`.
    fn clone_operand(&self) -> Node {
        match self {
            Node::BooleanKind(value) => Node::BooleanKind(*value),
            Node::NumberKind(value) => Node::NumberKind(*value),
            Node::StringKind(value) => Node::StringKind(value.clone()),
            Node::ReferenceKind {
                sheet_name,
                sheet_index,
                absolute_row,
                absolute_column,
                row,
                column,
            } => Node::ReferenceKind {
                sheet_name: sheet_name.clone(),
                sheet_index: *sheet_index,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
                row: *row,
                column: *column,
            },
            Node::RangeKind {
                sheet_name,
                sheet_index,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
            } => Node::RangeKind {
                sheet_name: sheet_name.clone(),
                sheet_index: *sheet_index,
                absolute_row1: *absolute_row1,
                absolute_column1: *absolute_column1,
                row1: *row1,
                column1: *column1,
                absolute_row2: *absolute_row2,
                absolute_column2: *absolute_column2,
                row2: *row2,
                column2: *column2,
            },
            Node::SheetSpanKind {
                first_sheet,
                last_sheet,
                first_index,
                last_index,
                reference,
            } => Node::SheetSpanKind {
                first_sheet: first_sheet.clone(),
                last_sheet: last_sheet.clone(),
                first_index: *first_index,
                last_index: *last_index,
                reference: reference.clone(),
            },
            Node::WrongReferenceKind {
                sheet_name,
                absolute_row,
                absolute_column,
                row,
                column,
            } => Node::WrongReferenceKind {
                sheet_name: sheet_name.clone(),
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
                row: *row,
                column: *column,
            },
            Node::WrongRangeKind {
                sheet_name,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
            } => Node::WrongRangeKind {
                sheet_name: sheet_name.clone(),
                absolute_row1: *absolute_row1,
                absolute_column1: *absolute_column1,
                row1: *row1,
                column1: *column1,
                absolute_row2: *absolute_row2,
                absolute_column2: *absolute_column2,
                row2: *row2,
                column2: *column2,
            },
            Node::OpRangeKind { left, right } => Node::OpRangeKind {
                left: left.clone(),
                right: right.clone(),
            },
            Node::OpIntersectKind { left, right } => Node::OpIntersectKind {
                left: left.clone(),
                right: right.clone(),
            },
            Node::OpUnionKind {
                left,
                right,
                written,
            } => Node::OpUnionKind {
                left: left.clone(),
                right: right.clone(),
                written: *written,
            },
            Node::FunctionKind { kind, args } => Node::FunctionKind {
                kind: kind.clone(),
                args: args.clone(),
            },
            Node::LambdaDefKind { parameters, body } => Node::LambdaDefKind {
                parameters: parameters.clone(),
                body: body.clone(),
            },
            Node::LambdaCallKind { lambda, args } => Node::LambdaCallKind {
                lambda: lambda.clone(),
                args: args.clone(),
            },
            Node::NamedFunctionKind { id, name, args } => Node::NamedFunctionKind {
                id: *id,
                name: name.clone(),
                args: args.clone(),
            },
            Node::ArrayKind(rows) => Node::ArrayKind(rows.clone()),
            Node::DefinedNameKind(name) => Node::DefinedNameKind(name.clone()),
            Node::TableNameKind(name) => Node::TableNameKind(name.clone()),
            Node::NamedVariableKind { name, id } => Node::NamedVariableKind {
                name: name.clone(),
                id: *id,
            },
            Node::ImplicitIntersection { automatic, child } => Node::ImplicitIntersection {
                automatic: *automatic,
                child: child.clone(),
            },
            Node::SpillRangeOperator { child } => Node::SpillRangeOperator {
                child: child.clone(),
            },
            Node::UnaryKind { kind, right } => Node::UnaryKind {
                kind: kind.clone(),
                right: right.clone(),
            },
            Node::ErrorKind(error) => Node::ErrorKind(error.clone()),
            Node::ParseErrorKind {
                formula,
                message,
                position,
                expecting,
            } => Node::ParseErrorKind {
                formula: formula.clone(),
                message: message.clone(),
                position: *position,
                expecting: expecting.clone(),
            },
            Node::EmptyArgKind => Node::EmptyArgKind,
            // A value operator: `left_chain` never hands one over; copy it whole.
            Node::OpSumKind { .. }
            | Node::OpProductKind { .. }
            | Node::OpPowerKind { .. }
            | Node::OpConcatenateKind { .. }
            | Node::CompareKind { .. } => self.clone(),
        }
    }
}

/// The precedence levels of the value operators, loosest first. Each level's
/// operands are chains of the next tighter level.
#[derive(Clone, Copy)]
pub(super) enum ChainLevel {
    Compare,
    Concat,
    Sum,
    Product,
    Power,
}

/// An operator of a chain, with what it needs to build its node.
enum ChainOperator {
    Compare(OpCompare),
    Concat,
    Sum(OpSum),
    Product(OpProduct),
    Power,
}

impl ChainOperator {
    /// The operator `token` is at `level`, if it is one.
    fn at(level: ChainLevel, token: &TokenType) -> Option<ChainOperator> {
        match (level, token) {
            (ChainLevel::Compare, TokenType::Compare(op)) => {
                Some(ChainOperator::Compare(op.clone()))
            }
            (ChainLevel::Concat, TokenType::And) => Some(ChainOperator::Concat),
            (ChainLevel::Sum, TokenType::Addition(op)) => Some(ChainOperator::Sum(op.clone())),
            (ChainLevel::Product, TokenType::Product(op)) => {
                Some(ChainOperator::Product(op.clone()))
            }
            (ChainLevel::Power, TokenType::Power) => Some(ChainOperator::Power),
            _ => None,
        }
    }

    /// The node `left <operator> right`.
    fn node(self, left: Node, right: Node) -> Node {
        let (left, right) = (Box::new(left), Box::new(right));
        match self {
            ChainOperator::Compare(kind) => Node::CompareKind { kind, left, right },
            ChainOperator::Concat => Node::OpConcatenateKind { left, right },
            ChainOperator::Sum(kind) => Node::OpSumKind { kind, left, right },
            ChainOperator::Product(kind) => Node::OpProductKind { kind, left, right },
            ChainOperator::Power => Node::OpPowerKind { left, right },
        }
    }
}

impl Parser {
    /// Parses `operand (operator operand)*` for the operators of `level`, as
    /// left-nested nodes (`1-2-3` is `(1-2)-3`), in a loop.
    pub(super) fn parse_chain(&mut self, level: ChainLevel) -> Node {
        let mut t = self.parse_chain_operand(level);
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut depth = self.depth;
        while let Some(operator) = ChainOperator::at(level, &self.lexer.peek_token()) {
            self.lexer.advance_token();
            let p = self.parse_chain_operand(level);
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            if let Err(error) = self.grow_chain(&t, depth, self.depth) {
                return error;
            }
            depth = self.depth;
            t = operator.node(t, p);
        }
        t
    }

    /// An operand of a `level` chain: a chain of the next tighter level, or a
    /// unary expression under `^`.
    fn parse_chain_operand(&mut self, level: ChainLevel) -> Node {
        match level {
            ChainLevel::Compare => self.parse_chain(ChainLevel::Concat),
            ChainLevel::Concat => self.parse_chain(ChainLevel::Sum),
            ChainLevel::Sum => self.parse_chain(ChainLevel::Product),
            ChainLevel::Product => self.parse_chain(ChainLevel::Power),
            ChainLevel::Power => self.parse_power(),
        }
    }
}
