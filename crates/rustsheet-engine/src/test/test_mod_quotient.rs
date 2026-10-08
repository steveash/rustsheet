#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=MOD(5,2)");
    model._set("A2", "=MOD()");
    model._set("A3", "=MOD(5, 2, 1)");
    model._set("A4", "=QUOTIENT(5, 2)");
    model._set("A5", "=QUOTIENT()");
    model._set("A6", "=QUOTIENT(5, 2, 1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"2");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

// Excel: MOD is #NUM! once |number/divisor| reaches 1.1259E+12 (probes/mod-quotient-boundary).
#[test]
fn huge_quotient_is_num() {
    let mut model = new_empty_model();
    model._set("A1", "=MOD(1125899999999,1)");
    model._set("A2", "=MOD(1125900000000,1)");
    model._set("A3", "=MOD(-1125900000000,-1)");
    model._set("A4", "=MOD(2251799999999,2)");
    model._set("A5", "=MOD(2251800000000,2)");
    model._set("A6", "=MOD(562950000000.5,0.5)");
    model._set("A7", "=MOD(1,1E-12)");
    model._set("A8", "=MOD(1.2,1E-12)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"1");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_ne!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
}

// Excel: MOD lifts element-wise over arrays (probes/geomean-overflow-cut-3 E4).
#[test]
fn mod_lifts_over_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(MOD(SEQUENCE(4),2))");
    model._set("A2", "=SUM(MOD(-SEQUENCE(4),3))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    // MOD takes the sign of the divisor: -1 -> 2, -2 -> 1, -3 -> 0, -4 -> 2
    assert_eq!(model._get_text("A2"), *"5");
}

#[test]
fn mod_lifts_over_ranges_with_error_and_zero_elements() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("B2", "=NA()");
    model._set("B3", "7");
    model._set("C1", "2");
    model._set("C2", "2");
    model._set("C3", "0");
    model._set("A1", "=MOD(B1:B3,C1:C3)");
    // a scalar divisor broadcasts and a size-1 column repeats
    model._set("D1", "=MOD(B1:B3,4)");
    // a shorter array leaves #N/A past its end
    model._set("E1", "=MOD(B1:B3,C1:C2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("D1"), *"1");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"3");
    assert_eq!(model._get_text("E3"), *"#N/A");
}
