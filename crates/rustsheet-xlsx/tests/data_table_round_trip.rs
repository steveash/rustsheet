#![allow(clippy::unwrap_used, reason = "tests")]

//! What-if data tables survive export and import (docs/excel-semantics/recalc.md, T-0441).

use rustsheet_engine::types::{DataTable, DataTableInput};
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;
use std::io::Read;

fn sheet_xml(path: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// A two-variable table is saved as Excel saves one, `<f t="dataTable" … dt2D="1" dtr="1" r1=row
/// r2=column/>` in its first result cell, and computes the same results once loaded again.
#[test]
fn a_two_variable_table_round_trips() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    for (row, column, input) in [
        (1, 1, "1"),
        (1, 2, "2"),
        (10, 1, "=A1*10+B1"),
        (10, 2, "100"),
        (10, 3, "200"),
        (11, 1, "1"),
        (12, 1, "2"),
        (1, 5, "=SUM(B11:C12)"),
    ] {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    let table = DataTable {
        range: (11, 2, 12, 3),
        row_input: Some(DataTableInput::Cell(1, 2)),
        column_input: Some(DataTableInput::Cell(1, 1)),
    };
    model.add_data_table(0, table.clone()).unwrap();
    model.evaluate();
    let path = "temp_data_table_round_trip.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let xml = sheet_xml(path);
    let mut reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        xml.contains(
            r#"<f t="dataTable" ref="B11:C12" dt2D="1" dtr="1" r1="B1" r2="A1"/><v>110</v>"#
        ),
        "{xml}"
    );
    assert_eq!(
        xml.matches("dataTable").count(),
        1,
        "only the first result cell has a formula"
    );
    assert_eq!(reloaded.workbook.worksheets[0].data_tables, [table]);
    reloaded.set_user_input(0, 1, 1, "5".to_string()).unwrap();
    reloaded.evaluate();
    let text = |row, column| reloaded.get_formatted_cell_value(0, row, column).unwrap();
    assert_eq!([text(11, 2), text(12, 3)], ["110", "220"]);
    assert_eq!(text(1, 5), "660");
}

/// Workbooks are untrusted: a `dataTable` `ref` as large as the sheet is refused on import, not
/// evaluated cell by cell (directive 5; `DataTable::check`, recalc.md DEVIATION).
#[test]
fn an_oversized_imported_table_is_refused() {
    use std::io::Write;
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
    model.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "5".to_string()).unwrap();
    let table = DataTable {
        range: (2, 2, 2, 2),
        row_input: None,
        column_input: Some(DataTableInput::Cell(1, 1)),
    };
    model.add_data_table(0, table).unwrap();
    model.evaluate();
    let path = "temp_data_table_oversized.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let hostile = "temp_data_table_oversized_hostile.xlsx";
    let mut writer = zip::ZipWriter::new(std::fs::File::create(hostile).unwrap());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().unwrap().to_string();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name == "xl/worksheets/sheet1.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert!(xml.contains(r#"ref="B2:B2""#), "{xml}");
            bytes = xml
                .replace(r#"ref="B2:B2""#, r#"ref="B2:XFD1048576""#)
                .into_bytes();
        }
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();
    let loaded = load_from_xlsx(hostile, "en", "UTC", "en");
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(hostile).unwrap();
    let error = loaded.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(error.contains("data table too large"), "{error}");
}
