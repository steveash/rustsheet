#![allow(clippy::unwrap_used)]

//! Running totals from the prefix cache (T-3456) give what SUM's walk gives: the same values to
//! the bit, the first error, the same circular marks and the same work charged, under iteration
//! too.

use crate::expressions::utils::number_to_column;
use crate::test::pass_cache_runs::{cell, Runs};
use crate::Model;

/// The states the cached run kept in its last pass.
fn kept(model: &Model) -> usize {
    model.evaluation.sum_prefixes.kept()
}

/// [`crate::test::pass_cache_runs::assert_alike`], counting the states kept.
fn assert_alike(cells: &[(String, String)], edits: &[(String, String)], iterate: bool) -> usize {
    let Runs { cached, .. } =
        crate::test::pass_cache_runs::assert_alike(cells, edits, iterate, &kept);
    cached
}

/// What column A holds at `row`: numbers that cancel to near zero, text, booleans, numeric text,
/// blanks, formulas evaluated only when read, and an error at row 40.
fn data(row: usize) -> &'static str {
    match row % 9 {
        _ if row == 40 => "=1/0",
        0 => "0.1",
        1 => "0.2",
        2 => "-0.3",
        3 => "text",
        4 => "TRUE",
        5 => "'7",
        6 => "",
        7 => "=Z1*3",
        _ => "1E+300",
    }
}

/// Running totals down, up (the longest first) and from a second start, over values, text,
/// booleans, blanks, unevaluated formulas and an error.
#[test]
fn running_totals_down_a_column_match_the_walk() {
    let mut cells = vec![cell("Z1".into(), "=Z2+0.7"), cell("Z2".into(), "0.4")];
    for row in 1..=60 {
        cells.push(cell(format!("A{row}"), data(row)));
        cells.push(cell(format!("B{row}"), &format!("=SUM($A$1:A{row})")));
        cells.push(cell(
            format!("C{row}"),
            &format!("=SUM($A$1:A{})", 61 - row),
        ));
        cells.push(cell(format!("D{row}"), &format!("=SUM(A$5:A{})", row + 4)));
        cells.push(cell(format!("E{row}"), &format!("=SUM(A{row}:$A$1)")));
    }
    assert!(assert_alike(&cells, &[], false) > 0);
}

/// Running totals across a row, as `SUM($I11:I11)` filled right, and a range of two rows, which
/// the cache leaves to the walk.
#[test]
fn running_totals_across_a_row_match_the_walk() {
    let mut cells = vec![cell("Z1".into(), "=Z2+0.7"), cell("Z2".into(), "0.4")];
    for column in 1..=40 {
        let name = number_to_column(column).unwrap();
        cells.push(cell(format!("{name}1"), data(column as usize)));
        cells.push(cell(format!("{name}2"), "3"));
        cells.push(cell(format!("{name}3"), &format!("=SUM($A1:{name}1)")));
        cells.push(cell(format!("{name}4"), &format!("=SUM($A1:{name}2)")));
    }
    assert!(assert_alike(&cells, &[], false) > 0);
}

/// A running total over its own column reads the cells it caches from (`B5 = SUM($B$1:B4)+1`),
/// one over its own cell is circular, a total over formulas that read a later total, and a total
/// whose walk reads shorter totals from its own start.
#[test]
fn running_totals_over_running_totals_match_the_walk() {
    let mut cells = vec![cell("B1".into(), "1")];
    for row in 2..=30 {
        cells.push(cell(
            format!("B{row}"),
            &format!("=SUM($B$1:B{})+1", row - 1),
        ));
        cells.push(cell(format!("C{row}"), &format!("=SUM($B$1:B{row})")));
        cells.push(cell(format!("D{row}"), &format!("=SUM($E$1:E{row})")));
        cells.push(cell(format!("E{row}"), &format!("=C{}", row + 1)));
    }
    cells.push(cell("C31".into(), "=SUM($C$1:C31)"));
    // G1 reads F30 first, whose walk reads F2..F29, each a running total from the same start,
    // and H40 reads past them.
    cells.push(cell("F1".into(), "1"));
    for row in 2..=30 {
        cells.push(cell(
            format!("F{row}"),
            &format!("=SUM($F$1:F{})+1", row - 1),
        ));
    }
    for row in 31..=40 {
        cells.push(cell(format!("F{row}"), "2"));
    }
    cells.push(cell("G1".into(), "=F30"));
    cells.push(cell("H40".into(), "=SUM($F$1:F40)"));
    assert!(assert_alike(&cells, &[], false) > 0);
}

