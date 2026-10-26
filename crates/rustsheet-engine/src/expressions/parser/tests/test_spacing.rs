//! Typed whitespace prints back where Excel keeps it and is dropped where Excel drops it
//! (docs/excel-semantics/syntax.md "Formula text keeps typed whitespace…",
//! probes/t2769-formula-text-whitespace, probes/t2776-*; SPEC-0023).

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::to_rc_format;
use crate::expressions::parser::tests::test_group::{context, parser};
use crate::expressions::parser::tests::utils::to_english_localized_string;
use crate::expressions::parser::Node;

/// `formula` parses, prints back as typed (from the tree and from its clone) and survives the R1C1
/// storage text.
fn assert_prints_back(formula: &str) {
    let mut parser = parser();
    let node = parser.parse(formula, &context());
    assert!(!matches!(node, Node::ParseErrorKind { .. }), "{formula}");
    assert_eq!(to_english_localized_string(&node, &context()), formula);
    assert_eq!(
        to_english_localized_string(&node.clone(), &context()),
        formula
    );
    let rc = to_rc_format(&node);
    parser.set_lexer_mode(LexerMode::R1C1);
    let reparsed = parser.parse(&rc, &context());
    parser.set_lexer_mode(LexerMode::A1);
    assert_eq!(reparsed, node, "{formula} via {rc}");
}

#[test]
fn kept_whitespace_prints_back() {
    for formula in [
        "1 + 2",
        "1 +2",
        "1+ 2",
        "1  +  2",
        "B1 + B2",
        " 1+2",
        "- 5",
        "-+ 5",
        "+  5",
        "+ 5",
        "+ \"a\"",
        "5 %",
        "SUM( 1 )",
        "SUM(B1:B2 )",
        "(1 )+2",
        "( 1)+2",
        "SUM(1, 2)",
        "@ B1:B2",
        "\"a\" & \"b\"",
        "B1 = 1",
        "SUM(B1:B2) * 2",
        "LET(x, 1, x + 1)",
        "LET(x, 1, y, 2, x + y)",
        "-  -5",
        "1 +  - 5",
        "(  1  )",
        "( (A1) )",
        "1*2 + 3",
        "IF( TRUE,1,2)",
        "LAMBDA(x, x + 1)(2)",
    ] {
        assert_prints_back(formula);
    }
}

#[test]
fn dropped_whitespace_prints_without_it() {
    let mut parser = parser();
    for (formula, text) in [
        ("1+2 ", "1+2"),
        ("1=1 ", "1=1"),
        ("SUM(1 ,2)", "SUM(1,2)"),
        ("IF( TRUE ,1,2)", "IF( TRUE,1,2)"),
        ("{1, 2}", "{1,2}"),
        ("SUM({ 1 , 2 })", "SUM({1,2})"),
        ("SUM(B1 :B2)", "SUM(B1:B2)"),
        ("SUM(B1: B2)", "SUM(B1:B2)"),
        // A plus that folds into a number keeps the spaces before it.
        ("  +5", "  5"),
        ("1+  +5", "1+  5"),
        ("1+ +5", "1+ 5"),
        // Line feeds are T-2782's: a run holding one prints nothing yet.
        ("1+\n2", "1+2"),
    ] {
        let node = parser.parse(formula, &context());
        assert_eq!(to_english_localized_string(&node, &context()), text);
    }
}

#[test]
fn a_space_count_wraps_at_256() {
    // `=1+` + 300 spaces + `2` prints 44 spaces (probes/t2776-formula-text-whitespace-shapes).
    let mut parser = parser();
    for (spaces, kept) in [(255, 255), (256, 0), (257, 1), (300, 44)] {
        let formula = format!("1+{}2", " ".repeat(spaces));
        let node = parser.parse(&formula, &context());
        let text = to_english_localized_string(&node, &context());
        assert_eq!(text, format!("1+{}2", " ".repeat(kept)), "{spaces}");
    }
}

#[test]
fn whitespace_is_transparent() {
    let mut parser = parser();
    for (spaced, bare) in [
        ("1 + 2", "1+2"),
        ("SUM( B1:B2 )", "SUM(B1:B2)"),
        ("LET(x, 1, y, 2, x + y)", "LET(x,1,y,2,x+y)"),
        ("( A1, B1 )", "(A1,B1)"),
        ("- 5 %", "-5%"),
    ] {
        let mut node = parser.parse(spaced, &context());
        assert!(node.has_spaces(), "{spaced}");
        node.strip_spaces();
        assert!(!node.has_spaces(), "{spaced}");
        assert_eq!(node, parser.parse(bare, &context()), "{spaced}");
    }
}

#[test]
fn a_spaced_operand_in_a_chain_survives_a_clone() {
    let node = parser().parse("1*2 + 3", &context());
    let Node::OpSumKind { left, .. } = &node else {
        panic!("not a sum: {node:?}");
    };
    // The run before `+` is the leaf `2`'s, so the chain `1*2` keeps its plain spine.
    assert!(matches!(**left, Node::OpProductKind { .. }), "{node:?}");
    assert_eq!(node.clone(), node);
    assert_eq!(
        to_english_localized_string(&node.clone(), &context()),
        "1*2 + 3"
    );
}
