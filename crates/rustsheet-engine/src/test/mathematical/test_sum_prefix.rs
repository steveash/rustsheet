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
