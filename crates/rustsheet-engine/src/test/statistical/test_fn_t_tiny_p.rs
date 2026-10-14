use crate::test::util::new_empty_model;

#[test]
fn formulas_near_the_limits_do_not_panic() {
    let mut model = new_empty_model();
    model._set("A1", "=T.INV.2T(1.9999999999999998,1)");
    model._set("A2", "=T.INV(1E-300,99999)");
    model._set("A3", "=CONFIDENCE.T(1E-300,1,100000)");
    model.evaluate();
    // p in (1,2) is #NUM! for us where Excel gives -63712901546918.43 (TODO(T-1542): fix and tighten).
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_ne!(model._get_text("A2"), *"#NUM!");
    assert_ne!(model._get_text("A3"), *"#NUM!");
}
