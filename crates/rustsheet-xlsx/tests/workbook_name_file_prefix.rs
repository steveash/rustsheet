#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;
use std::io::{Cursor, Read};

/// A workbook named `53aa` whose `Calc!A1` reads the workbook name `loc` through the sheet Feb.
fn model() -> Model {
    let mut model = Model::new_empty("53aa", "en", "UTC", "en").unwrap();
    model.new_sheet();
    model.rename_sheet("Sheet1", "Calc").unwrap();
    model.rename_sheet("Sheet2", "Feb").unwrap();
    model.define_name("loc", None, "=Feb!$A$1").unwrap();
    model.set_user_input(1, 1, 1, "5".to_string()).unwrap();
    model
        .set_user_input(0, 1, 1, "=SUM(Feb!loc)+1".to_string())
        .unwrap();
    model.evaluate();
    model
}

fn sheet1_xml(bytes: &[u8]) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// The file-prefixed name (`='53aa.xlsx'!loc`) is saved as `[0]!loc`, the own-workbook prefix
/// Excel loads as the workbook name (probes/workbook-name-own-book-prefix, excel-graph, T-2834),
/// and the reloaded workbook shows its own file name and keeps the value.
#[test]
fn a_file_prefixed_workbook_name_round_trips() {
    let model = model();
    assert_eq!(
        model.get_cell_formula(0, 1, 1).unwrap().unwrap(),
        "=SUM('53aa.xlsx'!loc)+1"
    );
    let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    assert!(sheet1_xml(&bytes).contains("<f>SUM([0]!loc)+1</f>"));
    let workbook = load_from_xlsx_bytes(&bytes, "Budget", "en", "UTC").unwrap();
    let mut reloaded = Model::from_workbook(workbook, "en").unwrap();
    reloaded.evaluate();
    assert_eq!(
        reloaded.get_cell_formula(0, 1, 1).unwrap().unwrap(),
        "=SUM(Budget.xlsx!loc)+1"
    );
    assert_eq!(reloaded.get_formatted_cell_value(0, 1, 1).unwrap(), "6");
}
