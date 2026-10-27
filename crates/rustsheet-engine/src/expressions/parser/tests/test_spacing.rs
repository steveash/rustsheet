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

/// A run between two commas or after the last one is the missing argument's own (T-2944,
/// probes/t2786-whitespace-structured-refs-runs I2:I10).
#[test]
fn whitespace_beside_a_missing_argument_prints_back() {
    for formula in [
        "IF(1, ,2)",
        "IF(1,  ,2)",
        "IF(1,\n,2)",
        "SUM(1, )",
        "SUM(1,  )",
        "SUM(1,\n)",
        "IF(1,2, )",
        "SUM(, 1)",
        "SUM(,1)",
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
        // A run keeps its line feeds and only the spaces after the last one
        // (probes/t2776-formula-text-whitespace-shapes C13, C19:C22).
        ("\n1+2", "\n1+2"),
        ("1+\n2", "1+\n2"),
        ("1+ \n 2", "1+\n 2"),
        ("1+\n \n2", "1+\n\n2"),
        ("1+          \n\n\n  2", "1+\n\n\n  2"),
        ("1+2 \n ", "1+2"),
    ] {
        let node = parser.parse(formula, &context());
        assert_eq!(to_english_localized_string(&node, &context()), text);
    }
}

#[test]
fn a_space_count_wraps_at_256() {
    // `=1+` + 300 spaces + `2` prints 44 spaces (probes/t2776-formula-text-whitespace-shapes), and
    // exactly 256 print none (probes/t2782-whitespace-run-wrap).
    let mut parser = parser();
    for (spaces, kept) in [(255, 255), (256, 0), (257, 1), (300, 44)] {
        let formula = format!("1+{}2", " ".repeat(spaces));
        let node = parser.parse(&formula, &context());
        let text = to_english_localized_string(&node, &context());
        assert_eq!(text, format!("1+{}2", " ".repeat(kept)), "{spaces}");
    }
}

#[test]
fn a_line_feed_count_wraps_at_256_on_its_own() {
    // Each count is one byte and wraps by itself; spaces before the last line feed are dropped
    // even when the line-feed count wraps to none (probes/t2782-whitespace-run-wrap).
    let mut parser = parser();
    for (run, kept) in [
        ("\n".repeat(255), "\n".repeat(255)),
        ("\n".repeat(256), String::new()),
        ("\n".repeat(257), "\n".to_string()),
        ("\n".repeat(300), "\n".repeat(44)),
        (format!("{}   ", "\n".repeat(256)), "   ".to_string()),
        (format!("   {}", "\n".repeat(256)), String::new()),
    ] {
        let node = parser.parse(&format!("1+{run}2"), &context());
        let text = to_english_localized_string(&node, &context());
        assert_eq!(text, format!("1+{kept}2"), "{run:?}");
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
