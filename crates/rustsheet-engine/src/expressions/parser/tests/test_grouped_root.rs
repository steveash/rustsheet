#![allow(clippy::panic, reason = "tests")]

//! A cell formula written wholly in parentheses keeps them: `=(A1-1)` parses to a
//! root `Group` and prints back unchanged (T-1017). Excel evidence: it skips near-zero
//! snapping for `=(Inputs!A2-1)` (probes-numeric/30bc5f8fab02d0a2, docs/excel-semantics/numbers.md
//! "Near-zero snapping in addition and subtraction").

use std::collections::HashMap;

use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::to_rc_format;
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

fn is_grouped(node: &Node) -> bool {
    matches!(node, Node::Group { .. })
}

#[test]
fn whole_formula_in_parentheses_is_a_root_group() {
    let mut parser = parser();
    let grouped = parser.parse("(A1-1)", &context());
    let bare = parser.parse("A1-1", &context());
    assert!(is_grouped(&grouped));
    assert!(!is_grouped(&bare));
    assert_ne!(grouped, bare);
    assert_eq!(to_english_localized_string(&grouped, &context()), "(A1-1)");
    assert_eq!(to_english_localized_string(&bare, &context()), "A1-1");
}

#[test]
fn printing_round_trips_through_rc_format() {
    let mut parser = parser();
    for formula in [
        "(A1-1)",
        "(SUM(A1:A3))",
        "(A1)",
        "(A1:A3)",
        "(-A1)",
        "(A1 B1)",
    ] {
        let node = parser.parse(formula, &context());
        assert!(is_grouped(&node), "{formula}");
        assert_eq!(to_english_localized_string(&node, &context()), formula);
        let rc = to_rc_format(&node);
        parser.set_lexer_mode(LexerMode::R1C1);
        let reparsed = parser.parse(&rc, &context());
        parser.set_lexer_mode(LexerMode::A1);
        assert_eq!(reparsed, node, "{formula}");
    }
}

#[test]
fn nested_outer_parentheses_keep_every_level() {
    // Excel keeps every written level around a whole formula (probes/near-zero-snap-groups B6,
    // docs/excel-semantics/numbers.md), one root `Group` with a level per pair (T-1161).
    let mut parser = parser();
    for formula in ["((A1-1))", "(((A1-1)))"] {
        let node = parser.parse(formula, &context());
        let Node::Group { levels, .. } = &node else {
            panic!("{formula} is not grouped");
        };
        assert_eq!(usize::from(*levels), formula.matches('(').count());
        assert_eq!(to_english_localized_string(&node, &context()), formula);
    }
}

#[test]
fn inner_groups_that_are_only_part_of_the_content_add_no_level() {
    let mut parser = parser();
    for formula in ["((A1)+(B1))", "(1+(B1))", "((A1)+B1)", "((A1-1)*2)"] {
        let node = parser.parse(formula, &context());
        let Node::Group { levels: 1, child } = &node else {
            panic!("{formula} is not grouped");
        };
        assert!(!is_grouped(child), "{formula} counted a second level");
    }
}

#[test]
fn parentheses_that_do_not_span_the_formula_are_not_a_root_group() {
    let mut parser = parser();
    for (formula, printed) in [
        // Every other group is printed back as written (SPEC-0007).
        ("(A1)-(B1)", "(A1)-(B1)"),
        ("(A1-1)*2", "(A1-1)*2"),
        ("(A1)%", "(A1)%"),
        ("-(A1-1)", "-(A1-1)"),
        ("SUM((A1-1))", "SUM((A1-1))"),
        ("(LAMBDA(x,x))(1)", "(LAMBDA(x,x))(1)"),
    ] {
        let node = parser.parse(formula, &context());
        assert!(!is_grouped(&node), "{formula}");
        assert_eq!(to_english_localized_string(&node, &context()), printed);
    }
}

#[test]
fn a_union_keeps_its_own_parentheses_and_is_not_wrapped() {
    let mut parser = parser();
    // Its parentheses are its own one-level `Group`, not a root group around it.
    let node = parser.parse("(A1,B2)", &context());
    assert!(
        matches!(&node, Node::Group { levels: 1, child } if matches!(**child, Node::OpUnionKind { .. }))
    );
    assert_eq!(to_english_localized_string(&node, &context()), "(A1,B2)");
}

#[test]
fn plain_parse_keeps_the_root_group() {
    // Defined-name bodies parse with the same rule as cell formulas (T-1392).
    let mut parser = parser();
    let node = parser.parse("(A1-1)", &context());
    assert!(is_grouped(&node));
}

#[test]
fn spaces_between_nested_parentheses_print_back() {
    // The spaces are written between the two groups, so they do not merge into one node of two
    // levels as `((A1))` does; both print as typed (SPEC-0023).
    let mut parser = parser();
    let node = parser.parse("( (A1) )", &context());
    assert!(matches!(node, Node::Group { levels: 1, .. }));
    assert_eq!(to_english_localized_string(&node, &context()), "( (A1) )");
    assert!(matches!(node.ungrouped(), Node::ReferenceKind { .. }));
}
