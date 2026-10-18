//! Timings for SPEC-0011's lifting driver (`functions/lift/driver.rs`), T-1784. Column A holds
//! runs of `RUN_LENGTH` equal values (0, 0, ..., 1, 1, ...) and column B the row number:
//! - a lifted whole-column COUNTIFS, `=SUM(COUNTIFS(A:A,A1:A100000))` (the `reuse_equal_runs`
//!   path: 10 walks, 100,000 slot comparisons);
//! - a SUMIFS with an array criterion, `=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))`
//!   (10 distinct criteria, one walk each);
//! - SWITCH lifted over 1,000,000 cells, where the driver itself dominates: a numeric result,
//!   `=SUM(SWITCH(A1:A1000000,0,1,1,2,3))`, and a text one, `=ROWS(SWITCH(A1:A1000000,0,"a","b"))`
//!   (every element a clone of a reused text result);
//! - PMT lifted over a 100,000-cell range, `=SUM(PMT(A1:A100000,10,100))` (`lift_call`'s in-place
//!   single-grid path with a `Lift` kernel, T-1372; a smoke number, SPEC-0011 Performance).
//!
//! - the quantile functions at their lift cap (T-1848): BETA.INV's full bisection and F.INV's
//!   377-step Newton search over 512 elements, T.INV.2T's bisection over 2,048 (the cases measured,
//!   which cost far less than the bound the lifting table charges).
//!
//! - the incomplete-gamma functions at their lift cap (T-1872): GAMMA.DIST's continued fraction at
//!   its 2^19-term cap (the bound's slower term) and CHISQ.DIST at the largest chi-squared shape,
//!   over 512 elements.
//!
//! `cargo bench -p rustsheet --bench lift [-- FILTER...]` (only the cases whose name contains a
//! FILTER, e.g. `SWITCH`)
//!
//! One warm-up run, then `RUNS` measured recalculations; each case prints the fastest, the median
//! and the spread.
//!
//! Compare a parent and a branch one case per process (`-- NAME`): run together, a later case
//! inherits the allocator state of the earlier ones and its times are not comparable (T-1784).

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};

const RUN_LENGTH: u32 = 10_000;
const RUNS: usize = 11;

/// A workbook with `rows` rows of runs in column A, the row number in column B, and `formula` in
/// C1.
fn workbook(rows: u32, formula: &str) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=rows {
            tx.set(format!("A{r}").as_str(), f64::from((r - 1) / RUN_LENGTH))?;
            tx.set(format!("B{r}").as_str(), f64::from(r))?;
        }
        tx.set("C1", Input::Formula(formula.to_string()))?;
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            "COUNTIFS, lifted whole column",
            100_000,
            "=SUM(COUNTIFS(A:A,A1:A100000))",
        ),
        (
            "SUMIFS, array criterion",
            100_000,
            "=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))",
        ),
        (
            "SWITCH, numeric result",
            1_000_000,
            "=SUM(SWITCH(A1:A1000000,0,1,1,2,3))",
        ),
        (
            "SWITCH, text result",
            1_000_000,
            "=ROWS(SWITCH(A1:A1000000,0,\"a\",\"b\"))",
        ),
        (
            "PMT, lifted over a range",
            100_000,
            "=SUM(PMT(A1:A100000,10,100))",
        ),
        (
            "BETA.INV, full bisection at the element cap",
            1,
            "=SUM(BETA.INV(SEQUENCE(512)*0+1E-300,0.5,1E6))",
        ),
        (
            "F.INV, 377-step Newton search at the element cap",
            1,
            "=SUM(F.INV(SEQUENCE(512)*0+3.718810279553989E-162,4532,43))",
        ),
        (
            "T.INV.2T, bisection at the element cap",
            1,
            "=SUM(T.INV.2T(SEQUENCE(2048)*0+1E-9,99999))",
        ),
        (
            "GAMMA.DIST, continued fraction at its term cap, at the element cap",
            1,
            "=ROWS(GAMMA.DIST(SEQUENCE(512)*0+1.0000001E300,1E300,1,TRUE))",
        ),
        (
            "CHISQ.DIST, the largest shape at the element cap",
            1,
            "=SUM(CHISQ.DIST(SEQUENCE(512)*0+1E10,1E10,TRUE))",
        ),
    ];
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    for (name, rows, formula) in cases {
        if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
            continue;
        }
        let mut wb = workbook(rows, formula)?;
        let t = common::timings(RUNS, || wb.recalculate());
        let result = wb.value("C1")?;
        writeln!(
            std::io::stdout(),
            "{name}: rows={rows} runs={RUNS} {t} C1={result:?}"
        )?;
    }
    Ok(())
}
