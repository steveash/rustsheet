#![allow(clippy::unwrap_used)]

//! Running totals from the prefix cache (T-3456) give what SUM's walk gives: the same values to
//! the bit, the first error, the same circular marks and the same work charged, under iteration
//! too.

use crate::expressions::types::CellReferenceIndex;
use crate::expressions::utils::{column_to_number, number_to_column};
use crate::test::util::new_empty_model;
use crate::types::Iteration;
use crate::Model;

/// Each cell's stored value and circular mark, and the work units the last recalculation spent, after `cells` are set and evaluated (then
/// `edits` set and evaluated again), with the pass caches on and off; and the states the cached
/// run kept in its last pass.
fn cached_and_walked(
    cells: &[(String, String)],
    edits: &[(String, String)],
    iterate: bool,
) -> (Vec<(String, String, String)>, usize) {
    let run = |no_pass_caches: bool| {
        let mut model = new_empty_model();
        if iterate {
            model.workbook.settings.iteration = Some(Iteration {
                max_iterations: 20,
                max_change: 0.001,
            });
        }
        model.evaluation.no_pass_caches = no_pass_caches;
        for (cell, value) in cells {
            model._set(cell, value);
        }
        model.evaluate();
        for (cell, value) in edits {
            model._set(cell, value);
        }
        if !edits.is_empty() {
            model.evaluate();
        }
        let mut values: Vec<String> = cells.iter().map(|(cell, _)| state(&model, cell)).collect();
        // The work charged: the cached path charges the range node as the walk does.
        let spent = model
            .evaluation
            .last_recalc
            .as_ref()
            .map(|last| last.spent());
        values.push(format!("spent {spent:?}"));
        (values, model.evaluation.sum_prefixes.kept())
    };
    let (cached, kept) = run(false);
    let (walked, _) = run(true);
    let names = cells
        .iter()
        .map(|(cell, _)| cell.clone())
        .chain(["(work)".to_string()]);
    (
        names
            .zip(cached.into_iter().zip(walked))
            .map(|(cell, (cached, walked))| (cell, cached, walked))
            .collect(),
        kept,
    )
}

/// The stored value of Sheet1's `cell`, exactly, and whether it is marked circular.
fn state(model: &Model, cell: &str) -> String {
    let split = cell.find(|c: char| c.is_ascii_digit()).unwrap();
    let at = CellReferenceIndex {
        sheet: 0,
        row: cell[split..].parse().unwrap(),
        column: column_to_number(&cell[..split]).unwrap(),
    };
    let circular = model
        .evaluation
        .circular
        .contains(&crate::evaluation::key(at));
    format!(
        "{:?} {} circular={circular}",
        model._get_cell(cell),
        model._get_text(cell)
    )
}

/// Asserts the cached and walked runs agree everywhere; returns the states kept.
fn assert_alike(cells: &[(String, String)], edits: &[(String, String)], iterate: bool) -> usize {
    let (results, kept) = cached_and_walked(cells, edits, iterate);
    for (cell, cached, walked) in &results {
        assert_eq!(cached, walked, "{cell}");
    }
    kept
}

fn cell(name: String, value: &str) -> (String, String) {
    (name, value.to_string())
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
