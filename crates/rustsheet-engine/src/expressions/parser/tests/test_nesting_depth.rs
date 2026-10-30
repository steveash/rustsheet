//! T-0931: formulas nested deeper than `MAX_FORMULA_DEPTH` are a parse error,
//! never a stack overflow; formulas up to the limit parse. T-0128: operator
//! chains as long as Excel's formula length allows parse, evaluate and print.

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::{
    Node, MAX_FORMULA_DEPTH, MAX_PARENTHESES, MAX_SIGNS, MAX_VALUE_OPERATORS, TOO_DEEP_MESSAGE,
    TOO_MANY_OPERATORS_MESSAGE,
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
            nested("ABS(", "1", ")", MAX_NESTING + 1),
            nested("( ", "1", ")", MAX_NESTING + 1),
        ] {
            assert!(is_too_deep(&parse(&formula)), "{formula}");
        }
    });
}

/// T-3191: a run of directly nested parentheses is parsed in a loop and merges into one group,
/// so Excel's 256 fit (probes/t3172-paren-256) and 257 are refused, as Excel refuses them.
#[test]
fn parenthesis_runs_nest_as_deep_as_excel_allows() {
    on_small_stack(|| {
        let node = parse(&nested("(", "1", ")", MAX_PARENTHESES));
        assert!(
            matches!(node, Node::Group { levels, .. } if usize::from(levels) == MAX_PARENTHESES)
        );
        assert!(is_too_deep(&parse(&nested(
            "(",
            "1",
            ")",
            MAX_PARENTHESES + 1
        ))));
        assert!(is_too_deep(&parse(&"(".repeat(MAX_PARENTHESES + 1))));
        // Groups in several runs count together (unprobed; syntax.md): split by a space, an
        // operand or a call, 256 in all fit and 257 do not.
        let half = MAX_PARENTHESES / 2;
        for (open, close) in [("( ", ")"), ("(1+", ")"), ("ABS(", ")")] {
            let fits = format!(
                "{open}{}{close}",
                nested("(", "1", ")", MAX_PARENTHESES - 1)
            );
            assert!(
                !matches!(parse(&fits), Node::ParseErrorKind { .. }),
                "{open}"
            );
            let past = format!("{open}{}{close}", nested("(", "1", ")", MAX_PARENTHESES));
            let past = if open == "ABS(" {
                format!("({past})")
            } else {
                past
            };
            assert!(is_too_deep(&parse(&past)), "{open}");
        }
        let split = nested(
            "(",
            &format!("1+{}", nested("(", "1", ")", half)),
            ")",
            half,
        );
        assert!(!matches!(parse(&split), Node::ParseErrorKind { .. }));
        let split = nested(
            "(",
            &format!("1+{}", nested("(", "1", ")", half + 1)),
            ")",
            half,
        );
        assert!(is_too_deep(&parse(&split)));
        let mut model = crate::test::util::new_empty_model();
        let formula = format!("={}", nested("(", "-A2", ")", MAX_PARENTHESES));
        model._set("A1", &formula);
        model._set("A2", "3");
        model.evaluate();
        assert_eq!(model._get_text("A1"), "-3");
        assert_eq!(model._get_formula("A1"), formula);
        // A called group at the end of a run, and inside one (probes/paren-call-*).
        model._set("B1", "=(((LAMBDA(x,x+1))))(2)");
        model._set("B2", "=((LAMBDA(x,x*2))(3)+1)");
        model.evaluate();
        assert_eq!(model._get_text("B1"), "3");
        assert_eq!(model._get_text("B2"), "7");
    });
}

/// T-3197: a call at the head of a group reserves one and a half levels of nesting and adds one
/// level to the tree, so Excel's 65 calls each wrapped in parentheses parse
/// (probes/t3097-nesting-paren-abs-65), up to 72 levels; one more is too deep.
#[test]
fn calls_wrapped_in_parentheses_nest_as_deep_as_excel_allows() {
    on_small_stack(|| {
        let levels = (2 * MAX_FORMULA_DEPTH - 4) / 7;
        for (open, close) in [("(ABS(", "))"), ("SUM((", "))"), ("( ABS( ", " ) )")] {
            for fits in [65, levels] {
                let formula = nested(open, "1", close, fits);
                assert!(
                    !matches!(parse(&formula), Node::ParseErrorKind { .. }),
                    "{open} {fits}"
                );
            }
            let past = nested(open, "1", close, levels + 1);
            assert!(is_too_deep(&parse(&past)), "{open}");
        }
        // A group at the head of a group still costs an operand's two levels.
        assert!(is_too_deep(&parse(&nested(
            "( ",
            "1",
            ")",
            MAX_NESTING + 1
        ))));
    });
}

