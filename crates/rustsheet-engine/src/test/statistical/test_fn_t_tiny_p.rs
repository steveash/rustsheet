use crate::test::util::new_empty_model;

#[test]
fn formulas_near_the_limits_do_not_panic() {
    let mut model = new_empty_model();
    model._set("A1", "=T.INV.2T(1.9999999999999998,1)");
    model._set("A2", "=T.INV(1E-300,99999)");
    model._set("A3", "=CONFIDENCE.T(1E-300,1,100000)");
    model.evaluate();
    // Excel gives -63712901546918.43; the last digits are T-1542's.
    let a1: f64 = model._get_text("A1").parse().unwrap_or(f64::NAN);
    assert!((a1 / -63_712_901_546_918.43 - 1.0).abs() < 1e-6, "{a1}");
    assert_ne!(model._get_text("A2"), *"#NUM!");
    assert_ne!(model._get_text("A3"), *"#NUM!");
}
