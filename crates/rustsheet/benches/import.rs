//! Timing of an xlsx load whose cost is its formulas (T-0499): 20,000 long COUNTIFS formulas
//! over a quoted sheet name, the shape of corpus-bench/52450, where the importer's A1-to-R1C1
//! pass (lex, parse, stringify) dominated the load.
//! `cargo bench -p rustsheet --bench import`

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, LoadOptions, SaveOptions, Workbook};
use rustsheet_engine::expressions::utils::number_to_column;

const ROWS: u32 = 1_000;
const COLUMNS: u32 = 20;
const RUNS: usize = 5;

/// A formula of corpus-bench/52450's shape (Sheet1!AA100 there), relative along rows and columns.
fn formula(row: u32, column: &str) -> String {
    let range = "'XA Data Pull'!$J:$J";
    let when = format!("\">=\"&{column}$1,{range},\"<=\"&{column}$1+TIME(23,59,59)");
    format!(
        "=IF($E{row}<>\"\",IF(COUNTIFS({range},{when},'XA Data Pull'!$K:$K,$B{row})=0,\"\",\
         COUNTIFS({range},{when},'XA Data Pull'!$K:$K,$B{row},'XA Data Pull'!$L:$L,\"<>\")),\"\")"
    )
}

/// The xlsx bytes of a workbook holding `ROWS * COLUMNS` such formulas in F2 onwards.
fn workbook_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    // From F (column 6) on.
    let columns = (6..6 + COLUMNS)
        .map(|c| number_to_column(i32::try_from(c).ok()?))
        .collect::<Option<Vec<_>>>()
        .ok_or("no such column")?;
    let wb = common::filled_workbook(|tx| {
        tx.add_sheet("XA Data Pull")?;
        for r in 2..=ROWS + 1 {
            for column in &columns {
                tx.set(
                    format!("{column}{r}").as_str(),
                    Input::Formula(formula(r, column)),
                )?;
            }
        }
        Ok(())
    })?;
    Ok(wb.to_xlsx_bytes(SaveOptions::default())?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let bytes = workbook_bytes()?;
    let opts = LoadOptions {
        recalc: false,
        ..LoadOptions::default()
    };
    let mut failed = None;
    let timings = common::timings(RUNS, || {
        if let Err(e) = Workbook::from_xlsx_bytes(&bytes, &opts) {
            failed = Some(e);
        }
    });
    if let Some(e) = failed {
        return Err(e.into());
    }
    writeln!(
        std::io::stdout(),
        "load {} formulas ({} bytes), no recalc: {timings}",
        ROWS * COLUMNS,
        bytes.len()
    )?;
    Ok(())
}
