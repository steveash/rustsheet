//! Imports an untrusted xlsx package through the public facade, then recalculates it.
//!
//! The limits are tighter than the defaults so a zip bomb is rejected quickly instead of
//! tripping libFuzzer's memory limit: the importer must refuse it, not inflate it.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rustsheet::{LoadOptions, Workbook};

fuzz_target!(|data: &[u8]| {
    let opts = LoadOptions {
        recalc: true,
        max_uncompressed_bytes: 16 << 20,
        max_zip_entries: 500,
        ..LoadOptions::default()
    };
    let Ok(wb) = Workbook::from_xlsx_bytes(data, &opts) else {
        return;
    };
    for sheet in wb.sheets() {
        if let Ok(Some(range)) = wb.used_range(sheet.id) {
            let _ = wb.values(range);
        }
    }
    let _ = wb.to_xlsx_bytes(rustsheet::SaveOptions::default());
});
