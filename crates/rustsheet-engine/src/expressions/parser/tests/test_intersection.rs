#![allow(clippy::panic)]

//! The intersection operator (a space between two references) and the rejection of input
//! left over after a complete formula (T-0105). Excel evidence: docs/excel-semantics/references.md
//! "Intersection operator (space)".

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::to_rc_format;
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::{Node, Parser, MAX_FORMULA_DEPTH, TOO_DEEP_MESSAGE};
use crate::expressions::types::CellReferenceRC;

fn context() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    }
}

fn parser() -> Parser {
    let worksheets = vec!["Sheet1".to_string(), "Data".to_string(), "Q'1".to_string()];
    new_parser(worksheets, vec![], HashMap::new())
}

fn is_intersection(node: &Node) -> bool {
    matches!(node, Node::OpIntersectKind { .. })
}

#[test]
fn space_between_references_is_an_intersection() {
    let mut parser = parser();
    let node = parser.parse("A1:B2 B1:B3", &context());
    let Node::OpIntersectKind { left, right, .. } = &node else {
        panic!("expected an intersection, got {node:?}");
    };
    assert!(matches!(**left, Node::RangeKind { .. }));
    assert!(matches!(**right, Node::RangeKind { .. }));
}

#[test]
fn intersection_chains_left_to_right() {
    let mut parser = parser();
    let node = parser.parse("Data!C1:D7 Data!C1:C12 Data!A6:E6", &context());
    let Node::OpIntersectKind { left, right, .. } = &node else {
        panic!("expected an intersection, got {node:?}");
    };
    assert!(is_intersection(left), "left operand: {left:?}");
    assert!(matches!(**right, Node::RangeKind { .. }));
}

#[test]
fn intersection_binds_tighter_than_other_operators() {
    let mut parser = parser();
    // `A B+1` is `(A B)+1`, `A B&""` is `(A B)&""`, `-A B` is `-(A B)`.
    let Node::OpSumKind { left, .. } = parser.parse("A1:A3 A2:C2+1", &context()) else {
        panic!("expected a sum");
    };
    assert!(is_intersection(&left));
    let Node::OpConcatenateKind { left, .. } = parser.parse("D:D 2:2&\"\"", &context()) else {
        panic!("expected a concatenation");
    };
    assert!(is_intersection(&left));
    let Node::UnaryKind { right, .. } = parser.parse("-A1:A3 A2", &context()) else {
        panic!("expected a negation");
    };
    assert!(is_intersection(&right));
    let Node::OpPowerKind { left, .. } = parser.parse("A1:A3 A2^2", &context()) else {
        panic!("expected a power");
    };
    assert!(is_intersection(&left));
    // The range operator binds tighter still: `A B:C` is `A (B:C)`.
    let Node::OpIntersectKind { right, .. } = parser.parse("A1 A1:A2:B3", &context()) else {
        panic!("expected an intersection");
    };
    assert!(matches!(*right, Node::OpRangeKind { .. }));
}

#[test]
fn intersection_operands_may_be_parenthesized_or_calls() {
    let mut parser = parser();
    for formula in [
        "(A1:B2) B1",
        "A1:B2 (B1:B3)",
        "(A1:B2 B1:B3)+1",
        "OFFSET(A1,0,0,3,3) B2",
        "SUM(A1:B2 B1:B3)",
        "Data!B:B Data!8:8",
        "'Q''1'!7:7 'Q''1'!E7",
        "$A$1:$D$12 A12",
        "Data!A7:A7:Data!C11 Data!A11",
    ] {
        let node = parser.parse(formula, &context());
        assert!(
            !matches!(node, Node::ParseErrorKind { .. }),
            "{formula}: {node:?}"
        );
    }
}

#[test]
fn whitespace_elsewhere_is_not_an_intersection() {
    let mut parser = parser();
    let cases = [
        // The spaces Excel keeps print back (SPEC-0023): not those at the end, before a comma,
        // in an array constant or around `:`.
        (" 1 + 2 ", " 1 + 2"),
        ("SUM( A1 , B1 )", "SUM( A1, B1 )"),
        ("SUM( A1:B2 )", "SUM( A1:B2 )"),
        ("SUM( A1 : B2 )", "SUM( A1:B2 )"),
        ("( A1 )", "( A1 )"),
        ("{ 1 , 2 ; 3 , 4 }", "{1,2;3,4}"),
        ("A1 + B1", "A1 + B1"),
        ("A1 -B1", "A1 -B1"),
        ("A1 & B1", "A1 & B1"),
        ("A1 = B1", "A1 = B1"),
        ("A1 * B1", "A1 * B1"),
        ("A1 ^ B1", "A1 ^ B1"),
        ("A1 %", "A1 %"),
        ("IF( A1 , B1 , C1 )", "IF( A1, B1, C1 )"),
        ("A1:B2 ", "A1:B2"),
    ];
    for (formula, expected) in cases {
        let node = parser.parse(formula, &context());
        assert_eq!(
            to_english_localized_string(&node, &context()),
            expected,
            "{formula}: {node:?}"
        );
    }
}