/// T-3191: a run whose groups hold more than the group inside them (`(((1)+1)*2)`) still costs
/// an operand per parenthesis, but no parser stack: it is bounded by the depth, not the stack.
#[test]
fn parenthesis_runs_around_operators_are_bounded_by_depth() {
    on_small_stack(|| {
        // Each level is a group (two levels) over a sum (one).
        let levels = (MAX_FORMULA_DEPTH - 2) / 3;
        let deepest = nested("(", "1", "+1)", levels);
        assert!(!matches!(parse(&deepest), Node::ParseErrorKind { .. }));
        assert!(is_too_deep(&parse(&nested("(", "1", "+1)", levels + 1))));
        let mut model = crate::test::util::new_empty_model();
        model._set("A1", &format!("={deepest}"));
        model.evaluate();
        assert_eq!(model._get_text("A1"), (levels + 1).to_string());
        assert_eq!(model._get_formula("A1"), format!("={deepest}"));
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
        // Each minus is its own node (`--x` coerces, T-0132), and so is each plus kept in the
        // formula text (T-2723), but a sign adds no depth: a run is bounded by `MAX_SIGNS`,
        // Excel's cap, and a very long one is a parse error, never a stack overflow (T-3193). A
        // minus directly before a number is not counted (Excel takes 1,024 before `1`); the
        // plus directly before a number folds into it and adds no node.
        let node = parse(&format!("{}1", "-".repeat(MAX_SIGNS + 1)));
        assert!(matches!(node, Node::UnaryKind { .. }));
        assert!(is_too_deep(&parse(&format!(
            "{}1",
            "-".repeat(MAX_SIGNS + 2)
        ))));
        assert!(is_too_deep(&parse(&format!("{}1", "-".repeat(10_000)))));
        let node = parse(&format!("{}\"a\"", "+".repeat(MAX_SIGNS)));
        assert!(matches!(node, Node::UnaryPlus { .. }));
        assert!(is_too_deep(&parse(&format!(
            "{}\"a\"",
            "+".repeat(MAX_SIGNS + 1)
        ))));
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
        let minuses = "-".repeat(MAX_SIGNS + 1);
        model._set("A1", &format!("={minuses}1"));
        model._set("A2", &format!("={}TRUE", &minuses[1..]));
        model.evaluate();
        assert_eq!(model._get_text("A1"), "1");
        assert_eq!(model._get_text("A2"), "-1");
        assert_eq!(model._get_formula("A1"), format!("={minuses}1"));
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
        let inner = format!(
            "{}+1",
            nested("ABS(", &chain("+", 30), ")", MAX_NESTING - 2)
        );
        assert!(
            !matches!(parse(&inner), Node::ParseErrorKind { .. }),
            "{inner}"
        );
        let too_deep = format!("{}+1", nested("ABS(", &chain("+", 30), ")", MAX_NESTING));
        assert!(is_too_deep(&parse(&too_deep)));
    });
}

/// `count` unary signs, `-` and `+` alternating from a minus, each followed by `space`.
fn signs(count: usize, space: &str) -> String {
    (0..count)
        .map(|i| if i % 2 == 0 { "-" } else { "+" })
        .collect::<Vec<_>>()
        .join(space)
        + space
}

#[test]
fn sign_runs_at_the_cap_survive_every_walk_under_the_deepest_nesting() {
    on_small_stack(|| {
        // The deepest operand nesting the depth bound admits, a group and a sum per level, with
        // a run of signs at the bottom: the deepest tree a formula can build (T-3193).
        let nested_over = |levels: usize, run: &str, cell: &str| {
            format!("{}{run}{cell}{}", "(1+".repeat(levels), ")".repeat(levels))
        };
        for space in ["", " ", "\n "] {
            let run = signs(MAX_SIGNS, space);
            let levels = (1..MAX_FORMULA_DEPTH)
                .rev()
                .find(|&levels| !is_too_deep(&parse(&nested_over(levels, &run, "C1"))))
                .unwrap_or_default();
            assert!(levels > 60, "{levels}");
            let mut model = crate::test::util::new_empty_model();
            model._set("C1", "2");
            model._set("A1", &format!("={}", nested_over(levels, &run, "C1")));
            model._set("A2", &format!("={run}C1"));
            model._set("A3", &format!("=-({}C1)", signs(MAX_SIGNS - 1, space)));
            model.evaluate();
            for cell in ["A1", "A2", "A3"] {
                assert!(!model._get_text(cell).starts_with('#'), "{cell}");
            }
            assert_eq!(model._get_formula("A2"), format!("={run}C1"));
            assert_eq!(model.insert_rows(0, 1, 1), Ok(()));
            assert_eq!(model.rename_sheet("Sheet1", "Renamed"), Ok(()));
            model.evaluate();
            assert_eq!(
                model._get_formula("A2"),
                format!("={}", nested_over(levels, &run, "C2"))
            );
            assert_eq!(model._get_formula("A3"), format!("={run}C2"));
            assert!(is_too_deep(&parse(&format!(
                "{}C1",
                signs(MAX_SIGNS + 1, space)
            ))));
            assert!(is_too_deep(&parse(&format!(
                "-({}C1)",
                signs(MAX_SIGNS, space)
            ))));
        }
    });
}
