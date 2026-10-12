#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=MROUND()");
    model._set("A2", "=MROUND(10)");
    model._set("A3", "=MROUND(10, 3)");
    model._set("A4", "=MROUND(10, 3, 1)");

    model._set("A5", "=TRUNC()");
    model._set("A6", "=TRUNC(10)");
    model._set("A7", "=TRUNC(10.22, 1)");
    model._set("A8", "=TRUNC(10, 3, 1)");

    model._set("A9", "=INT()");
    model._set("A10", "=INT(10.22)");
    model._set("A11", "=INT(10.22, 1)");
    model._set("A12", "=INT(10.22, 1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"9");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"10");
    assert_eq!(model._get_text("A7"), *"10.2");
    assert_eq!(model._get_text("A8"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"#ERROR!");
    assert_eq!(model._get_text("A10"), *"10");
    assert_eq!(model._get_text("A11"), *"#ERROR!");
    assert_eq!(model._get_text("A12"), *"#ERROR!");
}

/// TRUNC is ROUNDDOWN: a 15-digit decimal cut first, and the same huge `num_digits` wrap
/// (probes/trunc-decimal-stages, excel-graph, T-0297).
#[test]
fn trunc_rounds_down_in_decimal_like_rounddown() {
    let mut model = new_empty_model();
    for (cell, formula) in [
        ("A1", "=TRUNC(-(0.1+0.7),1)"),
        ("A2", "=TRUNC(1-2^-53)"),
        ("A3", "=TRUNC(123456789.123,-5)"),
        ("A4", "=TRUNC(123.456,-16)"),
        ("A5", "=TRUNC(123.456,65537)"),
        ("A6", "=TRUNC(2^1023*(2-2^-52),0)"),
        ("A7", "=TRUNC(-0.5)&\"\""),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    assert_eq!(model._get_text("A1"), "-0.8");
    assert_eq!(model._get_text("A2"), "1");
    assert_eq!(model._get_text("A3"), "123400000");
    assert_eq!(model._get_text("A4"), "0");
    assert_eq!(model._get_text("A5"), "123.4");
    assert_eq!(model._get_text("A6"), "#NUM!");
    assert_eq!(model._get_text("A7"), "0");
}
