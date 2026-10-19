//! Timings for the math number functions on SPEC-0011's lifting driver (T-1773). Column A holds
//! `row / 7` (a fractional number in every row):
//! - unlifted per-row cells, a full recalculation of 100,000 rows of `=ROUND(A{r},2)`,
//!   `=ABS(A{r})` or `=MOD(A{r},7)` in column B (the driver's no-grid path, once per cell);
//! - lifted over a 1,000,000-cell range: `=SUM(ROUND(A1:A1000000,2))`, `=SUM(ABS(A1:A1000000))`;
//! - EXP (T-1931), whose x87 model V6 costs far more per call than libm's `exp`: 100,000 rows of
//!   `=EXP(A{r}/100000)` and the lifted `=SUM(EXP(A1:A1000000/1000000))` (arguments in (0, 1/7],
//!   inside the range where every call runs the full F2XM1 series);
//! - a lifted error fan-out, `=SUM(IFERROR(ROUND(A1:A1000000&"x",2),0))`: every element is
//!   `#VALUE!`, which must allocate nothing (an empty message).
//!
//! `cargo bench -p rustsheet --bench math_lift [-- FILTER...]` (only the cases whose name
//! contains a FILTER). Compare a parent and a branch one case per process (`-- NAME`): run
//! together, a later case inherits the allocator state of the earlier ones (T-1784).

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};

const RUNS: usize = 11;

/// What a case puts in the workbook besides column A.
enum Shape {
    /// `formula` with `{r}` replaced by the row, in B1:B`rows`.
    PerRow(&'static str),
    /// One formula in B1.
    Single(&'static str),
}

/// A workbook with `rows` rows of `r / 7` in column A and the case's formulas in column B.
fn workbook(rows: u32, shape: &Shape) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=rows {
            tx.set(format!("A{r}").as_str(), f64::from(r) / 7.0)?;
            if let Shape::PerRow(formula) = shape {
                let formula = formula.replace("{r}", &r.to_string());
                tx.set(format!("B{r}").as_str(), Input::Formula(formula))?;
            }
        }
        if let Shape::Single(formula) = shape {
            tx.set("B1", Input::Formula((*formula).to_string()))?;
        }
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cases = [
        ("ROUND, per row", 100_000, Shape::PerRow("=ROUND(A{r},2)")),
        ("ABS, per row", 100_000, Shape::PerRow("=ABS(A{r})")),
        ("MOD, per row", 100_000, Shape::PerRow("=MOD(A{r},7)")),
        ("EXP, per row", 100_000, Shape::PerRow("=EXP(A{r}/100000)")),
        (
            "ROUND, lifted",
            1_000_000,
            Shape::Single("=SUM(ROUND(A1:A1000000,2))"),
        ),
        (
            "ABS, lifted",
            1_000_000,
            Shape::Single("=SUM(ABS(A1:A1000000))"),
        ),
        (
            "EXP, lifted",
            1_000_000,
            Shape::Single("=SUM(EXP(A1:A1000000/1000000))"),
        ),
        (
            "ROUND, lifted errors",
            1_000_000,
            Shape::Single("=SUM(IFERROR(ROUND(A1:A1000000&\"x\",2),0))"),
        ),
    ];
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    for (name, rows, shape) in cases {
        if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
            continue;
        }
        let mut wb = workbook(rows, &shape)?;
        let t = common::timings(RUNS, || wb.recalculate());
        let result = wb.value("B1")?;
        writeln!(
            std::io::stdout(),
            "{name}: rows={rows} runs={RUNS} {t} B1={result:?}"
        )?;
    }
    Ok(())
}
