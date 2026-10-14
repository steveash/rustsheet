#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_pi_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=PI(1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_atan2_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=ATAN2(1)");
    model._set("A2", "=ATAN2(1,1)");
    model._set("A3", "=ATAN2(1,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"0.785398163");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_fn_trigonometric_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SIN()");
    model._set("A2", "=COS()");
    model._set("A3", "=TAN()");

    model._set("A5", "=ASIN()");
    model._set("A6", "=ACOS()");
    model._set("A7", "=ATAN()");

    model._set("A9", "=SINH()");
    model._set("A10", "=COSH()");
    model._set("A11", "=TANH()");

    model._set("A13", "=ASINH()");
    model._set("A14", "=ACOSH()");
    model._set("A15", "=ATANH()");

    model._set("B1", "=SIN(1,2)");
    model._set("B2", "=COS(1,2)");
    model._set("B3", "=TAN(1,2)");

    model._set("B5", "=ASIN(1,2)");
    model._set("B6", "=ACOS(1,2)");
    model._set("B7", "=ATAN(1,2)");

    model._set("B9", "=SINH(1,2)");
    model._set("B10", "=COSH(1,2)");
    model._set("B11", "=TANH(1,2)");

    model._set("B13", "=ASINH(1,2)");
    model._set("B14", "=ACOSH(1,2)");
    model._set("B15", "=ATANH(1,2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"#ERROR!");
    assert_eq!(model._get_text("A10"), *"#ERROR!");
    assert_eq!(model._get_text("A11"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");

    assert_eq!(model._get_text("B5"), *"#ERROR!");
    assert_eq!(model._get_text("B6"), *"#ERROR!");
    assert_eq!(model._get_text("B7"), *"#ERROR!");

    assert_eq!(model._get_text("B9"), *"#ERROR!");
    assert_eq!(model._get_text("B10"), *"#ERROR!");
    assert_eq!(model._get_text("B11"), *"#ERROR!");
}

#[test]
fn test_fn_tan_pi2() {
    let mut model = new_empty_model();
    model._set("A1", "=TAN(PI()/2)");
    model.evaluate();

    // This is consistent with IEEE 754 but inconsistent with Excel
    assert_eq!(model._get_text("A1"), *"1.63312E+16");
}

#[test]
fn test_trigonometric_identity() {
    let mut model = new_empty_model();
    model._set("A1", "=COTH(1)*CSCH(1)");
    model._set("A2", "=COSH(1)/(SINH(1))^2");
    model._set("A3", "=A1=A2");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"TRUE");
}

// T-1202: an out-of-domain argument is `#NUM!` at once, so an enclosing function cannot swallow
// the NaN (gen-m5/cd883013bea3b305, gen-m6/3ee7ffc8d6ab061f, gen-m7/93976dd9a3578ed9).
#[test]
fn out_of_domain_inverse_functions_nested_are_num() {
    let mut model = new_empty_model();
    model._set("A1", "=SERIESSUM(ACOS(1E+15),0,1,1)");
    model._set("A2", "=ADDRESS(ACOSH(-1),4)");
    model._set("A3", "=CHOOSECOLS(0,ASIN(100))");
    model._set("A4", "=(1<ASIN(PI()))");
    model._set("A5", "=SUM(ACOS({2,1}))");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
}
