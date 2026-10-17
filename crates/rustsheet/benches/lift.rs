//! Timings for SPEC-0011's lifting driver (`functions/lift/driver.rs`), T-1784:
//! - a lifted whole-column COUNTIFS, `=SUM(COUNTIFS(A:A,A1:A100000))`, over a column of 10 runs
//!   of 10,000 equal values (the `reuse_equal_runs` path: 10 walks, 100,000 slot comparisons);
//! - a SUMIFS with an array criterion, `=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))`
//!   (10 distinct criteria, one walk each).
//!
//! `cargo bench -p rustsheet --bench lift`
//!
//! One warm-up run, then `RUNS` measured recalculations; each figure is the median.

use std::error::Error;
use std::io::Write;
use std::time::{Duration, Instant};

use rustsheet::{Input, Workbook, WorkbookOptions};

const ROWS: u32 = 100_000;
const RUN_LENGTH: u32 = 10_000;
const RUNS: usize = 5;

/// A workbook with runs of `RUN_LENGTH` equal values in A1:A`ROWS` (0, 0, ..., 1, 1, ...), the
/// row number in column B, and `formula` in C1.
fn workbook(formula: &str) -> Result<Workbook, Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.set_track_changes(false);
    wb.edit(|tx| {
        for r in 1..=ROWS {
            tx.set(format!("A{r}").as_str(), f64::from((r - 1) / RUN_LENGTH))?;
            tx.set(format!("B{r}").as_str(), f64::from(r))?;
        }
        tx.set("C1", Input::Formula(formula.to_string()))?;
        Ok(())
    })?;
    Ok(wb)
}

/// The median time of a full recalculation of `wb`.
fn median_recalc(wb: &mut Workbook) -> Duration {
    wb.recalculate(); // warm-up
    let mut v: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let t = Instant::now();
            wb.recalculate();
            t.elapsed()
        })
        .collect();
    v.sort();
    v[v.len() / 2]
}

fn main() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            "COUNTIFS, lifted whole column",
            "=SUM(COUNTIFS(A:A,A1:A100000))",
        ),
        (
            "SUMIFS, array criterion",
            "=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))",
        ),
    ];
    for (name, formula) in cases {
        let mut wb = workbook(formula)?;
        let time = median_recalc(&mut wb);
        let result = wb.value("C1")?;
        writeln!(
            std::io::stdout(),
            "{name}: rows={ROWS} runs={RUNS} (median) recalc={time:?} C1={result:?}"
        )?;
    }
    Ok(())
}
