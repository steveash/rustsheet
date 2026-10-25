//! Timings for SPEC-0009's aggregate-argument walker: SUM and AVERAGE over a numeric column of
//! 1,000,000 cells, SUM over the same column with an error in A2 (the walk ends there), and SUM
//! over a block three columns wide reaching past the used area (T-0405: cut at the used area).
//! Then criteria aggregates over shared ranges (T-0449): 2,000 SUMIFS and 2,000 COUNTIFS over
//! 50,000 rows, with five text criteria and 100 numeric thresholds.
//! `cargo bench -p rustsheet --bench aggregate` (`-- criteria` runs only the T-0449 case)
//!
//! One warm-up run, then `RUNS` measured recalculations; each case prints the fastest, the median
//! and the spread.

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};

const ROWS: u32 = 1_000_000;
const RUNS: usize = 5;

/// A workbook with 1..=ROWS in column A (A2 an error when `error_in_a2`) and `formula` in B1.
fn workbook(formula: &str, error_in_a2: bool) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=ROWS {
            tx.set(format!("A{r}").as_str(), f64::from(r))?;
        }
        if error_in_a2 {
            tx.set("A2", Input::Formula("=NA()".to_string()))?;
        }
        tx.set("B1", Input::Formula(formula.to_string()))?;
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut wb = criteria_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        std::io::stdout(),
        "SUMIFS + COUNTIFS: {CRITERIA_CALLS} each over {CRITERIA_ROWS} rows, runs={RUNS} {t}"
    )?;
    if std::env::args().any(|arg| arg == "criteria") {
        return Ok(());
    }
    let cases = [
        ("SUM", "=SUM(A1:A1000000)", false),
        ("AVERAGE", "=AVERAGE(A1:A1000000)", false),
        ("SUM, error in A2", "=SUM(A1:A1000000)", true),
        ("SUM, block past the used area", "=SUM(A1:C1048000)", false),
    ];
    for (name, formula, error_in_a2) in cases {
        let mut wb = workbook(formula, error_in_a2)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(std::io::stdout(), "{name}: rows={ROWS} runs={RUNS} {t}")?;
    }
    Ok(())
}
