#![allow(clippy::unwrap_used)]

use super::results_independent_of_used_area;
use crate::test::util::new_empty_model;

#[test]
fn test_hlookup_range_sorted() {
    let mut model = new_empty_model();
    // First row is the (sorted) search row, second row holds the results.
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A2", "a");
    model._set("B2", "b");
    model._set("C2", "c");
    model._set("E1", "=HLOOKUP(20,A1:C2,2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "b");
}

#[test]
fn test_hlookup_range_exact() {
    let mut model = new_empty_model();
    model._set("A1", "banana");
    model._set("B1", "apple");
    model._set("C1", "cherry");
    model._set("A2", "1");
    model._set("B2", "2");
    model._set("C2", "3");
    model._set("E1", r#"=HLOOKUP("apple",A1:C2,2,FALSE)"#);
    model.evaluate();
    assert_eq!(model._get_text("E1"), "2");
}

#[test]
fn test_hlookup_array_sorted() {
    let mut model = new_empty_model();
    model._set("A1", "=HLOOKUP(20,{10,20,30;\"a\",\"b\",\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_hlookup_array_exact() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=HLOOKUP("banana",{"apple","banana","cherry";1,2,3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_hlookup_array_wildcard() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=HLOOKUP("ban*",{"apple","banana","cherry";1,2,3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_hlookup_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=HLOOKUP(99,{10,20,30;\"a\",\"b\",\"c\"},2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_hlookup_array_row_index_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "=HLOOKUP(20,{10,20,30;\"a\",\"b\",\"c\"},3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn test_hlookup_array_sorted_largest_smaller() {
    let mut model = new_empty_model();
    // 25 is not present, sorted search returns the largest value <= 25, i.e. 20.
    model._set("A1", "=HLOOKUP(25,{10,20,30;\"a\",\"b\",\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

// A whole-row search gives the results of a full walk. Only the exact
// matches are pinned: sorted search over the empty tail awaits T-0126.
#[test]
fn test_hlookup_full_row_independent_of_used_area() {
    let data = [
        ("A1", "10"),
        ("B1", "20"),
        ("C1", "30"),
        ("A2", "a"),
        ("B2", "b"),
        ("C2", "c"),
    ];
    let results = results_independent_of_used_area(
        &data,
        "A3",
        "XFD5",
        &[
            "=HLOOKUP(20,1:2,2,FALSE)",
            "=HLOOKUP(99,1:2,2,FALSE)",
            "=HLOOKUP(1,Z1:XFD2,2,FALSE)",
            "=HLOOKUP(99,1:2,2)",
            "=HLOOKUP(FALSE,1:2,2,FALSE)",
            "=HLOOKUP(0,1:2,2,FALSE)",
            "=HLOOKUP(\"\",1:2,2,FALSE)",
        ],
    );
    assert_eq!(results[..3], ["b", "#N/A", "#N/A"]);
}

#[test]
fn test_hlookup_whole_row_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,HLOOKUP(5,4:4,1,FALSE))");
    model._set("A3", "=SEQUENCE(1,3,10)");
    model._set("A4", "=SEQUENCE(1,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
}
