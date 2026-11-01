//! T-1110: one cell nested as deep as the parser allows parses and evaluates on a 2 MiB thread
//! (the default test thread) even in an unoptimized build, whose frames are several times larger
//! than an optimized build's (directive 5).

use crate::expressions::parser::{MAX_FORMULA_DEPTH, MAX_PARENTHESES};
use crate::test::util::new_empty_model;

/// Deepest nesting of calls or parentheses the parser accepts: every operand costs two levels of
/// `MAX_FORMULA_DEPTH`, the innermost `1` included (expressions/parser/depth.rs).
const MAX_OPERANDS: usize = MAX_FORMULA_DEPTH / 2 - 1;

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
            let accepted = (1..=MAX_OPERANDS).rev().find(|&levels| {
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
        ("1*(", ")"),
    ] {
        let text = evaluate_deepest_on_2_mib_stack(open, close);
        assert_eq!(text.as_deref(), Some("1"), "{open}");
    }
    // A sign costs no level (T-3193), and a group holding only a signed group none (T-3338),
    // so `-(` nests past the depth bound, to the parenthesis cap (below).
    let (accepted, text) = deepest_on_2_mib_stack("-(", ")").unwrap_or_default();
    assert_eq!(accepted, MAX_OPERANDS);
    let negated = if accepted % 2 == 1 { "-1" } else { "1" };
    assert_eq!(text, negated);
}

/// T-3338: `-(` nests to Excel's 256-parenthesis cap (probes/t3266-nested-paren-256), and so do
/// its spaced and plus forms, and 200 levels around Excel's 824 more minus signs
/// (probes/t3266-nested-200-824): each parses, evaluates, prints as written, and survives a row
/// insert and a sheet rename (directive 5). The bare forms take a 2 MiB stack even unoptimized.
/// The others hold more nodes per level, a [`Node::Spaced`] around every sign and group or a
/// long run of signs, as many as the sign cap's own tests build (`MAX_SIGNS`), and the walks that
/// recurse per node (cloning the tree when it is stored) take what those take: 1 MiB optimized,
/// 16 MiB unoptimized (`on_small_stack` in `expressions/parser/tests/test_nesting_depth.rs`).
///
/// [`Node::Spaced`]: crate::expressions::parser::Node::Spaced
#[test]
fn signed_parentheses_at_the_parenthesis_cap_evaluate_on_a_small_stack() {
    let limit = MAX_PARENTHESES;
    let small = if cfg!(debug_assertions) {
        16 << 20
    } else {
        1 << 20
    };
    let minuses = "-".repeat(824);
    for (open, inner, close, levels, value, stack) in [
        ("-(", "B1", ")", limit, "2", 2 << 20),
        ("-(", "B1", ")", limit - 1, "-2", 2 << 20),
        ("+(", "B1", ")", limit, "2", 2 << 20),
        ("- ( ", "B1", " ) ", limit, "2", small),
        ("-(", "1", ")", 200, "1", small),
        ("-(", "-B1%", ")", limit, "-0.02", small),
    ] {
        let inner = if levels == 200 {
            format!("{minuses}{inner}")
        } else {
            inner.to_string()
        };
        let handle = std::thread::Builder::new()
            .stack_size(stack)
            .spawn(move || {
                let mut model = new_empty_model();
                let formula = nested(open, &inner, close, levels);
                model.set_user_input(0, 1, 2, "2".to_string()).ok()?;
                model.set_user_input(0, 1, 1, formula.clone()).ok()?;
                model.evaluate();
                let before = (model._get_formula("A1"), model._get_text("A1"));
                model.insert_rows(0, 1, 1).ok()?;
                model.rename_sheet("Sheet1", "Renamed").ok()?;
                model.evaluate();
                let moved = nested(open, &inner.replace("B1", "B2"), close, levels);
                let after = (model._get_formula("A2"), model._get_text("A2"));
                Some((formula, before, moved, after))
            });
        let result = handle.ok().and_then(|h| h.join().ok()).flatten();
        let (formula, (text, before), moved, (after, value_after)) = result.unwrap_or_default();
        assert_eq!(text, formula.trim_end(), "{open} {levels}");
        assert_eq!(before, value, "{open} {levels}");
        assert_eq!(after, moved.trim_end(), "{open} {levels}");
        assert_eq!(value_after, value, "{open} {levels}");
    }
    let mut model = new_empty_model();
    let past = nested("-(", "1", ")", limit + 1);
    assert!(model.set_user_input(0, 1, 1, past).is_err());
}

