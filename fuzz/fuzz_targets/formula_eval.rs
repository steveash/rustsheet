//! Evaluates an untrusted formula on a small model through the public facade.
//!
//! The model has numbers, text, a boolean, an error and a blank in range, a defined name and a
//! second sheet, so references, coercions and cross-sheet lookups are all reachable.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rustsheet::{Input, Value, Workbook, WorkbookOptions};

fn model() -> Option<Workbook> {
    let mut wb = Workbook::new(&WorkbookOptions::default()).ok()?;
    wb.edit(|tx| tx.add_sheet("Data")).ok()?;
    let inputs: [(&str, Input); 8] = [
        ("A1", Input::from(1.5)),
        ("A2", Input::from(-2.0)),
        ("A3", Input::Parse("text".into())),
        ("A4", Input::from(true)),
        ("A5", Input::Parse("#N/A".into())),
        ("B1", Input::Parse("=A1*2".into())),
        ("B2", Input::Parse("=SEQUENCE(2,2)".into())),
        ("Data!A1", Input::from(42.0)),
    ];
    for (cell, input) in inputs {
        wb.set(cell, input).ok()?;
    }
    wb.edit(|tx| tx.define_name("Rate", "Sheet1!$A$1")).ok()?;
    Some(wb)
}

fuzz_target!(|data: &[u8]| {
    let Ok(formula) = std::str::from_utf8(data) else {
        return;
    };
    let Some(mut wb) = model() else {
        return;
    };
    // An edit may reject the formula; that is fine, only panics are findings.
    if wb.set("C3", Input::Formula(format!("={formula}"))).is_err() {
        return;
    }
    let _: Option<Value> = wb.value("C3").ok();
    let _ = wb.formatted("C3");
    let _ = wb.cell("C3");
});
