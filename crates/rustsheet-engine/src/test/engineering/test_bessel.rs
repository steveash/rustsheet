use crate::cell::CellValue;
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

/// Excel: past the phase 134217727 the asymptotic form's sine and cosine do not answer, so
/// BESSELJ and BESSELY are `#NUM!` there and in every order recurring from it, while the order-1
/// phase, 1.57 lower, can still answer (probes/t2547-bessel-phase-cut A9:A12, A37:A41, A49:A50).
#[test]
fn asymptotic_phase_past_the_inner_trig_cut_is_num_error() {
    let mut model = new_empty_model();
    model._set("A1", "=BESSELJ(134217727.785398,0)");
    model._set("A2", "=BESSELJ(134217727.786398,0)");
    model._set("A3", "=BESSELY(134217727.786398,0)");
    model._set("A4", "=BESSELJ(134217728.5,1)");
    model._set("A5", "=BESSELJ(134217728.5,2)");
    model._set("A6", "=BESSELY(134217728.5,4)");
    model._set("A7", "=BESSELJ(-134217728,0)");
    model._set("A8", "=BESSELJ(-134217728,1)");
    model._set("A9", "=BESSELY(1E+15,1)");
    model._set("A10", "=BESSELY(134217727.785398,0)");
    model.evaluate();
    for cell in ["A2", "A3", "A5", "A6", "A7", "A9"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A4"), (-6.297_380_020_515_363e-5_f64).to_bits());
    assert_eq!(bits("A8"), 6.863_275_229_685_527e-5_f64.to_bits());
    assert_eq!(bits("A1"), (-2.020_590_456_739_121_3e-5_f64).to_bits());
    assert_eq!(bits("A10"), (-6.584_002_087_696_111e-5_f64).to_bits());
}

/// Excel drops a subnormal term `2n/x * K(n)` from BESSELK's upward recurrence, so near x = 700
/// higher orders repeat K0 and K1 (probes-numeric/m9-besselk-underflow).
#[test]
fn besselk_recurrence_flushes_a_subnormal_term() {
    let mut model = new_empty_model();
    model._set("A1", "=BESSELK(700,2)=BESSELK(700,0)");
    model._set("A2", "=BESSELK(702,4)=BESSELK(702,0)");
    model._set("A3", "=BESSELK(705,5)=BESSELK(705,1)");
    model._set("A4", "=BESSELK(700,3)");
    model._set("A5", "=BESSELK(702,4)");
    model.evaluate();
    for cell in ["A1", "A2", "A3"] {
        assert_eq!(model._get_text(cell), *"TRUE", "{cell}");
    }
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A4"), 4.699_795_140_140_56e-306_f64.to_bits());
    assert_eq!(bits("A5"), 6.310_849_433_265_845e-307_f64.to_bits());
}
