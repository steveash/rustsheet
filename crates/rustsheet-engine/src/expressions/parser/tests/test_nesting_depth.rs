//! T-0931: formulas nested deeper than `MAX_FORMULA_DEPTH` are a parse error,
//! never a stack overflow; formulas up to the limit parse. T-0128: operator
//! chains as long as Excel's formula length allows parse, evaluate and print.

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::{
    Node, MAX_FORMULA_DEPTH, MAX_VALUE_OPERATORS, TOO_DEEP_MESSAGE, TOO_MANY_OPERATORS_MESSAGE,
};
use crate::expressions::types::CellReferenceRC;

fn parse(formula: &str) -> Node {
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let cell = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    parser.parse(formula, &cell)
}

/// Runs `f` on a thread whose stack is what the limit is sized for: 1 MiB (the
/// WebAssembly default, the smallest common stack) in an optimized build. An
/// unoptimized build spends about ten times more stack per nesting level
/// (measured: ~23 KB vs ~2 KB before T-1110 split the largest frames, ~11 KB since), so it gets
/// 16 MiB.
fn on_small_stack(f: impl FnOnce() + Send + 'static) {
    let stack = if cfg!(debug_assertions) {
        16 << 20
    } else {
        1 << 20
    };
    let handle = std::thread::Builder::new().stack_size(stack).spawn(f);
    assert!(handle.is_ok_and(|h| h.join().is_ok()));
}

fn is_too_deep(node: &Node) -> bool {
    matches!(node, Node::ParseErrorKind { message, .. } if message == TOO_DEEP_MESSAGE)
}

fn has_too_many_operators(node: &Node) -> bool {
    matches!(node, Node::ParseErrorKind { message, .. } if message == TOO_MANY_OPERATORS_MESSAGE)
}

fn nested(open: &str, inner: &str, close: &str, levels: usize) -> String {
    format!("{}{inner}{}", open.repeat(levels), close.repeat(levels))
}

/// Deepest nesting of parentheses or calls that fits: every operand costs two
/// levels, the innermost `1` included.
const MAX_NESTING: usize = MAX_FORMULA_DEPTH / 2 - 1;

#[test]
fn excel_function_nesting_limit_fits() {
    on_small_stack(|| {
        // Excel allows 64 nested function levels.
        let node = parse(&nested("ABS(", "1", ")", 64));
        assert!(matches!(node, Node::FunctionKind { .. }));
    });
}

#[test]
fn nesting_at_the_limit_parses() {
    on_small_stack(|| {
        for formula in [
            nested("(", "1", ")", MAX_NESTING),
            nested("ABS(", "1", ")", MAX_NESTING),
        ] {
            let node = parse(&formula);
            assert!(!matches!(node, Node::ParseErrorKind { .. }), "{formula}");
        }
    });
}

#[test]
fn one_level_past_the_limit_is_too_deep() {
    on_small_stack(|| {
        for formula in [
            nested("(", "1", ")", MAX_NESTING + 1),
            nested("ABS(", "1", ")", MAX_NESTING + 1),
        ] {
            assert!(is_too_deep(&parse(&formula)), "{formula}");
        }
    });
}

#[test]
fn deep_formulas_of_every_shape_are_too_deep() {
    on_small_stack(|| {
        let levels = 10_000;
        for formula in [
            nested("(", "1", ")", levels),
            nested("SUM(", "1", ")", levels),
            nested("{", "1", "}", levels),
            nested("LAMBDA(x,", "x", ")", levels),
            nested("-(", "1", ")", levels),
            nested("", "1", "%", levels),
            nested("(", "A1", ",A1)", levels),
            nested("(A1,", "A1", ")", levels),
            format!("({})", vec!["A1"; levels].join(",")),
            // Unclosed: the error must come from the depth, not the missing `)`.
            "(".repeat(levels),
        ] {
            assert!(is_too_deep(&parse(&formula)), "{}", &formula[..40]);
        }
    });
}

