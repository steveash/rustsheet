#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_round_approximation() {
    let mut model = new_empty_model();
    model._set("A1", "=ROUND(1.05*(0.0284+0.0046)-0.0284,4)");
    model._set("A2", "=ROUNDDOWN(1.05*(0.0284+0.0046)-0.0284,5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.0063");
    assert_eq!(model._get_text("A2"), *"0.00625");
}

#[test]
fn fn_round_to_more_digits_than_a_double_holds() {
    // gen-m2/f7e6d351c8538ef5 A35, a339ea84ef64382e A138, 1b6ceed7489e0800 A337 (excel-graph).
    let mut model = new_empty_model();
    model._set("A1", "=ROUNDDOWN(0.5,123456789.123)");
    model._set("A2", "=ROUNDDOWN(1234.5,100000)");
    model._set("A3", "=ROUNDUP(42,100000)");
    model._set("A4", "=ROUND(1E-15,123456789)");
    model._set("A5", "=ROUNDDOWN(TRUE,-123456789.323)");
    model._set("A6", "=ROUND(12345,-400)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.5");
    assert_eq!(model._get_text("A2"), *"1234.5");
    assert_eq!(model._get_text("A3"), *"42");
    assert_eq!(model._get_text("A4"), *"1E-15");
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"0");
}
