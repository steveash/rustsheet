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
        // Two open: Excel refuses them (probes/t3283-typed-missing-paren).
        ("SUM(1+(2", false),
        ("SUM((1", false),
        ("SUM(SUM(1", false),
        ("ABS(-(1", false),
        ("SUM(1,(2", false),
        ("LAMBDA(x,x)(1", false),
        ("SUM(1)", false),
        ("\"SUM(\"", false),
    ] {
        assert_eq!(open_parenthesis_is_a_call(formula), is_call, "{formula}");
    }
}

/// T-3283 (probes/t3283-typed-double-star, t3213-typed-parse-errors A5): typed, `**` in a number is
/// its exponent marker `E`, and `**` anywhere an `E` would not be is refused. Stored text is unprobed
/// and reads `**` as before.
#[test]
fn typed_double_star_in_a_number_is_its_exponent() {
    let mut model = new_empty_model();
    model._set("Z1", "2");
    for (row, typed, formula, value) in [
        (1, "=1**2", "=100", "100"),
        (2, "=2**3", "=2000", "2000"),
        (3, "=1.5**2", "=150", "150"),
        (4, "=2**-1", "=0.2", "0.2"),
        (5, "=-2**2", "=-200", "-200"),
        (6, "=2**3*2", "=2000*2", "4000"),
        (7, "=SUM(1**2)", "=SUM(100)", "100"),
    ] {
        let result = model.set_typed_user_input(0, row, 1, typed.to_string());
        assert_eq!(result, Ok(()), "{typed}");
        model.evaluate();
        assert_eq!(
            model.get_cell_formula(0, row, 1).unwrap().as_deref(),
            Some(formula),
            "{typed}"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, row, 1).unwrap(),
            value,
            "{typed}"
        );
    }
    for typed in [
        "=1**",
        "=Z1**2",
        "=1***2",
        "=2** 3",
        "=(2)**3",
        "=2**0.5",
        "=1E2**1",
        "=\"a\"**2",
        "=2**3**2",
    ] {
        let result = model.set_typed_user_input(0, 9, 1, typed.to_string());
        let message = result.err().unwrap_or_default();
        assert!(
            message.starts_with(TYPED_FORMULA_DOES_NOT_PARSE),
            "{typed}: {message}"
        );
    }
    // Stored, `**` is not read as an exponent: the text stays a parse error (unprobed).
    model._set("A10", "=1**2");
    model.evaluate();
    assert_eq!(model._get_text("A10"), "#ERROR!");
}
