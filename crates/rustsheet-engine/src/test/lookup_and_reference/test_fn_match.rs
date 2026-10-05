#![allow(clippy::unwrap_used)]

use super::results_independent_of_used_area;
use crate::test::util::new_empty_model;

#[test]
fn test_match_array_row_vector() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(69,{24,43.5,52.8,69,269,387,770},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
}

#[test]
fn test_match_array_column_vector() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(52.8,{24;43.5;52.8;69;269;387;770},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_match_array_exact_string() {
    let mut model = new_empty_model();
    model._set("A1", r#"=MATCH("banana",{"apple","banana","cherry"},0)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_match_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(100,{24,43.5,52.8,69},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_match_array_ascending_default() {
    let mut model = new_empty_model();
    // match_type 1 (default): largest value <= target in an ascending array
    model._set("A1", "=MATCH(70,{24,43.5,52.8,69,269,387,770})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
}

#[test]
fn test_match_array_descending() {
    let mut model = new_empty_model();
    // match_type -1: smallest value >= target in a descending array.
    // Values >= 70 are 770, 387, 269; the smallest of those is 269 at position 3.
    model._set("A1", "=MATCH(70,{770,387,269,69,52.8,43.5,24},-1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_match_array_wildcard() {
    let mut model = new_empty_model();
    model._set("A1", r#"=MATCH("ban*",{"apple","banana","cherry"},0)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_match_array_not_a_vector() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(1,{1,2;3,4},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

// Whole-column and whole-row searches give the results of a full walk. Only
// the exact matches are pinned: how Excel's sorted search treats the empty
// tail is not yet oracle-probed (T-0126).
#[test]
fn test_match_whole_row_and_column_independent_of_used_area() {
    let data = [
        ("A1", "10"),
        ("A2", "20"),
        ("A3", "30"),
        ("B1", "10"),
        ("C1", "20"),
        ("D1", "30"),
    ];
    let column = results_independent_of_used_area(
        &data,
        "H3",
        "Z1048576",
        &[
            "=MATCH(20,A:A,0)",
            "=MATCH(99,A:A,0)",
            "=MATCH(1,A100:A200,0)",
            "=MATCH(\"*\",A:A,0)",
            "=MATCH(99,A:A,1)",
            "=MATCH(99,A:A,-1)",
            "=MATCH(5,A:A,1)",
            "=MATCH(0,A2:A1048576,1)",
        ],
    );
    assert_eq!(column[..3], ["2", "#N/A", "#N/A"]);
    let row = results_independent_of_used_area(
        &data,
        "H3",
        "XFD5",
        &[
            "=MATCH(20,1:1,0)",
            "=MATCH(99,1:1,0)",
            "=MATCH(1,Z1:XFD1,0)",
            "=MATCH(99,1:1,1)",
            "=MATCH(99,1:1,-1)",
        ],
    );
    assert_eq!(row[..3], ["3", "#N/A", "#N/A"]);
}

// The skipped part of a whole-column search vector is on record as seen empty,
// so a spill into it during the pass restarts the pass (as for XMATCH).
#[test]
fn test_match_and_lookup_whole_column_see_a_spill_beyond_the_used_area() {
    for formula in ["=MATCH(5,D:D,0)", "=LOOKUP(5,D1:D5)"] {
        let mut model = new_empty_model();
        model._set("A1", &format!("=SEQUENCE(1,1,{})", &formula[1..]));
        model._set("C1", "=SEQUENCE(3,1,10)");
        model._set("D1", "=SEQUENCE(5)");
        model.evaluate();
        assert_eq!(model._get_text("A1"), "5", "{formula}");
        assert_eq!(model.evaluation.restarts_in_last_evaluation, 1, "{formula}");
    }
}

// A 2-D lookup_array is #N/A, but an error in an earlier argument wins (T-0210).
#[test]
fn test_match_two_dimensional_lookup_array_is_na() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "=MATCH(2,A1:B2,0)");
    model._set("D2", "=MATCH(2,A1:B2)");
    model._set("D3", "=MATCH(1/0,A1:B2,0)");
    model._set("D4", "=MATCH(2,{1,2;3,4},0)");
    model._set("D5", "=MATCH(2,A1:B1,0)");
    model._set("D6", "=MATCH(2,A1:B2,1/0)");
    model._set("D7", "=MATCH(2,A1:B2,\"x\")");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"#N/A");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#DIV/0!");
    assert_eq!(model._get_text("D4"), *"#N/A");
    assert_eq!(model._get_text("D5"), *"2");
    assert_eq!(model._get_text("D6"), *"#DIV/0!");
    assert_eq!(model._get_text("D7"), *"#VALUE!");
}
