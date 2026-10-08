#![allow(clippy::unwrap_used)]

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
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
/// rejects as stored literals (T-0999) are saved as `#VALUE!` until the exporter writes `vm`
/// rich-value metadata (T-1014). Every other error keeps its literal.
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