#[test]
fn intersections_round_trip_through_stringify() {
    let mut parser = parser();
    let cases = [
        ("A1:B2 B1:B3", "A1:B2 B1:B3"),
        ("Data!A1:C7 Data!C7:E12", "Data!A1:C7 Data!C7:E12"),
        (
            "Data!C1:D7 Data!C1:C12 Data!A6:E6",
            "Data!C1:D7 Data!C1:C12 Data!A6:E6",
        ),
        (
            "(Data!C2:C11 Data!C6:E6)+0.1",
            "(Data!C2:C11 Data!C6:E6)+0.1",
        ),
        ("(D:D 2:2)&\"\"", "(D:D 2:2)&\"\""),
        ("Data!C8:B3 Data!B8", "Data!B3:C8 Data!B8"),
        ("'Q''1'!7:7 'Q''1'!E7", "'Q''1'!7:7 'Q''1'!E7"),
        ("$A$10:$D$12 A12", "$A$10:$D$12 A12"),
        ("A1 (B1:C3 C1)", "A1 (B1:C3 C1)"),
        ("(A1 B1):C3", "(A1 B1):C3"),
        // `@` applies to the whole intersection after it; Excel prints it bare, and keeps written
        // parentheses (T-1098, T-1109).
        ("@(A1:B2 B1)", "@(A1:B2 B1)"),
        ("@A1:B2 B1", "@A1:B2 B1"),
        ("A1 @A1", "A1 @A1"),
        ("A1 @A1 @A1", "A1 @A1 @A1"),
        ("(A1 @A1) B1", "(A1 @A1) B1"),
        // Written parentheses around an `@` are kept too (SPEC-0007, T-1343).
        ("A1 (@A1 B1)", "A1 (@A1 B1)"),
        ("(@A1) B1", "(@A1) B1"),
        // An `@` operand of `:` or `@` is parenthesized so the text re-parses (T-1102).
        ("A1:(@B1)", "A1:(@B1)"),
        ("(@(A1 B1)):C3", "(@(A1 B1)):C3"),
        ("@(@(A1 B1))", "@(@(A1 B1))"),
        // Every level written around an `@` operand is kept, whatever the operand (T-1116).
        ("@((A1 B1))", "@((A1 B1))"),
        ("@(A1:B1)", "@(A1:B1)"),
        ("@(A1)", "@(A1)"),
        ("@(((A1)))", "@(((A1)))"),
        ("(@A1:B2)#", "(@A1:B2)#"),
        ("-A1:A3 A2", "-A1:A3 A2"),
        ("SUM(A1:B2 B1:B3,C1)", "SUM(A1:B2 B1:B3,C1)"),
    ];
    for (formula, expected) in cases {
        let node = parser.parse(formula, &context());
        let text = to_english_localized_string(&node, &context());
        assert_eq!(text, expected, "{formula}: {node:?}");
        // Re-parsing the printed formula gives the same tree, in A1 and in R1C1 (the form
        // formulas are stored in).
        assert_eq!(parser.parse(&text, &context()), node, "{formula}");
        let rc = to_rc_format(&node);
        parser.set_lexer_mode(LexerMode::R1C1);
        let reparsed = parser.parse(&rc, &context());
        parser.set_lexer_mode(LexerMode::A1);
        assert_eq!(reparsed, node, "{formula} via {rc}");
    }
}

#[test]
fn trailing_input_is_a_parse_error() {
    let mut parser = parser();
    for formula in [
        "1 2",
        "1+2 3",
        "\"a\" \"b\"",
        "TRUE FALSE",
        "1 A1",
        "A1 1",
        "A1 B1 2",
        "SUM(A1))",
        "A1)",
        "(1+2)3",
        "A1 B1,",
        "{1,2} A1",
    ] {
        let node = parser.parse(formula, &context());
        assert!(
            matches!(node, Node::ParseErrorKind { .. }),
            "{formula}: {node:?}"
        );
    }
}

#[test]
fn long_intersection_chains_hit_the_depth_cap() {
    let mut parser = parser();
    let chain = |terms: usize| vec!["A1"; terms].join(" ");
    let node = parser.parse(&chain(MAX_FORMULA_DEPTH / 2), &context());
    assert!(is_intersection(&node), "{node:?}");
    let node = parser.parse(&chain(MAX_FORMULA_DEPTH * 4), &context());
    let Node::ParseErrorKind { message, .. } = node else {
        panic!("expected the depth error, got {node:?}");
    };
    assert_eq!(message, TOO_DEEP_MESSAGE);
    // U+3000 chains are capped the same way (security review: no unbounded chain).
    let ideographic = vec!["A1"; MAX_FORMULA_DEPTH * 4].join("\u{3000}");
    let node = parser.parse(&ideographic, &context());
    assert!(matches!(node, Node::ParseErrorKind { .. }), "{node:?}");
}

