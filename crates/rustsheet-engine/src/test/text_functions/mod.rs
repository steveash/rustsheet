#![allow(clippy::unwrap_used)]

mod broadcast_bound;
mod byte_variants;
mod fn_arraytotext;
mod fn_char_code;
mod fn_clean;
mod fn_fixed_huge_decimals;
mod fn_fixed_negative_zero;
mod fn_left_right_mid_arrays;
mod fn_regex;
mod fn_text_arrays;
mod fn_text_sections;
mod fn_textbefore;
mod fn_textjoin;
mod fn_textsplit;
mod fn_unicode;
mod fn_utf16_positions;
mod fn_valuetotext;
mod textsplit_bound;

use crate::test::util::new_empty_model;

// ── ASC ───────────────────────────────────────────────────────────────────────

#[test]
fn test_asc_leaves_fullwidth_unchanged() {
    let mut model = new_empty_model();
    // en-US Excel does not narrow: probes/t1626-asc, text.md "ASC and JIS in en-US" (T-1863).
    model._set("A1", "=ASC(\"\u{FF21}\u{FF22}\u{FF23}\")");
    model._set("A2", "=ASC(\"\u{3000}\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "\u{FF21}\u{FF22}\u{FF23}");
    assert_eq!(model._get_text("A2"), "\u{3000}");
}

#[test]
fn test_asc_passthrough() {
    let mut model = new_empty_model();
    model._set("A1", "=ASC(\"hello\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "hello");
}

// ── DBCS, PHONETIC ────────────────────────────────────────────────────────────

#[test]
fn test_dbcs_and_phonetic_as_the_web_oracle_evaluates_them() {
    // probes/text-modes-dbcs C3 and text-modes-phonetic{,-range,-number} C1 (T-1213).
    let mut model = new_empty_model();
    model._set("A1", "abc");
    model._set("A2", "5");
    model._set("B1", "=DBCS(A1)");
    model._set("B2", "=PHONETIC(A1)");
    model._set("B3", "=PHONETIC(A2)");
    model._set("B4", "=PHONETIC(A1:A2)");
    model._set("B5", "=DBCS()");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "abc");
    for cell in ["B2", "B3", "B4"] {
        assert_eq!(model._get_text(cell), "#N/A", "{cell}");
    }
    assert_eq!(model._get_text("B5"), "#ERROR!");
}

#[test]
fn test_asc_and_dbcs_lift_over_a_range_and_an_array() {
    // probes/text-modes-dbcs-lift: one result per element, spilled (T-1248; T-1411, T-1488).
    let mut model = new_empty_model();
    model._set("A1", "ab");
    model._set("A2", "5");
    model._set("C1", "=ASC(A1:A2)");
    model._set("D1", "=DBCS(A1:A2)");
    model._set("E1", "=ROWS(ASC({\"a\";\"b\"}))");
    model._set("F1", "=ROWS(DBCS(A1:A2))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "ab");
    assert_eq!(model._get_text("C2"), "5");
    assert_eq!(model._get_text("D1"), "ab");
    assert_eq!(model._get_text("D2"), "5");
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("F1"), "2");
}

// ── DOLLAR ────────────────────────────────────────────────────────────────────

#[test]
fn test_dollar_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=DOLLAR(1234.567)");
    model._set("A2", "=DOLLAR(1234.567, 0)");
    model._set("A3", "=DOLLAR(-1234.5, 2)");
    model._set("A4", "=DOLLAR(1234.5, -2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "$1,234.57");
    assert_eq!(model._get_text("A2"), "$1,235");
    assert_eq!(model._get_text("A3"), "($1,234.50)");
    assert_eq!(model._get_text("A4"), "$1,200");
}

// ── FIXED ─────────────────────────────────────────────────────────────────────

#[test]
fn test_fixed_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=FIXED(1234.567, 2)");
    model._set("A2", "=FIXED(1234.567, 0)");
    model._set("A3", "=FIXED(1234.567, 2, TRUE)");
    model._set("A4", "=FIXED(-1234.5, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1,234.57");
    assert_eq!(model._get_text("A2"), "1,235");
    assert_eq!(model._get_text("A3"), "1234.57");
    assert_eq!(model._get_text("A4"), "-1,234.50");
}

