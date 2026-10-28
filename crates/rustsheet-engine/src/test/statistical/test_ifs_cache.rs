#![allow(clippy::unwrap_used)]

//! The per-pass cache of the criteria aggregates (T-0449): a call through cached range values and
//! match sets gives what the walk gives.

use crate::test::util::new_empty_model;
use crate::Model;

/// Rows of one data block: below `ifs_cache::MIN_CACHED_CELLS`, so a call over one block walks.
const BLOCK: usize = 60;
/// Blocks stacked in A1:C240, a range the cache takes.
const COPIES: usize = 4;

/// One block's criteria values (A and B) and sum values (C, numbers small enough to add exactly,
/// with text, booleans, blanks and an error).
fn row_values(row: usize) -> (String, String, String) {
    const A: &[&str] = &[
        "1",
        "2",
        "'2",
        "TRUE",
        "FALSE",
        "abc",
        "ABC",
        "abd",
        "a*c",
        "",
        "=NA()",
        "0.3",
        "=0.1+0.2",
        "-1",
        "0",
        "=\"\"",
        "x",
        "1E+300",
        "'1/1/2024",
        "45292",
    ];
    const B: &[&str] = &[
        "t1", "u2", "Z", "", "5", "s", "T", "tt", "=1/0", "zz", "10", "'10",
    ];
    const C: &[&str] = &[
        "1", "2", "3", "", "TRUE", "'4", "5", "6", "7", "8", "9", "=0.5",
    ];
    let i = (row - 1) % BLOCK;
    let c = if i == 37 { "=NA()" } else { C[i % C.len()] };
    (
        A[i % A.len()].to_string(),
        B[(i * 7) % B.len()].to_string(),
        c.to_string(),
    )
}

fn fill(model: &mut Model) {
    for row in 1..=BLOCK * COPIES {
        let (a, b, c) = row_values(row);
        model._set(&format!("A{row}"), &a);
        model._set(&format!("B{row}"), &b);
        model._set(&format!("C{row}"), &c);
    }
}

/// `{f}` takes the ranges as `{a}`, `{b}`, `{c}`; criteria as `{x}` and `{y}`.
fn instantiate(template: &str, top: usize, bottom: usize, x: &str, y: &str) -> String {
    template
        .replace("{a}", &format!("$A${top}:$A${bottom}"))
        .replace("{b}", &format!("$B${top}:$B${bottom}"))
        .replace("{c}", &format!("$C${top}:$C${bottom}"))
        .replace("{x}", x)
        .replace("{y}", y)
}

const CRITERIA: &[&str] = &[
    "1",
    "\"1\"",
    "\">1\"",
    "\"<=0.3\"",
    "\"<>abc\"",
    "\"abc\"",
    "\"a*\"",
    "\"?b?\"",
    "\"a~*c\"",
    "\"\"",
    "\"=\"",
    "\"<>\"",
    "TRUE",
    "\"TRUE\"",
    "\"#N/A\"",
    "\">t\"",
    "\"<1/2/2024\"",
    "\"10\"",
    "\"<1\"",
    "\">=0\"",
    "\"<=-1\"",
    "\"<>1\"",
    "\"<>0\"",
    "0.3",
    "\"=0.3\"",
    "\"0.30000000000000004\"",
    "\">=1/1/2024\"",
    "\"=45292\"",
    "-1",
    "\"ABC\"",
    "\"=abd\"",
    "\"t1\"",
    "\"<>10\"",
    "\">TRUE\"",
    "\" 2 \"",
    "\"2\"",
];

/// Every criterion pair through COUNTIFS, SUMIFS and MAXIFS over the cached A1:C240 gives what
/// the walks over its four 60-row blocks give together.
#[test]
fn cached_criteria_aggregates_match_the_walks() {
    let mut model = new_empty_model();
    fill(&mut model);
    let templates = [
        ("COUNTIFS({a},{x},{b},{y})", "+"),
        ("SUMIFS({c},{a},{x},{b},{y})", "+"),
        ("COUNTIF({a},{x})", "+"),
        ("SUMIF({a},{x},{c})", "+"),
        ("MAXIFS({c},{a},{x})", ","),
    ];
    let mut cases = Vec::new();
    let mut row = 1;
    for (template, join) in templates {
        for x in CRITERIA {
            for y in CRITERIA.iter().step_by(3) {
                let cached = format!("={}", instantiate(template, 1, BLOCK * COPIES, x, y));
                let parts: Vec<String> = (0..COPIES)
                    .map(|k| instantiate(template, k * BLOCK + 1, (k + 1) * BLOCK, x, y))
                    .collect();
                let walked = if join == "+" {
                    format!("={}", parts.join("+"))
                } else {
                    format!("=MAX({})", parts.join(","))
                };
                model._set(&format!("E{row}"), &cached);
                model._set(&format!("F{row}"), &walked);
                cases.push((row, cached));
                row += 1;
            }
        }
    }
    model.evaluate();
    for (row, formula) in cases {
        assert_eq!(
            model._get_text(&format!("E{row}")),
            model._get_text(&format!("F{row}")),
            "{formula}"
        );
    }
}

