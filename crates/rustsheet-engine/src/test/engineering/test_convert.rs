use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_convert() {
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(1, "lbm", "kg")"#);
    model._set("A2", r#"=CONVERT(68, "F", "C")"#);
    model._set("A3", r#"=CONVERT(2.5, "ft", "sec")"#);
    model._set("A4", r#"=CONVERT(CONVERT(100,"ft","m"),"ft","m""#);

    model._set("B1", "6");

    model._set("A5", r#"=CONVERT(B1,"C","F")"#);
    model._set("A6", r#"=CONVERT(B1,"tsp","tbs")"#);
    model._set("A7", r#"=CONVERT(B1,"gal","l")"#);
    model._set("A8", r#"=CONVERT(B1,"mi","km")"#);
    model._set("A9", r#"=CONVERT(B1,"km","mi")"#);
    model._set("A10", r#"=CONVERT(B1,"in","ft")"#);
    model._set("A11", r#"=CONVERT(B1,"cm","in")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.45359237");
    assert_eq!(model._get_text("A2"), "20");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "9.290304");

    assert_eq!(model._get_text("A5"), "42.8");
    assert_eq!(model._get_text("A6"), "2");
    assert_eq!(model._get_text("A7"), "22.7124707"); //22.71741274");
    assert_eq!(model._get_text("A8"), "9.656064");
    assert_eq!(model._get_text("A9"), "3.728227153");
    assert_eq!(model._get_text("A10"), "0.5");
    assert_eq!(model._get_text("A11"), "2.362204724");
}

#[test]
fn convert_matches_excel_bits() {
    // Excel-cached values: ironcalc-calc ENGINEERING/CONVERT.xlsx (docs/excel-semantics/
    // engineering.md "CONVERT constants and evaluation order").
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(1,"sg","g")"#); // Weight!C3
    model._set("A2", r#"=CONVERT(0.05,"Mu","g")"#); // Weight!G5
    model._set("A3", r#"=CONVERT(4,"parsec","m")"#); // distance!G11
    model._set("A4", r#"=CONVERT(1,"pica","m")"#); // distance!C15
    model._set("A5", r#"=CONVERT(0.0002,"Mang3","m3")"#); // Volume!J17
    model._set("A6", r#"=CONVERT(0.0002,"Muk_pt","m3")"#); // Volume!J9
    model._set("A7", r#"=CONVERT(5,"cal","J")"#); // Energy!G5
    model._set("A8", r#"=CONVERT(5,"eV","J")"#); // Energy!G6
    model._set("A9", r#"=CONVERT(0.05,"Mdyn","N")"#); // Force!J3
    model._set("A10", r#"=CONVERT(1,"Picapt2","m2")"#); // Area!C21
    model._set("A11", r#"=CONVERT(1,"F","K")"#); // Temperature!C4
    model._set("A12", r#"=CONVERT(0.004,"Mm/h","m/s")"#); // Speed!J4
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 14_593.902_937_206_363_f64.to_bits());
    assert_eq!(bits("A2"), 8.302_693_91e-20_f64.to_bits());
    assert_eq!(bits("A3"), 1.234_271_032_512_621_3e17_f64.to_bits());
    assert_eq!(bits("A4"), 0.004_233_333_333_333_334_f64.to_bits());
    assert_eq!(bits("A5"), 2.000_000_000_000_000_2e-16_f64.to_bits());
    assert_eq!(bits("A6"), 0.113_652_250_000_000_02_f64.to_bits());
    assert_eq!(bits("A7"), 20.934_f64.to_bits());
    assert_eq!(bits("A8"), 8.010_882_434_999_999e-19_f64.to_bits());
    assert_eq!(bits("A9"), 0.5_f64.to_bits());
    assert_eq!(bits("A10"), 1.244_521_604_938_271_5e-7_f64.to_bits());
    assert_eq!(bits("A11"), 255.927_777_777_777_75_f64.to_bits());
    assert_eq!(bits("A12"), 1.111_111_111_111_111_f64.to_bits());
}
