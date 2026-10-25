#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn num(model: &mut crate::model::Model, cell: &str) -> f64 {
    model._get_text(cell).parse::<f64>().unwrap()
}

// VDB documented examples
// (https://support.microsoft.com/en-us/office/vdb-function-dde4e207-f3fa-488d-91d2-66d55e861d73)
#[test]
fn fn_vdb_microsoft_examples() {
    let mut model = new_empty_model();
    // First period of the double-declining-balance method (factor defaults to 2):
    // 2400 * 2 / 10 = 480
    model._set("A1", "=VDB(2400, 300, 10, 0, 1)");
    // Depreciation of the first 7/8 of a year with factor 1.5: 315
    model._set("A2", "=VDB(2400, 300, 10, 0, 0.875, 1.5)");
    // Months 6..18 with the default factor: ~396.31
    model._set("A3", "=VDB(2400, 300, 120, 6, 18)");
    model.evaluate();

    assert!((num(&mut model, "A1") - 480.0).abs() < 1e-6);
    assert!((num(&mut model, "A2") - 315.0).abs() < 1e-6);
    assert!((num(&mut model, "A3") - 396.306_138).abs() < 1e-3);
}

// The no_switch flag keeps the calculation on declining balance. With a slow
// (factor 1) declining balance the straight-line switch matters.
#[test]
fn fn_vdb_no_switch() {
    let mut model = new_empty_model();
    // First year is the same either way (240 = factor-1 declining balance).
    model._set("A1", "=VDB(2400, 300, 10, 0, 1, 1, TRUE)");
    // Whole life, allowing the switch, recovers cost - salvage = 2100.
    model._set("A2", "=VDB(2400, 300, 10, 0, 10, 1)");
    // Whole life with no switch stays on declining balance, recovering less.
    model._set("A3", "=VDB(2400, 300, 10, 0, 10, 1, TRUE)");
    model.evaluate();

    assert!((num(&mut model, "A1") - 240.0).abs() < 1e-6);
    assert!((num(&mut model, "A2") - 2100.0).abs() < 1e-6);
    assert!((num(&mut model, "A3") - 1_563.171_74).abs() < 1e-4);
    assert!(num(&mut model, "A3") < num(&mut model, "A2"));
}

#[test]
fn fn_vdb_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=VDB(2400, 300, 10, 5, 3)"); // start > end
    model._set("A2", "=VDB(2400, 300, -10, 0, 1)"); // life <= 0
    model._set("A3", "=VDB(2400, 300, 10)"); // too few args
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

// AMORLINC documented example
// (https://support.microsoft.com/en-us/office/amorlinc-function-7d417b45-f7f5-4dba-a0a5-3451a81079a8)
#[test]
fn fn_amorlinc_microsoft_example() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 1, 0.15, 1)",
    );
    model.evaluate();

    assert!((num(&mut model, "A1") - 360.0).abs() < 1e-6);
}

// AMORDEGRC documented example
// (https://support.microsoft.com/en-us/office/amordegrc-function-a14d0ca1-64a4-42eb-9b3d-b0dededf9e51)
#[test]
fn fn_amordegrc_microsoft_example() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=AMORDEGRC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 1, 0.15, 1)",
    );
    model.evaluate();

    assert!((num(&mut model, "A1") - 776.0).abs() < 1e-6);
}

// AMORLINC: period 0 returns the (prorated) first-period depreciation.
#[test]
fn fn_amorlinc_first_period() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 0, 0.15, 1)",
    );
    model.evaluate();

    // 134 / 366 * 0.15 * 2400
    let expected = 134.0 / 366.0 * 0.15 * 2400.0;
    assert!((num(&mut model, "A1") - expected).abs() < 1e-6);
}

#[test]
fn fn_amorlinc_errors() {
    let mut model = new_empty_model();
    // salvage >= cost
    model._set(
        "A1",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 2400, 1, 0.15, 1)",
    );
    // basis 2 (Actual/360) is not allowed
    model._set(
        "A2",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 1, 0.15, 2)",
    );
    // date_purchased >= first_period
    model._set(
        "A3",
        "=AMORLINC(2400, DATE(2009,1,1), DATE(2008,12,31), 300, 1, 0.15, 1)",
    );
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
}

