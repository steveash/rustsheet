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