#[test]
fn unary_minus_runs_nest_up_to_the_limit() {
    on_small_stack(|| {
        // Each minus is its own node (`--x` coerces, T-0132), and so is each plus
        // kept in the formula text (T-2723), so a run nests: one under the limit
        // parses, a very long one is a parse error, never a stack overflow. The
        // plus directly before a number folds into it and adds no node.
        let node = parse(&format!("{}1", "-".repeat(MAX_FORMULA_DEPTH - 3)));
        assert!(matches!(node, Node::UnaryKind { .. }));
        assert!(is_too_deep(&parse(&format!("{}1", "-".repeat(10_000)))));
        let node = parse(&format!("{}1", "+".repeat(MAX_FORMULA_DEPTH - 2)));
        assert!(matches!(node, Node::UnaryPlus { .. }));
        assert!(is_too_deep(&parse(&format!("{}1", "+".repeat(10_000)))));
        assert!(matches!(parse("+1"), Node::NumberKind(_)));
    });
}

#[test]
fn unary_minus_run_at_the_limit_evaluates() {
    on_small_stack(|| {
        // The longest run that parses must also evaluate (one iterative pass,
        // `apply_unary_chain`) and print on a small stack.
        let mut model = crate::test::util::new_empty_model();
        let minuses = "-".repeat(MAX_FORMULA_DEPTH - 3);
        model._set("A1", &format!("={minuses}TRUE"));
        model.evaluate();
        assert_eq!(model._get_text("A1"), "-1");
        assert_eq!(model._get_formula("A1"), format!("={minuses}TRUE"));
    });
}

#[test]
fn parser_is_reusable_after_a_too_deep_formula() {
    on_small_stack(|| {
        let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
        let cell = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };
        assert!(is_too_deep(
            &parser.parse(&nested("(", "1", ")", 1000), &cell)
        ));
        let node = parser.parse(&nested("(", "1", ")", MAX_NESTING), &cell);
        assert!(!matches!(node, Node::ParseErrorKind { .. }));
    });
}

/// Deepest nesting of unions that parses: each level is a parenthesis (two levels) holding a
/// union (one), `((A1,A1),A1)`.
const MAX_UNION_NESTING: usize = (MAX_FORMULA_DEPTH - 2) / 3;

#[test]
fn nested_unions_at_the_limit_parse_evaluate_and_print() {
    on_small_stack(|| {
        // T-0134: the union is one more parser and evaluator frame per level.
        let formula = nested("(", "A1", ",A1)", MAX_UNION_NESTING);
        assert!(matches!(
            parse(&formula).ungrouped(),
            Node::OpUnionKind { .. }
        ));
        assert!(is_too_deep(&parse(&nested(
            "(",
            "A1",
            ",A1)",
            MAX_UNION_NESTING + 1
        ))));
        let right = nested("(A1,", "A1", ")", MAX_UNION_NESTING);
        assert!(matches!(
            parse(&right).ungrouped(),
            Node::OpUnionKind { .. }
        ));
        let mut model = crate::test::util::new_empty_model();
        model._set("B1", &format!("={formula}"));
        model._set("B2", &format!("={right} A1"));
        model.evaluate();
        assert_eq!(model._get_text("B1"), "#VALUE!");
        assert_eq!(model._get_text("B2"), "#VALUE!");
        assert_eq!(model._get_formula("B2"), format!("={right} A1"));
    });
}

/// `=1<op>1<op>...` with `terms` operands.
fn chain(op: &str, terms: usize) -> String {
    vec!["1"; terms].join(op)
}

/// Excel's 8,192 character formula limit fits 4,096 terms of `1+1+...`.
const EXCEL_LONGEST_CHAIN: usize = 4096;

