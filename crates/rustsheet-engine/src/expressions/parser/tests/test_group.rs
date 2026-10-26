#![allow(clippy::panic, reason = "tests")]

//! Written parentheses around value expressions are a [`Node::Group`] (SPEC-0007, T-1340):
//! printed back as written, as Excel's FORMULATEXT and stored formula text keep them
//! (probes/at-operand-nested-groups, probes/power-chain-text; docs/excel-semantics/syntax.md).

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::{to_excel_string, to_rc_format};
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::{Node, Parser};
use crate::expressions::types::CellReferenceRC;

pub(super) fn context() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    }
}

pub(super) fn parser() -> Parser {
    new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new())
}

#[test]
fn written_groups_print_back_and_round_trip() {
    let mut parser = parser();
    for formula in [
        "((A1+B1)+C1)*D1",
        "A1+(B1)",
        "-(A1)",
        "(A1)*((B1))",
        "A1+(B1*C1)",
        "IF((A1>0),(B1),C1)",
        "SUM((A1)+(B1))",
        "(A1) (B1)",
        // `^` chains print as written (probes/power-chain-text).
        "A1^2^3",
        "(A1^2)^3",
        "A1^(2^3)",
        "-A1^2",
        "(-A1)^2",
        // Operands print bare where no parentheses were written (probes/operand-wrap-text).
        "SUM(A1)^2",
        "2^SUM(A1)",
        "2^-A1",
        "2^A2%",
        "SUM({1,2}^2)",
        "-A1%",
        "2*-A1",
        "-SUM(A1)^2",
        "2^-A1^2",
        "2^--A1",
    ] {
        assert_prints_back_and_round_trips(&mut parser, formula);
    }
}

/// `formula` prints back as written, in the stored xlsx text too (SPEC-0007, excel-graph `<f>`
/// texts), and its R1C1 text parses back to the same tree.
pub(super) fn assert_prints_back_and_round_trips(parser: &mut Parser, formula: &str) {
    let node = parser.parse(formula, &context());
    assert_eq!(to_english_localized_string(&node, &context()), formula);
    assert_eq!(
        to_english_localized_string(&node.clone(), &context()),
        formula
    );
    assert_eq!(to_excel_string(&node, &context()), formula);
    let rc = to_rc_format(&node);
    parser.set_lexer_mode(LexerMode::R1C1);
    let reparsed = parser.parse(&rc, &context());
    parser.set_lexer_mode(LexerMode::A1);
    assert_eq!(reparsed, node, "{formula}");
}

#[test]
fn directly_nested_levels_are_one_group() {
    let mut parser = parser();
    let node = parser.parse("A1+((B1))", &context());
    let Node::OpSumKind { right, .. } = node else {
        panic!("not a sum: {node:?}");
    };
    assert!(
        matches!(*right, Node::Group { levels: 2, ref child } if matches!(**child, Node::ReferenceKind { .. }))
    );
    // `((A1)+(B1))` is one level around a sum of two groups.
    let node = parser.parse("1*((A1)+(B1))", &context());
    let Node::OpProductKind { right, .. } = node else {
        panic!("not a product: {node:?}");
    };
    let Node::Group { levels: 1, child } = *right else {
        panic!("not one level: {right:?}");
    };
    assert!(matches!(*child, Node::OpSumKind { .. }));
}

/// A tree built in code with no `Group` prints the parentheses it needs to parse back to the
/// same tree: an equal-level right operand is wrapped, operators being left associative
/// (T-1344's one precedence rule).
#[test]
fn a_right_nested_tree_without_groups_prints_its_parentheses() {
    use crate::expressions::parser::ChainLeft;
    use crate::expressions::token::OpSum;
    let mut parser = parser();
    let operand = |formula: &str, parser: &mut Parser| parser.parse(formula, &context());
    let node = Node::OpSumKind {
        kind: OpSum::Minus,
        left: ChainLeft::new(operand("A1", &mut parser)),
        right: Box::new(operand("B1+C1", &mut parser)),
    };
    assert_eq!(to_english_localized_string(&node, &context()), "A1-(B1+C1)");
    let node = Node::OpPowerKind {
        left: ChainLeft::new(operand("A1", &mut parser)),
        right: Box::new(operand("2^3", &mut parser)),
    };
    assert_eq!(to_english_localized_string(&node, &context()), "A1^(2^3)");
}
