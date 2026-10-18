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

// UNVERIFIED: no oracle case for a 2-D array; #N/A assumed by analogy with ranges (T-0224).
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
    model._set("D5", "=MATCH(2,A1:B1,0)");
    model._set("D6", "=MATCH(2,A1:B2,1/0)");
    model._set("D7", "=MATCH(2,A1:B2,\"x\")");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"#N/A");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"2");
    assert_eq!(model._get_text("D6"), *"#DIV/0!");
    assert_eq!(model._get_text("D7"), *"#VALUE!");
}

/// Excel's values for approximate and exact lookups over whole columns with a
/// blank tail (probes/lookup-whole-column-blank-tail, Col sheet; T-0126).
#[test]
fn whole_column_blank_tail_matches_excel() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "10"),
        ("A2", "20"),
        ("A3", "30"),
        ("B1", "a"),
        ("B2", "b"),
        ("B3", "c"),
    ] {
        model._set(cell, value);
    }
    let cases = [
        ("=MATCH(99,A:A,1)", "3"),
        ("=MATCH(99,A:A,-1)", "#N/A"),
        ("=MATCH(5,A:A,1)", "#N/A"),
        ("=MATCH(9.99E+307,A:A)", "3"),
        ("=VLOOKUP(99,A:B,2)", "c"),
        ("=LOOKUP(20,A:A,B:B)", "b"),
        ("=LOOKUP(99,A:A,B:B)", "c"),
        ("=LOOKUP(30,A1:A3,E:E)", "0"),
        ("=MATCH(99,A1:A10,1)", "3"),
        ("=VLOOKUP(99,A1:B10,2)", "c"),
        ("=VLOOKUP(0,A:B,2,FALSE)", "#N/A"),
        ("=VLOOKUP(\"\",A:B,2,FALSE)", "#N/A"),
        ("=MATCH(0,A:A,0)", "#N/A"),
        ("=MATCH(G1,A:A,0)", "#N/A"),
        ("=VLOOKUP(G1,A:B,2)", "#N/A"),
        ("=XMATCH(0,A:A)", "#N/A"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("M{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("M{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn test_match_fractional_match_type() {
    let mut model = new_empty_model();
    // A negative fraction searches descending, sign only (gen-m2 A160/A241).
    model._set("A1", "=MATCH(5,{9,7,5,3},-0.5)");
    model._set("A2", "=MATCH(4,{9,7,5,3},-0.0001)");
    // 0.5 searches ascending (probes/match-fractional-match-type C1); 1E+15 stays ascending
    // without overflow.
    model._set("A3", "=MATCH(4,{1,3,5},0.5)");
    model._set("A4", "=MATCH(4,{1,3,5},1E+15)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "2");
}

#[test]
fn test_match_lifts_over_array_lookup_value() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "30");
    model._set("B2", "10");
    // A 1x1 array, a 2x1 array and a multi-cell range each give one position per element (T-1866).
    model._set("C1", "=MATCH({20},A1:A3,0)");
    model._set("D1", "=MATCH({30;10},A1:A3,0)");
    model._set("E1", "=MATCH(B1:B2,A1:A3,0)");
    model._set("F1", "=ROWS(MATCH(B1:B2,A1:A3,0))");
    model._set("G1", "=MATCH({20;99},A1:A3,0)");
    // An error element wins over an invalid match_type (probes/match-lookup-value-lift E17).
    model._set("H1", "=MATCH({#N/A;10},A1:A3,\"x\")");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("D2"), "1");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E2"), "1");
    assert_eq!(model._get_text("F1"), "2");
    assert_eq!(model._get_text("G1"), "2");
    assert_eq!(model._get_text("G2"), "#N/A");
    assert_eq!(model._get_text("H1"), "#N/A");
    assert_eq!(model._get_text("H2"), "#VALUE!");
}

#[test]
fn test_match_lifts_over_array_match_type() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "30");
    model._set("B2", "10");
    model._set("C1", "1");
    model._set("C2", "0");
    // An array match_type lifts, and broadcasts with lookup_value (probes/match-lookup-value-lift
    // E14:E16, T-1917).
    model._set("D1", "=MATCH(25,A1:A3,{0;1})");
    model._set("E1", "=MATCH(B1:B2,A1:A3,{0;1})");
    model._set("F1", "=ROWS(MATCH(20,A1:A3,{0;1}))");
    // A multi-cell range match_type still reads its top-left cell, and does not spill
    // (probes/match-type-nested-passthrough).
    model._set("G1", "=MATCH(25,A1:A3,C1:C2)");
    model._set("H1", "=MATCH(25,A1:A3,{0;\"x\"})");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
    assert_eq!(model._get_text("D2"), "2");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E2"), "1");
    assert_eq!(model._get_text("F1"), "2");
    assert_eq!(model._get_text("G1"), "2");
    assert_eq!(model._get_text("G2"), "");
    assert_eq!(model._get_text("H1"), "#N/A");
    assert_eq!(model._get_text("H2"), "#VALUE!");
}
