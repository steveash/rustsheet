use crate::test::util::new_empty_model;

#[test]
fn fn_besseli() {
    let mut model = new_empty_model();

    model._set("B1", "=BESSELI()");
    model._set("B2", "=BESSELI(1,2, 1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn fn_besselj() {
    let mut model = new_empty_model();

    model._set("B1", "=BESSELJ()");
    model._set("B2", "=BESSELJ(1,2, 1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn fn_besselk() {
    let mut model = new_empty_model();

    model._set("B1", "=BESSELK()");
    model._set("B2", "=BESSELK(1,2, 1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn fn_bessely() {
    let mut model = new_empty_model();

    model._set("B1", "=BESSELY()");
    model._set("B2", "=BESSELY(1,2, 1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

/// Excel: any negative order is `#NUM!`, checked before truncation
/// (probes/eng-gen-sample-edges E1:E10).
#[test]
fn negative_fractional_order_is_num_error() {
    let mut model = new_empty_model();
    model._set("A1", "=BESSELK(5,-0.0001)");
    model._set("A2", "=BESSELJ(5,-0.9999)");
    model._set("A3", "=BESSELY(5,-1E-300)");
    model._set("A4", "=BESSELI(5,-0.5)");
    model._set("A5", "=BESSELK(5,0.9999)=BESSELK(5,0)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    assert_eq!(model._get_text("A5"), *"TRUE");
}