#[test]
fn fn_forecast_ets_family_not_implemented() {
    let mut model = new_empty_model();
    model._set("A1", "=FORECAST.ETS(5, {1,2,3}, {1,2,3})");
    model._set("A2", "=FORECAST.ETS.CONFINT(5, {1,2,3}, {1,2,3})");
    model._set("A3", "=FORECAST.ETS.SEASONALITY({1,2,3}, {1,2,3})");
    model._set("A4", "=FORECAST.ETS.STAT({1,2,3}, {1,2,3}, 1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#N/IMPL!");
    assert_eq!(model._get_text("A2"), *"#N/IMPL!");
    assert_eq!(model._get_text("A3"), *"#N/IMPL!");
    assert_eq!(model._get_text("A4"), *"#N/IMPL!");
}

// Excel's exact doubles for VDB's two charge orders (docs/excel-semantics/financial.md
// "VDB charge order"; probes/fin-vdb-modes, fin-vdb-schedule, T-1975).
#[test]
fn fn_vdb_charge_order_bits() {
    let mut model = new_empty_model();
    model._set("A1", "=VDB(2400,300,10,7,10,2,FALSE)");
    model._set("A2", "=VDB(2400,300,10,7,10,2,TRUE)");
    model._set("A3", "=VDB(2400,300,10,0,10,3,TRUE)");
    model._set("A4", "=VDB(2400,0,10,5,6)");
    model._set("A5", "=VDB(1234.56,78.9,12,4,9,1.75,FALSE)");
    model._set("A6", "=VDB(2400,3000,10,0,1)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 203.316_480_000_000_07_f64.to_bits());
    assert_eq!(bits("A2"), 203.316_480_000_000_4_f64.to_bits());
    assert_eq!(bits("A3"), 2_099.999_999_999_999_5_f64.to_bits());
    assert_eq!(bits("A4"), 157.286_400_000_000_01_f64.to_bits());
    assert_eq!(bits("A5"), 379.885_448_741_749_9_f64.to_bits());
    assert_eq!(bits("A6"), (-600.0_f64).to_bits());
}

// RRI and DURATION use Excel's own operators (financial.md "DURATION and RRI operation order";
// probes/fin-mduration-rri-fit A13, A37, T-1975).
#[test]
fn fn_rri_and_duration_bits() {
    let mut model = new_empty_model();
    model._set("A1", "=RRI(10,1,200)");
    model._set(
        "A2",
        "=DURATION(DATE(2020,3,15),DATE(2025,9,30),0,0.06,2,0)",
    );
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 0.698_646_464_634_247_2_f64.to_bits());
    assert_eq!(bits("A2"), 5.541_666_666_666_668_f64.to_bits());
}

// DB keeps the remaining balance and DDB charges `min(old * rate, old - salvage)`, in Excel's
// operation order (financial.md "DB and DDB operation order"; ironcalc-calc DB_DDB.xlsx G20,
// G42, H10, probes/fin-argument-modes H95, T-2033).
#[test]
fn fn_db_ddb_operation_order_bits() {
    let mut model = new_empty_model();
    model._set("A1", "=DB(100,10,4,1,2)");
    model._set("A2", "=DB(23,10,4,5,2)");
    model._set("A3", "=DDB(100,10,4,3,1.3334)");
    model._set("A4", "=DDB(10000,1000,6,2,2)");
    model._set("A5", "=DDB(9695,282,20,8,1)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 7.299_999_999_999_999_f64.to_bits());
    assert_eq!(bits("A2"), 1.868_731_396_396_230_9_f64.to_bits());
    assert_eq!(bits("A3"), 14.814_814_787_037_497_f64.to_bits());
    assert_eq!(bits("A4"), 2_222.222_222_222_222_f64.to_bits());
    assert_eq!(bits("A5"), 338.519_004_281_445_3_f64.to_bits());
}
