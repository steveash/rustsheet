#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=FACT()");
    model._set("A2", "=FACTDOUBLE()");
    model._set("A3", "=FACT(3)");
    model._set("A4", "=FACTDOUBLE(3)");
    model._set("A5", "=FACT(1, 2)");
    model._set("A6", "=FACTDOUBLE(1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"6");
    assert_eq!(model._get_text("A4"), *"3");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

// Excel: numbers.md "FACT and FACTDOUBLE of a huge argument" (probes/fact-large-arguments).
#[test]
fn huge_arguments_are_num_without_looping() {
    let mut model = new_empty_model();
    for (i, f) in [
        "FACT(171)",
        "FACT(1E+15)",
        "FACT(1E+300)",
        "FACTDOUBLE(301)",
        "FACTDOUBLE(1E+15)",
        "FACTDOUBLE(1E+300)",
    ]
    .iter()
    .enumerate()
    {
        model._set(&format!("A{}", i + 1), &format!("={f}"));
    }
    model._set("B1", "=FACT(170.9)");
    model._set("B2", "=FACTDOUBLE(300)");
    model.evaluate();
    for i in 1..=6 {
        assert_eq!(model._get_text(&format!("A{i}")), *"#NUM!");
    }
    assert_eq!(model._get_text("B1"), *"7.2574E+306");
    assert_eq!(model._get_text("B2"), *"8.1544E+307");
}

#[test]
fn large_factorials_round_like_excel() {
    // Excel multiplies from n down: probes/fact-product-order-sweep A43, A171, B246, B247, B249
    // (the ascending product is off by up to ten ulps; T-1249).
    let mut model = new_empty_model();
    model._set("A1", "=FACT(42)");
    model._set("A2", "=FACT(170)");
    model._set("A3", "=FACTDOUBLE(297)");
    model._set("A4", "=FACTDOUBLE(298)");
    model._set("A5", "=FACTDOUBLE(300)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1.405_006_117_752_880_1e51_f64.to_bits());
    assert_eq!(bits("A2"), 7.257_415_615_308_004e306_f64.to_bits());
    assert_eq!(bits("A3"), 1.255_275_622_599_305_7e304_f64.to_bits());
    assert_eq!(bits("A4"), 2.718_138_023_126_864e305_f64.to_bits());
    assert_eq!(bits("A5"), 8.154_414_069_380_598e307_f64.to_bits());
}
