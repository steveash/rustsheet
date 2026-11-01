//! T-2762: typing a formula with a string literal over 4,095 UTF-16 units is refused and the cell
//! keeps its content, as Excel's Graph PATCH 400 does (T-2739 probes `t2739-typed-*`).

use crate::test::util::new_empty_model;

#[test]
fn overlong_literal_typed_is_refused_and_the_cell_is_unchanged() {
    let mut model = new_empty_model();
    assert_eq!(model.set_user_input(0, 1, 1, "=5".to_string()), Ok(()));
    let literal = |n: usize| format!("=LEN(\"{}\")", "a".repeat(n));
    assert!(model.set_user_input(0, 1, 1, literal(4_096)).is_err());
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=5");
    assert_eq!(model.set_user_input(0, 1, 2, literal(4_095)), Ok(()));
    model.evaluate();
    assert_eq!(model._get_text("B1"), "4095");
}
