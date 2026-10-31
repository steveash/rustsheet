//! Writes the seed corpus of the `bitcode_workbook` fuzz target: `Model::to_bytes()` of a few
//! small engine-written workbooks, since random bytes rarely decode as a workbook (T-2895).
//!
//! Usage: `cargo run --bin bitcode_seeds -- <directory>` (`seed_corpus.py` runs it).

use std::path::PathBuf;

use rustsheet_engine::Model;

/// Each seed: the edits that build it, as (sheet, row, column, input).
const SEEDS: &[&[(u32, i32, i32, &str)]] = &[
    // A number and a formula.
    &[(0, 1, 1, "1"), (0, 2, 1, "=A1+1")],
    // Text, a boolean, an error and a percentage, so shared strings and number formats.
    &[
        (0, 1, 1, "hello"),
        (0, 1, 2, "TRUE"),
        (0, 1, 3, "#N/A"),
        (0, 2, 1, "50%"),
    ],
    // The same formula down a column: one shared formula.
    &[
        (0, 1, 1, "1"),
        (0, 2, 1, "=A1*2"),
        (0, 3, 1, "=A2*2"),
        (0, 4, 1, "=A3*2"),
    ],
    // A dynamic array and a reference to its spill.
    &[(0, 1, 1, "=SEQUENCE(3,2)"), (0, 1, 4, "=SUM(A1#)")],
    // Two sheets referring to each other.
    &[(0, 1, 1, "=Sheet2!A1+1"), (1, 1, 1, "7")],
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or("usage: bitcode_seeds <directory>")?
        .into();
    std::fs::create_dir_all(&directory)?;
    for (index, edits) in SEEDS.iter().enumerate() {
        let mut model = Model::new_empty("seed", "en", "UTC", "en")?;
        if edits.iter().any(|(sheet, ..)| *sheet == 1) {
            model.new_sheet();
        }
        for (sheet, row, column, input) in edits.iter() {
            model.set_user_input(*sheet, *row, *column, (*input).to_string())?;
        }
        model.evaluate();
        std::fs::write(
            directory.join(format!("seed-{index}.bin")),
            model.to_bytes(),
        )?;
    }
    Ok(())
}
