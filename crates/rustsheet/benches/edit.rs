//! Timings for SPEC-0001's size envelope and budget gate (100k chained formulas).
//! `cargo bench -p rustsheet --bench edit`
//!
//! One warm-up run, then `RUNS` measured runs on fresh workbooks; each figure is the median.

use std::error::Error;
use std::io::Write;
use std::time::{Duration, Instant};

use rustsheet::{ChangeSet, EditError, Input, Workbook, WorkbookOptions};

const FORMULAS: u32 = 100_000;
const RUNS: usize = 5;

/// One run's timings and the number of changes the tracked edit reported.
struct Sample {
    recalc: Duration,
    untracked: Duration,
    tracked: Duration,
    rollback: Duration,
    changes: usize,
}

fn run() -> Result<Sample, Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.set_track_changes(false);
    wb.edit(|tx| {
        tx.set("A1", 1.0)?;
        for r in 2..=FORMULAS {
            tx.set(
                format!("A{r}").as_str(),
                Input::Formula(format!("=A{}+1", r - 1)),
            )?;
        }
        Ok(())
    })?;
    let t = Instant::now();
    wb.recalculate();
    let recalc = t.elapsed();
    let t = Instant::now();
    wb.set("B1", 1.0)?;
    let untracked = t.elapsed();
    wb.set_track_changes(true);
    let t = Instant::now();
    let cs = wb.set("B1", 2.0)?;
    let tracked = t.elapsed();
    let t = Instant::now();
    let r: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("A1", 5.0)?;
        Err(EditError::Engine("x".into()))
    });
    let rollback = t.elapsed();
    assert!(r.is_err());
    Ok(Sample {
        recalc,
        untracked,
        tracked,
        rollback,
        changes: cs.values.len(),
    })
}

fn median(samples: &[Sample], f: impl Fn(&Sample) -> Duration) -> Duration {
    let mut v: Vec<Duration> = samples.iter().map(f).collect();
    v.sort();
    v[v.len() / 2]
}

fn main() -> Result<(), Box<dyn Error>> {
    run()?; // warm-up
    let samples = (0..RUNS).map(|_| run()).collect::<Result<Vec<_>, _>>()?;
    let untracked = median(&samples, |s| s.untracked);
    let tracked = median(&samples, |s| s.tracked);
    writeln!(
        std::io::stdout(),
        "formulas={FORMULAS} runs={RUNS} (median) recalc={:?} set(untracked)={untracked:?} \
         set(tracked)={tracked:?} diff_overhead={:?} failed_edit_rollback={:?} changes={}",
        median(&samples, |s| s.recalc),
        tracked.saturating_sub(untracked),
        median(&samples, |s| s.rollback),
        samples[0].changes
    )?;
    Ok(())
}
