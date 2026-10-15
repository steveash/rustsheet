use crate::test::util::new_empty_model;

#[test]
fn ceiling_floor_family_lifts_over_both_arguments() {
    // gen-m7/e5a361112d83d0c3 Calc!A9 (ISO.CEILING); the rest follow arrays.md's lift rule.
    let mut model = new_empty_model();
    let cases = [
        ("ISO.CEILING({1.5,2.5},1)", "5"),
        ("ISO.CEILING(1.5,{1,2})", "4"),
        ("CEILING.PRECISE({1.5,2.5},1)", "5"),
        ("CEILING.PRECISE(1.5,{1,2})", "4"),
        ("CEILING.MATH({1.5,2.5},1)", "5"),
        ("CEILING.MATH(1.5,{1,2})", "4"),
        ("CEILING.MATH(-1.5,1,{0,1})", "-3"),
        ("CEILING({1.5,2.5},1)", "5"),
        ("CEILING(1.5,{1,2})", "4"),
        ("FLOOR.MATH({1.5,2.5},1)", "3"),
        ("FLOOR.MATH(1.5,{1,2})", "1"),
        ("FLOOR.MATH(-1.5,1,{0,1})", "-3"),
        ("FLOOR.PRECISE({1.5,2.5},1)", "3"),
        ("FLOOR.PRECISE(1.5,{1,2})", "1"),
        ("FLOOR({1.5,2.5},1)", "3"),
        ("FLOOR(1.5,{1,2})", "1"),
        ("MROUND({1.4,2.6},1)", "4"),
        ("MROUND(1.5,{1,2})", "4"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), &format!("=SUM({formula})"));
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn ceiling_math_error_number_is_broadcast_over_an_array_mode() {
    // gen-m9/89693ab0cafc29df Calc!A14: Excel lifts even though number is an error.
    let mut model = new_empty_model();
    model._set("A1", "=CEILING.MATH(1/0,1,{0;1})");
    model._set("B1", "=CEILING.MATH(1/0,1,0)");
    model.evaluate();
    // The error is broadcast over the 2x1 mode array, so it spills down A1:A2.
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("B1"), *"#DIV/0!");
}

#[test]
fn mround_keeps_its_scalar_rules() {
    let mut model = new_empty_model();
    model._set("A1", "=MROUND(TRUE,)");
    model._set("B1", "=MROUND({TRUE,2.6},1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("B1"), *"#VALUE!");
    assert_eq!(model._get_text("C1"), *"3");
}
