#![allow(clippy::unwrap_used)]

//! IFS and SWITCH lift element by element over array and range conditions and SWITCH
//! expressions (probes/ifs-switch-array-values, excel-graph; docs/excel-semantics/logic.md).

use crate::test::util::new_empty_model;

// A1:B2 = 1,2;3,4
fn model() -> crate::model::Model {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B1", "2"), ("A2", "3"), ("B2", "4")] {
        model._set(cell, value);
    }
    model
}

#[test]
fn ifs_lifts_over_array_and_range_conditions() {
    let mut model = model();
    model._set("D1", "=IFS({TRUE,FALSE},1,TRUE,2)");
    model._set("D3", "=IFS(A1:B2>1,5)");
    // an earlier FALSE scalar condition, then an array one
    model._set("D6", "=IFS(FALSE,9,{FALSE;TRUE},7,TRUE,8)");
    // an error element is that element's result
    model._set("F1", "=IFS({TRUE,#DIV/0!},1)");
    // a value argument broadcasts against the condition (probes/ifs-switch-lift-edges)
    model._set("H1", "=IFS({TRUE,FALSE},{10,20},TRUE,{30,40})");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("D3"), "#N/A");
    assert_eq!(model._get_text("E3"), "5");
    assert_eq!(model._get_text("D4"), "5");
    assert_eq!(model._get_text("E4"), "5");
    assert_eq!(model._get_text("D6"), "8");
    assert_eq!(model._get_text("D7"), "7");
    assert_eq!(model._get_text("F1"), "1");
    assert_eq!(model._get_text("G1"), "#DIV/0!");
    assert_eq!(model._get_text("H1"), "10");
    assert_eq!(model._get_text("I1"), "40");
}

#[test]
fn ifs_with_scalar_conditions_keeps_a_range_value() {
    let mut model = model();
    model._set("D1", "=SUM(IFS(TRUE,A1:B2))");
    model._set("D2", "=IFS(FALSE,1,TRUE,\"x\")");
    model._set("D3", "=IFS(FALSE,1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("D2"), "x");
    assert_eq!(model._get_text("D3"), "#N/A");
}

#[test]
fn switch_lifts_over_an_array_or_range_expression() {
    let mut model = model();
    model._set("D1", "=SWITCH({1,2},1,\"a\",2,\"b\")");
    model._set("D3", "=SWITCH(A1:B2,1,\"a\",\"z\")");
    // no default: #N/A where nothing matches
    model._set("D6", "=SWITCH({1,5},1,\"a\")");
    // an error element is that element's result
    model._set("D8", "=SWITCH({1,#N/A},1,\"a\",\"z\")");
    // a value argument broadcasts against the expression (probes/ifs-switch-lift-edges)
    model._set("D10", "=SWITCH({1,2},1,{10,20},{30,40})");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "a");
    assert_eq!(model._get_text("E1"), "b");
    assert_eq!(model._get_text("D3"), "a");
    assert_eq!(model._get_text("E3"), "z");
    assert_eq!(model._get_text("D4"), "z");
    assert_eq!(model._get_text("E4"), "z");
    assert_eq!(model._get_text("D6"), "a");
    assert_eq!(model._get_text("E6"), "#N/A");
    assert_eq!(model._get_text("D8"), "a");
    assert_eq!(model._get_text("E8"), "#N/A");
    assert_eq!(model._get_text("D10"), "10");
    assert_eq!(model._get_text("E10"), "40");
}

#[test]
fn switch_with_a_scalar_expression_keeps_a_range_value() {
    let mut model = model();
    model._set("D1", "=SUM(SWITCH(1,1,A1:B2))");
    model._set("D2", "=SWITCH(9,1,5)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("D2"), "#N/A");
}

/// An array case value lifts SWITCH over a scalar expression (probes/subtotal-take-array-selector-b
/// A10, T-2012), and a nest of them reads each case once: 2^depth reads would not finish.
#[test]
fn switch_lifts_over_an_array_case_value_and_nests_linearly() {
    let mut model = model();
    model._set("D1", "=SWITCH(1,{1,2},10,20,30,40)");
    let mut nested = "SWITCH(1,{1,2},10,20)".to_string();
    for _ in 0..40 {
        nested = format!("SWITCH(1,{nested},10,20)");
    }
    model._set("D3", &format!("={nested}"));
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("E1"), "40");
    assert!(!model._get_text("D3").is_empty());
}

/// A one-cell range condition (`INDIRECT("K1")`) picks a branch like a cell does, and a missing
/// `value_if_false` gives FALSE (probes/subtotal-take-array-selector-b A130, T-2012).
#[test]
fn if_reads_a_one_cell_range_condition_as_a_scalar() {
    let mut model = model();
    model._set("K1", "=FALSE");
    model._set("K2", "=TRUE");
    model._set("D1", "=IF(INDIRECT(\"K1\"),1,2)");
    model._set("D2", "=IF(INDIRECT(\"K1\"),1)");
    model._set("D3", "=IF(INDIRECT(\"K2\"),1,2)");
    model._set("D4", "=IF(INDIRECT(\"K1\"))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("D2"), "FALSE");
    assert_eq!(model._get_text("D3"), "1");
    assert_eq!(model._get_text("D4"), "#VALUE!");
}
