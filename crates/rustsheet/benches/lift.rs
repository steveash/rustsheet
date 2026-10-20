//! Timings for SPEC-0011's lifting driver (`functions/lift/driver.rs`), T-1784. Column A holds
//! runs of `RUN_LENGTH` equal values (0, 0, ..., 1, 1, ...) and column B the row number:
//! - a lifted whole-column COUNTIFS, `=SUM(COUNTIFS(A:A,A1:A100000))` (the `Reuse::AnyEqual`
//!   path: 10 walks, 100,000 elements hashed);
//! - a SUMIFS with an array criterion, `=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))`
//!   (10 distinct criteria, one walk each);
//! - the occurrence-count idiom over a 10,000-element criteria range of `CYCLE` distinct values
//!   that repeat apart (column D), `=SUM(COUNTIFS(D1:D10000,D1:D10000))` and
//!   `=SUM(SUMIFS(B1:B10000,D1:D10000,D1:D10000))` (T-0408: one walk per distinct value, where
//!   reusing only equal runs walked once per element);
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
//!   its 2^19-term cap, on its NaN path and converging with its rescales (the bound's slower term),
//!   and CHISQ.DIST at the largest chi-squared shape, over 512 elements.
//!
//! - HYPGEOM.DIST at its lift cap (T-1873): the cdf sum with 258,239 terms that are not known to
//!   be 0 (just under its 2^18-term cap, the population of 1.8E8 at its widest), and the cdf at
//!   its cap whose window is mostly exact zeros and subnormal terms (the slowest terms: `exp`'s
//!   subnormal path and additions with a subnormal operand), over 32 elements.
//!
//! - the financial functions' two paths (T-1823): PMT per row, unlifted, over 100,000 rows
//!   (`=PMT(A{r}/100,10,100)` in each C{r}; C1 is read), and RATE at its work cap, a
//!   non-converging `=SUM(RATE(A1:A100000,0,100,1))` (50 Newton steps an element).
//!
//! - OFFSET (T-1956, SPEC-0014): 100,000 scalar calls, one per row (`=SUM(OFFSET(A{r},0,0,1,2))`,
//!   the unlifted path that must stay allocation-free), and one OFFSET lifted over 100,000 rows into
//!   an array of references.
//!
//! - WORKDAY and NETWORKDAYS (T-1236): 20,000 scalar WORKDAY calls through MAP (the path a
//!   filled-down column takes, comparable with main), and WORKDAY and NETWORKDAYS lifted over a
//!   100,000-element array with 20 holidays, prepared once per call.
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
/// Column D's values, in its first `CYCLED_ROWS` rows: 0, 1, ..., `CYCLE` - 1, 0, 1, ...
const CYCLE: u32 = 100;
const CYCLED_ROWS: u32 = 10_000;
const RUNS: usize = 11;

/// A workbook with `rows` rows of runs in column A, the row number in column B, at most
/// `CYCLED_ROWS` rows of cycling values in column D, and `formula` in C1.
fn workbook(rows: u32, formula: &str) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=rows {
            tx.set(format!("A{r}").as_str(), f64::from((r - 1) / RUN_LENGTH))?;
            tx.set(format!("B{r}").as_str(), f64::from(r))?;
            if r <= CYCLED_ROWS {
                tx.set(format!("D{r}").as_str(), f64::from((r - 1) % CYCLE))?;
            }
        }
        if formula.contains("{r}") {
            // One scalar formula per row (the unlifted path), the template's `{r}` the row.
            for r in 1..=rows {
                let row = formula.replace("{r}", &r.to_string());
                tx.set(format!("C{r}").as_str(), Input::Formula(row))?;
            }
        } else {
            tx.set("C1", Input::Formula(formula.to_string()))?;
        }
        Ok(())
    })
}

/// Each case: its name, the rows of data, and the formula (`{r}`: one per row).
fn cases() -> Vec<(&'static str, u32, &'static str)> {
    vec![
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
            "COUNTIFS, occurrence count over repeated criteria",
            10_000,
            "=SUM(COUNTIFS(D1:D10000,D1:D10000))",
        ),
        (
            "SUMIFS, 10,000-element criteria range of repeated values",
            10_000,
            "=SUM(SUMIFS(B1:B10000,D1:D10000,D1:D10000))",
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
            "PMT, unlifted, one formula per row",
            100_000,
            "=PMT(A{r}/100,10,100)",
        ),
        (
            "OFFSET, unlifted, one formula per row",
            100_000,
            "=SUM(OFFSET(A{r},0,0,1,2))",
        ),
        (
            "OFFSET, lifted into an array of references",
            100_000,
            "=ROWS(OFFSET(A1,B1:B100000,0))",
        ),
        (
            "RATE, lifted, non-converging at the Newton cap",
            100_000,
            "=SUM(RATE(A1:A100000,0,100,1))",
        ),
        (
            "WORKDAY, scalar calls through MAP",
            20_000,
            "=SUM(MAP(B1:B20000,LAMBDA(d,WORKDAY(d+40000,5,B1:B20+45000))))",
        ),
        (
            "WORKDAY, lifted over an array",
            100_000,
            "=SUM(WORKDAY(B1:B100000+40000,5,B1:B20+45000))",
        ),
        (
            "NETWORKDAYS, lifted over an array",
            100_000,
            "=SUM(NETWORKDAYS(B1:B100000+40000,50000,B1:B20+45000))",
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
            "GAMMA.DIST, converging continued fraction at its term cap, at the element cap",
            1,
            "=ROWS(GAMMA.DIST(SEQUENCE(512)*0+1000000000000001,1E15,1,TRUE))",
        ),
        (
            "CHISQ.DIST, the largest shape at the element cap",
            1,
            "=SUM(CHISQ.DIST(SEQUENCE(512)*0+1E10,1E10,TRUE))",
        ),
        (
            "HYPGEOM.DIST, the cdf sum near its term cap, at the element cap",
            1,
            "=SUM(HYPGEOM.DIST(SEQUENCE(32)*0+45500000,90000000,90000000,180000000,TRUE))",
        ),
        (
            "HYPGEOM.DIST, the cdf sum at its term cap through subnormal terms, at the element cap",
            1,
            "=ROWS(HYPGEOM.DIST(SEQUENCE(32)*0+99994114707,200000000000,200000000000,400000000000,TRUE))",
        ),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let cases = cases();
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
