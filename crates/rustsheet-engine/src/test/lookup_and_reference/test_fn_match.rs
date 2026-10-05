#![allow(clippy::unwrap_used)]

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
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn test_match_whole_row_and_column_bounded_by_used_range() {
    // Each formula sits in a fresh model at H3 so it does not extend the used rows.
    let cases = [
        ("=MATCH(20,A:A,0)", "2"),
        ("=MATCH(99,A:A,0)", "#N/A"),
        ("=MATCH(99,A:A,1)", "3"),
        ("=MATCH(20,1:1,0)", "3"),
        ("=MATCH(99,1:1,0)", "#N/A"),
        // Range that starts below (or right of) all data is entirely empty.
        ("=MATCH(1,A100:A200,0)", "#N/A"),
        ("=MATCH(1,Z1:ZZ1,0)", "#N/A"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        for (cell, value) in [
            ("A1", "10"),
            ("A2", "20"),
            ("A3", "30"),
            ("B1", "10"),
            ("C1", "20"),
            ("D1", "30"),
        ] {
            model._set(cell, value);
        }
        model._set("H3", formula);
        model.evaluate();
        assert_eq!(model._get_text("H3"), expected, "{formula}");
    }
}