#[test]
fn long_at_operand_chains_hit_the_depth_cap() {
    // `@` recurses into the intersection with no operand in between (T-0149).
    let mut parser = parser();
    let chain = |terms: usize| format!("A1{}", " @A1".repeat(terms));
    let node = parser.parse(&chain(MAX_FORMULA_DEPTH / 8), &context());
    assert!(is_intersection(&node), "{node:?}");
    let node = parser.parse(&chain(MAX_FORMULA_DEPTH * 4), &context());
    let Node::ParseErrorKind { message, .. } = node else {
        panic!("expected the depth error, got {node:?}");
    };
    assert_eq!(message, TOO_DEEP_MESSAGE);
    let node = parser.parse(&"@".repeat(MAX_FORMULA_DEPTH * 4), &context());
    assert!(matches!(node, Node::ParseErrorKind { .. }), "{node:?}");
}

#[test]
fn only_spaces_and_line_feeds_separate_operands() {
    let mut parser = parser();
    for separator in [" ", "  ", "\n", " \n ", "\n\n"] {
        let node = parser.parse(&format!("A1:A3{separator}A2:B2"), &context());
        assert!(is_intersection(&node), "{separator:?}: {node:?}");
    }
    // Excel refuses a workbook holding these (T-0138).
    for separator in ["\r", "\r\n", "\t", "\u{a0}", " \t "] {
        let node = parser.parse(&format!("A1:A3{separator}A2:B2"), &context());
        assert!(
            matches!(node, Node::ParseErrorKind { .. }),
            "{separator:?}: {node:?}"
        );
    }
}

#[test]
fn ideographic_space_makes_a_name_error() {
    // Excel reads it as part of a name: `#NAME?` (probes/intersection-whitespace H7).
    let mut parser = parser();
    // The text is kept, as an undefined name (evaluates to #NAME?, prints as written).
    let formula = "A1:A3\u{3000}A2:B2";
    let node = parser.parse(formula, &context());
    assert!(
        matches!(&node, Node::NamedVariableKind { name, id: None } if name == formula),
        "{node:?}"
    );
    assert_eq!(to_rc_format(&node), formula);
    let node = parser.parse("SUM(A1:A3\u{3000}A2:B2)", &context());
    assert_eq!(to_rc_format(&node), "SUM(A1:A3\u{3000}A2:B2)");
}

#[test]
fn intersection_operands_must_be_references() {
    let mut parser = parser();
    for formula in [
        "A1 (1)",
        "#N/A A1",
        "#N/A #REF!",
        "(1/0) A1",
        "A1 (1/0)",
        "(1/0) (NA())",
        "(1/0) Z99",
        "A1 SUM(A1:A2)",
        "SUM(A1:A2) A1",
        "A1 NA()",
        "A1:A3 ABS(A2)",
        "A1:A3 ROW(A2)",
        "A1:A3 IFERROR(A2:B2,0)",
        "A1:A3 IFNA(A2:B2,0)",
        "A1:A3 CHOOSEROWS(A1:B3,2)",
        "A2:B2 CHOOSECOLS(A1:B3,1)",
        "A1:A3 FILTER(A1:B3,A1:A3=2)",
        "A1:A3 VSTACK(A2:B2)",
        "A1:A3 SORT(A2:B2)",
        "A1:A3 UNIQUE(A2:B2)",
        "A1 (B1+C1)",
        "A1 (-B1)",
        "A1 (B1&\"\")",
        "A1 (B1=1)",
        "A1 (B1%)",
        "A1 (B1*C1)",
        "A1 (B1/C1)",
        "A1 (B1^2)",
        "A1:A3 VLOOKUP(2,A1:B3,1)",
        "A1:A3 MATCH(2,A1:A3,0)",
        "A1:A3 ROWS(A2:B2)",
        "A1:A3 HLOOKUP(2,A2:B3,1)",
        "A1:A3 LOOKUP(2,A1:A3,A1:A3)",
    ] {
        let node = parser.parse(formula, &context());
        assert!(
            matches!(node, Node::ParseErrorKind { .. }),
            "{formula}: {node:?}"
        );
    }
    for formula in [
        "A1 #REF!",
        "#REF! A1",
        "A1:A3 INDEX(A1:B3,2,0)",
        "OFFSET(A2,0,0,1,2) A1:A3",
        "A1:A3 IF(TRUE,A2:B2)",
        "A1:A3 INDIRECT(\"A2:B2\")",
        "A1:A3 CHOOSE(1,A2:B2)",
        "A1:A3 XLOOKUP(2,A1:A3,A1:B3)",
        "A1:A3 LET(x,A2:B2,x)",
        "A1:A3 SWITCH(1,1,A2:B2)",
        "A1:A3 IFS(TRUE,A2:B2)",
        "A1:A3 TAKE(A2:B3,1)",
        "A1:A3 DROP(A1:B2,1)",
        "A1:A3 TRIMRANGE(A2:B2)",
    ] {
        let node = parser.parse(formula, &context());
        assert!(is_intersection(&node), "{formula}: {node:?}");
    }
}
