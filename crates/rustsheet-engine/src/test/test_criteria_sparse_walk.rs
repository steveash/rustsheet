#![allow(clippy::unwrap_used)]

// The *IF(S) functions walk the cells stored in their ranges, not every position of the used
// area: a blank position matches or fails as a whole and is counted without being read (T-2711).
// The work is counted in cell reads (`Evaluation::cell_reads`), not wall-clock time (T-2282).

use crate::constants::LAST_ROW;
use crate::test::util::new_empty_model;

/// A model whose first sheet holds `J1048576 = 1` and the given cells, so the used area of a
/// whole-column range is the whole sheet height.
fn model_beside_last_row(cells: &[(&str, &str)]) -> crate::Model {
    let mut model = new_empty_model();
    model._set(&format!("J{LAST_ROW}"), "1");
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model
}

/// Evaluates `formula` in Z1 and returns its text and the cells the evaluation read.
fn evaluate_counting_reads(model: &mut crate::Model, formula: &str) -> (String, u64) {
    model._set("Z1", formula);
    let before = model.evaluation.cell_reads;
    model.evaluate();
    (model._get_text("Z1"), model.evaluation.cell_reads - before)
}

#[test]
fn criteria_walks_beside_the_last_row_read_a_few_cells() {
    let cases = [
        ("=COUNTIF(A:J,1)", "1"),
        // Every position but A5 and J1048576 is blank in A:J; I1048576 is blank in A:J and
        // is J1048576 = 1 in B:K, which is not "x": 10 x 1048576 - 3 + 1.
        ("=COUNTIFS(A:J,\"\",B:K,\"<>x\")", "10485758"),
        ("=SUMIFS(A:A,B:B,\"\")", "3"),
        ("=SUMIF(B:B,\"\",A:A)", "3"),
        ("=AVERAGEIFS(A:A,B:B,\"\")", "3"),
        ("=MAXIFS(A:J,A:J,\">0\")", "3"),
        ("=MINIFS(A:J,A:J,\">0\")", "1"),
    ];
    for (formula, expected) in cases {
        let mut model = model_beside_last_row(&[("A5", "3")]);
        let (value, reads) = evaluate_counting_reads(&mut model, formula);
        assert_eq!(value, expected, "{formula}");
        assert!(reads < 50, "{formula} read {reads} cells");
    }
}

#[test]
fn a_criterion_matching_blanks_still_counts_every_blank_position() {
    let mut model = model_beside_last_row(&[]);
    let (value, reads) = evaluate_counting_reads(&mut model, "=COUNTIF(A:J,\"\")");
    assert_eq!(value, "10485759");
    assert!(reads < 50, "read {reads} cells");
}

#[test]
fn criteria_ranges_line_up_position_by_position() {
    // Criteria ranges at different offsets and on different sheets line up position by
    // position, whatever each sheet's used area: Sheet2!A100 lies past Sheet1's used area.
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "1");
    model._set("B2", "1");
    model._set("Sheet2!A100", "x");
    model._set("Z1", "=COUNTIFS(A:A,\"\",Sheet2!A:A,\"x\")");
    model._set("Z2", "=COUNTIFS(A1:A3,1,B2:B4,\"\")");
    model._set("Z3", "=COUNTIFS(A1:A3,\"\",B2:B4,\"\")");
    model._set("Z4", "=COUNTIFS(A1:A3,\"\",B1:B3,\"\")");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "1");
    assert_eq!(model._get_text("Z2"), "0");
    assert_eq!(model._get_text("Z3"), "2");
    assert_eq!(model._get_text("Z4"), "1");
}

#[test]
fn the_first_matched_error_in_row_major_order_is_the_result() {
    let mut model = new_empty_model();
    model._set("B1", "=1/0");
    model._set("A2", "=NA()");
    model._set("C1", "y");
    model._set("C2", "x");
    model._set("D1", "x");
    model._set("D2", "x");
    // Row-major: B1 (#DIV/0!) comes before A2 (#N/A), but only where the criterion matches.
    model._set("Z1", "=SUMIFS(A1:B2,C1:D2,\"x\")");
    model._set("Z2", "=SUMIFS(A1:B2,C1:D2,\"y\")");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#DIV/0!");
    assert_eq!(model._get_text("Z2"), "0");
}

#[test]
fn a_spill_into_a_walked_range_is_counted_whatever_the_order() {
    // The spill cells of A1 are written while the pass runs; the walk records each range as read
    // empty, so a spill landing in it restarts the pass and the count sees every spilled cell.
    for count_first in [true, false] {
        let mut model = new_empty_model();
        let count = ("C1", "=COUNTIF(A:A,\">0\")+SUMIFS(A:A,A:A,\">1\")");
        let spill = ("A1", "=SEQUENCE(3)");
        let order = if count_first {
            [count, spill]
        } else {
            [spill, count]
        };
        for (cell, formula) in order {
            model._set(cell, formula);
        }
        model.evaluate();
        assert_eq!(model._get_text("C1"), "8", "count first: {count_first}");
    }
}

#[test]
fn a_lifted_countifs_is_priced_by_every_criteria_range_it_walks() {
    // DEVIATION (criteria.md "*IF(S) lift array criteria"): the walk of COUNTIFS visits the
    // stored cells of every criteria range, so the lift cap prices them all. A:A holds one cell
    // and B:B 100,000: 1,000 elements x 2 criteria x 100,001 positions is past 2^27, 10 are not.
    let mut model = new_empty_model();
    model._set("A1", "1");
    for row in 1..=100_000 {
        model._set(&format!("B{row}"), "1");
    }
    model._set("Z1", "=SUM(COUNTIFS(A:A,SEQUENCE(1000),B:B,\">0\"))");
    model._set("Z2", "=SUM(COUNTIFS(A:A,SEQUENCE(10),B:B,\">0\"))");
    model._set("Z3", "=SUM(SUMIFS(A:A,B:B,SEQUENCE(1000),A:A,1))");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#ERROR!");
    assert_eq!(model._get_text("Z2"), "1");
    assert_eq!(model._get_text("Z3"), "1");
}
