//! Timings for SPEC-0009's aggregate-argument walker: SUM and AVERAGE over a numeric column of
//! 1,000,000 cells, SUM over the same column with an error in A2 (the walk ends there), and SUM
//! over a block three columns wide reaching past the used area (T-0405: cut at the used area),
//! and SUM over the whole sheet with a far cell in XFD1048576 (T-1690: walked over its cells).
//! Then a 3-D TEXTJOIN over whole columns beside a far cell, and the same SUM repeated by a
//! LAMBDA (T-2710: blank runs counted, not read).
//! Then criteria aggregates over shared ranges (T-0449): 2,000 SUMIFS and 2,000 COUNTIFS over
//! 50,000 rows, with five text criteria and 100 numeric thresholds. Then aggregates over
//! whole-column expressions (T-0469): 400 `MAX(IF(F:F=A1,G:G,""))` over 2,000 rows.
//! `cargo bench -p rustsheet --bench aggregate` (`-- criteria` and `-- whole_column` run only
//! the T-0449 or the T-0469 case)
//!
//! One warm-up run, then `RUNS` measured recalculations; each case prints the fastest, the median
//! and the spread.

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};

const ROWS: u32 = 1_000_000;
const RUNS: usize = 5;

/// A workbook with 1..=ROWS in column A (A2 an error when `error_in_a2`, and 1 in XFD1048576
/// when `far_cell`) and `formula` in B1.
fn workbook(formula: &str, error_in_a2: bool, far_cell: bool) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=ROWS {
            tx.set(format!("A{r}").as_str(), f64::from(r))?;
        }
        if error_in_a2 {
            tx.set("A2", Input::Formula("=NA()".to_string()))?;
        }
        if far_cell {
            tx.set("XFD1048576", 1.0)?;
        }
        tx.set("B1", Input::Formula(formula.to_string()))?;
        Ok(())
    })
}

/// Two sheets each holding only 1 in J1048576, with `formula` in Sheet1!Z1: every whole-column
/// walk of `A:J` spans the sheet height over one stored cell.
fn far_cell_workbook(formula: &str) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        tx.add_sheet("Two")?;
        tx.set("Sheet1!J1048576", 1.0)?;
        tx.set("Two!J1048576", 1.0)?;
        tx.set("Sheet1!Z1", Input::Formula(formula.to_string()))?;
        Ok(())
    })
}

/// Rows of the criteria ranges of the T-0449 case.
const CRITERIA_ROWS: u32 = 50_000;
/// SUMIFS calls, and as many COUNTIFS.
const CRITERIA_CALLS: u32 = 2_000;
const REGIONS: [&str; 5] = ["North", "South", "East", "West", "Central"];

/// A region in A, a tag in B (half before "t", half after) and an amount in C, with
/// `CRITERIA_CALLS` SUMIFS in E and as many COUNTIFS in F.
fn criteria_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=CRITERIA_ROWS {
            let region = REGIONS[(r % 5) as usize];
            let tag = format!("{}{r}", if r % 2 == 0 { "alpha" } else { "zeta" });
            tx.set(format!("A{r}").as_str(), region)?;
            tx.set(format!("B{r}").as_str(), tag.as_str())?;
            tx.set(format!("C{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        let n = CRITERIA_ROWS;
        for r in 1..=CRITERIA_CALLS {
            let region = REGIONS[(r % 5) as usize];
            let threshold = (r % 100) * 10;
            let sumifs =
                format!("=SUMIFS($C$1:$C${n},$A$1:$A${n},\"{region}\",$B$1:$B${n},\">t\")");
            let countifs =
                format!("=COUNTIFS($A$1:$A${n},\"{region}\",$C$1:$C${n},\">{threshold}\")");
            tx.set(format!("E{r}").as_str(), Input::Formula(sumifs))?;
            tx.set(format!("F{r}").as_str(), Input::Formula(countifs))?;
        }
        Ok(())
    })
}

/// Rows of data in the T-0469 case.
const WHOLE_COLUMN_ROWS: u32 = 2_000;
/// Whole-column MAX(IF(..)) calls.
const WHOLE_COLUMN_CALLS: u32 = 400;

/// A key in F (one of 50) and an amount in G, with `WHOLE_COLUMN_CALLS` keys in A and
/// `MAX(IF(F:F=A1,G:G,""))` beside each in B, as in corpus-bench 17049.
fn whole_column_workbook() -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=WHOLE_COLUMN_ROWS {
            tx.set(format!("F{r}").as_str(), format!("k{}", r % 50).as_str())?;
            tx.set(format!("G{r}").as_str(), f64::from((r * 37) % 1000))?;
        }
        for r in 1..=WHOLE_COLUMN_CALLS {
            tx.set(format!("A{r}").as_str(), format!("k{}", r % 50).as_str())?;
            let formula = format!("=MAX(IF(F:F=A{r},G:G,\"\"))");
            tx.set(format!("B{r}").as_str(), Input::Formula(formula))?;
        }
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut wb = whole_column_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "MAX(IF(F:F=A1,G:G,\"\")): {WHOLE_COLUMN_CALLS} calls over {WHOLE_COLUMN_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "whole_column") {
        return Ok(());
    }
    let mut wb = criteria_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "SUMIFS + COUNTIFS: {CRITERIA_CALLS} each over {CRITERIA_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "criteria") {
        return Ok(());
    }
    let far_cell_cases = [
        (
            "TEXTJOIN 3-D, whole columns beside a far cell",
            "=LEN(TEXTJOIN(\",\",FALSE,Sheet1:Two!A:J))",
        ),
        (
            "SUM(A:J) beside a far cell, 100 times in a LAMBDA",
            "=SUM(BYROW(SEQUENCE(100),LAMBDA(r,SUM(Two!A:J))))",
        ),
    ];
    for (name, formula) in far_cell_cases {
        let mut wb = far_cell_workbook(formula)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(std::io::stdout(), "{name}: runs={RUNS} {t}")?;
    }
    let cases = [
        ("SUM", "=SUM(A1:A1000000)", false, false),
        ("AVERAGE", "=AVERAGE(A1:A1000000)", false, false),
        ("SUM, error in A2", "=SUM(A1:A1000000)", true, false),
        (
            "SUM, block past the used area",
            "=SUM(A1:C1048000)",
            false,
            false,
        ),
        (
            "SUM, whole sheet with a far cell",
            "=SUM(A:XFD)",
            false,
            true,
        ),
    ];
    for (name, formula, error_in_a2, far_cell) in cases {
        let mut wb = workbook(formula, error_in_a2, far_cell)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(std::io::stdout(), "{name}: rows={ROWS} runs={RUNS} {t}")?;
    }
    Ok(())
}
