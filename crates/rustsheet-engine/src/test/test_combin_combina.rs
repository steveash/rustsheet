#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=COMBIN(5,2)");
    model._set("A2", "=COMBINA(5,2)");
    model._set("A3", "=COMBIN()");
    model._set("A4", "=COMBINA()");
    model._set("A5", "=COMBIN(2)");
    model._set("A6", "=COMBINA(2)");
    model._set("A7", "=COMBIN(1, 2, 3)");
    model._set("A8", "=COMBINA(1, 2, 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"10");
    assert_eq!(model._get_text("A2"), *"15");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#ERROR!");
}

#[test]
fn combin_overflow_is_bounded() {
    // The running product overflows within a few hundred factors, so a huge `k` returns #NUM!
    // at once instead of looping `k` times (directive 5). COMBIN(1030,515) is #NUM! in Excel
    // (probes/combin-algorithm-sweep).
    let mut model = new_empty_model();
    model._set("A1", "=COMBIN(1030,515)");
    model._set("A2", "=COMBIN(1E+15,5E+14)");
    model._set("A3", "=COMBIN(1E+300,1E+299)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
}

#[test]
fn combin_rounds_like_excel() {
    // probes/combin-algorithm-sweep (T-1598): the exact binomial is 84, 210, 455 and 252.
    let mut model = new_empty_model();
    model._set("A1", "=COMBIN(9,3)");
    model._set("A2", "=COMBIN(10,6)");
    model._set("A3", "=COMBIN(15,3)");
    model._set("A4", "=COMBIN(10,5)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 83.999_999_999_999_99_f64.to_bits());
    assert_eq!(bits("A2"), 209.999_999_999_999_97_f64.to_bits());
    assert_eq!(bits("A3"), 455.000_000_000_000_06_f64.to_bits());
    assert_eq!(bits("A4"), 252.0_f64.to_bits());
}
