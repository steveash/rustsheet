#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_f_dist_sanity() {
    let mut model = new_empty_model();
    model._set("A1", "=F.DIST(15, 6, 4, TRUE)");
    model._set("A2", "=F.DIST(15, 6, 4, FALSE)");
    model._set("A3", "=F.DIST(15, 6, 4)");
    model._set("A4", "=F.DIST(15, 6, 4, TRUE, FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.989741952");
    assert_eq!(model._get_text("A2"), *"0.001271447");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_f_dist_density_at_zero() {
    // probes-numeric/t1049-fdist A27, A28 (T-2095).
    let mut model = new_empty_model();
    model._set("A1", "=F.DIST(0,1,7,FALSE)");
    model._set("A2", "=F.DIST(0,2,7,FALSE)");
    model._set("A3", "=F.DIST(0,2.9,7,FALSE)");
    model._set("A4", "=F.DIST(0,3,7,FALSE)");
    model._set("A5", "=F.DIST(0,1,7,TRUE)");
    model._set("A6", "=F.DIST.RT(0,1,7)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"1");
}

#[test]
fn test_fn_f_dist_rt_sanity() {
    let mut model = new_empty_model();

    // Valid call
    model._set("A1", "=F.DIST.RT(15, 6, 4)");
    // Too few args
    model._set("A2", "=F.DIST.RT(15, 6)");
    // Too many args
    model._set("A3", "=F.DIST.RT(15, 6, 4, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.010258048");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_fn_f_inv_sanity() {
    let mut model = new_empty_model();

    // Valid call: left-tail inverse
    model._set("A1", "=F.INV(0.9897419523940, 6, 4)");

    // Too many args
    model._set("A2", "=F.INV(0.5, 6, 4, 2)");

    // Too few args
    model._set("A3", "=F.INV(0.5, 6)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"15");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_fn_f_inv_rt_sanity() {
    let mut model = new_empty_model();

    // Valid call: left-tail inverse
    model._set("A1", "=F.INV.RT(0.0102580476059808, 6, 4)");

    // Too many args
    model._set("A2", "=F.INV.RT(0.5, 6, 4, 2)");

    // Too few args
    model._set("A3", "=F.INV.RT(0.5, 6)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"15");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn f_inv_rt_deep_upper_tail_is_solved_on_its_own_tail() {
    // statrs' estimate solves the rounded 1 - p, 7% off for F.INV.RT(1E-16,9,3); the refinement on
    // F.DIST.RT gives Excel's 60647458731.09812 and FINV(1E-16,1,7) = 183379.00311608665 (gen-m9
    // 0a9226016e8f5f33 A45, 146b46e74f2dac3b A117; T-0974).
    let mut model = new_empty_model();
    model._set("A1", "=F.INV.RT(1E-16,9,3)");
    model._set("A2", "=FINV(1E-16,1,7)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"60647458731");
    assert_eq!(model._get_text("A2"), *"183379.0031");
}
