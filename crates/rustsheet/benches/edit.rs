//! Timings for SPEC-0001's size envelope and budget gate (100k chained formulas).
//! `cargo bench -p rustsheet --bench edit`

use std::error::Error;
use std::io::Write;
use std::time::Instant;

use rustsheet::{ChangeSet, EditError, Input, Workbook, WorkbookOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    let n = 100_000u32;
    wb.set_track_changes(false);
    wb.edit(|tx| {
        tx.set("A1", 1.0)?;
        for r in 2..=n {
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
    writeln!(
        std::io::stdout(),
        "formulas={n} recalc={recalc:?} set(untracked)={untracked:?} set(tracked)={tracked:?} \
         diff_overhead={:?} failed_edit_rollback={rollback:?} changes={}",
        tracked.saturating_sub(untracked),
        cs.values.len()
    )?;
    Ok(())
}
