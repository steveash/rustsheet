//! Timings for GETPIVOTDATA's per-pivot index (SPEC-0015 "Untrusted-input limits", T-2083). A
//! 100,000-record source (Region: 100 items, Product: 10, Sales: the row number) feeds one Sum
//! pivot with Region rows and Product columns:
//! - 1,000 GETPIVOTDATA cells, each a different Region/Product intersection (1,000 queries, one
//!   index build);
//! - 1,000 GETPIVOTDATA cells asking the same grand total (one query, memoized);
//! - one call lifted over 100 Region items, `=SUM(GETPIVOTDATA("Sales",$H$1,"Region",F1:F100))`.
//!
//! Each recalculation rebuilds the index (it is dropped at each evaluation), so the timings
//! include its one pass over the records.
//!
//! `cargo bench -p rustsheet --bench pivot [-- FILTER...]`

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, PivotFunction, PivotTableInput, Workbook};

const RECORDS: u32 = 100_000;
const REGIONS: u32 = 100;
const PRODUCTS: u32 = 10;
const RUNS: usize = 11;

/// The source in A1:C100001, the pivot registered at H1:S103 and the region names in F1:F100,
/// then `formulas` in column E.
fn workbook(formulas: &[String]) -> Result<Workbook, Box<dyn Error>> {
    let mut wb = common::filled_workbook(|tx| {
        tx.set("A1", "Region")?;
        tx.set("B1", "Product")?;
        tx.set("C1", "Sales")?;
        for r in 0..RECORDS {
            let row = r + 2;
            tx.set(
                format!("A{row}").as_str(),
                format!("R{}", r % REGIONS).as_str(),
            )?;
            tx.set(
                format!("B{row}").as_str(),
                format!("P{}", (r / REGIONS) % PRODUCTS).as_str(),
            )?;
            tx.set(format!("C{row}").as_str(), f64::from(r))?;
        }
        for r in 0..REGIONS {
            tx.set(format!("F{}", r + 1).as_str(), format!("R{r}").as_str())?;
        }
        Ok(())
    })?;
    let input = PivotTableInput {
        name: "Bench".into(),
        source: wb.range_ref(&format!("A1:C{}", RECORDS + 1))?,
        location: wb.range_ref("H1:S103")?,
        row: "Region".into(),
        column: Some("Product".into()),
        value: "Sales".into(),
        function: PivotFunction::Sum,
        caption: "Sum of Sales".into(),
    };
    wb.edit(|tx| {
        tx.add_pivot_table(&input)?;
        for (i, formula) in formulas.iter().enumerate() {
            tx.set(
                format!("E{}", i + 1).as_str(),
                Input::Formula(formula.clone()),
            )?;
        }
        Ok(())
    })?;
    Ok(wb)
}

fn cases() -> Vec<(&'static str, Vec<String>)> {
    let intersections = (0..1_000)
        .map(|i| {
            format!(
                r#"=GETPIVOTDATA("Sales",$H$1,"Region","R{}","Product","P{}")"#,
                i % REGIONS,
                (i / REGIONS) % PRODUCTS
            )
        })
        .collect();
    let totals = vec![r#"=GETPIVOTDATA("Sales",$H$1)"#.to_string(); 1_000];
    let lifted = vec![r#"=SUM(GETPIVOTDATA("Sales",$H$1,"Region",F1:F100))"#.to_string()];
    vec![
        ("1,000 distinct intersections", intersections),
        ("1,000 equal grand totals", totals),
        ("one call lifted over 100 items", lifted),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    let mut out = std::io::stdout().lock();
    for (name, formulas) in cases() {
        if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
            continue;
        }
        let mut wb = workbook(&formulas)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(out, "{name}: {t}")?;
    }
    Ok(())
}
