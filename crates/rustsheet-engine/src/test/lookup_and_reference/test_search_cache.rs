#![allow(clippy::unwrap_used)]

//! The per-pass search cache and the exact-match indexes (T-0448): a lookup through a cached,
//! indexed vector finds what a fresh walk finds.

use crate::test::util::new_empty_model;
use crate::Model;

/// Rows of the shared lookup range: above `search_cache::MIN_CACHED_CELLS`, so it is cached.
const ROWS: usize = 120;

/// The values of A1:A120 (B holds each row's number): numbers that MATCH tells apart and XMATCH
/// does not, text in several cases, accents and hyphens, booleans, an error, blanks, duplicates
/// and wildcard characters.
const DATA: &[&str] = &[
    "1",
    "'1",
    "TRUE",
    "FALSE",
    "0",
    "-0",
    "0.3",
    "=0.1+0.2",
    "abc",
    "ABC",
    "Abc",
    "co-op",
    "coop",
    "café",
    "cafe",
    "=NA()",
    "",
    "a*b",
    "axxb",
    "~",
    "ü",
    "Straße",
    "STRASSE",
    "1E+300",
    "123456789012345",
    "123456789012345.1",
    "=\"\"",
    "x",
    "abc",
    "1",
];

/// Lookup values, written as formula operands.
const LOOKUPS: &[&str] = &[
    "1",
    "\"1\"",
    "TRUE",
    "FALSE",
    "0",
    "0.3",
    "0.1+0.2",
    "\"abc\"",
    "\"ABC\"",
    "\"coop\"",
    "\"co-op\"",
    "\"cafe\"",
    "\"CAFÉ\"",
    "\"a*b\"",
    "\"a~*b\"",
    "\"?bc\"",
    "\"~\"",
    "\"~~\"",
    "\"Ü\"",
    "\"strasse\"",
    "1E+300",
    "123456789012345.1",
    "123456789012345",
    "\"x\"",
    "\"missing\"",
    "\"\"",
    "-0",
];

/// Formula templates: `{v}` is the lookup value, `{r}` the last row of the lookup range.
const TEMPLATES: &[&str] = &[
    "=MATCH({v},$A$1:$A${r},0)",
    "=VLOOKUP({v},$A$1:$B${r},2,FALSE)",
    "=XMATCH({v},$A$1:$A${r})",
    "=XMATCH({v},$A$1:$A${r},0,-1)",
    "=XLOOKUP({v},$A$1:$A${r},$B$1:$B${r})",
    "=XLOOKUP({v},$A$1:$A${r},$B$1:$B${r},\"none\",0,-1)",
    "=XLOOKUP({v},$A$1:$A${r},$B$1:$B${r},\"none\",2)",
];

fn fill_data(model: &mut Model) {
    for row in 1..=ROWS {
        let value = DATA[(row - 1) % DATA.len()];
        model._set(&format!("A{row}"), value);
        model._set(&format!("B{row}"), &row.to_string());
    }
}

/// Every lookup value through every template: each formula over the shared A1:A120 (cached and,
/// from its second search on, indexed) gives what the same formula gives over a range of its own
/// length (A1:A121, A1:A122, ...: blank rows past the data, never shared, so always walked).
#[test]
fn indexed_lookups_find_what_the_walk_finds() {
    let mut model = new_empty_model();
    fill_data(&mut model);
    let mut cases = Vec::new();
    let mut row = 1;
    for template in TEMPLATES {
        for (i, value) in LOOKUPS.iter().enumerate() {
            let shared = template
                .replace("{v}", value)
                .replace("{r}", &ROWS.to_string());
            let own = template
                .replace("{v}", value)
                .replace("{r}", &(ROWS + 1 + cases.len() + i).to_string());
            model._set(&format!("D{row}"), &shared);
            model._set(&format!("E{row}"), &own);
            cases.push((row, shared));
            row += 1;
        }
    }
    model.evaluate();
    for (row, formula) in cases {
        assert_eq!(
            model._get_text(&format!("D{row}")),
            model._get_text(&format!("E{row}")),
            "{formula}"
        );
    }
}

