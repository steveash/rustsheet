#![allow(clippy::unwrap_used, reason = "tests")]

//! Excel snaps a near-zero result of a cell formula's outermost `+`/`-` or a SUM call to 0, but not of
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

// Excel flushes the 8-ULP bound when it is subnormal: addends of 1.5·2^-973 still snap (to a
// subnormal total), those of 1.5·2^-974 never do (probes/t2714-snap-tiny-addends).
#[test]
fn the_bound_flushes_below_the_smallest_normal() {
    let at = |p: i32| {
        let a = 1.5 * 2f64.powi(-p);
        let b = -ulps_above(a, 1.0);
        snap_near_zero(a, b, a + b)
    };
    assert_eq!(at(973), 0.0);
    let kept = at(974);
    assert!(kept != 0.0 && kept.abs() < f64::MIN_POSITIVE);
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
fn sum_snaps_everywhere_but_grouped_and_inner_arithmetic_do_not() {
    let mut model = new_empty_model();
    model._set("A1", "=0.1+0.2-0.3");
    model._set("A2", "=SUM(0.1,0.2,-0.3)");
    model._set("A3", "=(0.1+0.2-0.3)");
    model._set("A4", "=(0.1+0.2-0.3)*1");
    model._set("A5", "=0.1+0.2-0.3+0");
    model._set("A6", "=(SUM(0.1,0.2,-0.3))");
    model._set("A7", "=SUM(0.1,0.2,-0.3)*1");
    model._set("A8", "=SUM(0.1,0.2,-0.3,0)");
    for row in 1..=8 {
        model._set(&format!("B{row}"), &format!("=A{row}=0"));
    }
    model.evaluate();
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("B2"), "TRUE");
    assert_eq!(model._get_text("B3"), "FALSE");
    assert_eq!(model._get_text("B4"), "FALSE");
    assert_eq!(model._get_text("B5"), "FALSE");
    // probes/near-zero-sum-forms: SUM snaps inside a group or an operation, but not after a zero addend.
    assert_eq!(model._get_text("B6"), "TRUE");
    assert_eq!(model._get_text("B7"), "TRUE");
    assert_eq!(model._get_text("B8"), "FALSE");
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

/// AGGREGATE(9) and SUBTOTAL(9/109) snap like SUM (probes/near-zero-aggregate-forms B1-B5).
#[test]
fn aggregate_and_subtotal_sum_snap() {
    let mut model = new_empty_model();
    model._set("A1", "0.1");
    model._set("A2", "0.2");
    model._set("A3", "-0.3");
    model._set("B1", "=AGGREGATE(9,4,A1:A3)=0");
    model._set("B2", "=SUBTOTAL(9,A1:A3)=0");
    model._set("B3", "=SUBTOTAL(109,A1:A3)+0=0");
    model.evaluate();
    for cell in ["B1", "B2", "B3"] {
        assert_eq!(model._get_text(cell), "TRUE", "{cell}");
    }
}

/// The root `+`/`-` snaps every element of an array result on its own two addends, with the
/// scalar bound: 7 ULP of 1 snaps on the `+` side, 4 ULP of the smaller addend does not on the
/// `-` side (probes/t3047-snap-array-operands S!E1:L3).
#[test]
fn array_operands_snap_element_by_element() {
    let mut model = new_empty_model();
    for (row, (a, b, c)) in [
        ("0.1", "0.2", "0.3"),
        ("1", "1", "2"),
        ("0.1", "0.2", "0.1"),
        ("1", "-0.9", "0.1"),
    ]
    .iter()
    .enumerate()
    {
        let row = row + 1;
        model._set(&format!("A{row}"), a);
        model._set(&format!("B{row}"), b);
        model._set(&format!("C{row}"), c);
    }
    model._set("E1", "=A1:A4+B1:B4-C1:C4");
    model._set("F1", "=-C1:C4+A1:A4+B1:B4");
    model._set("G1", "=0+{0.1,1}+{0.2,1}-{0.3,2}");
    model._set("G3", "=0.1+{0.2;0.2}-0.3");
    model._set("H3", "=0+{0.1}+0.2-0.3");
    model._set("I1", "=1+{3;7;8}*2^-52-1");
    model._set("J1", "=1-{3;4;7}*2^-52-1");
    model._set("K1", "=-1+(1+{3;7;8}*2^-52)");
    model._set("M1", "=INDEX(E1#,1)=0");
    model._set("M2", "=INDEX(A1:A4+B1:B4-C1:C4,1)=0");
    model.evaluate();
    let zero = [
        "E1", "E2", "E4", "F1", "F2", "F4", "G1", "H1", "G3", "G4", "H3", "I1", "I2", "J1", "K1",
        "K2",
    ];
    for cell in zero {
        assert_eq!(model._get_text(cell), "0", "{cell}");
    }
    assert_eq!(model._get_text("E3"), "0.2");
    assert_eq!(model._get_text("I3"), "1.77636E-15");
    assert_eq!(model._get_text("J2"), "-8.88178E-16");
    assert_eq!(model._get_text("J3"), "-1.55431E-15");
    assert_eq!(model._get_text("K3"), "1.77636E-15");
    // A reader of the spill sees the snapped value; an operator under a call is not snapped.
    assert_eq!(model._get_text("M1"), "TRUE");
    assert_eq!(model._get_text("M2"), "FALSE");
}

/// A legacy formula intersects its range operands with its own row and snaps the result
/// (probes/t3047-snap-array-operands L!E1:E4).
#[test]
fn legacy_intersected_ranges_snap_per_row() {
    let mut model = new_empty_model();
    for (row, (a, b, c)) in [("0.1", "0.2", "0.3"), ("1", "-0.9", "0.1")]
        .iter()
        .enumerate()
    {
        let row = row + 1;
        model._set(&format!("A{row}"), a);
        model._set(&format!("B{row}"), b);
        model._set(&format!("C{row}"), c);
    }
    for row in 1..=2 {
        model
            .set_cell_with_legacy_formula(0, row, 5, "=A1:A2+B1:B2-C1:C2")
            .unwrap();
    }
    model.evaluate();
    assert_eq!(model._get_text("E1"), "0");
    assert_eq!(model._get_text("E2"), "0");
}
