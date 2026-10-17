#![allow(clippy::unwrap_used)]

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
