#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// probes/date-argument-modes (excel-graph, 2026-10-09): the ToolPak-era date functions lift over
// an array literal but are #VALUE! for a multi-cell range (docs/excel-semantics/dates.md).
#[test]
fn array_literal_lifts_and_range_is_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "45000.25");
    model._set("A2", "45031.5");
    model._set("A3", "45100.75");
    model._set("B1", "=INDEX(EDATE({45000.25;45031.5;45100.75},1),2)");
    model._set("B2", "=ROWS(EOMONTH({45000;45031},1))");
    model._set("B3", "=INDEX(EOMONTH({45000;45031},1),1)");
    model._set("B4", "=INDEX(NETWORKDAYS({45000;45031},45040),1)");
    model._set("B5", "=INDEX(WORKDAY({45000;45031},{1,2}),2,2)");
    model._set("B6", "=INDEX(WEEKNUM(45000,{1;2}),2)");
    model._set("B7", "=INDEX(YEARFRAC(45000,{45100;45200}),1)");
    model._set("B8", "=EDATE(A1:A3,1)");
    model._set("B9", "=INDEX(EDATE({45000;45031},TRUE),1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"45061");
    assert_eq!(model._get_text("B2"), *"2");
    assert_eq!(model._get_text("B3"), *"45046");
    assert_eq!(model._get_text("B4"), *"29");
    assert_eq!(model._get_text("B5"), *"45034");
    assert_eq!(model._get_text("B6"), *"12");
    assert_eq!(model._get_text("B7"), *"0.272222222");
    assert_eq!(model._get_text("B8"), *"#VALUE!");
    assert_eq!(model._get_text("B9"), *"#VALUE!");
}
