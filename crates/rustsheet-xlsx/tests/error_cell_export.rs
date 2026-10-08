#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;
use std::io::Read;

/// The `<v>` of every `t="e"` cell in sheet 1, in file order.
fn stored_errors(path: &str) -> Vec<String> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml.split("<c ")
        .filter(|cell| cell.contains("t=\"e\""))
        .map(|cell| {
            let v = cell.split("<v>").nth(1).unwrap();
            v.split("</v>").next().unwrap().to_string()
        })
        .collect()
}

/// SPEC-0003 X1: `#GETTING_DATA` is saved as `#N/A` (Excel's stored form); the kinds Excel
/// rejects as stored literals (T-0999) are saved as `#VALUE!` plus `vm` rich-value metadata
/// (T-1014, see `rich_errors_round_trip`). Every other error keeps its literal.
#[test]
fn error_cells_are_saved_in_excels_stored_form() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    let inputs = [
        "=#GETTING_DATA",
        "=#CALC!",
        "=#SPILL!",
        "=#DIV/0!",
        "=#N/A",
        "=1/0+1",
    ];
    for (i, input) in inputs.iter().enumerate() {
        model
            .set_user_input(0, i as i32 + 1, 1, (*input).to_string())
            .unwrap();
    }
    model.evaluate();
    let path = "temp_error_cell_export.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let stored = stored_errors(path);
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        stored,
        ["#N/A", "#VALUE!", "#VALUE!", "#DIV/0!", "#N/A", "#DIV/0!"]
    );
}

/// SPEC-0003 X2 (T-1014): every rich kind, as a formula result and as a constant, is saved in
/// Excel's form (`#VALUE!` plus `vm`) and comes back as itself, so ERROR.TYPE of it does too.
#[test]
fn rich_errors_round_trip() {
    // ERROR.TYPE.xlsx (Excel-authored) holds `#CALC!` (A8) and `#SPILL!` (A14) as constants.
    let mut model = load_from_xlsx(
        "tests/calc_tests/INFORMATION/ERROR.TYPE.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    let formulas = [
        ("=#CALC!", "#CALC!"),
        ("=#SPILL!", "#SPILL!"),
        (r#"=IMAGE("https://example.com/a.png")"#, "#BLOCKED!"),
        (r#"=STOCKHISTORY("MSFT",45000)"#, "#CONNECT!"),
        (r#"=FIELDVALUE(1,"Price")"#, "#FIELD!"),
    ];
    for (i, (formula, _)) in formulas.iter().enumerate() {
        let row = i as i32 + 1;
        model
            .set_user_input(0, row, 20, (*formula).to_string())
            .unwrap();
        model
            .set_user_input(0, row, 21, format!("=ERROR.TYPE(T{row})"))
            .unwrap();
    }
    model.evaluate();
    let path = "temp_rich_errors_round_trip.xlsx";
    save_to_xlsx(&model, path).unwrap();
    // The cached values, not a recalculation, are what is checked.
    let reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    let text = |row, column| reloaded.get_formatted_cell_value(0, row, column).unwrap();
    assert_eq!(text(8, 1), "#CALC!");
    assert_eq!(text(14, 1), "#SPILL!");
    for (i, (_, expected)) in formulas.iter().enumerate() {
        let row = i as i32 + 1;
        assert_eq!(text(row, 20), *expected, "T{row}");
        assert_eq!(
            text(row, 21),
            model.get_formatted_cell_value(0, row, 21).unwrap(),
            "U{row}"
        );
    }
}
