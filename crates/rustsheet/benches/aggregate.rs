//! Timings for SPEC-0009's aggregate-argument walker: SUM and AVERAGE over a numeric column of
//! 1,000,000 cells, and SUM over the same column with an error in A2 (the walk ends there).
//! `cargo bench -p rustsheet --bench aggregate`
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

fn main() -> Result<(), Box<dyn Error>> {
    let cases = [
        ("SUM", "=SUM(A1:A1000000)", false),
        ("AVERAGE", "=AVERAGE(A1:A1000000)", false),
        ("SUM, error in A2", "=SUM(A1:A1000000)", true),
    ];
    for (name, formula, error_in_a2) in cases {
        let mut wb = workbook(formula, error_in_a2)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(std::io::stdout(), "{name}: rows={ROWS} runs={RUNS} {t}")?;
    }
    Ok(())
}
