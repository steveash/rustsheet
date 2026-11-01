//! Decodes untrusted bytes as a bitcode workbook through `Model::from_bytes`, then evaluates it
//! and makes a few reads and edits (T-2895, SPEC-0026 decision 5).
//!
//! Decoded bitcode is untrusted (CLAUDE.md directive 5): `Model::from_workbook` refuses dangling
//! indices (T-2913) and anchors claiming more spill cells than are stored (T-3426), and what
//! loads must evaluate, edit and save without a panic. Random bytes rarely decode, so the seed
//! corpus is engine-written workbooks (`seeds/bitcode_seeds.rs`, run by `seed_corpus.py`).
#![no_main]

use libfuzzer_sys::fuzz_target;
use rustsheet_engine::Model;

fuzz_target!(|data: &[u8]| {
    let Ok(mut model) = Model::from_bytes(data, "en") else {
        return;
    };
    model.evaluate();
    let sheets = u32::try_from(model.workbook.worksheets.len()).unwrap_or(0);
    for sheet in 0..sheets.min(4) {
        for (row, column) in [(1, 1), (2, 2), (3, 1)] {
            let _ = model.get_formatted_cell_value(sheet, row, column);
        }
        let _ = model.set_user_input(sheet, 1, 1, "=SUM(A2:B3)+1".to_string());
        let _ = model.set_user_input(sheet, 4, 2, "=SEQUENCE(3)".to_string());
        let _ = model.insert_rows(sheet, 2, 1);
        let _ = model.delete_columns(sheet, 1, 1);
    }
    model.evaluate();
    let bytes = model.to_bytes();
    let _ = Model::from_bytes(&bytes, "en");
});
