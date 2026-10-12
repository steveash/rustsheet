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

#[test]
fn test_hlookup_search_before_index() {
    // probes/lookup-index-order H37–H46 (excel-graph).
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "2"),
        ("C1", "3"),
        ("A2", "x"),
        ("A3", "10"),
        ("E1", "a"),
        ("E2", "'1.5"),
    ] {
        model._set(cell, value);
    }
    model._set("H1", "=HLOOKUP(1,A1:C3,0)");
    model._set("H2", "=HLOOKUP(1,A1:C3,4)");
    model._set("H3", "=HLOOKUP(9,A1:C3,4,FALSE)");
    model._set("H4", "=HLOOKUP(9,A1:C3,\"a\",FALSE)");
    model._set("H5", "=HLOOKUP(1,A1:C3,E1)");
    model._set("H6", "=HLOOKUP(9,A1:C3,E1,FALSE)");
    model._set("H7", "=HLOOKUP(1,A1:C3,E2)");
    model._set("H8", "=HLOOKUP(1,A1:C3,-1E+15)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "#VALUE!");
    assert_eq!(model._get_text("H2"), "#REF!");
    assert_eq!(model._get_text("H3"), "#N/A");
    assert_eq!(model._get_text("H4"), "#VALUE!");
    assert_eq!(model._get_text("H5"), "#REF!");
    assert_eq!(model._get_text("H6"), "#N/A");
    assert_eq!(model._get_text("H7"), "1");
    assert_eq!(model._get_text("H8"), "#REF!");
}

/// Whole-row lookups with a blank tail (probes/lookup-whole-column-blank-tail,
/// Row sheet; T-0126).
#[test]
fn whole_row_blank_tail_matches_excel() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "10"),
        ("B1", "20"),
        ("C1", "30"),
        ("A2", "a"),
        ("B2", "b"),
        ("C2", "c"),
    ] {
        model._set(cell, value);
    }
    let cases = [
        ("=HLOOKUP(99,1:2,2)", "c"),
        ("=HLOOKUP(FALSE,1:2,2,FALSE)", "#N/A"),
        ("=HLOOKUP(0,1:2,2,FALSE)", "#N/A"),
        ("=HLOOKUP(5,1:2,2)", "#N/A"),
        ("=HLOOKUP(99,A1:J2,2)", "c"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 10), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 10)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn test_hlookup_index_function_index_is_a_reference() {
    // The HLOOKUP twin of the VLOOKUP case (shared `lookup_index_arg`; probes/lookup-index-shapes
    // H2, H3, H11, H12 probe VLOOKUP).
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "2"),
        ("A2", "x"),
        ("B2", "y"),
        ("E1", "a"),
        ("E2", "=1/0"),
    ] {
        model._set(cell, value);
    }
    model._set("H1", "=HLOOKUP(2,A1:B2,INDEX(E1:E2,1))");
    model._set("H2", "=HLOOKUP(2,A1:B2,INDEX(E1:E2,2))");
    model._set("H3", "=HLOOKUP(9,A1:B2,INDEX(E1:E2,1),FALSE)");
    model._set("H4", "=HLOOKUP(9,A1:B2,INDEX(E1:E2,2),FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "#REF!");
    assert_eq!(model._get_text("H2"), "#REF!");
    assert_eq!(model._get_text("H3"), "#N/A");
    assert_eq!(model._get_text("H4"), "#N/A");
}

#[test]
fn test_lookup_array_index_and_range_lookup() {
    // probes/vlookup-array-index-range-lookup (T-1269, excel-graph): a scalar lookup lifts over an
    // index array after validating every element; a lifted lookup or range_lookup reads only the
    // first element of the index, and range_lookup lifts with the lookup value.
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "10"),
        ("B1", "20"),
        ("C1", "30"),
        ("A2", "a"),
        ("B2", "b"),
        ("C2", "c"),
    ] {
        model._set(cell, value);
    }
    model._set("E1", "=HLOOKUP(20,A1:C2,{2,1})");
    model._set("E4", "=HLOOKUP(20,A1:C2,{2,0})");
    model._set("E5", "=HLOOKUP(20,A1:C2,{9,0})");
    model._set("E6", "=HLOOKUP(99,A1:C2,{9,0},FALSE)");
    model._set("G1", "=HLOOKUP({20;30},A1:C2,{2;1})");
    model._set("J1", "=HLOOKUP(25,A1:C2,2,{FALSE;TRUE})");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "b");
    assert_eq!(model._get_text("F1"), "20");
    assert_eq!(model._get_text("E4"), "#VALUE!");
    assert_eq!(model._get_text("E5"), "#REF!");
    assert_eq!(model._get_text("E6"), "#N/A");
    assert_eq!(model._get_text("G1"), "b");
    assert_eq!(model._get_text("G2"), "c");
    assert_eq!(model._get_text("J1"), "#N/A");
    assert_eq!(model._get_text("J2"), "b");
}

#[test]
fn test_lookup_pick_stays_a_reference_for_an_if_index() {
    // probes/lookup-pick-index-shapes A3, A9, B2 (T-1377, excel-graph): an `IF` index that picks a
    // cell keeps the lookup pick a reference, so COUNT skips the picked FALSE like a plain cell.
    let mut model = new_empty_model();
    for (cell, value) in [
        ("F5", "42"),
        ("G5", "2"),
        ("F6", "FALSE"),
        ("A1", "1"),
        ("B1", "FALSE"),
        ("A2", "1"),
        ("B2", "5"),
    ] {
        model._set(cell, value);
    }
    model._set("H1", "=COUNT(HLOOKUP(42,F5:G6,G5))");
    model._set("H2", "=COUNT(HLOOKUP(42,F5:G6,IF(TRUE,G5)))");
    model._set("H3", "=COUNT(HLOOKUP(42,F5:G6,2))");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "0");
    assert_eq!(model._get_text("H2"), "0");
    assert_eq!(model._get_text("H3"), "1");
}
