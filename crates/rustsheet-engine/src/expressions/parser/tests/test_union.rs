#![allow(clippy::panic)]

//! The union operator, a comma directly inside parentheses (`(A1,B2)`), T-0134. Excel evidence:
//! docs/excel-semantics/references.md "Union (comma) reference in a value context is #VALUE!"
//! (gen-m1/c940d3cfb0d3b083).

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::to_rc_format;
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::{Node, Parser, MAX_FORMULA_DEPTH, TOO_DEEP_MESSAGE};
use crate::expressions::types::CellReferenceRC;
use crate::functions::Function;

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

fn is_union(node: &Node) -> bool {
    matches!(node, Node::OpUnionKind { .. })
}

fn is_parse_error(node: &Node) -> bool {
    matches!(node, Node::ParseErrorKind { .. })
}

#[test]
fn comma_inside_parentheses_is_a_union() {
    let mut parser = parser();
    let node = parser.parse("(Data!B6,Data!E11)", &context());
    let Node::OpUnionKind { left, right } = &node else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(**left, Node::ReferenceKind { .. }));
    assert!(matches!(**right, Node::ReferenceKind { .. }));
    // Spaces around the comma are not intersections.
    assert!(is_union(&parser.parse("( A1 , B2 )", &context())));
}

#[test]
fn comma_in_a_call_still_separates_arguments() {
    let mut parser = parser();
    let node = parser.parse("SUM(A1,B2)", &context());
    let Node::FunctionKind { kind, args } = &node else {
        panic!("expected a call, got {node:?}");
    };
    assert_eq!(*kind, Function::Sum);
    assert_eq!(args.len(), 2);
    // A parenthesis inside the call holds one union argument.
    let node = parser.parse("SUM((A1,B2),C3)", &context());
    let Node::FunctionKind { args, .. } = &node else {
        panic!("expected a call, got {node:?}");
    };
    assert_eq!(args.len(), 2);
    assert!(is_union(&args[0]), "{:?}", args[0]);
    // A call inside the parenthesis takes its own arguments back.
    let node = parser.parse("(INDEX(A1:B2,1,1),C3)", &context());
    let Node::OpUnionKind { left, .. } = &node else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(&**left, Node::FunctionKind { args, .. } if args.len() == 3));
    // Array constants keep their column separator.
    assert!(matches!(
        parser.parse("SUM({1,2})", &context()),
        Node::FunctionKind { .. }
    ));
}

#[test]
fn comma_outside_parentheses_is_not_a_union() {
    let mut parser = parser();
    for formula in [
        "A1,B2",
        "A1 B1,C1",
        "(A1,B2),C3",
        "(A1,)",
        "(,A1)",
        "(A1,B2",
    ] {
        let node = parser.parse(formula, &context());
        assert!(is_parse_error(&node), "{formula}: {node:?}");
    }
}

#[test]
fn unions_nest_and_chain() {
    let mut parser = parser();
    // `((A2,E8),B6)` and `(A2,E8,B6)` are the same left-associative tree.
    let nested = parser.parse("((Data!A2,Data!E8),Data!B6)", &context());
    let Node::OpUnionKind { left, right } = &nested else {
        panic!("expected a union, got {nested:?}");
    };
    assert!(is_union(left));
    assert!(matches!(**right, Node::ReferenceKind { .. }));
    assert_eq!(
        parser.parse("(Data!A2,Data!E8,Data!B6)", &context()),
        nested
    );
    // A right operand that is a union keeps its parentheses.
    let node = parser.parse("(A1,(B2,C3))", &context());
    let Node::OpUnionKind { right, .. } = &node else {
        panic!("expected a union, got {node:?}");
    };
    assert!(is_union(right));
}

#[test]
fn union_binds_looser_than_intersection_and_tighter_than_the_rest() {
    let mut parser = parser();
    // `(A1:B2 B1,C1)` is `((A1:B2 B1),C1)`.
    let node = parser.parse("(A1:B2 B1,C1)", &context());
    let Node::OpUnionKind { left, .. } = &node else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(**left, Node::OpIntersectKind { .. }));
    // A union cut by an intersection: `(C1:C12,E1:E12) C9`.
    let node = parser.parse("(Data!C1:C12,Data!E1:E12) Data!C9", &context());
    let Node::OpIntersectKind { left, .. } = &node else {
        panic!("expected an intersection, got {node:?}");
    };
    assert!(is_union(left));
    // `(A1+B1,C1)` is `A1+(B1,C1)` (UNVERIFIED precedence, references.md).
    let node = parser.parse("(A1+B1,C1)", &context());
    let Node::OpSumKind { right, .. } = &node else {
        panic!("expected a sum, got {node:?}");
    };
    assert!(is_union(right));
    for formula in ["(A1,B2)/1", "(A1,B2)&\"\"", "-(A1,B2)", "(A1,B2)%"] {
        let node = parser.parse(formula, &context());
        assert!(!is_parse_error(&node), "{formula}: {node:?}");
    }
}

