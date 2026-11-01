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
use std::ops::{Deref, DerefMut};

use super::{Node, Parser};

/// Chains up to this many links are walked without allocating (the common case:
/// `=A1*2+B1`); longer ones collect their links once.
const SHORT_CHAIN: usize = 16;

/// The left operand of a value operator node. It derefs to the boxed node; its
/// `Clone` copies the chain below it in a loop (a derived `Clone` overflowed a
/// 1 MiB optimized stack at about 4,600 links), so `Node` keeps a derived
/// `Clone`.
#[derive(PartialEq, Debug)]
pub struct ChainLeft(Box<Node>);

impl ChainLeft {
    /// `node` as the left operand of a value operator.
    pub fn new(node: Node) -> ChainLeft {
        ChainLeft(Box::new(node))
    }
}

impl Deref for ChainLeft {
    type Target = Node;
    fn deref(&self) -> &Node {
        &self.0
    }
}

impl DerefMut for ChainLeft {
    fn deref_mut(&mut self) -> &mut Node {
        &mut self.0
    }
}

impl Clone for ChainLeft {
    fn clone(&self) -> Self {
        let (links, first) = left_chain(self);
        // `first` is not a value operator, so its derived `Clone` does not walk this chain.
        let mut node = first.clone();
        for link in links {
            node = relink(link, node);
        }
        ChainLeft::new(node)
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
pub(crate) fn value_operands_mut(node: &mut Node) -> Result<(&mut Node, &mut Node), &mut Node> {
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
pub(crate) fn left_chain(node: &Node) -> (ChainLinks<'_>, &Node) {
    let mut length = 0;
    let mut first = node;
    while let Some((left, _)) = value_operands(first) {
        length += 1;
        first = left;
    }
    let links = if length <= SHORT_CHAIN {
        ChainLinks::Short {
            top: node,
            remaining: length,
        }
    } else {
        let mut links = Vec::with_capacity(length);
        let mut link = node;
        while let Some((left, _)) = value_operands(link) {
            links.push(link);
            link = left;
        }
        ChainLinks::Long(links)
    };
    (links, first)
}

/// The links of a chain, innermost first (see [`left_chain`]).
pub(crate) enum ChainLinks<'a> {
    /// A short chain: each link is found by walking down from `top` again,
    /// which costs at most [`SHORT_CHAIN`] steps and no allocation.
    Short { top: &'a Node, remaining: usize },
    /// A long chain's links, outermost first (popped from the end).
    Long(Vec<&'a Node>),
}

impl<'a> Iterator for ChainLinks<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<&'a Node> {
        match self {
            ChainLinks::Short { top, remaining } => {
                *remaining = remaining.checked_sub(1)?;
                let mut link = *top;
                for _ in 0..*remaining {
                    link = value_operands(link)?.0;
                }
                Some(link)
            }
            ChainLinks::Long(links) => links.pop(),
        }
    }
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
    let left = ChainLeft::new(left);
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
        other => other.clone(),
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
        let (left, right) = (ChainLeft::new(left), Box::new(right));
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
        let t = self.parse_chain_operand(level);
        if matches!(t, Node::ParseErrorKind { .. }) || !self.at_chain_operator(level) {
            return t;
        }
        self.parse_chain_links(level, t)
    }

    /// Whether the next token is an operator of `level`. A method of its own, like
    /// [`Self::parse_chain_links`], so that the frame `parse_chain` keeps on the stack for every
    /// nesting level of a formula stays small in unoptimized builds (T-1110).
    fn at_chain_operator(&mut self, level: ChainLevel) -> bool {
        ChainOperator::at(level, self.lexer.peek()).is_some()
    }

    /// The links of a chain after its first operand `t`.
    fn parse_chain_links(&mut self, level: ChainLevel, mut t: Node) -> Node {
        let mut depth = self.depth;
        while let Some(operator) = ChainOperator::at(level, self.lexer.peek()) {
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
