#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_t_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: cumulative (left-tail CDF)
    model._set("A1", "=T.DIST(2, 10, TRUE)");
    // Valid: probability density function (PDF)
    model._set("B1", "=T.DIST(2, 10, FALSE)");

    // Wrong number of arguments
    model._set("A2", "=T.DIST(2, 10)");
    model._set("A3", "=T.DIST(2, 10, TRUE, FALSE)");

    // Domain error: df < 1 -> #NUM!
    model._set("A4", "=T.DIST(2, 0, TRUE)");
    model._set("A5", "=T.DIST(2, -1, TRUE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.963305983");
    assert_eq!(model._get_text("B1"), *"0.061145766");

    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
}

#[test]
fn test_fn_t_dist_rt_smoke() {
    let mut model = new_empty_model();

    // Valid: right tail probability
    model._set("A1", "=T.DIST.RT(2, 10)");

    // Wrong number of arguments
    model._set("A2", "=T.DIST.RT(2)");
    model._set("A3", "=T.DIST.RT(2, 10, TRUE)");

    // Domain error: df < 1
    model._set("A4", "=T.DIST.RT(2, 0)");
    model._set("A5", "=T.DIST.RT(2, -1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.036694017");

    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
}

#[test]
fn test_fn_t_dist_2t_smoke() {
    let mut model = new_empty_model();

    // Valid: two-tailed probability
    model._set("A1", "=T.DIST.2T(2, 10)");

    // In the limit case of x = 0, the two-tailed probability is 1.0
    model._set("A4", "=T.DIST.2T(0, 10)");

    // Wrong number of arguments
    model._set("A2", "=T.DIST.2T(2)");
    model._set("A3", "=T.DIST.2T(2, 10, TRUE)");

    // Domain errors:
    // x < 0 -> #NUM!
    model._set("A5", "=T.DIST.2T(-0.001, 10)");
    // df < 1 -> #NUM!
    model._set("A6", "=T.DIST.2T(2, 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.073388035");
    assert_eq!(model._get_text("A4"), *"1");

    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}

#[test]
fn test_fn_t_inv_smoke() {
    let mut model = new_empty_model();

    // Valid: upper and lower tail
    model._set("A1", "=T.INV(0.95, 10)");
    model._set("A2", "=T.INV(0.05, 10)");
    // limit case:
    model._set("B2", "=T.INV(0.95, 1)");

    // Wrong number of arguments
    model._set("A3", "=T.INV(0.95)");
    model._set("A4", "=T.INV(0.95, 10, 1)");

    // Domain errors:
    // p <= 0 or >= 1
    model._set("A5", "=T.INV(0, 10)");
    model._set("A6", "=T.INV(1, 10)");
    // df < 1
    model._set("A7", "=T.INV(0.95, 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.812461123");
    assert_eq!(model._get_text("A2"), *"-1.812461123");
    assert_eq!(model._get_text("B2"), *"6.313751515");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}

#[test]
fn test_fn_t_inv_2t_smoke() {
    let mut model = new_empty_model();

    // Valid: two-tailed critical values
    model._set("A1", "=T.INV.2T(0.1, 10)");
    model._set("A2", "=T.INV.2T(0.05, 10)");

    // p = 1 should give t = 0 (both tails outside are 1.0, so cut at the mean)
    model._set("A3", "=T.INV.2T(1, 10)");

    model._set("A7", "=T.INV.2T(1.5, 10)");

    // Wrong number of arguments
    model._set("A4", "=T.INV.2T(0.1)");
    model._set("A5", "=T.INV.2T(0.1, 10, 1)");

    // Domain errors:
    // p <= 0 or p > 1
    model._set("A6", "=T.INV.2T(0, 10)");
    // df < 1
    model._set("A8", "=T.INV.2T(0.1, 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.812461123");
    assert_eq!(model._get_text("A2"), *"2.228138852");
    assert_eq!(model._get_text("A3"), *"0");

    // NB: Excel returns -0.699812061 for T.INV.2T(1.5, 10)
    // which seems inconsistent with its documented behavior
    assert_eq!(model._get_text("A7"), *"#NUM!");

    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
}

#[test]
fn test_fn_t_dist_arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=T.DIST()");
    model._set("A2", "=T.DIST(8)");
    model._set("A3", "=T.DIST(8, 3)");
    model._set("A4", "=T.DIST(8, 3, TRUE)");
    model._set("A5", "=T.DIST(8, 3, TRUE, 1)");

    model._set("B1", "=T.INV()");
    model._set("B2", "=T.INV(A4)");
    model._set("B3", "=T.INV(A4, 3)");
    model._set("B4", "=T.INV(A4, 3, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"0.997961711");
    assert_eq!(model._get_text("A5"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"8");
    assert_eq!(model._get_text("B4"), *"#ERROR!");
}

// T-1518: nightly fuzz input `T.INV(0.9,5E10)` hung; Excel gives #NUM! above df = 1E10
// (probes-numeric case t1518-t-df-cut), with the pdf staying finite.
#[test]
fn test_t_family_df_upper_limit() {
    let mut model = new_empty_model();
    let over = [
        "=T.INV(0.9,5E10)",
        "=T.INV(0.9,10000000001)",
        "=T.INV.2T(0.1,1.2E10)",
        "=T.DIST(2,1.2E10,TRUE)",
        "=T.DIST.RT(2,1.2E10)",
        "=T.DIST.2T(2,1.2E10)",
        "=CONFIDENCE.T(0.05,1,10000000002)",
    ];
    for (i, f) in over.iter().enumerate() {
        model._set(&format!("A{}", i + 1), f);
    }
    // At the limit (and a fractional df that truncates to it) the functions still evaluate.
    model._set("B1", "=T.INV(0.9,1E10)");
    model._set("B2", "=T.INV(0.9,10000000000.5)");
    model._set("B3", "=T.DIST(2,1E10,TRUE)");
    model._set("B4", "=T.DIST(2,1E11,FALSE)");
    model._set("B5", "=CONFIDENCE.T(0.05,1,10000000001)");
    model.evaluate();

    for i in 1..=over.len() {
        assert_eq!(model._get_text(&format!("A{i}")), *"#NUM!", "{}", over[i - 1]);
    }
    assert_eq!(model._get_text("B1"), *"1.281551842");
    assert_eq!(model._get_text("B2"), *"1.281551842");
    assert_eq!(model._get_text("B3"), *"0.977249868");
    assert_eq!(model._get_text("B4"), *"0.053990967");
    assert_eq!(model._get_text("B5"), *"1.96E-05");
}
