#![allow(clippy::unwrap_used)]

// Ranges read as a whole. A function that walks a whole column or row visits
// only the sheet's used area; the remainder is put on record as one
// rectangle read empty (`clip_to_used_area`, `record_seen_empty_range`), so
// that a spill into it restarts the pass like a spill into a recorded cell,
// and so that a whole-column read does not record one entry per cell.

use crate::test::util::{balanced_union, new_empty_model};
use crate::Model;

fn restarts(model: &Model) -> u32 {
    model.evaluation.restarts_in_last_evaluation
}

fn anchor_order(model: &Model) -> Vec<String> {
    model
        .evaluation
        .anchor_order
        .iter()
        .map(|a| {
            let column = (b'A' + (a.column - 1) as u8) as char;
            format!("{column}{}", a.row)
        })
        .collect()
}

//         ║       A          |       B        |       C        |       D        |
// ════════╬══════════════════╪════════════════╪════════════════╪════════════════╪
//    1    ║ =SEQUENCE(1,1,SUM(D:D)) |         | =SEQUENCE(3,1,10) | =SEQUENCE(5) |
//
// A1 is an anchor that sums a whole column, and runs first. The used area
// ends at row 3 (C1's spill) when the sum clips D:D, and D1, evaluated on
// demand by the walk, spills to row 5. The rows the walk skipped were on
// record: D1's commit contradicts them, and A1 is recomputed after D1.
// Before the record existed, A1 kept the sum of the three rows it walked.
#[test]
fn a_spill_beyond_the_used_area_restarts_the_anchor_that_summed_the_column() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,SUM(D:D))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "15");
    assert_eq!(model._get_text("D5"), "5");
    assert_eq!(restarts(&model), 1);
    assert_eq!(super::oracle::violations(&mut model), Vec::<String>::new());
    let order = anchor_order(&model);
    let position = |cell: &str| order.iter().position(|c| c == cell).unwrap();
    assert!(position("D1") < position("A1"));

    model.evaluate();
    assert_eq!(model._get_text("A1"), "15");
    assert_eq!(restarts(&model), 0);
}

// The spill lands entirely outside the used area at the time of the read:
// B7:B10 is the used area, D12 spills below it. Same mechanism.
#[test]
fn a_spill_entirely_beyond_the_used_area_is_seen() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,SUM(D:D))");
    model._set("B7", "=SEQUENCE(4)");
    model._set("D12", "=SEQUENCE(2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(restarts(&model), 1);
    assert_eq!(super::oracle::violations(&mut model), Vec::<String>::new());
}

// COUNTIF clips differently: it counts the cells beyond the used area as
// empty without visiting them. They are on record all the same.
#[test]
fn a_countif_over_a_whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,COUNTIF(D:D,\">0\"))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(restarts(&model), 1);
    assert_eq!(super::oracle::violations(&mut model), Vec::<String>::new());
}

// A whole-row read, the other direction of the clip.
#[test]
fn a_spill_beyond_the_used_area_of_a_whole_row_is_seen() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,SUM(3:3))");
    model._set("A3", "=SEQUENCE(1,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "15");
    assert_eq!(super::oracle::violations(&mut model), Vec::<String>::new());
}

// A reader that is not an anchor runs after every anchor has spilled, so it
// needs no restart at all.
#[test]
fn an_ordinary_formula_summing_a_whole_column_needs_no_restart() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(D:D)");
    model._set("D1", "=SEQUENCE(3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(restarts(&model), 0);
}

// The skipped remainder of a whole-column read is one rectangle, not a
// record per cell: the records after an evaluation hold nothing beyond
// the cells actually visited.
#[test]
fn a_whole_column_read_records_one_rectangle() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(D:D)");
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("D3", "");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert!(
        model.evaluation.seen.len() <= 4,
        "{:?}",
        model.evaluation.seen
    );
    assert_eq!(model.evaluation.seen_ranges.len(), 1);
    let range = model.evaluation.seen_ranges[0];
    assert_eq!((range.column1, range.column2), (4, 4));
    assert!(range.row1 <= 4);
    assert_eq!(range.row2, crate::constants::LAST_ROW);
}

// A union area is recorded as one rectangle (T-1497), and a union walked once per
// row of a MAP records its rectangles once: the spill check scans what was read,
// not how often.
#[test]
fn a_repeated_union_walk_records_its_rectangles_once() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(MAP(F1:F1000,LAMBDA(r,SUM((D:D,E1:E10)))))");
    model._set("D1", "1");
    model._set("E2", "2");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3000");
    let union_reads: Vec<_> = model
        .evaluation
        .seen_ranges
        .iter()
        .filter(|range| range.column1 >= 4)
        .collect();
    assert_eq!(union_reads.len(), 2, "{union_reads:?}");
}

// XLOOKUP clips its whole-column arrays like the others; it has no test
// file of its own, so its case lives here. The other clipped functions are
// tested the same way in their own files.
#[test]
fn an_xlookup_over_a_whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,XLOOKUP(5,D:D,D:D))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(restarts(&model), 1);
    assert_eq!(super::oracle::violations(&mut model), Vec::<String>::new());

    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(restarts(&model), 0);
}

