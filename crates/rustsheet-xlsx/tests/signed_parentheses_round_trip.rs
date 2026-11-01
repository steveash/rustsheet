#![allow(clippy::unwrap_used, reason = "tests")]

//! T-3338: `-(` nested to Excel's 256-parenthesis cap (probes/t3266-nested-paren-256), and 200
//! levels of it around Excel's 824 more minus signs (probes/t3266-nested-200-824), survive an
//! xlsx round trip on a small stack: export prints them, import parses them back, and both
//! evaluate (directive 5). The stack is what the parser's depth tests give: 1 MiB optimized,
//! 16 MiB unoptimized (`on_small_stack` in the engine's `test_nesting_depth.rs`).

use std::io::Cursor;

use rustsheet_xlsx::base::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

fn nested(open: &str, inner: &str, levels: usize) -> String {
    format!("={}{inner}{}", open.repeat(levels), ")".repeat(levels))
}

#[test]
fn signed_parentheses_at_the_cap_round_trip_on_a_small_stack() {
    let stack = if cfg!(debug_assertions) {
        16 << 20
    } else {
        1 << 20
    };
    let handle = std::thread::Builder::new().stack_size(stack).spawn(|| {
        let formulas = [
            nested("-(", "B1", 256),
            nested("-(", "B1", 255),
            nested("-(", &format!("{}1", "-".repeat(824)), 200),
        ];
        let mut model = Model::new_empty("signed", "en", "UTC", "en").unwrap();
        model.set_user_input(0, 1, 2, "2".to_string()).unwrap();
        for (row, formula) in (1..).zip(&formulas) {
            model.set_user_input(0, row, 1, formula.clone()).unwrap();
        }
        model.evaluate();
        let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
            .unwrap()
            .into_inner();
        let workbook = load_from_xlsx_bytes(&bytes, "signed", "en", "UTC").unwrap();
        let mut reloaded = Model::from_workbook(workbook, "en").unwrap();
        reloaded.evaluate();
        let read = |model: &Model| -> Vec<(Option<String>, String)> {
            (1..=3)
                .map(|row| {
                    (
                        model.get_cell_formula(0, row, 1).unwrap(),
                        model.get_formatted_cell_value(0, row, 1).unwrap(),
                    )
                })
                .collect()
        };
        (formulas, read(&model), read(&reloaded))
    });
    let (formulas, before, after) = handle.unwrap().join().unwrap();
    let expected: Vec<_> = formulas
        .into_iter()
        .zip(["2", "-2", "1"])
        .map(|(formula, value)| (Some(formula), value.to_string()))
        .collect();
    assert_eq!(before, expected);
    assert_eq!(after, expected);
}
