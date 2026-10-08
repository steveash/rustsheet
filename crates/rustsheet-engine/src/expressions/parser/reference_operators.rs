//! The reference operators, which bind tighter than every other operator: the union (a
//! comma inside parentheses, `(A1,B2)`), the intersection (a space, `A1:B2 B1:B3`), the range
//! (`:`), `@` and the spill operator `#`. Also the check that a formula has no input left over
//! after its expression.
//!
//! Excel evidence: docs/excel-semantics/references.md "Intersection operator (space)" (gen-m1,
//! T-0105) and "Union (comma) reference in a value context is #VALUE!" (gen-m1, T-0134).

use crate::expressions::parser::depth::{MAX_FORMULA_DEPTH, PRIMARY_DEPTH};
use crate::expressions::parser::{ExpectedTokens, Node, Parser};
use crate::expressions::token::{Error, OpUnary, TokenType};
use crate::functions::Function;
use crate::locale::Locale;

use super::stringify::argument_separator;

impl Parser {
    /// `'(' expr (',' expr)* ')'`, after the `(`. Directly inside it, and nowhere else, the
    /// argument separator is the union operator: `(A1,B2)` is one union, `SUM(A1,B2)` two
    /// arguments. The union is the loosest operator inside its parenthesis, so `(A1,B1 C1)` is
    /// `(A1,(B1 C1))`. Its operands must be references, or a negated one: Excel loads `(-A1,B1)`
    /// but refuses `(1,A1)` and `(A1+B1,C1)` (docs/excel-semantics/references.md, "Union:
    /// parsing and evaluation", T-0187).
    pub(super) fn parse_parenthesized(&mut self) -> Node {
        let after_at = std::mem::take(&mut self.at_paren);
        // `@((A1 B1))`: the inner group is a level of the same operand.
        self.at_paren = after_at && self.lexer.peek_token() == TokenType::LeftParenthesis;
        // The first group parsed is the one the formula's first `(` opens.
        let leading = std::mem::take(&mut self.leading_paren);
        let mut t = self.parse_expr();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let separator = self.get_argument_separator_token();
        let mut union_depth = self.depth;
        while self.lexer.peek_token() == separator {
            self.lexer.advance_token();
            let p = self.parse_expr();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            if !is_union_operand(&t) || !is_union_operand(&p) {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Range],
                    position: self.lexer.get_position() as usize,
                    message: "Union needs references".to_string(),
                };
            }
            if let Err(error) = self.grow(union_depth, self.depth) {
                return error;
            }
            union_depth = self.depth;
            t = Node::OpUnionKind {
                left: Box::new(t),
                right: Box::new(p),
                written: 0,
            };
        }
        self.inner_depth = self.inner_depth.max(union_depth);
        let close = self.lexer.get_position_index();
        if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: err.position,
                message: err.message,
            };
        }
        if leading && self.lexer.peek_token() == TokenType::EOF {
            self.grouped_root = true;
        }
        // Excel keeps every level of written parentheses around a union (`((A1,B2))`).
        if let Node::OpUnionKind { written, .. } = &mut t {
            *written = written.saturating_add(1);
        }
        // An `@` operand prints its written levels back (`ImplicitIntersection::written`).
        if after_at {
            let inner = self.at_group.take().filter(|&(_, end)| end == close);
            let levels = inner.map_or(0, |(levels, _)| levels).saturating_add(1);
            self.at_group = Some((levels, self.lexer.get_position_index()));
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
        // An `@` group is the leftmost operand of an intersection only if it closes exactly there
        // and holds a single operand: `@(A1):B1 C1` and `@(A1 B1) C1` would print parentheses
        // the user did not write around that node (T-1134).
        let mut stray_group = matches!(t, Node::OpIntersectKind { .. } | Node::OpUnionKind { .. })
            || self
                .at_group
                .is_some_and(|(_, end)| end != self.lexer.get_position_index());
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
                || !(name_error || (is_reference_operand(&t) && !has_negated_operand(&t)))
                || !is_reference_operand(&p)
                || has_negated_operand(&p)
            {
                return self.intersection_error();
            }
            as_name |= name_error;
            if std::mem::take(&mut stray_group) {
                self.at_group = None;
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

    /// `'@' intersect | primary ['#']`. The space binds tighter than `@`: Excel evaluates
    /// `=@A1:A3 A2:B2` in row 11 to 2, `@(A1:A3 A2:B2)`, where `=@A1:A3` alone is `#VALUE!`
    /// (probes/intersection-operands H11, probes/intersection-at-operands; T-0149).
    fn parse_implicit(&mut self) -> Node {
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::At {
            self.lexer.advance_token();
            // `@` recurses into the intersection without an operand in between (`A1 @A1
            // @A1 ...`), so it reserves nesting like an operand does to bound the recursion.
            if self.nesting + PRIMARY_DEPTH > MAX_FORMULA_DEPTH {
                return self.too_deep();
            }
            self.at_paren = self.lexer.peek_token() == TokenType::LeftParenthesis;
            self.at_group = None;
            let t = if self.lexer.peek_token() == TokenType::At {
                // `@@A1`: an operand cannot start with `@`.
                self.parse_primary()
            } else {
                self.nesting += PRIMARY_DEPTH;
                let t = self.parse_intersection();
                self.nesting -= PRIMARY_DEPTH;
                t
            };
            if let Node::ParseErrorKind { .. } = t {
                return t;
            }
            if let Err(error) = self.grow(self.depth, 0) {
                return error;
            }
            // Whole-operand parentheses (`@(A1)`) are `written`; a group that is only the leftmost
            // operand of an intersection (`@(A1) B1`) is `lead`.
            let group = self.at_group.take();
            let whole = group.filter(|&(_, end)| end == self.lexer.get_position_index());
            let written = whole.map_or(0, |(levels, _)| levels);
            let lead = match group {
                Some((levels, _))
                    if whole.is_none() && matches!(t, Node::OpIntersectKind { .. }) =>
                {
                    levels
                }
                _ => 0,
            };
            self.at_paren = false;
            return Node::ImplicitIntersection {
                automatic: false,
                child: Box::new(t),
                written,
                lead,
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

    /// The call `kind(args)`, or a parse error when an argument that must be a reference is not
    /// one. Excel for the web refuses a workbook (`FileCorruptTryRepair`) holding SUBTOTAL with
    /// a non-reference argument after the function number (`{1,2}`, `A1:A2*1`, `1`, `"A1"`,
    /// `-A1`, `#N/A`, `SUM(A1:A2)`, `FILTER(..)`, `VSTACK(..)`), an empty one (`SUBTOTAL(9,A1,)`)
    /// or a nested SUBTOTAL, and AREAS of `1` or `"A1"`; the operands the intersection accepts all
    /// load (docs/excel-semantics/references.md "SUBTOTAL and AREAS take references only",
    /// T-1063, T-1163).
    pub(super) fn function_call(&self, kind: Function, args: Vec<Node>) -> Node {
        let first_reference = match kind {
            Function::Subtotal => 1,
            Function::Areas => 0,
            _ => return Node::FunctionKind { kind, args },
        };
        let refused = args
            .iter()
            .skip(first_reference)
            .any(|arg| matches!(arg, Node::EmptyArgKind) || !is_reference_operand(arg));
        if refused {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: self.lexer.get_position() as usize,
                message: "This argument must be a reference".to_string(),
            };
        }
        Node::FunctionKind { kind, args }
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

/// Whether `node` may be an operand of a union: a reference expression, or a negated one
/// (Excel loads `(-A1,B1)` and prints it unchanged, probes/union-neg-operand H1:H2, T-0187; the
/// tree shape, a union of `-A1` and `B1`, is an engine choice).
fn is_union_operand(node: &Node) -> bool {
    // `(-A1,B1)`, `(--A1,B1)`, `(---A1,B1)` and `(----A1,B1)` all load in Excel and are `#VALUE!`
    // (probes/union-operand-{double-minus,triple-minus,quad-minus}, T-1031), so any number of
    // minuses is accepted.
    let mut operand = node;
    while let Node::UnaryKind {
        kind: OpUnary::Minus,
        right,
    } = operand
    {
        operand = right;
    }
    is_reference_operand(operand)
}

/// Whether `node` is, or holds at any depth of union, a union with a negated operand, the shape
/// Excel refuses (`FileCorruptTryRepair`) when it is an operand of an intersection: `(-A1,B1) A1`,
/// `(--A1,B1) A1`, `(A1,-B1) A1`, `(-(A1,B2),C1) A1`, `((-A1,B1),C1) A1` and `A1 (-A1,B1)`
/// (docs/excel-semantics/references.md "Union: parsing and evaluation", T-1024, T-1031).
fn has_negated_operand(node: &Node) -> bool {
    match node {
        Node::OpUnionKind { left, right, .. } => {
            matches!(**left, Node::UnaryKind { .. })
                || matches!(**right, Node::UnaryKind { .. })
                || has_negated_operand(left)
                || has_negated_operand(right)
        }
        _ => false,
    }
}

/// Whether `node` is a reference expression, the only kind of operand Excel accepts for an
/// intersection and for SUBTOTAL and AREAS arguments ([`Parser::function_call`])
/// (docs/excel-semantics/references.md "Intersection operands must be
/// references"): values, value operators, error literals other than `#REF!` and value
/// functions make Excel refuse the formula. The accepted calls are the reference-returning
/// functions below (probes/intersection-function-operands; T-0161); value functions such as
/// SUM, NA, ABS, ROW, ROWS, MATCH, VLOOKUP, HLOOKUP, LOOKUP, IFERROR, IFNA, CHOOSEROWS, CHOOSECOLS, FILTER,
/// SORT, UNIQUE and VSTACK are refused, even those that pass a range through; functions not
/// probed are refused by inference.
pub(super) fn is_reference_operand(node: &Node) -> bool {
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
/// call, a parenthesis, `@` (`=A1 @A1` is 1, probes/intersection-operands H10), or the
/// `#REF!` Excel writes in place of a deleted reference.
fn starts_reference(token: &TokenType) -> bool {
    matches!(
        token,
        TokenType::Reference { .. }
            | TokenType::At
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
/// anything the parser does not take as a reference operand (`A1 (1)`). An `@` operand binds
/// looser than the space, so it prints bare, as Excel's FORMULATEXT does (`A1 @A1`,
/// probes/intersection-at-formulatext, T-1098), only as a right operand that nothing follows:
/// the left operand `A1 @A1` of `(A1 @A1) B1` keeps its parentheses, since bare `A1 @A1 B1`
/// is `A1 @(A1 B1)` (T-0149).
pub(crate) fn intersection_operand(text: String, node: &Node, is_left: bool) -> String {
    let bare = match node {
        Node::OpIntersectKind { right, .. } => {
            is_left && !matches!(**right, Node::ImplicitIntersection { .. })
        }
        Node::ImplicitIntersection { .. } => !is_left,
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
/// than the intersection (`:`, `#`): an intersection there needs parentheses, `(A1 B1):C3`,
/// and so does an `@`, which would otherwise take the rest of the operator's text
/// (`A1:(@B1)`, `(@(A1 B1)):C3`; T-1102).
pub(crate) fn tight_operand(text: String, node: &Node) -> String {
    if matches!(
        node,
        Node::OpIntersectKind { .. } | Node::ImplicitIntersection { .. }
    ) {
        format!("({text})")
    } else {
        text
    }
}

/// `text` (the printed `node`) as the operand of `@`. An intersection prints bare, as Excel does
/// (`@A1:B2 B1`, T-1098), but a nested `@` needs parentheses, `@(@(A1 B1))` (T-1102), and any operand
/// keeps the `written` levels it had, `@(A1 B1)`, `@(A1)`, `@((A1 B1))` (T-1109, T-1116).
pub(crate) fn at_operand(text: String, node: &Node, written: u8) -> String {
    // A nested `@` needs one level even when none was written (programmatic trees).
    let levels = if matches!(node, Node::ImplicitIntersection { .. }) {
        written.max(1)
    } else if matches!(node, Node::OpUnionKind { .. }) {
        // A union prints the levels it was written in itself, `@(A1,B2)`.
        0
    } else {
        written
    };
    (0..levels).fold(text, |text, _| format!("({text})"))
}

/// The text of the intersection chain `child` whose leftmost operand was written in `lead` levels
/// of parentheses, `(A1) B1` (`@(A1) B1`, T-1134); `None` when there are none to print. `sub`
/// prints an operand.
pub(crate) fn lead_group_text(
    child: &Node,
    lead: u8,
    sub: &dyn Fn(&Node) -> String,
) -> Option<String> {
    let mut rights = Vec::new();
    let mut leftmost = child;
    while let Node::OpIntersectKind { left, right } = leftmost {
        rights.push(&**right);
        leftmost = left;
    }
    if lead == 0 || rights.is_empty() {
        return None;
    }
    let mut text = (0..lead).fold(sub(leftmost), |text, _| format!("({text})"));
    for right in rights.into_iter().rev() {
        text = format!("{text} {}", intersection_operand(sub(right), right, false));
    }
    Some(text)
}

/// The printed `@` operand `child` with the parentheses it was written in: `written` around the
/// whole operand, `lead` around its leftmost operand only. `sub` prints a node.
pub(crate) fn at_text(
    child: &Node,
    written: u8,
    lead: u8,
    sub: &dyn Fn(&Node) -> String,
) -> String {
    let operand =
        lead_group_text(child, lead, sub).unwrap_or_else(|| at_operand(sub(child), child, written));
    format!("@{operand}")
}

/// `text` (the printed `node`) as the callee of a call, `(A1)(A2)`: a LAMBDA prints bare
/// (`LAMBDA(x,x)(1)`) and a union with its own parentheses (`union_text`); any other callee was
/// written in parentheses, and bare, `A1(A2)`, it would not parse back (T-0149).
pub(crate) fn callee_operand(text: String, node: &Node) -> String {
    if matches!(node, Node::LambdaDefKind { .. } | Node::OpUnionKind { .. }) {
        text
    } else {
        format!("({text})")
    }
}

/// The printed union `node` (an `OpUnionKind`) of `left_text` and `right_text` (the printed
/// operands), with the parentheses it was written in. It always carries at least one: without
/// them the comma would be an argument separator (`SUM((A1,B2))`) or left over input. A left
/// operand that is itself a union is printed without its own, `(A1,B2,C3)`, which parses back
/// to the same left-associative tree, unless it was written in parentheses (`((A1,B2),C3)`,
/// Excel keeps those parentheses, T-0193, and every extra level, `((A1,B2))`, T-1024).
pub(crate) fn union_text(
    node: &Node,
    left_text: &str,
    right_text: &str,
    locale: &Locale,
) -> String {
    // Callers pass only unions; any other node prints as a plain one-level union of its texts.
    let (left, written) = match node {
        Node::OpUnionKind { left, written, .. } => (&**left, *written),
        other => (other, 1),
    };
    let left_text = match left {
        Node::OpUnionKind { written: 0, .. } => left_text
            .strip_prefix('(')
            .and_then(|text| text.strip_suffix(')'))
            .unwrap_or(left_text),
        _ => left_text,
    };
    let levels = written.max(1);
    format!(
        "{}{left_text}{}{right_text}{}",
        "(".repeat(levels),
        argument_separator(locale),
        ")".repeat(levels)
    )
}