/// The index keeps the first match for MATCH, VLOOKUP and XMATCH, and the last for a
/// last-to-first XMATCH or XLOOKUP; text matches case-insensitively and never matches a number.
#[test]
fn duplicates_case_and_types() {
    let mut model = new_empty_model();
    fill_data(&mut model);
    // Three searches each, so the third certainly goes through the index.
    for row in 1..=3 {
        model._set(&format!("D{row}"), "=MATCH(\"ABC\",$A$1:$A$120,0)");
        model._set(&format!("E{row}"), "=XMATCH(\"abc\",$A$1:$A$120,0,-1)");
        model._set(&format!("F{row}"), "=MATCH(1,$A$1:$A$120,0)");
        model._set(
            &format!("G{row}"),
            "=XLOOKUP(\"1\",$A$1:$A$120,$B$1:$B$120)",
        );
    }
    model.evaluate();
    // "abc" is first at A9; its last copy is A119 (row 29 of the fourth pass through DATA).
    assert_eq!(model._get_text("D3"), "9");
    assert_eq!(model._get_text("E3"), "119");
    // The number 1 is first at A1; the text "1" first at A2.
    assert_eq!(model._get_text("F3"), "1");
    assert_eq!(model._get_text("G3"), "2");
}

/// A range of formulas evaluated by the lookups that read it, then edited: each evaluation
/// starts with an empty cache, so the second sees the new values.
#[test]
fn the_cache_lives_for_one_pass() {
    let mut model = new_empty_model();
    for row in 1..=ROWS {
        model._set(&format!("C{row}"), &row.to_string());
        model._set(&format!("A{row}"), &format!("=C{row}*10"));
    }
    for row in 1..=5 {
        model._set(&format!("D{row}"), "=MATCH(500,$A$1:$A$120,0)");
        model._set(&format!("E{row}"), "=XMATCH(500,$A$1:$A$120)");
    }
    model.evaluate();
    assert_eq!(model._get_text("D5"), "50");
    assert_eq!(model._get_text("E5"), "50");
    model._set("C50", "7");
    model._set("C60", "50");
    model.evaluate();
    assert_eq!(model._get_text("D5"), "60");
    assert_eq!(model._get_text("E5"), "60");
}

/// A cycle through the range (a cell of it looks the range up) is never cached: its cell gets
/// its value only after the lookups that met it running, and every lookup sees what the walk
/// sees.
#[test]
fn a_cycle_through_the_range_is_not_cached() {
    let mut model = new_empty_model();
    fill_data(&mut model);
    model._set("A40", "=MATCH(\"x\",$A$1:$A$120,0)");
    for row in 1..=3 {
        model._set(&format!("D{row}"), "=MATCH(0,$A$1:$A$120,0)");
        model._set(
            &format!("E{row}"),
            &format!("=MATCH(0,$A$1:$A${},0)", 120 + row),
        );
    }
    model.evaluate();
    for row in 1..=3 {
        assert_eq!(
            model._get_text(&format!("D{row}")),
            model._get_text(&format!("E{row}"))
        );
    }
}

/// A dynamic array spills into a range that a lookup read earlier in the same pass: the pass
/// restarts with the anchor first (the walk recorded what it saw there), and the restarted pass
/// starts with an empty cache, so every lookup sees the spilled values as the walk does. B1 is
/// itself an anchor, so it runs before A50 in the first pass and reads A51:A59 still empty.
#[test]
fn a_spill_into_a_cached_range_restarts_with_an_empty_cache() {
    let mut model = new_empty_model();
    for row in 60..=ROWS {
        model._set(&format!("A{row}"), &row.to_string());
    }
    model._set("B1", "=MATCH({1005;1009},$A$51:$A$120,0)");
    for row in 3..=5 {
        model._set(&format!("B{row}"), "=MATCH(1005,$A$51:$A$120,0)");
        model._set(
            &format!("C{row}"),
            &format!("=MATCH(1005,$A$51:$A${},0)", 120 + row),
        );
    }
    model._set("A50", "=SEQUENCE(10,1,1000)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "5");
    assert_eq!(model._get_text("B2"), "9");
    for row in 3..=5 {
        assert_eq!(model._get_text(&format!("B{row}")), "5");
        assert_eq!(model._get_text(&format!("C{row}")), "5");
    }
}
