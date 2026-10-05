//! T-0931: formulas nested deeper than `MAX_FORMULA_DEPTH` are a parse error,
//! never a stack overflow; formulas up to the limit parse.

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::{Node, MAX_FORMULA_DEPTH, TOO_DEEP_MESSAGE};
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
/// (measured: ~23 KB vs ~2 KB), so it gets 16 MiB.
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
        // An operator chain: the first operand is two levels, each operator one.
        let chain = vec!["1"; MAX_FORMULA_DEPTH - 1].join("+");
        assert!(matches!(parse(&chain), Node::OpSumKind { .. }));
    });
}

#[test]
fn one_level_past_the_limit_is_too_deep() {
    on_small_stack(|| {
        for formula in [
            nested("(", "1", ")", MAX_NESTING + 1),
            nested("ABS(", "1", ")", MAX_NESTING + 1),
            vec!["1"; MAX_FORMULA_DEPTH].join("+"),
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
            format!("1{}", "&1".repeat(levels)),
            format!("1{}", "^1".repeat(levels)),
            format!("1{}", "*1".repeat(levels)),
            format!("1{}", "=1".repeat(levels)),
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
        // Each minus is its own node (`--x` coerces, T-0132), so a run nests:
        // one under the limit parses, a very long one is a parse error, never a
        // stack overflow. Plus signs are no-ops and add no nodes.
        let node = parse(&format!("{}1", "-".repeat(MAX_FORMULA_DEPTH - 3)));
        assert!(matches!(node, Node::UnaryKind { .. }));
        assert!(is_too_deep(&parse(&format!("{}1", "-".repeat(10_000)))));
        assert!(matches!(
            parse(&format!("{}1", "+".repeat(10_000))),
            Node::NumberKind(_)
        ));
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