/// Criteria ranges of formulas evaluated by the first call, then edited: each evaluation starts
/// with an empty cache, so the second sees the new values.
#[test]
fn the_cache_lives_for_one_pass() {
    let mut model = new_empty_model();
    for row in 1..=100 {
        model._set(&format!("D{row}"), &(row % 10).to_string());
        model._set(&format!("A{row}"), &format!("=D{row}*1"));
    }
    for row in 1..=3 {
        model._set(&format!("E{row}"), "=COUNTIF($A$1:$A$100,7)");
        model._set(
            &format!("F{row}"),
            "=SUMIFS($A$1:$A$100,$A$1:$A$100,\">5\")",
        );
    }
    model.evaluate();
    assert_eq!(model._get_text("E3"), "10");
    assert_eq!(model._get_text("F3"), "300");
    model._set("D1", "7");
    model.evaluate();
    assert_eq!(model._get_text("E3"), "11");
    assert_eq!(model._get_text("F3"), "307");
}

/// A cycle through a criteria range: its circular cell keeps the range uncached, so a second call
/// walks and sees what the walk sees.
#[test]
fn a_cycle_through_a_criteria_range_is_not_cached() {
    let mut model = new_empty_model();
    for row in 1..=100 {
        model._set(&format!("A{row}"), &(row % 7).to_string());
    }
    model._set("A50", "=E1");
    model._set("E1", "=COUNTIF($A$1:$A$100,3)");
    model._set("E2", "=COUNTIF($A$1:$A$100,3)");
    model._set("E3", "=COUNTIF($A$1:$A$50,3)+COUNTIF($A$51:$A$100,3)");
    model.evaluate();
    assert_eq!(model._get_text("E2"), model._get_text("E3"));
}

/// A dynamic array spills into a criteria range that a call read earlier in the same pass: the
/// pass restarts with the anchor first and an empty cache, so every call counts the spilled
/// values. E1 is itself an anchor (an array criterion), so it reads A51:A59 before A50 runs.
#[test]
fn a_spill_into_a_cached_range_restarts_with_an_empty_cache() {
    let mut model = new_empty_model();
    for row in 60..=150 {
        model._set(&format!("A{row}"), "1");
    }
    model._set("E1", "=COUNTIF($A$51:$A$150,{1000;1})");
    for row in 3..=5 {
        model._set(&format!("E{row}"), "=COUNTIF($A$51:$A$150,\">=1000\")");
    }
    model._set("A50", "=SEQUENCE(10,1,1000)");
    model.evaluate();
    // A50 holds 1000 and lies outside the range; A51:A59 hold 1001..1009.
    assert_eq!(model._get_text("E1"), "0");
    assert_eq!(model._get_text("E2"), "91");
    for row in 3..=5 {
        assert_eq!(model._get_text(&format!("E{row}")), "9");
    }
}

/// Directive 5: distinct text criteria are charged their text (at most 255 characters, longer is
/// `#VALUE!`), so the match sets of a pass stay within their budget however many such calls
/// there are; the calls past it walk. Two criteria each, so the calls take match sets (one
/// criterion is counted from the range's index).
#[test]
fn long_criteria_are_charged_to_the_budget() {
    let mut model = new_empty_model();
    for row in 1..=64 {
        model._set(&format!("A{row}"), "x");
    }
    for row in 1..=9000 {
        model._set(
            &format!("E{row}"),
            &format!("=COUNTIFS($A$1:$A$64,REPT(\"a\",250)&{row},$A$1:$A$64,\"<>\")"),
        );
    }
    model._set("F1", "=COUNTIF($A$1:$A$64,\"x\")");
    model.evaluate();
    let (used, budget) = model.evaluation.ifs_cache.match_bits();
    assert!(used <= budget, "{used} > {budget}");
    assert!(used > budget / 10 * 9, "the budget was not used: {used}");
    assert_eq!(model._get_text("E9000"), "0");
    assert_eq!(model._get_text("F1"), "64");
}

