#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_textsplit_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b,c\",\",\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
}

#[test]
fn test_textsplit_row_delimiter() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b|c,d\",\",\",\"|\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("A2"), "c");
    assert_eq!(model._get_text("B2"), "d");
}

#[test]
fn test_textsplit_ignore_empty() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,,b\",\",\",,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
}

#[test]
fn test_textsplit_case_insensitive() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"aXbXc\",\"x\",,FALSE,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
}

#[test]
fn test_textsplit_pad_with() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b|c\",\",\",\"|\",,,\"X\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("A2"), "c");
    assert_eq!(model._get_text("B2"), "X");
}

#[test]
fn test_textsplit_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── TEXTSPLIT with array col_delimiter ────────────────────────────────────────

#[test]
fn test_textsplit_array_col_delim_inline() {
    // Inline array of two column delimiters: "," and ";"
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b;c\",{\",\",\";\"},)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
}

#[test]
fn test_textsplit_array_col_delim_from_range() {
    // Column delimiter list in a range A10:A11
    let mut model = new_empty_model();
    model._set("A10", ",");
    model._set("A11", ";");
    model._set("A1", "=TEXTSPLIT(\"a,b;c\",A10:A11)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
}

#[test]
fn test_textsplit_array_col_delim_three_delimiters() {
    // Three delimiters: comma, semicolon, pipe
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b;c|d\",{\",\",\";\",\"|\"})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
    assert_eq!(model._get_text("D1"), "d");
}

#[test]
fn test_textsplit_array_col_delim_with_row_delim() {
    // Array col delimiters + single row delimiter
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b;c|d,e;f\",{\",\",\";\"},\"|\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
    assert_eq!(model._get_text("A2"), "d");
    assert_eq!(model._get_text("B2"), "e");
    assert_eq!(model._get_text("C2"), "f");
}

#[test]
fn test_textsplit_array_row_delim() {
    // Array of two row delimiters: "|" and "#"
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"a,b|c,d#e,f\",\",\",{\"|\",\"#\"})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("A2"), "c");
    assert_eq!(model._get_text("B2"), "d");
    assert_eq!(model._get_text("A3"), "e");
    assert_eq!(model._get_text("B3"), "f");
}

#[test]
fn test_textsplit_array_col_delim_preserves_order() {
    // When multiple delimiters are present, the earliest one in the text wins
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"hello;world,foo\",{\",\",\";\"},)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "hello");
    assert_eq!(model._get_text("B1"), "world");
    assert_eq!(model._get_text("C1"), "foo");
}

#[test]
fn test_textsplit_array_col_delim_case_insensitive() {
    // Array delimiter with case-insensitive matching
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"aXbYc\",{\"x\",\"y\"},,FALSE,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
}

// T-1794: match_mode 1 once located delimiters in `to_lowercase()` of the text and sliced the
// original with those byte offsets; İ lowercases to two chars, so the offset ran past the end.
#[test]
fn test_textsplit_case_insensitive_non_ascii_length_change() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"İİİ,a\",\",\",,,1)");
    // Kelvin sign (3 bytes) lowercases to ASCII k (1 byte), shifting offsets the other way.
    model._set("A3", "=TEXTSPLIT(\"\u{212A}\u{212A},b\",\",\",,,1)");
    model._set("A5", "=TEXTSPLIT(\"İ|İ,c\",\",\",\"|\",,1)");
    model._set("A8", "=TEXTSPLIT(\"ÄxäyÄ\",\"ä\",,,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "İİİ");
    assert_eq!(model._get_text("B1"), "a");
    assert_eq!(model._get_text("A3"), "\u{212A}\u{212A}");
    assert_eq!(model._get_text("B3"), "b");
    assert_eq!(model._get_text("A5"), "İ");
    assert_eq!(model._get_text("A6"), "İ");
    assert_eq!(model._get_text("B6"), "c");
    assert_eq!(model._get_text("A8"), "");
    assert_eq!(model._get_text("B8"), "x");
    assert_eq!(model._get_text("C8"), "y");
    assert_eq!(model._get_text("D8"), "");
}

// Excel (probes/textsplit-casefold-length): İ matches "i", Kelvin sign does not match "k",
// and ß / ẞ do not match "ss".
#[test]
fn test_textsplit_case_insensitive_matches_excel_folding() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTSPLIT(\"aİbİc\",\"i\",,,1)");
    model._set("A2", "=TEXTSPLIT(\"a\u{212A}bkc\",\"k\",,,1)");
    model._set("A3", "=TEXTSPLIT(\"aßbSSc\",\"ss\",,,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("B1"), "b");
    assert_eq!(model._get_text("C1"), "c");
    assert_eq!(model._get_text("A2"), "a\u{212A}b");
    assert_eq!(model._get_text("B2"), "c");
    assert_eq!(model._get_text("A3"), "aßb");
    assert_eq!(model._get_text("B3"), "c");
}
