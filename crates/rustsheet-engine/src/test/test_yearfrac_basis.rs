#![allow(clippy::panic)]
use crate::{cell::CellValue, test::util::new_empty_model};

#[test]
fn test_yearfrac_basis_2_actual_360() {
    let mut model = new_empty_model();

    // Non-leap span of exactly 360 days should result in 1.0
    model._set("A1", "=YEARFRAC(44561,44921,2)");

    // Leap-year span of 366 days: Jan 1 2020 → Jan 1 2021
    model._set("A2", "=YEARFRAC(43831,44197,2)");

    // Reverse order should yield negative value
    model._set("A3", "=YEARFRAC(44921,44561,2)");

    model.evaluate();

    // 360/360
    assert_eq!(model._get_text("A1"), *"1");

    // 366/360 ≈ 1.0166666667 (tolerance 1e-10)
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!A2") {
        assert!((v - 1.016_666_666_7).abs() < 1e-10);
    } else {
        panic!("Expected numeric value in A2");
    }

    // always positive A1
    assert_eq!(model._get_text("A3"), *"1");
}

#[test]
fn test_yearfrac_basis_3_actual_365() {
    let mut model = new_empty_model();

    // Non-leap span of exactly 365 days should result in 1.0
    model._set("B1", "=YEARFRAC(44561,44926,3)");

    // Leap-year span of 366 days
    model._set("B2", "=YEARFRAC(43831,44197,3)");

    // Same date should be 0
    model._set("B3", "=YEARFRAC(44561,44561,3)");

    model.evaluate();

    // 365/365
    assert_eq!(model._get_text("B1"), *"1");

    // 366/365 ≈ 1.002739726 (tolerance 1e-10)
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B2") {
        assert!((v - 1.002_739_726).abs() < 1e-10);
    } else {
        panic!("Expected numeric value in B2");
    }

    // Same date
    assert_eq!(model._get_text("B3"), *"0");
}

#[test]
// Only basis 1 has Excel evidence; the other bases' equality is our own expectation (dates.md).
fn test_yearfrac_reversed_dates_swap_for_every_basis() {
    let mut model = new_empty_model();
    for basis in 0..=4 {
        model._set(
            &format!("A{}", basis + 1),
            &format!("=YEARFRAC(24708,52874,{basis})"),
        );
        model._set(
            &format!("B{}", basis + 1),
            &format!("=YEARFRAC(52874,24708,{basis})"),
        );
    }
    model.evaluate();
    for row in 1..=5 {
        assert_eq!(
            model._get_text(&format!("A{row}")),
            model._get_text(&format!("B{row}"))
        );
    }
    // gen-m7/c263b43217a5898b Calc!A38 (Excel oracle)
    let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!B2") else {
        panic!("Expected numeric value in B2");
    };
    assert!((v - 77.112_951_912_951_91).abs() < 1e-12);
}

// Serial 0 is day 0 of January 1900 to the 30/360 count of basis 0: gen-m9/a2e51f48717af2a1
// Calc!A32 (Excel 0.9995471537332011 = (0.3 + 33268/360)^-0.0001), T-1409. Other bases stay #NUM!.
#[test]
fn test_yearfrac_basis_0_accepts_serial_0() {
    let mut model = new_empty_model();
    model._set("A1", "=YEARFRAC(33752,0)=33268/360");
    model._set("A2", "=YEARFRAC(0,33752)=33268/360");
    model._set("A3", "=YEARFRAC(33752,0.5)=33268/360");
    model._set("A4", "=YEARFRAC(0,0)");
    model._set("A5", "=((0.3+YEARFRAC(33752,0))^SUM((-0.0001)))");
    model._set("A6", "=YEARFRAC(33752,0,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0.999547154");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}
