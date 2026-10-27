#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_gauss_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=GAUSS(-3)");
    model._set("A2", "=GAUSS(-2.3)");
    model._set("A3", "=GAUSS(-1.7)");
    model._set("A4", "=GAUSS(0)");
    model._set("A5", "=GAUSS(0.5)");
    model._set("A6", "=GAUSS(1)");
    model._set("A7", "=GAUSS(1.3)");
    model._set("A8", "=GAUSS(3)");
    model._set("A9", "=GAUSS(4)");

    model._set("G6", "=GAUSS()");
    model._set("G7", "=GAUSS(1, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-0.498650102");
    assert_eq!(model._get_text("A2"), *"-0.48927589");
    assert_eq!(model._get_text("A3"), *"-0.455434537");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0.191462461");
    assert_eq!(model._get_text("A6"), *"0.341344746");
    assert_eq!(model._get_text("A7"), *"0.403199515");
    assert_eq!(model._get_text("A8"), *"0.498650102");
    assert_eq!(model._get_text("A9"), *"0.499968329");

    assert_eq!(model._get_text("G6"), *"#ERROR!");
    assert_eq!(model._get_text("G7"), *"#ERROR!");
}

#[test]
fn test_fn_gauss_booleans() {
    let mut model = new_empty_model();
    model._set("A1", "=GAUSS(TRUE)");
    model._set("A2", "=GAUSS(FALSE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.341344746");
    assert_eq!(model._get_text("A2"), *"0");
}

#[test]
fn test_fn_gauss_tiny_argument_is_z_times_phi_zero() {
    // Excel (gen-m9 oracle, T-2434): the leading term below ~2E-15, the cancelled Phi(z) - 0.5 above.
    let mut model = new_empty_model();
    model._set("A1", "=GAUSS(1E-300)");
    model._set("A2", "=GAUSS(-1E-300)");
    model._set("A3", "=GAUSS(1E-10)");
    model._set("A4", "=GAUSS(4E-15)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3.98942E-301");
    assert_eq!(model._get_text("A2"), *"-3.98942E-301");
    assert_eq!(model._get_text("A3"), *"3.98943E-11");
    assert_eq!(model._get_text("A4"), *"1.55431E-15");
}
