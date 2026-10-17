//! Timing and workbook set-up shared by the `harness = false` benches (`mod common;` in each).
//! Criterion is not a dependency, so each bench times its runs with `timings` and prints the
//! fastest, the median and the spread.

use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};

use rustsheet::{EditError, Txn, Workbook, WorkbookOptions};

/// The fastest, median and slowest of a bench's measured runs.
pub struct Timings {
    pub min: Duration,
    pub median: Duration,
    pub max: Duration,
}

impl fmt::Display for Timings {
    /// `min=… median=… spread=…`, the spread being max − min. Compare the fastest across builds:
    /// on a shared machine it is far steadier than the median.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "min={:?} median={:?} spread={:?}",
            self.min,
            self.median,
            self.max.saturating_sub(self.min)
        )
    }
}

/// The timings of `runs` calls of `run`, after one warm-up call.
pub fn timings(runs: usize, mut run: impl FnMut()) -> Timings {
    run();
    let mut v: Vec<Duration> = (0..runs.max(1))
        .map(|_| {
            let t = Instant::now();
            run();
            t.elapsed()
        })
        .collect();
    v.sort();
    Timings {
        min: v[0],
        median: v[v.len() / 2],
        max: v[v.len() - 1],
    }
}

/// A new workbook, change tracking off, with the cells `fill` sets in one edit.
pub fn filled_workbook(
    fill: impl FnOnce(&mut Txn<'_>) -> Result<(), EditError>,
) -> Result<Workbook, Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.set_track_changes(false);
    wb.edit(fill)?;
    Ok(wb)
}
