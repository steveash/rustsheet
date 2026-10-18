//! Timings for SPEC-0010's text hot paths, recorded before `Text` replaces `String` in the
//! engine's values (T-1576, T-1580 report them before/after, budget ±3%): recalculation of
//! `=A1&"x"`, `=MID(A1,2,3)` and `=LEFT(A1,2)` over a text column of `ROWS` cells, of
//! `CONCATENATE_CELLS` many-argument `CONCATENATE`s just over the 32,767-unit limit, and
//! `Text::push` of a `CONCATENATE`-like argument list that starts with a lone surrogate.
//! `cargo bench -p rustsheet --bench text`
//!
//! One warm-up run, then `RUNS` measured runs; each case prints the fastest, the median and the
//! spread.

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};
use rustsheet_engine::Text;

const ROWS: u32 = 100_000;
const RUNS: usize = 11;
/// `CONCATENATE` takes at most 255 arguments; 255 cells of `ARG_LEN` units join to 33,150
/// units, so every result is truncated to 32,767.
const ARGS: u32 = 255;
const ARG_LEN: usize = 130;
const CONCATENATE_CELLS: u32 = 1_000;

/// Builds the formula of row `r`.
type RowFormula = fn(u32) -> String;

/// A workbook with text in A1:A`ROWS` and `formula(r)` in column B of each row.
fn column_workbook(formula: RowFormula) -> Result<Workbook, Box<dyn Error>> {
    common::filled_workbook(|tx| {
        for r in 1..=ROWS {
            tx.set(format!("A{r}").as_str(), format!("row {r} café text"))?;
            tx.set(format!("B{r}").as_str(), Input::Formula(formula(r)))?;
        }
        Ok(())
    })
}

/// A workbook with `ARG_LEN`-unit text in A1:A`ARGS` and, in column B, `CONCATENATE_CELLS`
/// copies of `CONCATENATE(A1,…,A255)`.
fn concatenate_workbook() -> Result<Workbook, Box<dyn Error>> {
    let args: Vec<String> = (1..=ARGS).map(|r| format!("A{r}")).collect();
    let formula = format!("=CONCATENATE({})", args.join(","));
    common::filled_workbook(|tx| {
        for r in 1..=ARGS {
            tx.set(format!("A{r}").as_str(), "x".repeat(ARG_LEN))?;
        }
        for r in 1..=CONCATENATE_CELLS {
            tx.set(format!("B{r}").as_str(), Input::Formula(formula.clone()))?;
        }
        Ok(())
    })
}

/// `CONCATENATE_CELLS` argument lists built with `Text::push`, each a lone low surrogate followed
/// by `ARGS` arguments of `ARG_LEN` units, truncated to the cell limit.
fn push_lone_first() -> usize {
    let lone = Text::from("😀").slice_utf16(1, 2);
    let arg = Text::from("x".repeat(ARG_LEN));
    let mut total = 0;
    for _ in 0..CONCATENATE_CELLS {
        let mut joined = lone.clone();
        for _ in 0..ARGS {
            joined.push(&arg);
        }
        joined.truncate_utf16(32767);
        total += joined.len_utf16();
    }
    total
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut out = std::io::stdout();
    let cases: [(&str, RowFormula); 3] = [
        ("A1&\"x\"", |r| format!("=A{r}&\"x\"")),
        ("MID(A1,2,3)", |r| format!("=MID(A{r},2,3)")),
        ("LEFT(A1,2)", |r| format!("=LEFT(A{r},2)")),
    ];
    for (name, formula) in cases {
        let mut wb = column_workbook(formula)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(out, "{name}: rows={ROWS} runs={RUNS} {t}")?;
    }
    let mut wb = concatenate_workbook()?;
    let t = common::timings(RUNS, || wb.recalculate());
    writeln!(
        out,
        "CONCATENATE({ARGS} x {ARG_LEN} units): cells={CONCATENATE_CELLS} runs={RUNS} {t}"
    )?;
    let mut units = 0;
    let t = common::timings(RUNS, || units = push_lone_first());
    writeln!(
        out,
        "Text::push, lone first ({ARGS} x {ARG_LEN} units): lists={CONCATENATE_CELLS} runs={RUNS} {t} units={units}"
    )?;
    Ok(())
}
