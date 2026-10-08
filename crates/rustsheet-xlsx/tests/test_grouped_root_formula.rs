#![allow(clippy::unwrap_used)]

//! A cell formula written wholly in parentheses keeps them through the model and an xlsx round
//! trip (T-1017): Excel tells `=(A1-1)` apart from `=A1-1` (docs/excel-semantics/numbers.md
//! "Near-zero snapping in addition and subtraction").

use std::io::Cursor;

use rustsheet_xlsx::base::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

#[test]
fn outer_parentheses_survive_the_model_and_an_xlsx_round_trip() {
    let mut model = Model::new_empty("grouped", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "3".to_string()).unwrap();
    model
        .set_user_input(0, 1, 2, "=(A1-1)".to_string())
        .unwrap();
    model.set_user_input(0, 1, 3, "=A1-1".to_string()).unwrap();
    model
        .set_user_input(0, 1, 4, "=(SUM(A1,1))".to_string())
        .unwrap();
    model.evaluate();
    let formulas = |model: &Model| -> Vec<Option<String>> {
        (2..=4)
            .map(|column| model.get_cell_formula(0, 1, column).unwrap())
            .collect()
    };
    let expected = vec![
        Some("=(A1-1)".to_string()),
        Some("=A1-1".to_string()),
        Some("=(SUM(A1,1))".to_string()),
    ];
    assert_eq!(formulas(&model), expected);
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 1, 4).unwrap(), "4");

    let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let workbook = load_from_xlsx_bytes(&bytes, "grouped", "en", "UTC").unwrap();
    let mut reloaded = Model::from_workbook(workbook, "en").unwrap();
    assert_eq!(formulas(&reloaded), expected);
    reloaded.evaluate();
    assert_eq!(reloaded.get_formatted_cell_value(0, 1, 2).unwrap(), "2");
}
