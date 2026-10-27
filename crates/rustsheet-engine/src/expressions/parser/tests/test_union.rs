#![allow(clippy::panic, reason = "tests")]

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
    matches!(node.ungrouped(), Node::OpUnionKind { .. })
}

fn is_parse_error(node: &Node) -> bool {
    matches!(node, Node::ParseErrorKind { .. })
}

#[test]
fn comma_inside_parentheses_is_a_union() {
    let mut parser = parser();
    let node = parser.parse("(Data!B6,Data!E11)", &context());
    let Node::OpUnionKind { left, right, .. } = node.ungrouped() else {
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
    let Node::OpUnionKind { left, .. } = node.ungrouped() else {
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
fn malformed_unions_are_parse_errors() {
    // A bare comma is a union (`=A1,B2`, T-1073, test/test_bare_union.rs); these are not.
    let mut parser = parser();
    for formula in ["A1,", "(A1,)", "(,A1)", "(A1,B2"] {
        let node = parser.parse(formula, &context());
        assert!(is_parse_error(&node), "{formula}: {node:?}");
    }
}

#[test]
fn unions_nest_and_chain() {
    let mut parser = parser();
    // `((A2,E8),B6)` keeps the group; `(A2,E8,B6)` is the same tree without it.
    let nested = parser.parse("((Data!A2,Data!E8),Data!B6)", &context());
    let Node::OpUnionKind { left, right, .. } = nested.ungrouped() else {
        panic!("expected a union, got {nested:?}");
    };
    assert!(is_union(left));
    assert!(matches!(**right, Node::ReferenceKind { .. }));
    assert_ne!(
        parser.parse("(Data!A2,Data!E8,Data!B6)", &context()),
        nested
    );
    // A right operand that is a union keeps its parentheses.
    let node = parser.parse("(A1,(B2,C3))", &context());
    let Node::OpUnionKind { right, .. } = node.ungrouped() else {
        panic!("expected a union, got {node:?}");
    };
    assert!(is_union(right));
}

#[test]
fn union_binds_looser_than_intersection_and_tighter_than_the_rest() {
    let mut parser = parser();
    // `(A1:B2 B1,C1)` is `((A1:B2 B1),C1)`.
    let node = parser.parse("(A1:B2 B1,C1)", &context());
    let Node::OpUnionKind { left, .. } = node.ungrouped() else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(**left, Node::OpIntersectKind { .. }));
    // A union cut by an intersection: `(C1:C12,E1:E12) C9`.
    let node = parser.parse("(Data!C1:C12,Data!E1:E12) Data!C9", &context());
    let Node::OpIntersectKind { left, .. } = &node else {
        panic!("expected an intersection, got {node:?}");
    };
    assert!(is_union(left));
    for formula in ["(A1,B2)/1", "(A1,B2)&\"\"", "-(A1,B2)", "(A1,B2)%"] {
        let node = parser.parse(formula, &context());
        assert!(!is_parse_error(&node), "{formula}: {node:?}");
    }
}

#[test]
fn union_operands_must_be_references() {
    let mut parser = parser();
    for formula in [
        "(A1+B1,C1)",
        "(-1,A1)",
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
        // The spaces after `(` and the comma and before `)` print back; the one before the comma
        // does not (SPEC-0023).
        ("( A1 , B2 )", "( A1, B2 )"),
        ("(Data!E4,Data!E4)", "(Data!E4,Data!E4)"),
        ("(Data!E12,Data!A9)/1", "(Data!E12,Data!A9)/1"),
        ("(Data!C12,Data!E9)&\"\"", "(Data!C12,Data!E9)&\"\""),
        ("((Data!A2,Data!E8),Data!B6)", "((Data!A2,Data!E8),Data!B6)"),
        ("(Data!A2,Data!E8,Data!B6)", "(Data!A2,Data!E8,Data!B6)"),
        ("(-A1,B1)", "(-A1,B1)"),
        ("(A1,-B1)", "(A1,-B1)"),
        ("((A1,B2),C3,D4)", "((A1,B2),C3,D4)"),
        ("(A1,B2,(C3,D4))", "(A1,B2,(C3,D4))"),
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
    // T-0130: `Data!B8:Data!E12` is one range; T-0188: it also survives the R1C1 storage form.
    let mut parser = parser();
    let node = parser.parse("(Data!B8:Data!E12,A1)", &context());
    let Node::OpUnionKind { left, .. } = node.ungrouped() else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(**left, Node::OpRangeKind { .. }), "{left:?}");
    let text = to_english_localized_string(&node, &context());
    assert_eq!(text, "(Data!B8:Data!E12,A1)");
    assert_eq!(parser.parse(&text, &context()), node);
    assert_r1c1_round_trip(&mut parser, "(Data!B8:Data!E12,A1)");
}

/// T-0188: the R1C1 lexer ends a range token at the first corner when the second corner carries
/// a sheet prefix, as the A1 lexer does, so the stored form re-parses to the same tree.
#[test]
fn second_corner_sheet_prefix_survives_r1c1() {
    let mut parser = parser();
    for formula in [
        "SUM(Data!B8:Data!E12,A1)",
        "(Data!B8:Data!E12,A1)",
        "Data!B8:Data!E12 Data!B12",
    ] {
        assert_r1c1_round_trip(&mut parser, formula);
    }
}

fn assert_r1c1_round_trip(parser: &mut Parser, formula: &str) {
    let node = parser.parse(formula, &context());
    assert!(!is_parse_error(&node), "{formula}: {node:?}");
    let rc = to_rc_format(&node);
    parser.set_lexer_mode(LexerMode::R1C1);
    let reparsed = parser.parse(&rc, &context());
    parser.set_lexer_mode(LexerMode::A1);
    assert_eq!(reparsed, node, "{formula} via {rc}");
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

#[test]
fn negated_reference_is_a_union_operand() {
    // Excel loads and prints `(-A1,B1)` unchanged (probes/union-neg-operand H2); the engine
    // reads it as a union of `-A1` and `B1`, not as `-(A1,B1)`.
    let mut parser = parser();
    let node = parser.parse("(-A1,B1)", &context());
    let Node::OpUnionKind { left, .. } = node.ungrouped() else {
        panic!("expected a union, got {node:?}");
    };
    assert!(matches!(**left, Node::UnaryKind { .. }));
}

#[test]
fn any_number_of_minuses_is_a_union_operand() {
    // Excel loads two, three and four minuses (probes/union-operand-{double,triple,quad}-minus).
    let mut parser = parser();
    for formula in ["(--A1,B1)", "(---A1,B1)", "(----A1,B1)"] {
        assert!(is_union(&parser.parse(formula, &context())), "{formula}");
    }
}

#[test]
fn union_with_a_negated_operand_cannot_be_cut_by_an_intersection() {
    // Excel refuses all of these (`FileCorruptTryRepair`, no oracle value): references.md "Union:
    // parsing and evaluation", operand forms (T-0998, T-1031 probes).
    let mut parser = parser();
    for formula in [
        "(-A1,B1) A1",
        "(-A1,B1) B1",
        "A1 (-A1,B1)",
        "((-A1,B1),C1) A1",
        "(-(A1,B2),C1) A1",
        "(--A1,B1) A1",
        "(A1,-B1) A1",
    ] {
        assert!(
            is_parse_error(&parser.parse(formula, &context())),
            "{formula}"
        );
    }
    // Without the negation the cut is fine.
    assert!(!is_parse_error(&parser.parse("(A1,B1) A1", &context())));
}

#[test]
fn parentheses_around_a_union_keep_every_level() {
    // probes/union-operand-nested-parens: FORMULATEXT keeps each level Excel was given.
    let mut parser = parser();
    for formula in [
        "(A1,B2)",
        "((A1,B2))",
        "((A1,B2),C1)",
        "(((A1,B2),C1))",
        "((((A1,B2))))",
    ] {
        let node = parser.parse(formula, &context());
        assert_eq!(to_english_localized_string(&node, &context()), formula);
    }
}

/// A cut of a parenthesized union keeps the parentheses written around it (T-1176,
/// probes/union-cut-precedence F6); other cuts print as before (the whole-formula levels are the grouped root's).
#[test]
fn parenthesized_cut_of_a_union_keeps_its_parentheses() {
    let mut parser = parser();
    for (formula, expected) in [
        ("((A1,B1) B1)", "((A1,B1) B1)"),
        ("(((A1,B1) B1))", "(((A1,B1) B1))"),
        ("(A1 B1)", "(A1 B1)"),
        ("(A1:B1 B1)", "(A1:B1 B1)"),
        // The wrapper levels the formula, `@` and `:` track themselves are not counted twice.
        ("@((A1,B1) B1)", "@((A1,B1) B1)"),
        ("SUM(((A1,B1) B1):C1)", "SUM(((A1,B1) B1):C1)"),
        ("SUM(((A1,B1) B1))", "SUM(((A1,B1) B1))"),
        // As either operand of another cut it prints its own levels only.
        ("C1 ((A1,B1) B1)", "C1 ((A1,B1) B1)"),
        ("A1:B1 ((A1,B1) B1)", "A1:B1 ((A1,B1) B1)"),
        ("((A1,B1) B1) ((A1,B1) B1)", "((A1,B1) B1) ((A1,B1) B1)"),
        ("((A1,B1) @B1) C1", "((A1,B1) @B1) C1"),
    ] {
        let node = parser.parse(formula, &context());
        let text = to_english_localized_string(&node, &context());
        assert_eq!(text, expected);
        // Printing is a fixed point: a re-parse gains no level.
        let reparsed = parser.parse(&text, &context());
        assert_eq!(to_english_localized_string(&reparsed, &context()), expected);
    }
}

/// The parentheses of a union, and of a cut of a parenthesized union, are `Group` nodes as
/// written (SPEC-0007 step 3): `((A1,B1) B1)` is `Group{1, Intersect(Group{1, Union}, B1)}`,
/// at the root as anywhere else.
#[test]
fn union_and_cut_parentheses_are_groups() {
    let mut parser = parser();
    let node = parser.parse("((A1,B1) B1)", &context());
    let Node::Group { levels: 1, child } = &node else {
        panic!("expected one group around the cut, got {node:?}");
    };
    let Node::OpIntersectKind { left, .. } = &**child else {
        panic!("expected a cut, got {child:?}");
    };
    assert!(
        matches!(&**left, Node::Group { levels: 1, child } if is_union(child)),
        "{left:?}"
    );
    let node = parser.parse("((A1,B2))", &context());
    assert!(
        matches!(&node, Node::Group { levels: 2, child } if is_union(child)),
        "{node:?}"
    );
}

/// SUBTOTAL and AREAS read a parenthesized union (or a cut of one) as a reference, so the call
/// is a scalar, not a dynamic formula, through the union's `Group` (T-1342).
#[test]
fn subtotal_and_areas_of_a_grouped_union_are_scalar() {
    use crate::expressions::parser::static_analysis::{run_static_analysis_on_node, StaticResult};
    let mut parser = parser();
    for formula in [
        "AREAS((A1,B1))",
        "SUBTOTAL(9,(A1,B1))",
        "SUBTOTAL(9,((A1,B1) B1))",
        "AREAS(((A1,B1)))",
    ] {
        let node = parser.parse(formula, &context());
        assert!(
            matches!(run_static_analysis_on_node(&node), StaticResult::Scalar),
            "{formula}"
        );
    }
}
