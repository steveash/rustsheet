#![allow(clippy::unwrap_used, reason = "tests")]

//! Range operands of operators read from the per-pass range-values cache (T-3471) give what the
//! cell walk gives: values to the bit, errors, circular marks and the work charged, under
//! iteration too; and they read fewer cells.

use crate::test::pass_cache_runs::{assert_alike, cell};
use crate::Model;

fn cell_reads(model: &Model) -> usize {
    usize::try_from(model.evaluation.cell_reads).unwrap()
}

/// What column A holds at `row`: numbers, text, booleans, numeric text, blanks, formulas
/// evaluated only when read, and errors.
fn data(row: usize) -> String {
    match row % 8 {
        0 => format!("{row}"),
        1 => "text".to_string(),
        2 => "TRUE".to_string(),
        3 => "'7".to_string(),
        4 => String::new(),
        5 => format!("=Z1*{row}"),
        6 if row == 46 => "=1/0".to_string(),
        6 => "-0.5".to_string(),
        _ => "Text".to_string(),
    }
}

/// Operands of several kinds of operator, some read before the range is cached, most after.
fn readers(range: &str, at: usize) -> Vec<String> {
    vec![
        format!("=SUMPRODUCT(({range}=H{at})*1)"),
        format!("=SUMPRODUCT(({range}>0)*1)"),
        format!("=SUM(IFERROR({range}*2,0))"),
        format!("=CONCAT(IFERROR(IF({range}<>\"\",\"x\",\"-\"),\"e\"))"),
        format!("=SUMPRODUCT(IFERROR(-{range},0))"),
        format!("=SUMPRODUCT(ISERROR({range}&\"\")*1)"),
        format!("=CONCAT(IFERROR({range}&\"|\",\"e|\"))"),
    ]
}

/// The readers, in columns C to F, of A1:A100, a two-column block A1:B80, the B spill, and a
/// range past the used area (left to the walk).
fn workbook() -> Vec<(String, String)> {
    let mut cells = vec![cell("Z1".into(), "=Z2+0.25"), cell("Z2".into(), "1")];
    for row in 1..=100 {
        cells.push(cell(format!("A{row}"), &data(row)));
    }
    cells.push(cell("B1".into(), "=SEQUENCE(80)"));
    let criteria = ["text", "TEXT", "7", "TRUE", "", "40"];
    for (index, criterion) in criteria.iter().enumerate() {
        cells.push(cell(format!("H{}", index + 1), criterion));
    }
    let ranges = ["$A$1:$A$100", "$A$1:$B$80", "$B$1:$B$80", "$A$1:$A$400"];
    for (column, range) in ["C", "D", "E", "F"].iter().zip(ranges) {
        for row in 1..=30 {
            let reader = &readers(range, row / 7 % 6 + 1)[row % 7];
            cells.push(cell(format!("{column}{row}"), reader));
        }
    }
    cells
}

#[test]
fn range_operands_match_the_walk() {
    let runs = assert_alike(&workbook(), &[], false, &cell_reads);
    assert!(runs.cached < runs.walked, "{} {}", runs.cached, runs.walked);
}

/// A reader inside its own range is circular, and so are the readers that read it; an edit
/// recalculates.
#[test]
fn range_operands_over_a_circular_cell_match_the_walk() {
    let mut cells = workbook();
    cells.push(cell("A60".into(), "=SUMPRODUCT(($A$1:$A$100=1)*1)"));
    let edits = [cell("Z2".into(), "3")];
    assert_alike(&cells, &edits, false, &cell_reads);
}

/// Under iterative calculation: a cycle member inside the range, and an edit that recalculates.
#[test]
fn range_operands_over_a_cycle_match_the_walk() {
    let mut cells = workbook();
    cells.push(cell("K1".into(), "=K1+1"));
    cells.push(cell("A61".into(), "=K1"));
    let edits = [cell("Z2".into(), "3")];
    assert_alike(&cells, &edits, true, &cell_reads);
}
