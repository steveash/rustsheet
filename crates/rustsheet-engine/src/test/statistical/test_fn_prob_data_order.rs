#![allow(clippy::unwrap_used)]
//! PROB's data-argument order (T-2468; statistics.md "PROB: an error data argument first").

use crate::test::util::new_empty_model;

#[test]
fn prob_checks_data_arguments_before_limits() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "0.5");
    model._set("B2", "0.5");
    model._set("D1", "1");
    model._set("D2", "=NA()");
    model._set("D3", "3");
    model._set("F1", "0.2");
    model._set("F2", "0.3");
    model._set("F3", "0.5");
    model._set("C1", "=PROB(1/0,{0.5;0.5},1)");
    model._set("C2", "=PROB({1;2},NA(),\"a\")");
    model._set("C5", "=PROB(A1:A3,B1:B2,1)");
    model._set("C6", "=PROB(D1:D3,F1:F3,\"a\")");
    model._set("C7", "=PROB(H1,B1,1)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"#DIV/0!");
    assert_eq!(model._get_text("C2"), *"#N/A");
    assert_eq!(model._get_text("C5"), *"#N/A");
    assert_eq!(model._get_text("C6"), *"#VALUE!");
    assert_eq!(model._get_text("C7"), *"#VALUE!");
}