// T-1463: `MAX_AREAS` copies of A:A walked once per row of a MAP over an empty sheet record one
// rectangle, not one per area per walk (2 x 2^24 before T-1497). The walk budget stops the
// formula (`#VALUE!`), as it does on main.
#[test]
fn many_copies_of_a_whole_column_in_a_map_record_one_rectangle() {
    let mut model = new_empty_model();
    let union = balanced_union("A:A", crate::reference_operators::MAX_AREAS);
    model._set(
        "Z1",
        &format!("=LET(u,{union},SUM(MAP(SEQUENCE(16),LAMBDA(r,SUM(u)))))"),
    );
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "#VALUE!");
    assert_eq!(model.evaluation.seen_ranges.len(), 1);
}

// DEVIATION (references.md "Seen-range cap"): one formula may put at most MAX_SEEN_RANGES
// distinct rectangles on record. Each whole row OFFSET picks below the used area is one, so a MAP
// over MAX_SEEN_RANGES rows fits and one more row is `#VALUE!`, uncatchable by IFERROR; the
// formulas next to it keep their own budgets.
#[test]
fn a_formula_records_at_most_max_seen_ranges_rectangles() {
    use crate::area_budget::{MAX_SEEN_RANGES, TOO_MANY_SEEN_RANGES};
    let rows = |n: usize| format!("MAP(SEQUENCE({n}),LAMBDA(r,SUM(OFFSET(1:1,r,0))))");
    let mut model = new_empty_model();
    model._set("A1", &format!("=SUM({})", rows(MAX_SEEN_RANGES)));
    model._set("B1", &format!("=SUM({})", rows(MAX_SEEN_RANGES + 1)));
    model._set(
        "C1",
        &format!("=IFERROR(SUM({}),1)", rows(MAX_SEEN_RANGES + 1)),
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "0");
    assert_eq!(model._get_text("B1"), "#VALUE!");
    assert_eq!(model._get_error_message("B1"), TOO_MANY_SEEN_RANGES);
    assert_eq!(model._get_text("C1"), "#VALUE!");
    assert!(model.evaluation.seen_ranges.len() <= 3 * MAX_SEEN_RANGES);
}

// T-1512: a spill's commit check costs about its cells plus the rectangles on record, not their
// product. The anchors A1 and B1 each put 50,000 whole-row rectangles on record (rows 200,001 to
// 300,000, below the used area) and spill 50,000 cells; then C2 spills 100,000 cells above the
// rectangles. Checking every cell against every rectangle is over 10^10 tests.
#[test]
fn a_large_spill_is_checked_against_many_rectangles_in_near_linear_time() {
    use std::time::{Duration, Instant};
    let rows =
        |from: usize| format!("=MAP(SEQUENCE(50000,1,{from}),LAMBDA(r,SUM(OFFSET(1:1,r,0))))");
    let mut model = new_empty_model();
    model._set("A1", &rows(200_000));
    model._set("B1", &rows(250_000));
    model._set("C2", "=SEQUENCE(100000)");
    let start = Instant::now();
    model.evaluate();
    model.evaluate();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(60), "{elapsed:?}");
    assert_eq!(model._get_text("A50000"), "0");
    assert_eq!(model._get_text("C100001"), "100000");
    assert_eq!(model.evaluation.seen_ranges.len(), 100_000);
    assert_eq!(restarts(&model), 0);
}

// The spill check meets a rectangle at the first cell, row by row, that the spill writes in it;
// the anchor's own cell is not a write.
#[test]
fn a_rectangle_is_met_at_the_first_cell_the_spill_writes_in_it() {
    use crate::expressions::types::CellReferenceIndex;
    use crate::seen_ranges::SeenRange;
    let anchor = CellReferenceIndex {
        sheet: 0,
        row: 2,
        column: 3,
    };
    let range = |sheet, row1, column1, row2, column2| SeenRange {
        root: anchor,
        sheet,
        row1,
        column1,
        row2,
        column2,
    };
    // A spill of 3 wide by 4 high: C2:E5.
    let area = (3, 4);
    assert_eq!(
        range(0, 1, 1, 9, 9).first_cell_written(anchor, area),
        Some((2, 4))
    );
    assert_eq!(
        range(0, 1, 1, 9, 3).first_cell_written(anchor, area),
        Some((3, 3))
    );
    assert_eq!(
        range(0, 4, 5, 9, 9).first_cell_written(anchor, area),
        Some((4, 5))
    );
    assert_eq!(range(0, 2, 3, 2, 3).first_cell_written(anchor, area), None);
    assert_eq!(range(0, 6, 1, 9, 9).first_cell_written(anchor, area), None);
    assert_eq!(range(0, 1, 6, 9, 9).first_cell_written(anchor, area), None);
    assert_eq!(range(1, 1, 1, 9, 9).first_cell_written(anchor, area), None);
    assert_eq!(
        range(0, 1, 1, 9, 9).first_cell_written(anchor, (1, 1)),
        None
    );
}