#[test]
fn operator_chains_as_long_as_excel_allows_evaluate_and_print() {
    on_small_stack(|| {
        let mut model = crate::test::util::new_empty_model();
        let cases = [
            ("A1", chain("+", EXCEL_LONGEST_CHAIN), "4096"),
            ("A2", chain("*", EXCEL_LONGEST_CHAIN), "1"),
            ("A4", chain("-", EXCEL_LONGEST_CHAIN), "-4094"),
            // `1=1` is TRUE, and TRUE is not the number 1, so every later link is FALSE.
            ("A5", chain("=", EXCEL_LONGEST_CHAIN), "FALSE"),
            (
                "A6",
                format!("LEN({})", chain("&", EXCEL_LONGEST_CHAIN)),
                "4096",
            ),
            // Mixed precedence: each `2*3` is a product under the sum chain.
            ("A7", vec!["2*3"; 1365].join("+"), "8190"),
            // A chain inside the deepest nesting, and nesting inside a chain.
            (
                "A8",
                nested("ABS(", &chain("+", EXCEL_LONGEST_CHAIN - 64), ")", 60),
                "4032",
            ),
            (
                "A9",
                format!(
                    "{}+{}",
                    chain("+", 2048),
                    nested("ABS(", "1", ")", MAX_NESTING - 1)
                ),
                "2049",
            ),
        ];
        for (cell, formula, _) in &cases {
            model._set(cell, &format!("={formula}"));
        }
        model.evaluate();
        for (cell, formula, expected) in &cases {
            assert_eq!(model._get_text(cell), *expected, "{cell}");
            assert_eq!(model._get_formula(cell), format!("={formula}"), "{cell}");
        }
        // A power chain prints as written, `=1^1^1` (probes/power-chain-text, T-1331).
        let power = format!("={}", chain("^", EXCEL_LONGEST_CHAIN));
        model._set("A3", &power);
        model.evaluate();
        assert_eq!(model._get_text("A3"), "1");
        assert_eq!(model._get_formula("A3"), power);
        // A chain of references evaluates, and moves with its cell.
        let references = vec!["A1"; EXCEL_LONGEST_CHAIN].join("+");
        model._set("B1", &format!("={references}"));
        model.evaluate();
        assert_eq!(model._get_text("B1"), "16777216");
        let cell = |row| crate::expressions::types::CellReferenceIndex {
            sheet: 0,
            row,
            column: 2,
        };
        let area = crate::expressions::types::Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 2,
            height: 1,
        };
        let moved =
            model.move_cell_value_to_area(&format!("={references}"), &cell(1), &cell(2), &area);
        assert_eq!(
            moved,
            Ok(format!("={}", vec!["A2"; EXCEL_LONGEST_CHAIN].join("+")))
        );
        let tree = parse(&chain("&", EXCEL_LONGEST_CHAIN));
        assert!(tree.clone() == tree);
    });
}

#[test]
fn operators_up_to_the_cap_parse() {
    on_small_stack(|| {
        // The cap counts every value operator in the formula, not one chain.
        let at_cap = chain("+", MAX_VALUE_OPERATORS + 1);
        assert!(matches!(parse(&at_cap), Node::OpSumKind { .. }));
        let split = format!(
            "SUM({},{})",
            chain("*", MAX_VALUE_OPERATORS / 2 + 1),
            chain("&", MAX_VALUE_OPERATORS / 2 + 1)
        );
        assert!(matches!(parse(&split), Node::FunctionKind { .. }));
        for op in ["+", "-", "*", "/", "^", "&", "=", "<>"] {
            let formula = chain(op, MAX_VALUE_OPERATORS + 2);
            assert!(has_too_many_operators(&parse(&formula)), "{op}");
        }
        let split = format!(
            "SUM({},{})",
            chain("*", MAX_VALUE_OPERATORS / 2 + 1),
            chain("&", MAX_VALUE_OPERATORS / 2 + 2)
        );
        assert!(has_too_many_operators(&parse(&split)));
        // Very long chains are refused, never a stack overflow.
        assert!(has_too_many_operators(&parse(&chain("+", 100_000))));
    });
}

#[test]
fn chain_links_add_no_depth() {
    on_small_stack(|| {
        // The deepest nesting still fits with a long chain at every level: a chain is one
        // level whatever its length.
        let inner = format!("{}+1", nested("(", &chain("+", 30), ")", MAX_NESTING - 2));
        assert!(
            !matches!(parse(&inner), Node::ParseErrorKind { .. }),
            "{inner}"
        );
        let too_deep = format!("{}+1", nested("(", &chain("+", 30), ")", MAX_NESTING));
        assert!(is_too_deep(&parse(&too_deep)));
    });
}
