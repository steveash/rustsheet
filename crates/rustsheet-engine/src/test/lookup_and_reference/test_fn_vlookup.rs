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
