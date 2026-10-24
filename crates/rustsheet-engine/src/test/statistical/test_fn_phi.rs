#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_phi_smoke() {
    let mut model = new_empty_model();

    model._set("A1", "=PHI(0)");
    model._set("A2", "=PHI(1)");
    model._set("A3", "=PHI(-1)");

    // Wrong number of arguments -> #ERROR!
    model._set("A4", "=PHI()");
    model._set("A5", "=PHI(0, 1)");

    model.evaluate();

    // Standard values
    assert_eq!(model._get_text("A1"), *"0.39894228");
    assert_eq!(model._get_text("A2"), *"0.241970725");
    assert_eq!(model._get_text("A3"), *"0.241970725");

    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
}

#[test]
fn phi_reads_a_boolean_as_a_number() {
    let mut model = new_empty_model();
    // probes-numeric/t1049-phi A32, A33.
    model._set("A1", "=PHI(TRUE)");
    model._set("A2", "=PHI(FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.241970725");
    assert_eq!(model._get_text("A2"), *"0.39894228");
}
