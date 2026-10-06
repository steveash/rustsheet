//! Parses an untrusted formula and prints it back in every notation the engine writes.
//!
//! Any input must yield a node (possibly a parse-error node), never a panic, stack overflow
//! or unbounded allocation (CLAUDE.md directive 5).
#![no_main]

use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;
use rustsheet_engine::expressions::parser::new_parser_english;
use rustsheet_engine::expressions::parser::stringify::{
    to_english_string, to_excel_string, to_rc_format,
};
use rustsheet_engine::expressions::types::CellReferenceRC;

fuzz_target!(|data: &[u8]| {
    let Ok(formula) = std::str::from_utf8(data) else {
        return;
    };
    let sheets = vec!["Sheet1".to_string(), "Other Sheet".to_string()];
    let names = vec![
        ("Rate".to_string(), None, "Sheet1!$A$1".to_string()),
        ("Local".to_string(), Some(1), "'Other Sheet'!$B$2:$C$3".to_string()),
    ];
    let mut parser = new_parser_english(sheets, names, HashMap::new());
    let context = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        column: 3,
        row: 5,
    };
    let node = parser.parse(formula, &context);
    let english = to_english_string(&node, &context);
    let _ = to_excel_string(&node, &context);
    let _ = to_rc_format(&node);
    // Re-parsing printed output exercises the printer's quoting and parenthesisation.
    let _ = parser.parse(&english, &context);
});
