//! T-1110: one cell nested as deep as the parser allows parses and evaluates on a 2 MiB thread
//! (the default test thread) even in an unoptimized build, whose frames are several times larger
//! than an optimized build's (directive 5).

use crate::expressions::parser::MAX_FORMULA_DEPTH;
use crate::test::util::new_empty_model;

/// Deepest nesting of calls or parentheses the parser accepts: every operand costs two levels of
/// `MAX_FORMULA_DEPTH`, the innermost `1` included (expressions/parser/depth.rs).
const MAX_NESTING: usize = MAX_FORMULA_DEPTH / 2 - 1;

fn nested(open: &str, inner: &str, close: &str, levels: usize) -> String {
    format!("={}{inner}{}", open.repeat(levels), close.repeat(levels))
}

/// Sets A1 to the deepest nesting of `open`..`close` the parser accepts, evaluates it on a thread
/// with a 2 MiB stack and returns A1's text and the levels accepted. An operator in `open` costs a
/// level of its own, so shapes reach different depths.
fn deepest_on_2_mib_stack(open: &'static str, close: &'static str) -> Option<(usize, String)> {
    let (accepted, mut texts) = deepest_with_inner_on_2_mib_stack(open, "1", close)?;
    Some((accepted, texts.swap_remove(0)))
}

/// [`deepest_on_2_mib_stack`] around `inner` instead of `1`; returns the texts of A1 and A2.
fn deepest_with_inner_on_2_mib_stack(
    open: &'static str,
    inner: &'static str,
    close: &'static str,
) -> Option<(usize, Vec<String>)> {
    let handle = std::thread::Builder::new()
        .stack_size(2 << 20)
        .spawn(move || {
            let mut model = new_empty_model();
            let accepted = (1..=MAX_NESTING).rev().find(|&levels| {
                model
                    .set_user_input(0, 1, 1, nested(open, inner, close, levels))
                    .is_ok()
            })?;
            model.evaluate();
            Some((accepted, vec![model._get_text("A1"), model._get_text("A2")]))
        })
        .ok()?;
    handle.join().ok()?
}

/// [`deepest_on_2_mib_stack`]'s text, for a shape that nests one call or group per level.
fn evaluate_deepest_on_2_mib_stack(open: &'static str, close: &'static str) -> Option<String> {
    let (accepted, text) = deepest_on_2_mib_stack(open, close)?;
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

/// SPEC-0011 (T-1770): a lifted call through the driver at every level,
/// `EDATE(EDATE(…{45000;45031}…,0),0)`, evaluates at the parser's limit on a 2 MiB stack and
/// spills both elements.
#[test]
fn nested_lifted_edate_at_the_nesting_limit_evaluates_on_a_2_mib_stack() {
    let deepest = deepest_with_inner_on_2_mib_stack("EDATE(", "{45000;45031}", ",0)");
    let (accepted, texts) = deepest.unwrap_or_default();
    assert!(accepted > 64, "EDATE: {accepted}");
    assert_eq!(texts, vec!["45000".to_string(), "45031".to_string()]);
}

/// SPEC-0007: a call and a written group alternating, `SUM((SUM((…))))`, so every level is a
/// `Group` node under a call, evaluates at the parser's limit on a 2 MiB stack. Each level is two
/// operands, so it nests half as deep as a one-operand shape.
#[test]
fn alternating_calls_and_groups_at_the_nesting_limit_evaluate_on_a_2_mib_stack() {
    let deepest = deepest_on_2_mib_stack("SUM((", "))");
    assert_eq!(deepest, Some((MAX_NESTING / 2, "1".to_string())));
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
        let nested_over = |cell: &str| {
            format!(
                "={}0.1+0.2-0.3+{cell}{}",
                "(".repeat(levels),
                ")".repeat(levels)
            )
        };
        let formula = nested_over("B1");
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
        assert_eq!(model._get_formula("A2"), nested_over("B2"), "{levels}");
        assert_eq!(model._get_text("A2"), "5.55112E-17", "{levels}");
    }
}
