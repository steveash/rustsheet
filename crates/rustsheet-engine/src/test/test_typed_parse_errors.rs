//! T-3213: Excel refuses a typed formula that does not parse (Graph PATCH 400), and the cell keeps
//! its content; it closes a call's missing `)` but not a group's (probes/t3213-typed-parse-errors).
//! Stored text keeps today's parse-error cell.

use crate::test::util::new_empty_model;
use crate::typed_input::open_parenthesis_is_a_call;
use crate::TYPED_FORMULA_DOES_NOT_PARSE;

#[test]
fn typed_formulas_that_do_not_parse_are_refused() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    for formula in [
        "=1+",
        "=(1",
        "=1)",
        "=+",
        "=1 2",
        "=\"a",
        "=NoSuchTable[Col]",
    ] {
        let result = model.set_typed_user_input(0, 1, 1, formula.to_string());
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{formula}: {message}"
        );
    }
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
}

#[test]
fn typed_calls_missing_their_parenthesis_are_closed() {
    let mut model = new_empty_model();
    for (row, formula) in [(1, "=SUM(1"), (2, "=SUM(1,,"), (3, "=LOG10(100")] {
        let result = model.set_typed_user_input(0, row, 1, formula.to_string());
        assert_eq!(result, Ok(()), "{formula}");
    }
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=SUM(1)");
    assert_eq!(model._get_formula("A2"), "=SUM(1,,)");
    assert_eq!(model._get_text("A2"), "1");
    assert_eq!(model._get_text("A3"), "2");
}

#[test]
fn stored_formulas_that_do_not_parse_keep_their_text() {
    let mut model = new_empty_model();
    model._set("A1", "=1+");
    model._set("A2", "=(1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "1");
}

#[test]
fn only_a_calls_open_parenthesis_is_closed() {
    for (formula, is_call) in [
        ("SUM(1", true),
        ("LOG10(2", true),
        ("SUM(1,,", true),
        ("\"(\"&SUM(1", true),
        ("'a(b'!A1+SUM(1", true),
        ("T[(x]+SUM(1", true),
        ("(1", false),
        ("1+(2", false),
        // The open `(` is the group's; the call's is closed (unprobed).
        ("SUM(1+(2", false),
        ("SUM((1", false),
        ("SUM(1)", false),
        ("\"SUM(\"", false),
    ] {
        assert_eq!(open_parenthesis_is_a_call(formula), is_call, "{formula}");
    }
}