#[test]
fn union_operands_must_be_references() {
    let mut parser = parser();
    for formula in [
        "(1,A1)",
        "(A1,1)",
        "(A1,\"a\")",
        "(A1,TRUE)",
        "(A1,#N/A)",
        "(A1,SUM(B1))",
        "(A1,(B1+1))",
        "(A1,{1,2})",
    ] {
        let node = parser.parse(formula, &context());
        assert!(is_parse_error(&node), "{formula}: {node:?}");
    }
    for formula in [
        "(A1,#REF!)",
        "(A1,INDEX(B1:B3,2))",
        "(A1,OFFSET(B1,1,0))",
        "(A1,@B1:B3)",
        "(A1:A3 A2,B2)",
    ] {
        let node = parser.parse(formula, &context());
        assert!(is_union(&node), "{formula}: {node:?}");
    }
}

#[test]
fn unions_round_trip_through_stringify() {
    let mut parser = parser();
    let cases = [
        ("(A1,B2)", "(A1,B2)"),
        ("( A1 , B2 )", "(A1,B2)"),
        ("(Data!E4,Data!E4)", "(Data!E4,Data!E4)"),
        ("(Data!E12,Data!A9)/1", "(Data!E12,Data!A9)/1"),
        ("(Data!C12,Data!E9)&\"\"", "(Data!C12,Data!E9)&\"\""),
        ("((Data!A2,Data!E8),Data!B6)", "(Data!A2,Data!E8,Data!B6)"),
        ("(A1,(B2,C3))", "(A1,(B2,C3))"),
        (
            "(Data!C1:C12,Data!E1:E12) Data!C9",
            "(Data!C1:C12,Data!E1:E12) Data!C9",
        ),
        ("A1:A3 (A2:B2,A2)", "A1:A3 (A2:B2,A2)"),
        ("SUM((A1,B2))", "SUM((A1,B2))"),
        ("SUM((A1,B2),C3)", "SUM((A1,B2),C3)"),
        ("-(A1,B2)", "-(A1,B2)"),
        ("(A1,B2)^2", "(A1,B2)^2"),
        ("(A1+B1,C1)", "A1+(B1,C1)"),
        ("(A1:B2 B1,C1)", "(A1:B2 B1,C1)"),
        ("('Q''1'!A1,Data!B1:B2)", "('Q''1'!A1,Data!B1:B2)"),
    ];
    for (formula, expected) in cases {
        let node = parser.parse(formula, &context());
        let text = to_english_localized_string(&node, &context());
        assert_eq!(text, expected, "{formula}: {node:?}");
        assert_eq!(parser.parse(&text, &context()), node, "{formula}");
        // Formulas are stored in R1C1.
        let rc = to_rc_format(&node);
        parser.set_lexer_mode(LexerMode::R1C1);
        let reparsed = parser.parse(&rc, &context());
        parser.set_lexer_mode(LexerMode::A1);
        assert_eq!(reparsed, node, "{formula} via {rc}");
    }
}

#[test]
fn sheet_qualified_corner_pairs_are_union_operands() {
    // T-0130: `Data!B8:Data!E12` is one range. Only A1 is checked: the R1C1 lexer does not
    // read a second sheet prefix yet (T-0184, which affects any such range).
    let mut parser = parser();
    let node = parser.parse("(Data!B8:Data!E12,A1)", &context());
    let Node::OpUnionKind { left, .. } = &node else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(**left, Node::OpRangeKind { .. }), "{left:?}");
    let text = to_english_localized_string(&node, &context());
    assert_eq!(text, "(Data!B8:Data!E12,A1)");
    assert_eq!(parser.parse(&text, &context()), node);
}

#[test]
fn long_unions_hit_the_depth_cap() {
    let mut parser = parser();
    let union = |terms: usize| format!("({})", vec!["A1"; terms].join(","));
    let node = parser.parse(&union(MAX_FORMULA_DEPTH / 2), &context());
    assert!(is_union(&node), "{node:?}");
    let node = parser.parse(&union(MAX_FORMULA_DEPTH * 4), &context());
    let Node::ParseErrorKind { message, .. } = node else {
        panic!("expected the depth error, got {node:?}");
    };
    assert_eq!(message, TOO_DEEP_MESSAGE);
    // Nested unions: test_nesting_depth.rs (they need a sized stack).
}
