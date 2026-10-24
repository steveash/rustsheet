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
// Basis 1: gen-m7/c263b43217a5898b; bases 0, 2, 3, 4: probes/yearfrac-reversed-bases (dates.md).
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

    // Day 31 edge: the reversed basis-0 pair is 76/360, not 75/360 (probes/yearfrac-reversed-bases E1).
    model._set("D1", "=YEARFRAC(DATE(2024,3,31),DATE(2024,1,15),0)*360");
    model.evaluate();
    let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref("Sheet1!D1") else {
        panic!("Expected numeric value in D1");
    };
    assert!((v - 76.0).abs() < 1e-9);
}

// Serial 0 is day 0 of January 1900 to the 30/360 count of basis 0: gen-m9/a2e51f48717af2a1
// Calc!A32 (Excel 0.9995471537332011 = (0.3 + 33268/360)^-0.0001), T-1409.
#[test]
fn test_yearfrac_basis_0_serial_0_and_negative_serial() {
    let mut model = new_empty_model();
    model._set("A1", "=YEARFRAC(33752,0)=33268/360");
    model._set("A2", "=YEARFRAC(0,33752)=33268/360");
    model._set("A3", "=YEARFRAC(33752,0.5)=33268/360");
    model._set("A4", "=YEARFRAC(0,0)");
    model._set("A5", "=((0.3+YEARFRAC(33752,0))^SUM((-0.0001)))");
    model._set("A6", "=YEARFRAC(-0.5,33752)");
    model._set("B1", "=YEARFRAC(31,0)=YEARFRAC(0,31)");
    model._set("B2", "=YEARFRAC(59,0)=YEARFRAC(0,59)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"TRUE");
    assert_eq!(model._get_text("B2"), *"TRUE");
    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0.999547154");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}

/// The numeric value of `cell`, panicking otherwise.
fn number_at(model: &crate::Model, cell: &str) -> f64 {
    match model.get_cell_value_by_ref(&format!("Sheet1!{cell}")) {
        Ok(CellValue::Number(v)) => v,
        other => panic!("{cell}: expected a number, got {other:?}"),
    }
}

// Serial 0 is a date on every basis, and next to the fictitious 1900-02-29 each basis reads the
// dates its own way (probes/yearfrac-serial-zero, excel-graph, T-2237; dates.md "YEARFRAC accepts
// serial 0").
#[test]
fn test_yearfrac_serial_0_and_1900_02_29_on_every_basis() {
    let mut model = new_empty_model();
    let cells = [
        // Basis 0: Excel's own calendar, 1900-02-28 is February's last day (day 30).
        ("A1", "=YEARFRAC(60,59,0)", -1.0 / 360.0),
        ("A2", "=YEARFRAC(60,61,0)", 2.0 / 360.0),
        ("A3", "=YEARFRAC(0,60,0)", 59.0 / 360.0),
        ("A4", "=YEARFRAC(0,366,0)", 1.002_777_777_777_777_8),
        // Basis 1: serial days over the real calendar's average year, 1900-1992.
        ("A5", "=YEARFRAC(0,33752,1)", 92.408_619_877_531_8),
        ("A6", "=YEARFRAC(1,33752,1)", 92.405_882_006_594_4),
        ("A7", "=YEARFRAC(59,60,1)", 1.0 / 365.0),
        ("A8", "=YEARFRAC(0,60,1)", 60.0 / 365.0),
        // Bases 2 and 3: serial days.
        ("A9", "=YEARFRAC(0.5,33752,2)", 33752.0 / 360.0),
        ("A10", "=YEARFRAC(0.9999,60,3)", 60.0 / 365.0),
        // Basis 4: serial 0 is day 0, serial 60 day 29.
        ("A11", "=YEARFRAC(0,31,4)", 30.0 / 360.0),
        ("A12", "=YEARFRAC(0,366,4)", 1.0),
        ("A13", "=YEARFRAC(60,61,4)", 2.0 / 360.0),
        ("A14", "=YEARFRAC(0.5,0.25,4)", 0.0),
        // The discount securities share the parts, sign included (probes/discount-serial-zero).
        (
            "A15",
            "=ACCRINTM(59,60,0.1,1000,0)",
            -0.277_777_777_777_777_8,
        ),
        ("A16", "=DISC(0,366,95,100,1)", 0.049_863_387_978_142_12),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        let actual = number_at(&model, cell);
        assert!(
            (actual - expected).abs() <= 1e-15 * expected.abs().max(1.0),
            "{formula}: {actual} != {expected}"
        );
    }
}

#[test]
fn test_negative_basis_is_num_error() {
    // Excel: any basis below 0 is #NUM!, even -1E-300 or -0.5, while 4.9 truncates to 4
    // (probes/fin-bond-negative-basis).
    let mut model = new_empty_model();
    model._set("A1", "=YEARFRAC(39083,39448,-0.5)");
    model._set("A2", "=YEARFRAC(39083,39448,-1E-300)");
    model._set("A3", "=YEARFRAC(39083,39448,4.9)");
    model._set("A4", "=COUPDAYS(39448,43831,2,-0.5)");
    model._set("A5", "=PRICE(39448,43831,0.0575,0.065,100,2,-1E-300)");
    model._set("A6", "=ODDLYIELD(39448,39630,39264,0.08,98,100,2,-0.5)");
    model._set("A7", "=COUPDAYS(39448,43831,2,4.9)");
    model._set("A8", "=AMORLINC(2400,39679,39813,300,1,0.15,-0.5)");
    model._set("A9", "=ACCRINT(39083,39264,39448,0.08,1000,2,-1E-300)");
    model.evaluate();
    for cell in ["A1", "A2", "A4", "A5", "A6", "A8", "A9"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A7"), *"180");
}

// Serials 1-60 on bases 0 and 1: basis 0 reads the displayed date (serial 60 is 29 February, with
// no end-of-February adjustment), basis 1 counts serial days over the real calendar's average year
// (probes/yearfrac-early-1900 A1-A22, A61-A70, excel-graph, T-2265; dates.md "Serials below 61 in
// DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC").
#[test]
fn test_yearfrac_serials_1_to_60_on_bases_0_and_1() {
    let mut model = new_empty_model();
    let cells = [
        ("A1", "=YEARFRAC(1,45000,0)", 123.205_555_555_555_55),
        ("A2", "=YEARFRAC(59,45000,0)", 123.041_666_666_666_67),
        ("A3", "=YEARFRAC(60,45000,0)", 123.044_444_444_444_45),
        ("A4", "=YEARFRAC(60.5,366,0)", 0.838_888_888_888_888_9),
        ("A5", "=YEARFRAC(60,425,0)", 0.997_222_222_222_222_2),
        ("A6", "=YEARFRAC(1,45000,1)", 123.203_267_829_542_94),
        ("A7", "=YEARFRAC(59,366,1)", 0.841_095_890_410_958_9),
        ("A8", "=YEARFRAC(60,45000,1)", 123.041_731_066_460_58),
        ("A9", "=YEARFRAC(60.5,366,1)", 0.838_356_164_383_561_6),
        ("A10", "=YEARFRAC(60,1520,1)", 3.997_809_419_496_166_4),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        let actual = number_at(&model, cell);
        assert!(
            (actual - expected).abs() <= 1e-15 * expected.abs().max(1.0),
            "{formula}: {actual} != {expected}"
        );
    }
}
