//! A unary plus is kept in the formula text except directly before a number literal, where it
//! folds into the number (probes/t2723-unary-plus-formula-text, excel-graph; T-2723;
//! docs/excel-semantics/syntax.md "A typed unary plus stays in the formula text").

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::{to_excel_string, to_rc_format};
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::{Node, Parser};
use crate::expressions::types::CellReferenceRC;

fn context() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    }
}

fn parser() -> Parser {
    new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new())
}

#[test]
fn a_plus_off_a_number_prints_back_and_round_trips() {
    let mut parser = parser();
    for formula in [
        "+\"a\"",
        "+TRUE",
        "+B1",
        "+B1:B2",
        "+SUM(1,2)",
        "+(1)",
        "+-5",
        "++\"a\"",
        "1++\"2\"",
        "+{1,2}",
        "+#N/A",
        "+\"a\"&\"b\"",
        "SUM(+\"1\",2)",
        "IF(TRUE,+\"a\")",
        "+(+\"a\")",
        "T(INDEX(+\"a\",0,0))",
        "N(INDEX(+TRUE,0))",
        "+B1%",
        "+B1^2",
        "2^+B1",
        "-+B1",
    ] {
        let node = parser.parse(formula, &context());
        assert_eq!(to_english_localized_string(&node, &context()), formula);
        assert_eq!(to_excel_string(&node, &context()), formula);
        let rc = to_rc_format(&node);
        parser.set_lexer_mode(LexerMode::R1C1);
        let reparsed = parser.parse(&rc, &context());
        parser.set_lexer_mode(LexerMode::A1);
        assert_eq!(reparsed, node, "{formula}");
    }
}

#[test]
fn a_plus_on_a_lambda_call_is_kept() {
    let node = parser().parse("+LAMBDA(x,x)(1)", &context());
    assert!(matches!(node, Node::UnaryPlus { .. }), "{node:?}");
    let text = to_english_localized_string(&node, &context());
    assert_eq!(text, "+LAMBDA(x,x)(1)");
}

#[test]
fn a_plus_directly_before_a_number_folds() {
    let mut parser = parser();
    for (formula, text) in [
        ("+5", "5"),
        ("+1E+3", "1000"),
        ("+.5", "0.5"),
        ("+5%", "5%"),
        ("++5", "+5"),
        ("1++2", "1+2"),
        ("-+5", "-5"),
        ("N(INDEX(+5,0,0))", "N(INDEX(5,0,0))"),
    ] {
        let node = parser.parse(formula, &context());
        assert_eq!(
            to_english_localized_string(&node, &context()),
            text,
            "{formula}"
        );
    }
    // A space breaks the fold, and the plus prints without it (the tree keeps no spaces).
    let node = parser.parse("+ 5", &context());
    assert!(matches!(node, Node::UnaryPlus { .. }), "{node:?}");
}
