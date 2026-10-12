#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// docs/excel-semantics/lookup.md "XLOOKUP with an array lookup_value looks up each element"
// (probes/xlookup-array-lookup-value, probes/xlookup-array-lookup-value-args; T-1264).

#[test]
fn whole_columns_with_many_distinct_values_are_not_refused() {
    // The cost is the evaluated cells of the lookup column, not its 1048576 rows, so 300
    // distinct lookups over whole columns stay far below the visit cap.
    let mut model = new_empty_model();
    for row in 1..=300 {
        model._set(&format!("A{row}"), &row.to_string());
        model._set(&format!("B{row}"), &row.to_string());
        model._set(&format!("C{row}"), &(row * 2).to_string());
    }
    model._set("E1", "=SUM(XLOOKUP(A1:A300,B:B,C:C))");
    model._set("E2", "=SUM(XLOOKUP(A:A,B:B,C:C))");
    model.evaluate();
    assert_eq!(model._get_text("E1"), *"90300");
    assert_eq!(model._get_text("E2"), *"90300");
}

#[test]
fn lifted_lookup_over_a_computed_array_hits_the_visit_cap() {
    // A computed lookup_array is charged its length per element (CLAUDE.md directive 5):
    // 2^20 lookups over a 2^20-element array would be ~10^12 comparisons. DEVIATION
    // (decision 0014): Excel computes it; we refuse past the shared cell-visit cap.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=ROWS(XLOOKUP(SEQUENCE(1048576),SEQUENCE(1048576),SEQUENCE(1048576)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn modes_and_argument_errors_lift_per_element() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("B1", "one"),
        ("B2", "two"),
        ("B3", "three"),
    ] {
        model._set(cell, value);
    }
    // match_mode and search_mode lift pairwise with lookup_value (A49, A52).
    model._set("D1", "=XLOOKUP({2.5;2.5},A1:A3,B1:B3,,{0;-1})");
    model._set("E1", "=XLOOKUP({1;1},{1;1},{\"f\";\"l\"},,0,{1;-1})");
    // An argument error is repeated per element, after a lookup_value error (A3, A64).
    model._set("F1", "=XLOOKUP({2;#N/A},A1:A3,B1:B3,,5)");
    // An array if_not_found gives its first element, a reference #VALUE! (A27, A58).
    model._set("G1", "=XLOOKUP({2;9},A1:A3,B1:B3,{\"x\";\"y\"})");
    model._set("H1", "=XLOOKUP({9;2},A1:A3,B1:B3,A1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"#N/A");
    assert_eq!(model._get_text("D2"), *"two");
    assert_eq!(model._get_text("E1"), *"f");
    assert_eq!(model._get_text("E2"), *"l");
    assert_eq!(model._get_text("F1"), *"#VALUE!");
    assert_eq!(model._get_text("F2"), *"#N/A");
    assert_eq!(model._get_text("G1"), *"two");
    assert_eq!(model._get_text("G2"), *"x");
    assert_eq!(model._get_text("H1"), *"#VALUE!");
    assert_eq!(model._get_text("H2"), *"two");
}

#[test]
fn row_lookup_with_a_taller_return_range_gives_the_matched_column() {
    // probes/xlookup-array-lookup-value-args C33 (scalar) and A33 (lifted: first cell only).
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "a"),
        ("B1", "b"),
        ("A2", "10"),
        ("B2", "20"),
        ("A3", "100"),
        ("B3", "200"),
    ] {
        model._set(cell, value);
    }
    model._set("D1", "=XLOOKUP(\"b\",A1:B1,A2:B3)");
    model._set("E1", "=XLOOKUP({\"b\";\"a\"},A1:B1,A2:B3)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"20");
    assert_eq!(model._get_text("D2"), *"200");
    assert_eq!(model._get_text("E1"), *"20");
    assert_eq!(model._get_text("E2"), *"10");
}