/// Blocks inside a cached range with its columns (prefixes such as `$A$1:$A$100` of the cached
/// `$A$1:$A$240`, and an inner block) are read from it and their match sets cut from its, through
/// its index (T-0465): each call gives what the walks over its pieces of at most 60 rows give.
#[test]
fn blocks_inside_a_cached_range_match_the_walks() {
    let mut model = new_empty_model();
    fill(&mut model);
    let all = BLOCK * COPIES;
    // Cache and index A1:C240 first: two criteria on each column.
    model._set(
        "D1",
        &format!("=COUNTIFS($A$1:$A${all},1,$B$1:$B${all},\"t1\")"),
    );
    model._set(
        "D2",
        &format!("=COUNTIFS($A$1:$A${all},2,$B$1:$B${all},\"T\")"),
    );
    model._set("D3", &format!("=SUMIFS($C$1:$C${all},$A$1:$A${all},3)"));
    let mut cases = Vec::new();
    let mut row = 1;
    let blocks = (64..=all)
        .step_by(11)
        .map(|bottom| (1, bottom))
        .chain([(70, 200), (65, 128)]);
    for (top, bottom) in blocks {
        // The walked pieces: at most 60 rows each, below the cache's minimum.
        let pieces: Vec<(usize, usize)> = (top..=bottom)
            .step_by(BLOCK)
            .map(|first| (first, (first + BLOCK - 1).min(bottom)))
            .collect();
        for template in [
            "COUNTIFS({a},{x},{b},{y})",
            "SUMIFS({c},{a},{x},{b},{y})",
            "COUNTIF({a},{x})",
        ] {
            for x in CRITERIA.iter().step_by(2) {
                let y = "\"<>zz\"";
                let cached = format!("={}", instantiate(template, top, bottom, x, y));
                let parts: Vec<String> = pieces
                    .iter()
                    .map(|&(first, last)| instantiate(template, first, last, x, y))
                    .collect();
                model._set(&format!("E{row}"), &cached);
                model._set(&format!("F{row}"), &format!("={}", parts.join("+")));
                cases.push((row, cached));
                row += 1;
            }
        }
    }
    model.evaluate();
    let cache = &model.evaluation.ifs_cache;
    assert!(cache.windows > 0, "no block was cut from its range");
    assert!(cache.indexed() >= 2, "the ranges were not indexed");
    for (row, formula) in cases {
        assert_eq!(
            model._get_text(&format!("E{row}")),
            model._get_text(&format!("F{row}")),
            "{formula}"
        );
    }
}

/// The index orders numbers by value, ties and 15-digit equality included: a number criterion
/// through the index gives what the walk gives over values that differ past 15 digits, negative
/// zero, and dates (T-0465).
#[test]
fn indexed_number_criteria_match_the_walk() {
    let mut model = new_empty_model();
    let values = [
        "1",
        "=1+2^-52",
        "=1-2^-53",
        "0",
        "=-0",
        "1E-17",
        "-1E-17",
        "0.1",
        "=0.1+0.2",
        "0.3",
        "45292",
        "=DATE(2024,1,1)",
        "'45292",
        "-5",
        "1E+300",
        "=-1E+300",
        "2",
        "2",
        "3",
    ];
    for row in 1..=190 {
        model._set(&format!("A{row}"), values[row % values.len()]);
    }
    let criteria = [
        "1",
        "\"<1\"",
        "\"<=1\"",
        "\">1\"",
        "\">=1\"",
        "\"<>1\"",
        "0",
        "\"<>0\"",
        "\"<0\"",
        "\">0\"",
        "\"=1E-17\"",
        "0.3",
        "\"1/1/2024\"",
        "\"<1/1/2024\"",
        "\">=1/1/2024\"",
        "2",
        "\">2\"",
        "\"<=2\"",
        "-5",
        "\"<-1E+299\"",
    ];
    let mut row = 1;
    for criterion in criteria {
        let cached = format!("=COUNTIF($A$1:$A$190,{criterion})");
        let walked = format!(
            "=COUNTIF($A$1:$A$60,{criterion})+COUNTIF($A$61:$A$120,{criterion})\
             +COUNTIF($A$121:$A$180,{criterion})+COUNTIF($A$181:$A$190,{criterion})"
        );
        model._set(&format!("E{row}"), &cached);
        model._set(&format!("F{row}"), &walked);
        row += 1;
    }
    model.evaluate();
    assert_eq!(model.evaluation.ifs_cache.indexed(), 1);
    for row in 1..row {
        assert_eq!(
            model._get_text(&format!("E{row}")),
            model._get_text(&format!("F{row}")),
            "row {row}"
        );
    }
}
