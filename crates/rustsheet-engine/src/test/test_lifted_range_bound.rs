#![allow(clippy::unwrap_used)]

//! Lifting a range into an array is bounded before any cell is read (T-0047, CLAUDE.md
//! directive 5). Excel values: probes/lifted-range-size, probes/lifted-range-rows.

use crate::test::util::new_empty_model;

#[test]
fn whole_row_range_lifted_by_date_is_bounded() {
    let mut model = new_empty_model();
    // The formula_eval fuzz reproducer: 1,889 x 16,384 cells (31M).
    // DEVIATION (decision 0014): Excel builds it (ROWS = 1889, lifted-range-rows C1) and the
    // anchor shows #SPILL! (C3); we refuse arrays above one whole column.
    model._set("C1", "=DATE(11:1899,12,1)");
    model._set("C2", "=ROWS(DATE(11:1899,12,1))");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn whole_column_range_lifts() {
    let mut model = new_empty_model();
    model._set("A1", "-2");
    model._set("A2", "3");
    model._set("C4", "=ROWS(ABS(A:A))");
    model._set("C5", "=SUM(ABS(A:A))");
    model._set("C6", "=INDEX(ABS(A:A),2)");
    model._set("C7", "=ABS(A:A)");
    model._set("C8", "=SUM(--ISNUMBER(A:A))");
    model._set("C10", "=ROWS(A:A+0)");

    model.evaluate();

    // lifted-range-size C4-C8, C10 (excel-graph, 2026-10-06).
    assert_eq!(model._get_text("C4"), *"1048576");
    assert_eq!(model._get_text("C5"), *"5");
    assert_eq!(model._get_text("C6"), *"3");
    assert_eq!(model._get_text("C7"), *"#SPILL!");
    assert_eq!(model._get_text("C8"), *"2");
    assert_eq!(model._get_text("C10"), *"1048576");
}

#[test]
fn bare_range_spilled_from_a_cell_is_bounded() {
    let mut model = new_empty_model();
    // Fits the grid from A1, so only the allocation bound stops it (DEVIATION, decision 0014).
    model._set("A1", "=B:C");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn sumif_criteria_range_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("D1", "=SUMIF(A1:A2,B:C,A1:A2)");

    model.evaluate();

    // DEVIATION (decision 0014): a 2-column criteria grid is above the shared bound.
    assert_eq!(model._get_text("D1"), *"#ERROR!");
}
