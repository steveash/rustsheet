#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_geomean_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=GEOMEAN()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_geomean_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "'2");
    // B5 is empty
    model._set("B6", "true");
    model._set("A1", "=GEOMEAN(B1:B6)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.817120593");
}

#[test]
fn test_fn_geomean_zero_value_returns_num_error() {
    let mut model = new_empty_model();
    model._set("A1", "0");
    model._set("A2", "1");
    model._set("A3", "2");
    model._set("B1", "=GEOMEAN(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#NUM!");
}

#[test]
fn test_fn_geomean_negative_value_returns_num_error() {
    let mut model = new_empty_model();
    model._set("A1", "-1");
    model._set("A2", "1");
    model._set("B1", "=GEOMEAN(A1:A2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#NUM!");
}

#[test]
fn test_fn_geomean_array_literal() {
    let mut model = new_empty_model();
    // Array literal {1,2,3} — previously fell through to the ignored arm
    model._set("A1", "=GEOMEAN({1,2,3})");
    model.evaluate();
    let v: f64 = model._get_text("A1").parse().unwrap();
    assert!((v - 1.817120593).abs() < 1e-7);
}

#[test]
fn test_fn_geomean_large_values_no_overflow() {
    let mut model = new_empty_model();
    // Previously: product = 1e200 * 1e200 = inf → wrong answer
    model._set("A1", "1e200");
    model._set("A2", "1e200");
    model._set("B1", "=GEOMEAN(A1:A2)");
    model.evaluate();
    let v: f64 = model._get_text("B1").parse().unwrap();
    assert!(
        (v - 1e200).abs() < 1e190,
        "GEOMEAN of two 1e200 values should be 1e200, got {v}"
    );
}

#[test]
fn test_fn_geomean_mixed_magnitudes_stay_finite() {
    let mut model = new_empty_model();
    // Neither the product (inf / 0) nor a partial product may leave the double range.
    model._set("B1", "=GEOMEAN(1E154,1E300)");
    model._set("B2", "=GEOMEAN(1E-154,1E-300)");
    model._set("B3", "=GEOMEAN(1E300,1E300,1E300)");
    model.evaluate();
    for (cell, want) in [("B1", 1e227), ("B2", 1e-227), ("B3", 1e300)] {
        let v: f64 = model
            ._get_text(cell)
            .parse()
            .unwrap_or_else(|_| panic!("{cell}: {}", model._get_text(cell)));
        assert!(((v - want) / want).abs() < 1e-9, "{cell}: got {v}");
    }
}

// Excel is #NUM! when the product of the values' significands in [1, 2) overflows, whatever the
// plain product does (numbers.md, "GEOMEAN is #NUM! when the product of significands overflows";
// probes/geomean-overflow-cut*, T-1112).
#[test]
fn test_fn_geomean_significand_product_overflow_is_num() {
    let mut model = new_empty_model();
    model._set("A1", "=GEOMEAN(IF(SEQUENCE(1028),5E+204))");
    model._set("A2", "=GEOMEAN(IF(SEQUENCE(1029),5E+204))");
    model._set("A3", "=GEOMEAN(IF(ISODD(SEQUENCE(1750)),1.5,0.75))");
    model._set("A4", "=GEOMEAN(IF(ISODD(SEQUENCE(1752)),1.5,0.75))");
    model._set("A5", "=GEOMEAN(IF(SEQUENCE(2000),2))");
    model._set("A6", "=GEOMEAN(IF(SEQUENCE(1770),1E+300))");
    model._set("A7", "=GEOMEAN(IF(SEQUENCE(1771),1E+300))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5E+204");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "1.060660172");
    // the plain product, 1.125^876, is in range
    assert_eq!(model._get_text("A4"), "#NUM!");
    // powers of two have significand 1
    assert_eq!(model._get_text("A5"), "2");
    assert_eq!(model._get_text("A6"), "1E+300");
    assert_eq!(model._get_text("A7"), "#NUM!");
}

#[test]
fn test_fn_geomean_exact_roots_are_exact() {
    // `exp(ln(P)/n)` is one ulp off on each; Excel gives the exact root (numbers.md, "GEOMEAN
    // in-range root"; probes/geomean-single-and-copies, probes/geomean-root-refinement).
    // A parenthesized difference shows the last bit: `=` compares at 15 digits and a bare `X-b`
    // snaps near-zero results to 0 (numbers.md, "Near-zero snapping in addition and subtraction").
    let mut model = new_empty_model();
    model._set("A1", "=(GEOMEAN(3,3)-3)");
    model._set("A2", "=(GEOMEAN(1,9)-3)");
    model._set("A3", "=(GEOMEAN(IF(SEQUENCE(50),7))-7)");
    model._set("A4", "=(GEOMEAN(IF(SEQUENCE(3),0.1))-0.1)");
    model._set("A5", "=(GEOMEAN(3)-3)");
    // not exact roots: left as `exp(ln(P)/n)`, which is Excel's value here
    model._set("A6", "=(GEOMEAN(1,3)-SQRT(3))");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4", "A5"] {
        assert_eq!(model._get_text(cell), "0", "{cell}");
    }
    // 1.7320508075688774 (Excel, ironcalc-calc GEOMEAN C23), one ulp above the correctly rounded root
    assert_eq!(model._get_text("A6"), "2.22045E-16");
}
