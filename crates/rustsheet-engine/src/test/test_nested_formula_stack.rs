//! T-1110: one cell nested as deep as the parser allows parses and evaluates on a 2 MiB thread
//! (the default test thread) even in an unoptimized build, whose frames are several times larger
//! than an optimized build's (directive 5).

use crate::expressions::parser::MAX_FORMULA_DEPTH;
use crate::test::util::new_empty_model;

/// Deepest nesting of calls or parentheses the parser accepts: every operand costs two levels of
/// `MAX_FORMULA_DEPTH`, the innermost `1` included (expressions/parser/depth.rs).
const MAX_NESTING: usize = MAX_FORMULA_DEPTH / 2 - 1;

fn nested(open: &str, close: &str, levels: usize) -> String {
    format!("={}1{}", open.repeat(levels), close.repeat(levels))
}

/// Sets A1 to the deepest nesting of `open`..`close` the parser accepts, evaluates it on a thread
/// with a 2 MiB stack and returns A1's text. An operator in `open` costs a level of its own, so
/// shapes reach different depths.
fn evaluate_deepest_on_2_mib_stack(open: &'static str, close: &'static str) -> Option<String> {
    let handle = std::thread::Builder::new()
        .stack_size(2 << 20)
        .spawn(move || {
            let mut model = new_empty_model();
            let accepted = (1..=MAX_NESTING).rev().find(|&levels| {
                model
                    .set_user_input(0, 1, 1, nested(open, close, levels))
                    .is_ok()
            })?;
            model.evaluate();
            Some((accepted, model._get_text("A1")))
        })
        .ok()?;
    let (accepted, text) = handle.join().ok()??;
    // Nothing shallower than Excel's 64 nested functions is the parser's limit.
    assert!(accepted > 64, "{open}: {accepted}");
    Some(text)
}

#[test]
fn formulas_at_the_nesting_limit_evaluate_on_a_2_mib_stack() {
    for (open, close) in [
        ("ABS(", ")"),
        ("(", ")"),
        ("IF(TRUE,", ")"),
        ("SUM(", ")"),
        ("-(", ")"),
        ("1*(", ")"),
    ] {
        let text = evaluate_deepest_on_2_mib_stack(open, close);
        assert_eq!(text.as_deref(), Some("1"), "{open}");
    }
}

/// T-1169: the reported shape, `=` + N `(` + `0.1+0.2-0.3+B1` + N `)`, on the default test thread
/// (2 MiB) in whatever build runs the tests. Up to the limit it parses, evaluates, prints, and
/// survives the edits that rewrite every formula (row insert, sheet rename); past it, it is
/// refused as too deep. Either way the process does not abort.
#[test]
fn parenthesised_chain_nested_to_the_limit_and_past_it_never_overflows() {
    // The innermost operator chain costs one level more than a lone operand.
    let limit = MAX_NESTING - 1;
    for levels in [limit - 1, limit, limit + 1, MAX_FORMULA_DEPTH, 300] {
        let formula = format!(
            "={}0.1+0.2-0.3+B1{}",
            "(".repeat(levels),
            ")".repeat(levels)
        );
        let mut model = new_empty_model();
        let result = model.set_user_input(0, 1, 1, formula.clone());
        if levels > limit {
            assert!(result.is_err(), "{levels} levels were accepted");
            continue;
        }
        assert!(result.is_ok(), "{levels} levels were refused: {result:?}");
        model.evaluate();
        assert_eq!(model._get_text("A1"), "5.55112E-17", "{levels}");
        assert_eq!(model._get_formula("A1"), formula, "{levels}");
        assert_eq!(model.insert_rows(0, 1, 1), Ok(()), "{levels}");
        assert_eq!(model.rename_sheet("Sheet1", "Renamed"), Ok(()), "{levels}");
        model.evaluate();
        let moved = format!(
            "={}0.1+0.2-0.3+B2{}",
            "(".repeat(levels),
            ")".repeat(levels)
        );
        assert_eq!(model._get_formula("A2"), moved, "{levels}");
        assert_eq!(model._get_text("A2"), "5.55112E-17", "{levels}");
    }
}