/// SPEC-0023: whitespace costs no nesting level, so a shape typed with spaces at every level, a
/// `Node::Spaced` around each operand and each sign's operand, nests exactly as deep as without
/// them (`=SUM( - SUM( - …` past Excel's 64 nested functions), and its formula text prints, survives
/// a row insert and a sheet rename and evaluates on a 2 MiB stack (decision 0127).
#[test]
fn spaced_shapes_nest_as_deep_as_bare_ones_on_a_2_mib_stack() {
    for (open, close, bare_open, bare_close) in [
        ("( ", " )", "(", ")"),
        ("SUM( ", " )", "SUM(", ")"),
        ("SUM( - ", " )", "SUM(-", ")"),
        ("- ( ", " ) ", "-(", ")"),
        ("IF( TRUE, ", " )", "IF(TRUE,", ")"),
    ] {
        let (spaced, _) = deepest_on_2_mib_stack(open, close).unwrap_or_default();
        let (bare, _) = deepest_on_2_mib_stack(bare_open, bare_close).unwrap_or_default();
        assert_eq!(spaced, bare, "{open}");
        assert!(spaced > 64, "{open}: {spaced}");
        let handle = std::thread::Builder::new()
            .stack_size(2 << 20)
            .spawn(move || {
                let mut model = new_empty_model();
                let formula = nested(open, "B1", close, spaced);
                model.set_user_input(0, 1, 1, formula.clone()).ok()?;
                model.evaluate();
                let before = (model._get_formula("A1"), model._get_text("A1"));
                model.insert_rows(0, 1, 1).ok()?;
                model.rename_sheet("Sheet1", "Renamed").ok()?;
                model.evaluate();
                let moved = nested(open, "B2", close, spaced);
                Some((formula, before, moved, model._get_formula("A2")))
            });
        let result = handle.ok().and_then(|h| h.join().ok()).flatten();
        let (formula, (text, value), moved, after) = result.unwrap_or_default();
        assert_eq!(text, formula.trim_end(), "{open}");
        assert_eq!(value, "0", "{open}");
        assert_eq!(after, moved.trim_end(), "{open}");
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
/// operands, a group (two levels of nesting) and the call at its head (one and a half, T-3197), so
/// it nests 72 levels, past Excel's 65 nested calls (probes/t3097-nesting-paren-abs-65); the
/// outermost operand and the innermost `1` take the rest. Measured in an unoptimized build: 72
/// levels need about 1.9 MiB, as 127 levels of `ABS(` do.
#[test]
fn alternating_calls_and_groups_at_the_nesting_limit_evaluate_on_a_2_mib_stack() {
    let levels = (2 * MAX_FORMULA_DEPTH - 4) / 7;
    assert_eq!(levels, 72);
    for (open, close) in [("SUM((", "))"), ("(ABS(", "))"), ("( SUM( ", " ) )")] {
        let deepest = deepest_on_2_mib_stack(open, close);
        assert_eq!(deepest, Some((levels, "1".to_string())), "{open}");
    }
}

/// T-1169: the reported shape, `=` + N `(` + `0.1+0.2-0.3+B1` + N `)`, on the default test thread
/// (2 MiB) in whatever build runs the tests. Up to the limit it parses, evaluates, prints, and
/// survives the edits that rewrite every formula (row insert, sheet rename); past it, it is
/// refused as too deep. Either way the process does not abort. The parentheses are one run, one
/// group node, so the limit is Excel's 256 (T-3191).
#[test]
fn parenthesised_chain_nested_to_the_limit_and_past_it_never_overflows() {
    let limit = MAX_PARENTHESES;
    for levels in [MAX_OPERANDS, limit - 1, limit, limit + 1, 300] {
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
