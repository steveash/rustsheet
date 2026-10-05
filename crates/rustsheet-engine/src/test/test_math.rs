#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_sqrt_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SQRT(4)");
    model._set("A2", "=SQRT()");
    model._set("A3", "=SQRT(4, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_fn_sqrtpi_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SQRTPI()");
    model._set("A2", "=SQRTPI(4, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

// Excel: probes/min-max-value-args; docs/excel-semantics/coercion.md (T-0217).
#[test]
fn test_fn_min_max_text_value_arguments() {
    let mut model = new_empty_model();
    model._set("B1", "a");
    model._set("B2", "5");
    model._set("A1", "=MIN(\"a\")");
    model._set("A2", "=MAX(CONCATENATE(\"x\"))");
    model._set("A3", "=MIN(\"2\")");
    model._set("A4", "=MAX(B1:B2)");
    model._set("A5", "=MIN(B1)");
    model._set("A6", "=MIN(IFERROR(B1,1))");
    model._set("A7", "=MIN(TRUE,5)");
    model._set("A8", "=MAX(-5,)");
    model._set("A9", "=MIN({TRUE,5})");
    model._set("A10", "=MAX({\"9\",-1})");
    model._set("A11", "=MAX(IF(TRUE,B1),B2)");
    model._set("A12", "=MIN(IFERROR(B9,7),3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"2");
    assert_eq!(model._get_text("A4"), *"5");
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"#VALUE!");
    assert_eq!(model._get_text("A7"), *"1");
    assert_eq!(model._get_text("A8"), *"0");
    assert_eq!(model._get_text("A9"), *"5");
    assert_eq!(model._get_text("A10"), *"-1");
    assert_eq!(model._get_text("A11"), *"5");
    assert_eq!(model._get_text("A12"), *"0");
}
