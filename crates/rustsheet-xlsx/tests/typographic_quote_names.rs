#![allow(clippy::unwrap_used, reason = "tests")]

//! A formula whose typographic quotes Excel reads as name characters (T-0481,
//! docs/excel-semantics/references.md "Undefined names may contain typographic quotes") keeps
//! them on export and on reload: the formula text is not rewritten into a string literal.

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;

#[test]
fn typographic_quote_names_round_trip() {
    let formula = "=IF(B1=\"domestic\",“D”,IF(B1=\"international\",“I”,“N/A”))";
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model
        .set_user_input(0, 1, 2, "domestic".to_string())
        .unwrap();
    model.set_user_input(0, 1, 1, formula.to_string()).unwrap();
    model.evaluate();
    let path = "temp_typographic_quote_names.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let mut reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    reloaded.evaluate();
    assert_eq!(
        reloaded.get_cell_formula(0, 1, 1).unwrap().as_deref(),
        Some(formula)
    );
    assert_eq!(
        reloaded.get_formatted_cell_value(0, 1, 1).unwrap(),
        "#NAME?"
    );
}
