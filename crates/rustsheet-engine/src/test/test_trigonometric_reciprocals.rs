#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=CSC()");
    model._set("A2", "=SEC()");
    model._set("A3", "=COT()");

    model._set("A4", "=CSCH()");
    model._set("A5", "=SECH()");
    model._set("A6", "=COTH()");

    model._set("A7", "=ACOT()");
    model._set("A8", "=ACOTH()");

    model._set("B1", "=CSC(1, 2)");
    model._set("B2", "=SEC(1, 2)");
    model._set("B3", "=COT(1, 2)");

    model._set("B4", "=CSCH(1, 2)");
    model._set("B5", "=SECH(1, 2)");
    model._set("B6", "=COTH(1, 2)");

    model._set("B7", "=ACOT(1, 2)");
    model._set("B8", "=ACOTH(1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");

    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");

    assert_eq!(model._get_text("B4"), *"#ERROR!");
    assert_eq!(model._get_text("B5"), *"#ERROR!");
    assert_eq!(model._get_text("B6"), *"#ERROR!");

    assert_eq!(model._get_text("B7"), *"#ERROR!");
    assert_eq!(model._get_text("B8"), *"#ERROR!");
}

#[test]
fn reciprocals_divide_like_the_operator() {
    // probes/trig-reciprocal-special-angles (excel-graph): 1 over the rounded TAN, SIN or COS,
    // rounded to 64 bits and then to a double, like `1/TAN(PI()/4)`.
    let mut model = new_empty_model();
    model._set("A1", "=COT(PI()/4)");
    model._set("A2", "=CSC(PI()/6)");
    model._set("A3", "=SEC(PI()/3)");
    model._set("A4", "=COT(3*PI()/4)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1.0_f64.to_bits());
    assert_eq!(bits("A2"), 2.0_f64.to_bits());
    assert_eq!(bits("A3"), 1.999_999_999_999_999_6_f64.to_bits());
    assert_eq!(bits("A4"), (-0.999_999_999_999_999_8_f64).to_bits());
}
