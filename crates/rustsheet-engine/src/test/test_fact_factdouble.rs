#![allow(clippy::unwrap_used)]

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
