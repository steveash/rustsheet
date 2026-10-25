//! Timings for exact-match lookups into one large range (T-0448): 5,000 each of
//! `VLOOKUP(k,$A$1:$B$20000,2,FALSE)`, `INDEX($B$1:$B$20000,MATCH(k,$A$1:$A$20000,0))` and
//! `XLOOKUP(k,$A$1:$A$20000,$B$1:$B$20000)` over 20,000 shuffled keys, once with number keys and
//! once with text keys.
//! `cargo bench -p rustsheet --bench lookup`
//!
//! One warm-up run, then `RUNS` measured recalculations; each case prints the fastest, the median
//! and the spread.

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, Workbook};

const ROWS: u32 = 20_000;
const LOOKUPS: u32 = 5_000;
const RUNS: usize = 3;

/// The keys 1..=ROWS in a fixed pseudo-random order (a full-period LCG walk, so no repeats).
fn shuffled_keys() -> Vec<u32> {
    // 7919 is coprime with ROWS, so `i * 7919 mod ROWS` visits every residue once.
    (0..ROWS).map(|i| (i * 7919) % ROWS + 1).collect()
}

/// Keys in A, values in B, lookup keys in C, and the three lookup formulas in D, E and F.
fn workbook(text_keys: bool) -> Result<Workbook, Box<dyn Error>> {
    let key = |k: u32| -> Input {
        if text_keys {
            Input::from(format!("Key{k}"))
        } else {
            Input::from(f64::from(k))
        }
    };
    common::filled_workbook(|tx| {
        for (i, k) in shuffled_keys().into_iter().enumerate() {
            let r = i + 1;
            tx.set(format!("A{r}").as_str(), key(k))?;
            tx.set(format!("B{r}").as_str(), f64::from(k) * 2.0)?;
        }
        for r in 1..=LOOKUPS {
            // Look up every fourth key, spread over the range.
            tx.set(format!("C{r}").as_str(), key(r * 4))?;
            let formulas = [
                format!("=VLOOKUP(C{r},$A$1:$B${ROWS},2,FALSE)"),
                format!("=INDEX($B$1:$B${ROWS},MATCH(C{r},$A$1:$A${ROWS},0))"),
                format!("=XLOOKUP(C{r},$A$1:$A${ROWS},$B$1:$B${ROWS})"),
            ];
            for (column, formula) in ["D", "E", "F"].iter().zip(formulas) {
                tx.set(format!("{column}{r}").as_str(), Input::Formula(formula))?;
            }
        }
        Ok(())
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    for (name, text_keys) in [("number keys", false), ("text keys", true)] {
        let mut wb = workbook(text_keys)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(
            std::io::stdout(),
            "{name}: {} lookups into {ROWS} rows, runs={RUNS} {t}",
            LOOKUPS * 3
        )?;
    }
    Ok(())
}
