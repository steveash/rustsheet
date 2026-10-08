#![allow(clippy::unwrap_used, reason = "tests")]

//! Excel snaps a near-zero result of a cell formula's outermost `+`/`-` or SUM to 0, but not of
//! a wholly parenthesized formula or of an inner operation (docs/excel-semantics/numbers.md
//! "Near-zero snapping in addition and subtraction"; probes/literal-digits C4,
//! probes-numeric/30bc5f8fab02d0a2).

use crate::arithmetic::snap_near_zero;
use crate::test::util::new_empty_model;

/// `b·(1 + k·2^-52)`: `b` plus `k` ULP of a normal `b`.
fn ulps_above(b: f64, k: f64) -> f64 {
    b * (1.0 + k * f64::EPSILON)
}

#[test]
fn snaps_below_eight_ulp_of_the_smaller_addend() {
    // 7 ULP of 1.0 snaps, 8 ULP is kept.
    let seven = ulps_above(1.0, 7.0);
    assert_eq!(snap_near_zero(seven, -1.0, seven - 1.0), 0.0);
    let eight = ulps_above(1.0, 8.0);
    assert_eq!(snap_near_zero(eight, -1.0, eight - 1.0), eight - 1.0);
    // The smaller addend sets the scale: 1 - 4·2^-52 has half the ULP of 1.
    let below = 1.0 - 4.0 * f64::EPSILON;
    assert_eq!(snap_near_zero(1.0, -below, 1.0 - below), 1.0 - below);
}

#[test]
fn only_opposite_signed_nonzero_addends_snap() {
    let tiny = f64::EPSILON;
    assert_eq!(snap_near_zero(1.0, 1.0, 2.0), 2.0);
    assert_eq!(snap_near_zero(0.0, tiny, tiny), tiny);
    assert_eq!(snap_near_zero(tiny, 0.0, tiny), tiny);
    assert_eq!(snap_near_zero(-0.0, tiny, tiny), tiny);
    // A subnormal addend has scale 0, so nothing snaps.
    let subnormal = f64::MIN_POSITIVE / 4.0;
    assert_eq!(
        snap_near_zero(subnormal, -subnormal / 2.0, subnormal / 2.0),
        subnormal / 2.0
    );
}

#[test]
fn outermost_sum_snaps_but_grouped_and_inner_do_not() {
    let mut model = new_empty_model();
    model._set("A1", "=0.1+0.2-0.3");
    model._set("A2", "=SUM(0.1,0.2,-0.3)");
    model._set("A3", "=(0.1+0.2-0.3)");
    model._set("A4", "=(0.1+0.2-0.3)*1");
    model._set("A5", "=0.1+0.2-0.3+0");
    for row in 1..=5 {
        model._set(&format!("B{row}"), &format!("=A{row}=0"));
    }
    model.evaluate();
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("B2"), "TRUE");
    assert_eq!(model._get_text("B3"), "FALSE");
    assert_eq!(model._get_text("B4"), "FALSE");
    assert_eq!(model._get_text("B5"), "FALSE");
}

/// Every written level of parentheses around a whole formula is kept, and the group still
/// skips snapping (probes/near-zero-snap-groups B3, B6).
#[test]
fn whole_formula_keeps_every_parenthesis_level() {
    let mut model = new_empty_model();
    model._set("A1", "=((0.1+0.2-0.3))");
    model._set("A2", "=(((0.1+0.2-0.3)))");
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=((0.1+0.2-0.3))");
    assert_eq!(model._get_formula("A2"), "=(((0.1+0.2-0.3)))");
    assert!(model._get_text("A1") != "0");
}
