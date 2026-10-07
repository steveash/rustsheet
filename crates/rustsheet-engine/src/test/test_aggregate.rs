#![allow(clippy::unwrap_used)]

//! AGGREGATE options over hidden rows and nested SUBTOTAL cells, which the probe case format
//! cannot express (docs/excel-semantics/numbers.md "AGGREGATE", UNVERIFIED parts).
use crate::test::util::new_empty_model;

#[test]
fn hidden_rows_and_nested_subtotals() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "4");
    model._set("A4", "=SUBTOTAL(9,A1:A2)");
    model.set_row_hidden(0, 2, true).unwrap();
    model._set("B1", "=AGGREGATE(9,4,A1:A4)");
    model._set("B2", "=AGGREGATE(9,5,A1:A4)");
    model._set("B3", "=AGGREGATE(9,0,A1:A4)");
    model._set("B4", "=AGGREGATE(9,1,A1:A4)");
    model._set("B5", "=AGGREGATE(14,5,A1:A4,1)");
    model.evaluate();

    // Option 4 keeps everything; 5 drops hidden row 2; 0 drops the SUBTOTAL cell; 1 both.
    assert_eq!(model._get_text("B1"), *"10");
    assert_eq!(model._get_text("B2"), *"8");
    assert_eq!(model._get_text("B3"), *"7");
    assert_eq!(model._get_text("B4"), *"5");
    assert_eq!(model._get_text("B5"), *"4");
}

#[test]
fn array_function_num_spills() {
    // probes/aggregate-args Misc!A37:B37: one AGGREGATE per element (SUM, then MAX).
    let mut model = new_empty_model();
    for (i, v) in [3, 1, 4, 1, 5, 9].iter().enumerate() {
        model._set(&format!("E{}", i + 1), &v.to_string());
    }
    model._set("A1", "=AGGREGATE({9,4},0,E1:E6)");
    model._set("A3", "=AGGREGATE(9,0,E1:E6)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"23");
    assert_eq!(model._get_text("B1"), *"9");
    assert_eq!(model._get_text("A3"), *"23");
}

#[test]
fn lifted_calls_over_large_arrays_hit_the_visit_cap() {
    // 2^20 elements, each reading a 2^20-element computed array: far above the 2^27 cap
    // (directive 5), so the call is #ERROR! instead of running ~10^12 steps.
    let mut model = new_empty_model();
    model._set("A1", "=AGGREGATE(14,6,SEQUENCE(1048576),SEQUENCE(1048576))");
    model._set("A2", "=AGGREGATE(14,6,SEQUENCE(1000),{1,2,3})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"1000");
    assert_eq!(model._get_text("C2"), *"998");
}
