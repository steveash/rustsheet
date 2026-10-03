#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Expected values (General format, 6 digits) are Excel-cached (MOD_QUOTIENT.xlsx C20/C21/C25/C26).
#[test]
fn test_mod_exact_remainder() {
    let mut model = new_empty_model();
    model._set("A1", "=MOD(10.0000000001,0.1)");
    model._set("A2", "=MOD(10.0000000001,-0.1)");
    model._set("A3", "=MOD(5,1E-10)");
    model._set("A4", "=MOD(5,-1E-10)");
    model._set("A5", "=MOD(-7,3)");
    model._set("A6", "=MOD(7,-3)");
    model._set("A7", "=MOD(-6,3)");
    model._set("A8", "=1/MOD(-6,3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "9.99995E-11");
    assert_eq!(model._get_text("A2"), "-0.1");
    assert_eq!(model._get_text("A3"), "9.99998E-11");
    assert_eq!(model._get_text("A4"), "-1.82161E-16");
    assert_eq!(model._get_text("A5"), "2");
    assert_eq!(model._get_text("A6"), "-2");
    assert_eq!(model._get_text("A7"), "0");
    assert_eq!(model._get_text("A8"), "#DIV/0!");
}
