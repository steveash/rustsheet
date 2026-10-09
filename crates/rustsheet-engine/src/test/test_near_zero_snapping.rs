#![allow(clippy::unwrap_used, reason = "tests")]

//! Excel snaps a near-zero result of a cell formula's outermost `+`/`-` and of every SUM call to
//! 0, but not of a wholly parenthesized `+`/`-` formula or of an inner operation (docs/excel-semantics/numbers.md
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
    assert_eq!(model._get_text("A1"), "5.55112E-17");
    assert_eq!(model._get_text("A2"), "5.55112E-17");
}

/// A SUM call snaps wherever it sits, and a trailing zero argument stops it (the last two
/// addends are `-0.3` and `0` or `0.2` and `0`): probes/near-zero-snap-sum-forms (T-1131).
#[test]
fn sum_call_snaps_anywhere_but_not_with_a_trailing_zero() {
    let mut model = new_empty_model();
    for (cell, formula) in [
        ("A1", "=SUM(0.1,0.2,-0.3)*1"),
        ("A2", "=(SUM(0.1,0.2,-0.3))"),
        ("A3", "=IF(TRUE,SUM(0.1,0.2,-0.3))"),
        ("A4", "=SUM(SUM(0.1,0.2,-0.3))"),
        ("A5", "=SUM({0.1,0.2,-0.3})"),
        ("A6", "=SUM(0.1,0.2,-0.3,0)"),
    ] {
        model._set(cell, formula);
        model._set(&cell.replace('A', "B"), &format!("={cell}=0"));
    }
    model.evaluate();
    for row in 1..=5 {
        assert_eq!(model._get_text(&format!("B{row}")), "TRUE", "row {row}");
    }
    assert_eq!(model._get_text("B6"), "FALSE");
    assert_eq!(model._get_text("A6"), "5.55112E-17");
}
