#![allow(clippy::unwrap_used)]

use super::results_independent_of_used_area;
use crate::test::util::new_empty_model;

// ── exact match (default) ─────────────────────────────────────────────────────

#[test]
fn test_xmatch_exact_found() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("A3", "cherry");
    model._set("C1", "=XMATCH(\"banana\",A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
}

#[test]
fn test_xmatch_exact_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("C1", "=XMATCH(\"cherry\",A1:A2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#N/A");
}

#[test]
fn test_xmatch_exact_number() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("C1", "=XMATCH(20,A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
}

// ── next-smaller / next-larger (unsorted) ────────────────────────────────────

#[test]
fn test_xmatch_next_smaller() {
    // match_mode=-1: exact or next smaller
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("A2", "10");
    model._set("A3", "15");
    // Looking for 12: no exact match, next smaller is 10 at position 2
    model._set("C1", "=XMATCH(12,A1:A3,-1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
}

#[test]
fn test_xmatch_next_larger() {
    // match_mode=1: exact or next larger
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("A2", "10");
    model._set("A3", "15");
    // Looking for 12: no exact match, next larger is 15 at position 3
    model._set("C1", "=XMATCH(12,A1:A3,1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

// ── wildcard match ────────────────────────────────────────────────────────────

#[test]
fn test_xmatch_wildcard() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "apricot");
    model._set("A3", "banana");
    // "ap*" should match the first entry starting with "ap"
    model._set("C1", "=XMATCH(\"ap*\",A1:A3,2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
}

#[test]
fn test_xmatch_wildcard_question_mark() {
    let mut model = new_empty_model();
    model._set("A1", "cat");
    model._set("A2", "bat");
    model._set("A3", "hat");
    // "?at" matches all three; should return first (position 1)
    model._set("C1", "=XMATCH(\"?at\",A1:A3,2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
}

// ── reverse search ────────────────────────────────────────────────────────────

#[test]
fn test_xmatch_reverse_search() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "x");
    // Last-to-first: last "x" is at position 3
    model._set("C1", "=XMATCH(\"x\",A1:A3,0,-1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

// ── binary search ─────────────────────────────────────────────────────────────

#[test]
fn test_xmatch_binary_ascending() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "3");
    model._set("A3", "5");
    model._set("A4", "7");
    // Exact match in sorted array
    model._set("C1", "=XMATCH(5,A1:A4,0,2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

#[test]
fn test_xmatch_binary_descending() {
    let mut model = new_empty_model();
    model._set("A1", "7");
    model._set("A2", "5");
    model._set("A3", "3");
    model._set("A4", "1");
    model._set("C1", "=XMATCH(5,A1:A4,0,-2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
}

// ── regex match (match_mode=3) ────────────────────────────────────────────────

#[test]
fn test_xmatch_regex_basic() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");
    // Matches words ending in "day"
    model._set("C1", "=XMATCH(\"[A-Z][a-z]*day\",A1:A3,3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
}

#[test]
fn test_xmatch_regex_second_match() {
    let mut model = new_empty_model();
    model._set("A1", "cat");
    model._set("A2", "Cart");
    model._set("A3", "car");
    // Capital letter followed by lowercase letters ending in "t"
    model._set("C1", "=XMATCH(\"[A-Z][a-z]*t\",A1:A3,3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
}

#[test]
fn test_xmatch_regex_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("C1", "=XMATCH(\"^\\d+$\",A1:A2,3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#N/A");
}

#[test]
fn test_xmatch_regex_invalid_pattern() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("C1", "=XMATCH(\"[\",A1:A1,3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#VALUE!");
}

#[test]
fn test_xmatch_regex_reverse_search() {
    let mut model = new_empty_model();
    model._set("A1", "Bay");
    model._set("A2", "Cat");
    model._set("A3", "Day");
    // Last-to-first: last match for capital+lowercase+y pattern is "Day" at position 3
    model._set("C1", "=XMATCH(\"[A-Z][a-z]*y\",A1:A3,3,-1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

// ── regex mode: non-string lookup_value → #N/A (matches Excel) ───────────────

#[test]
fn test_xmatch_regex_non_string_lookup_is_na() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    // Excel returns #N/A (not found) when lookup_value is not a string in regex mode
    model._set("C1", "=XMATCH(42,A1:A2,3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#N/A");
}

// ── row vector ────────────────────────────────────────────────────────────────

#[test]
fn test_xmatch_row_vector() {
    let mut model = new_empty_model();
    model._set("A1", "red");
    model._set("B1", "green");
    model._set("C1", "blue");
    model._set("A3", "=XMATCH(\"green\",A1:C1)");
    model.evaluate();
    assert_eq!(model._get_text("A3"), "2");
}

// ── array constants as lookup_array ─────────────────────────────────────────────

#[test]
fn test_xmatch_array_constant_wildcard() {
    let mut model = new_empty_model();
    // Wildcard match against an in-formula array constant.
    model._set("A1", r#"=XMATCH("ban*", {"apple","banana","cherry"}, 2)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_xmatch_array_constant_numeric() {
    let mut model = new_empty_model();
    // Exact numeric match against a numeric array constant.
    model._set("A1", "=XMATCH(30, {10,20,30,40,50})");
    // Exact-or-next-larger against a numeric array constant.
    model._set("A2", "=XMATCH(35, {10,20,30,40,50}, 1)");
    // A column-oriented array constant works too.
    model._set("A3", "=XMATCH(30, {10;20;30;40;50})");
    // Not found is #N/A, not #VALUE!.
    model._set("A4", "=XMATCH(99, {10,20,30})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "4");
    assert_eq!(model._get_text("A3"), "3");
    assert_eq!(model._get_text("A4"), "#N/A");
}

// ── whole-column lookup array and a spill beyond the used area ───────────────

// XMATCH evaluates a whole-column lookup array only up to the sheet's used
// area (the rest is a virtual empty tail). When it runs inside an anchor, the
// array it looks in may be spilled on demand during the walk, past the
// evaluated part. The skipped part is on record, so the spill restarts the
// pass and the match is found.
#[test]
fn test_xmatch_whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,XMATCH(5,D:D))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

#[test]
fn test_xmatch_whole_row_and_column_search_the_used_range() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("C1", "=XMATCH(20,A:A)");
    model._set("C2", "=XMATCH(99,A:A)");
    model._set("C3", "=XMATCH(20,1:1)");
    model._set("C4", "=XMATCH(1,A100:A200)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("C2"), "#N/A");
    assert_eq!(model._get_text("C3"), "#N/A");
    assert_eq!(model._get_text("C4"), "#N/A");
}

// The blank tail of a whole-column/row search vector is virtual (T-0127): every
// match and search mode must see what a full walk sees, wherever the used area
// ends. The first loop compares the engine with itself across used areas (an empty
// cell compares as 0), so it pins no Excel value; the Excel-backed ones are the
// asserts below (probes/lookup-whole-column-blank-tail, T-0126).
#[test]
fn test_xmatch_whole_column_and_row_independent_of_used_area() {
    let data = [
        ("A1", "10"),
        ("A2", "20"),
        ("A3", "30"),
        ("B1", "10"),
        ("C1", "20"),
        ("D1", "30"),
    ];
    for vector in ["A:A", "1:1"] {
        let mut formulas = Vec::new();
        for lookup in ["20", "0"] {
            for match_mode in [-1, 0, 1] {
                for search_mode in [1, -1, 2, -2] {
                    formulas.push(format!(
                        "=XMATCH({lookup},{vector},{match_mode},{search_mode})"
                    ));
                }
            }
        }
        let refs: Vec<&str> = formulas.iter().map(String::as_str).collect();
        let filler = if vector == "A:A" { "Z1048576" } else { "XFD5" };
        results_independent_of_used_area(&data, "H3", filler, &refs);
    }
    // XMATCH(0,A:A): the first empty cell is row 4 (full walk, not truncated).
    let model_value = |formula: &str| {
        let mut model = new_empty_model();
        for (cell, value) in data {
            model._set(cell, value);
        }
        model._set("H3", formula);
        model.evaluate();
        model._get_text("H3")
    };
    // Excel: a blank never matches an exact XMATCH
    // (probes/lookup-whole-column-blank-tail Col!M14, Row!A17; T-0126).
    assert_eq!(model_value("=XMATCH(0,A:A)"), "#N/A");
    assert_eq!(model_value("=XMATCH(0,A:A,0,-1)"), "#N/A");
    assert_eq!(model_value("=XMATCH(0,1:1)"), "#N/A");
    // A blank lookup value equals the whole tail: the equal-run walk of the
    // binary searches must jump it, not step a million cells (directive 5).
    for formula in [
        "=XMATCH(Z1,A:A,1,2)",
        "=XMATCH(Z1,A:A,-1,2)",
        "=XMATCH(Z1,A:A,1,-2)",
        "=XMATCH(Z1,A:A,-1,-2)",
    ] {
        model_value(formula);
    }
}

/// Excel's linear XMATCH/XLOOKUP over blanks (probes/xmatch-xlookup-blanks, Col sheet; T-1018).
#[test]
fn linear_modes_over_blanks_match_excel() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "5"),
        ("A3", "10"),
        ("A5", "20"),
        ("B1", "a"),
        ("B2", "b"),
        ("B3", "c"),
        ("B4", "d"),
        ("B5", "e"),
        ("B6", "f"),
    ] {
        model._set(cell, value);
    }
    let cases = [
        ("=XMATCH(0,A1:A6)", "#N/A"),
        ("=XMATCH(7,A1:A6,-1)", "1"),
        ("=XMATCH(7,A1:A6,1)", "3"),
        ("=XMATCH(0,A1:A6,-1)", "#N/A"),
        ("=XMATCH(0,A1:A6,1)", "1"),
        ("=XMATCH(99,A:A,-1)", "5"),
        // A blank sorts as text, so it is the "next larger" of a number nothing else exceeds.
        ("=XMATCH(99,A:A,1)", "2"),
        ("=XMATCH(25,A1:A6,1)", "2"),
        ("=XMATCH(0,A1:A6,2)", "#N/A"),
        ("=XMATCH(0,A1:A6,0,-1)", "#N/A"),
        ("=XMATCH(G1,A1:A6)", "2"),
        ("=XMATCH(G1,A:A)", "2"),
        ("=XMATCH(G1,A1:A6,-1)", "2"),
        ("=XMATCH(G1,A1:A6,2)", "2"),
        ("=XLOOKUP(0,A1:A6,B1:B6,\"nf\")", "nf"),
        ("=XLOOKUP(0,A:A,B:B,\"nf\")", "nf"),
        ("=XLOOKUP(0,A1:A6,B1:B6,\"nf\",-1)", "nf"),
        ("=XLOOKUP(0,A1:A6,B1:B6,\"nf\",1)", "a"),
        ("=XLOOKUP(G1,A1:A6,B1:B6,\"nf\")", "b"),
        ("=XLOOKUP(0,A1:A6,B1:B6,\"nf\",0,-1)", "nf"),
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
