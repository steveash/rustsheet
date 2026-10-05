#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=MOD(5,2)");
    model._set("A2", "=MOD()");
    model._set("A3", "=MOD(5, 2, 1)");
    model._set("A4", "=QUOTIENT(5, 2)");
    model._set("A5", "=QUOTIENT()");
    model._set("A6", "=QUOTIENT(5, 2, 1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"2");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

// Excel: MOD is #NUM! once |number/divisor| reaches 1.1259E+12 (probes/mod-quotient-boundary).
#[test]
fn huge_quotient_is_num() {
    let mut model = new_empty_model();
    model._set("A1", "=MOD(1125899999999,1)");
    model._set("A2", "=MOD(1125900000000,1)");
    model._set("A3", "=MOD(-1125900000000,-1)");
    model._set("A4", "=MOD(2251799999999,2)");
    model._set("A5", "=MOD(2251800000000,2)");
    model._set("A6", "=MOD(562950000000.5,0.5)");
    model._set("A7", "=MOD(1,1E-12)");
    model._set("A8", "=MOD(1.2,1E-12)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"1");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
}
