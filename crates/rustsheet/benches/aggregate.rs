//! Timings for SPEC-0009's aggregate-argument walker: SUM and AVERAGE over a numeric column of
//! 1,000,000 cells, and SUM over the same column with an error in A2 (the walk ends there).
//! `cargo bench -p rustsheet --bench aggregate`
//!
//! One warm-up run, then `RUNS` measured recalculations; each figure is the median.

use std::error::Error;
use std::io::Write;
use std::time::{Duration, Instant};

use rustsheet::{Input, Workbook, WorkbookOptions};

const ROWS: u32 = 1_000_000;
const RUNS: usize = 5;

/// A workbook with 1..=ROWS in column A (A2 an error when `error_in_a2`) and `formula` in B1.
fn workbook(formula: &str, error_in_a2: bool) -> Result<Workbook, Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.set_track_changes(false);
    wb.edit(|tx| {
        for r in 1..=ROWS {
            tx.set(format!("A{r}").as_str(), f64::from(r))?;
        }
        if error_in_a2 {
            tx.set("A2", Input::Formula("=NA()".to_string()))?;
        }
        tx.set("B1", Input::Formula(formula.to_string()))?;
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
        ("SUM", "=SUM(A1:A1000000)", false),
        ("AVERAGE", "=AVERAGE(A1:A1000000)", false),
        ("SUM, error in A2", "=SUM(A1:A1000000)", true),
    ];
    for (name, formula, error_in_a2) in cases {
        let mut wb = workbook(formula, error_in_a2)?;
        writeln!(
            std::io::stdout(),
            "{name}: rows={ROWS} runs={RUNS} (median) recalc={:?}",
            median_recalc(&mut wb)
        )?;
    }
    Ok(())
}
