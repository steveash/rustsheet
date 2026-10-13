#![allow(clippy::unwrap_used)]

use super::results_independent_of_used_area;
use crate::test::util::new_empty_model;

#[test]
fn test_vlookup_range_sorted() {
    let mut model = new_empty_model();
    // First column is the (sorted) search column, second column holds the results.
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model._set("D1", "=VLOOKUP(20,A1:B3,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "b");
}

#[test]
fn test_vlookup_range_exact() {
    let mut model = new_empty_model();
    model._set("A1", "banana");
    model._set("A2", "apple");
    model._set("A3", "cherry");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("D1", r#"=VLOOKUP("apple",A1:B3,2,FALSE)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
}

#[test]
fn test_vlookup_array_sorted() {
    let mut model = new_empty_model();
    model._set("A1", "=VLOOKUP(20,{10,\"a\";20,\"b\";30,\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_vlookup_array_exact() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=VLOOKUP("banana",{"apple",1;"banana",2;"cherry",3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_vlookup_array_wildcard() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=VLOOKUP("ban*",{"apple",1;"banana",2;"cherry",3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_vlookup_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=VLOOKUP(99,{10,\"a\";20,\"b\";30,\"c\"},2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_vlookup_array_column_index_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "=VLOOKUP(20,{10,\"a\";20,\"b\";30,\"c\"},3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn test_vlookup_array_sorted_largest_smaller() {
    let mut model = new_empty_model();
    // 25 is not present, sorted search returns the largest value <= 25, i.e. 20.
    model._set("A1", "=VLOOKUP(25,{10,\"a\";20,\"b\";30,\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

// A whole-column search gives the results of a full walk. Only the exact
// matches are pinned: sorted search over the empty tail awaits T-0126.
#[test]
fn test_vlookup_full_column_independent_of_used_area() {
    let data = [
        ("A1", "10"),
        ("A2", "20"),
        ("A3", "30"),
        ("B1", "a"),
        ("B2", "b"),
        ("B3", "c"),
    ];
    let results = results_independent_of_used_area(
        &data,
        "D1",
        "Z1048576",
        &[
            "=VLOOKUP(20,A:B,2,FALSE)",
            "=VLOOKUP(99,A:B,2,FALSE)",
            "=VLOOKUP(1,A100:B200,2,FALSE)",
            "=VLOOKUP(99,A:B,2)",
            "=VLOOKUP(0,A:B,2,FALSE)",
            "=VLOOKUP(\"\",A:B,2,FALSE)",
        ],
    );
    assert_eq!(results[..3], ["b", "#N/A", "#N/A"]);
}

/// Data for the index-order tests: keys 1..3 in A, text in B, numbers in C, and index candidates
/// in E (text, numeric text, an error, a boolean); F1 stays blank.
fn index_order_model() -> crate::Model {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("B1", "x"),
        ("B2", "y"),
        ("B3", "z"),
        ("C1", "10"),
        ("C2", "20"),
        ("C3", "30"),
        ("E1", "a"),
        ("E2", "'1.5"),
        ("E3", "=1/0"),
        ("E4", "TRUE"),
    ] {
        model._set(cell, value);
    }
    model
}

#[test]
fn test_vlookup_not_found_beats_any_index() {
    // probes/lookup-index-order H19–H23, H26, H27 (excel-graph): the search runs first.
    let mut model = index_order_model();
    model._set("H1", "=VLOOKUP(9,A1:C3,-1,FALSE)");
    model._set("H2", "=VLOOKUP(9,A1:C3,4,FALSE)");
    model._set("H3", "=VLOOKUP(9,A1:C3,E1,FALSE)");
    model._set("H4", "=VLOOKUP(9,A1:C3,E3,FALSE)");
    model._set("H5", "=VLOOKUP(9,A1:C3,F1,FALSE)");
    model._set("H6", "=VLOOKUP(0,A1:C3,-1)");
    model._set("H7", "=VLOOKUP(5,{1,\"p\";2,\"q\"},3,FALSE)");
    model.evaluate();
    for cell in ["H1", "H2", "H3", "H4", "H5", "H6", "H7"] {
        assert_eq!(model._get_text(cell), "#N/A", "{cell}");
    }
}

#[test]
fn test_vlookup_found_validates_index() {
    // probes/lookup-index-order H1–H18 (excel-graph).
    let mut model = index_order_model();
    let cases = [
        ("=VLOOKUP(2,A1:C3,-1)", "#VALUE!"),
        ("=VLOOKUP(2,A1:C3,0.5)", "#VALUE!"),
        ("=VLOOKUP(2,A1:C3,-1E+15)", "#REF!"),
        ("=VLOOKUP(2,A1:C3,-2147483649)", "#REF!"),
        // probes/lookup-index-shapes H19-H24: the cut-off is the 32-bit minimum itself.
        ("=VLOOKUP(2,A1:C3,-2147483648)", "#REF!"),
        ("=VLOOKUP(2,A1:C3,-2147483647)", "#VALUE!"),
        ("=VLOOKUP(2,A1:C3,-32768)", "#VALUE!"),
        ("=VLOOKUP(2,A1:C3,4)", "#REF!"),
        ("=VLOOKUP(2,A1:C3,1E+15)", "#REF!"),
        // A referenced cell that is not a number is #REF!, a blank one is 0.
        ("=VLOOKUP(2,A1:C3,E1)", "#REF!"),
        ("=VLOOKUP(2,A1:C3,E3)", "#REF!"),
        ("=VLOOKUP(2,A1:C3,F1)", "#VALUE!"),
        // Numeric text and booleans coerce.
        ("=VLOOKUP(2,A1:C3,E2)", "2"),
        ("=VLOOKUP(2,A1:C3,\"2.9\")", "y"),
        ("=VLOOKUP(2,A1:C3,E4)", "2"),
        ("=VLOOKUP(2,A1:C3,TRUE)", "2"),
    ];
    let cells: Vec<String> = (1..=cases.len()).map(|i| format!("H{i}")).collect();
    for (cell, (formula, _)) in cells.iter().zip(cases) {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, (formula, expected)) in cells.iter().zip(cases) {
        assert_eq!(model._get_text(cell), expected, "{formula}");
    }
}

#[test]
fn test_vlookup_value_index_fails_before_search() {
    // probes/lookup-index-order H9, H13, H24, H28, H33 (excel-graph): a value (not a reference)
    // index is coerced before the search, so it fails even when nothing is found.
    let mut model = index_order_model();
    model._set("H1", "=VLOOKUP(0,A1:C3,\"a\")");
    model._set("H2", "=VLOOKUP(9,A1:C3,1/0,FALSE)");
    model._set("H3", "=VLOOKUP(2,A1:C3,\"a\",1/0)");
    model._set("H4", "=VLOOKUP(2,A1:C3,\"\")");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "#VALUE!");
    assert_eq!(model._get_text("H2"), "#DIV/0!");
    assert_eq!(model._get_text("H3"), "#VALUE!");
    assert_eq!(model._get_text("H4"), "#VALUE!");
}

#[test]
fn test_vlookup_index_function_index_is_a_reference() {
    // probes/lookup-index-shapes H2, H3, H11, H12 (excel-graph): an INDEX(...) index is a
    // reference, so a text or error cell is #REF! once the value is found and not found is #N/A.
    let mut model = index_order_model();
    model._set("H1", "=VLOOKUP(2,A1:C3,INDEX(E1:E4,1))");
    model._set("H2", "=VLOOKUP(2,A1:C3,INDEX(E1:E4,3))");
    model._set("H3", "=VLOOKUP(9,A1:C3,INDEX(E1:E4,1),FALSE)");
    model._set("H4", "=VLOOKUP(9,A1:C3,INDEX(E1:E4,3),FALSE)");
    model._set("H5", "=VLOOKUP(2,A1:C3,INDEX(E1:E4,4))");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "#REF!");
    assert_eq!(model._get_text("H2"), "#REF!");
    assert_eq!(model._get_text("H3"), "#N/A");
    assert_eq!(model._get_text("H4"), "#N/A");
    assert_eq!(model._get_text("H5"), "2");
}

#[test]
fn array_lookup_value_lifts_element_by_element() {
    // probes/vlookup-array-lookup-value (excel-graph, T-0270). The spill shape follows the
    // lookup value, so a 2-row lookup in a dynamic cell spills without the scalar-cell assert.
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("B1", "one"), ("B2", "two")] {
        model._set(cell, value);
    }
    model._set("D1", "=VLOOKUP({2;9},A1:B2,2,FALSE)");
    model._set("E1", "=VLOOKUP(A1:A2,A1:B2,2,FALSE)");
    model._set("F1", "=VLOOKUP({2;9},A1:B2,0,FALSE)");
    model._set("G1", "=HLOOKUP({1,2},A1:B2,2,FALSE)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"two");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("E1"), *"one");
    assert_eq!(model._get_text("E2"), *"two");
    // Search first: the found element fails the index, the missing one is #N/A.
    assert_eq!(model._get_text("F1"), *"#VALUE!");
    assert_eq!(model._get_text("F2"), *"#N/A");
    assert_eq!(model._get_text("G1"), *"2");
    assert_eq!(model._get_text("H1"), *"#N/A");
}

#[test]
fn range_lookup_range_lifts_and_spills() {
    // gen-m2/04cfd82f54dbc2b2 Calc!A19 (excel-graph, T-0203): a multi-cell range range_lookup lifts like an array; the
    // first element is range_lookup FALSE (exact), the second TRUE (approximate).
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "3"), ("B1", "one"), ("B2", "three")] {
        model._set(cell, value);
    }
    model._set("C1", "FALSE");
    model._set("C2", "TRUE");
    model._set("D1", "=VLOOKUP(2,A1:B2,2,C1:C2)");
    // A spill collision: F2 is occupied.
    model._set("F1", "=VLOOKUP(2,A1:B2,2,C1:C2)");
    model._set("F2", "x");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"#N/A");
    assert_eq!(model._get_text("D2"), *"one");
    assert_eq!(model._get_text("F1"), *"#SPILL!");
}

#[test]
fn lifted_lookup_over_an_array_table_hits_the_visit_cap() {
    // A computed table is charged its length per element (T-0270, CLAUDE.md directive 5): 2^20
    // lookups over a 2^20-row array would be ~10^12 comparisons. DEVIATION (decision 0014):
    // Excel computes it; we refuse past the shared cell-visit cap.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=ROWS(VLOOKUP(SEQUENCE(1048576),SEQUENCE(1048576,2),2,FALSE))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn failed_arguments_still_lift_with_a_range_range_lookup() {
    // gen-m2/f7e6d351c8538ef5 Calc!A296 and 942bdade381b7537 Calc!A381 (excel-graph, T-1468): an
    // error lookup value or index does not stop a multi-cell range_lookup lifting, so the
    // formula spills (here: two rows) instead of returning the error.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("C1", "TRUE");
    model._set("C2", "TRUE");
    model._set("D1", "=VLOOKUP(1/0,A1:A2,1,C1:C2)");
    model._set("E1", "=VLOOKUP(1,A1:A2,1/0,C1:C2)");
    model._set("F1", "=VLOOKUP(1/0,A1:A2,1,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"#DIV/0!");
    assert_eq!(model._get_text("D2"), *"#DIV/0!");
    // Hand expectation: the element value of an error index was never seen (lookup.md, unprobed).
    assert!(model._get_text("E1").starts_with('#'));
    assert!(model._get_text("E2").starts_with('#'));
    assert_eq!(model._get_text("F1"), *"#DIV/0!");
    assert_eq!(model._get_text("F2"), *"");
}