/// Under iterative calculation: a cycle member inside the totals' range, and an edit that
/// recalculates.
#[test]
fn running_totals_over_a_cycle_match_the_walk() {
    let mut cells = vec![cell("K1".into(), "=K1+1"), cell("H1".into(), "1")];
    for row in 1..=30 {
        let value = if row == 12 { "=K1" } else { "=$H$1*2" };
        cells.push(cell(format!("A{row}"), value));
        cells.push(cell(format!("B{row}"), &format!("=SUM($A$1:A{row})")));
    }
    let edits = [cell("H1".into(), "5")];
    assert_alike(&cells, &edits, true);
    assert_alike(&cells, &edits, false);
}

/// `starts` totals, each from its own start at an error cell, so that each entry holds no state.
fn error_starts(starts: usize) -> Vec<(String, String)> {
    let mut cells = Vec::new();
    for row in 1..=starts + 1 {
        cells.push(cell(format!("A{row}"), "=1/0"));
    }
    for row in 1..=starts {
        cells.push(cell(
            format!("B{row}"),
            &format!("=SUM(A{row}:A{})", row + 1),
        ));
    }
    cells
}

/// The model of `cells` evaluated with the cache's limit set to `limit`.
fn evaluated_with_limit(cells: &[(String, String)], limit: usize) -> Model {
    let mut model = crate::test::util::new_empty_model();
    model.evaluation.sum_prefixes.limit = limit;
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model.evaluate();
    model
}

/// An entry that stopped at an error in its first cell holds no state and still counts against
/// the limit (T-3470), so a lowered limit bounds the entries and the walk answers past it.
#[test]
fn entries_without_states_count_against_the_limit() {
    let cells = error_starts(20);
    assert_eq!(assert_alike(&cells, &[], false), 20);
    // A new entry reserves room for every state it may keep (here two) before it reads.
    let model = evaluated_with_limit(&cells, 5);
    let kept = model.evaluation.sum_prefixes.kept();
    assert!((1..=5).contains(&kept), "{kept}");
    for row in 1..=20 {
        assert_eq!(model._get_text(&format!("B{row}")), "#DIV/0!", "B{row}");
    }
}

/// A hit keeps nothing new, so at a full limit the totals already kept are still read from the
/// cache: the cells read are those of a limit with room to spare.
#[test]
fn hits_need_no_room_under_the_limit() {
    let mut cells = Vec::new();
    for row in 1..=10 {
        cells.push(cell(format!("A{row}"), &row.to_string()));
        cells.push(cell(format!("B{row}"), &format!("=SUM($A$1:A{row})")));
        cells.push(cell(
            format!("C{row}"),
            &format!("=SUM($A$1:A{})", 11 - row),
        ));
    }
    let reads = |limit: usize| {
        let model = evaluated_with_limit(&cells, limit);
        assert_eq!(model.evaluation.sum_prefixes.kept(), 10, "{limit}");
        assert_eq!(model._get_text("C1"), "55");
        model.evaluation.cell_reads
    };
    assert_eq!(reads(10), reads(1_000));
}

/// A hit on an entry that stopped at an error needs no room either: at a full limit the longer
/// totals from that start read the error from the cache, not their cells.
#[test]
fn error_hits_need_no_room_under_the_limit() {
    let mut cells = vec![cell("A1".into(), "=1/0"), cell("A2".into(), "2")];
    for row in 1..=10 {
        cells.push(cell(format!("B{row}"), &format!("=SUM($A$1:A{})", row + 1)));
    }
    let reads = |limit: usize| {
        let model = evaluated_with_limit(&cells, limit);
        assert_eq!(model.evaluation.sum_prefixes.kept(), 1, "{limit}");
        assert_eq!(model._get_text("B10"), "#DIV/0!");
        model.evaluation.cell_reads
    };
    assert_eq!(reads(2), reads(1_000));
}