// ── NUMBERVALUE ───────────────────────────────────────────────────────────────

#[test]
fn test_numbervalue_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=NUMBERVALUE(\"1234.56\")");
    model._set("A2", "=NUMBERVALUE(\"1.234,56\", \",\", \".\")");
    model._set("A3", "=NUMBERVALUE(\"3.5%\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1234.56");
    assert_eq!(model._get_text("A2"), "1234.56");
    assert_eq!(model._get_text("A3"), "0.035");
}

#[test]
fn test_numbervalue_error() {
    let mut model = new_empty_model();
    model._set("A1", "=NUMBERVALUE(\"abc\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

// ── PROPER ────────────────────────────────────────────────────────────────────

#[test]
fn test_proper_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=PROPER(\"hello world\")");
    model._set("A2", "=PROPER(\"HELLO WORLD\")");
    model._set("A3", "=PROPER(\"it's a test\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "Hello World");
    assert_eq!(model._get_text("A2"), "Hello World");
    assert_eq!(model._get_text("A3"), "It'S A Test");
}

// ── TRIM ──────────────────────────────────────────────────────────────────────

#[test]
fn test_trim_collapse_internal_runs() {
    let mut model = new_empty_model();
    // Leading/trailing spaces trimmed AND internal runs collapsed to a single space.
    model._set("A1", "=TRIM(\"  hello   world  \")");
    // Interior-only collapse (ends already correct) — the load-bearing regressions.
    model._set("A2", "=TRIM(\"a    b\")");
    model._set("A3", "=TRIM(\"no  extra\")");
    // No change needed.
    model._set("A4", "=TRIM(\"single\")");
    // All spaces collapse to empty string.
    model._set("A5", "=TRIM(\"   \")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "hello world");
    assert_eq!(model._get_text("A2"), "a b");
    assert_eq!(model._get_text("A3"), "no extra");
    assert_eq!(model._get_text("A4"), "single");
    assert_eq!(model._get_text("A5"), "");
}

#[test]
fn test_trim_only_ascii_space() {
    let mut model = new_empty_model();
    // Tabs (0x09) must NOT be collapsed or trimmed — TRIM is 0x20-only.
    model._set("A1", "=TRIM(\"a\"&CHAR(9)&CHAR(9)&\"b\")");
    // Non-breaking space (0xA0) must NOT be trimmed.
    model._set("A2", "=TRIM(CHAR(160)&\"x\"&CHAR(160))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a\u{9}\u{9}b");
    assert_eq!(model._get_text("A2"), "\u{a0}x\u{a0}");
}

// ── REPLACE ───────────────────────────────────────────────────────────────────

#[test]
fn test_replace_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=REPLACE(\"abcdef\", 2, 3, \"XY\")");
    model._set("A2", "=REPLACE(\"hello\", 1, 0, \"start-\")");
    model._set("A3", "=REPLACE(\"hello\", 6, 0, \"-end\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "aXYef");
    assert_eq!(model._get_text("A2"), "start-hello");
    assert_eq!(model._get_text("A3"), "hello-end");
}

// ── VALUE ─────────────────────────────────────────────────────────────────────

#[test]
fn test_value_date_text_with_clock_suffix() {
    // probes/date-text-grammar C30: VALUE("July 1, 2026 10:00") = 46204.416666666664.
    let mut model = new_empty_model();
    model._set("A1", "=VALUE(\"July 1, 2026 10:00\")");
    model._set("A2", "=VALUE(\"1/2/2020 6 PM\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "46204.41667");
    // probes/date-text-grammar C63.
    assert_eq!(model._get_text("A2"), "43832.75");
}
