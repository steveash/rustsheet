#![allow(clippy::unwrap_used, reason = "tests")]

//! T-1552: a formula typed in RustSheet whose legacy reading would intersect a range is saved as
//! a one-cell dynamic array, so Excel (and our import) do not intersect it on load.

use std::io::{Cursor, Read};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// B1:B6 = 1,2,3,1,2,3 and the given formulas in column D from row 10, off B1:B6's rows, where a
/// legacy reading of the range is `#VALUE!`.
fn model_with(formulas: &[&str]) -> Model {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    for (row, v) in [1, 2, 3, 1, 2, 3].iter().enumerate() {
        model
            .set_user_input(0, row as i32 + 1, 2, v.to_string())
            .unwrap();
    }
    for (i, formula) in formulas.iter().enumerate() {
        model
            .set_user_input(0, i as i32 + 10, 4, (*formula).to_string())
            .unwrap();
    }
    model.evaluate();
    model
}

fn save(model: &Model) -> Vec<u8> {
    save_xlsx_to_writer(model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
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

/// The `<c>` element of `cell` in the sheet XML.
fn cell_xml<'a>(xml: &'a str, cell: &str) -> &'a str {
    let start = xml.find(&format!("<c r=\"{cell}\"")).unwrap();
    let end = start + xml[start..].find("</c>").unwrap();
    &xml[start..end]
}

fn reloaded_values(bytes: &[u8], rows: usize) -> Vec<String> {
    let workbook = load_from_xlsx_bytes(bytes, "m", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    (0..rows)
        .map(|i| model.get_formatted_cell_value(0, i as i32 + 10, 4).unwrap())
        .collect()
}

/// Excel saves a dynamic formula whose result is one cell as `<c cm="1"><f t="array" ref="F3">`
/// (Excel-authored `calc_tests/LOOKUP_AND_REFERENCE/ROW_COLUM.xlsx` sheet6 F3 `SUM(ROW(C2:C4))`;
/// docs/excel-semantics/xlsx-storage.md). A plain `<f>` is read as legacy and intersected.
#[test]
fn a_formula_a_legacy_reading_would_intersect_round_trips_as_dynamic() {
    let model = model_with(&["=SUM(B1:B6+0)", "=SUMPRODUCT(IF(B1:B6>1,B1:B6,0))"]);
    let bytes = save(&model);
    let xml = sheet1_xml(&bytes);
    for (cell, formula) in [
        ("D10", "SUM(B1:B6+0)"),
        ("D11", "SUMPRODUCT(IF(B1:B6&gt;1,B1:B6,0))"),
    ] {
        let c = cell_xml(&xml, cell);
        assert!(c.contains(" cm=\"1\""), "{c}");
        assert!(
            c.contains(&format!("<f t=\"array\" ref=\"{cell}\">{formula}</f>")),
            "{c}"
        );
    }
    assert_eq!(reloaded_values(&bytes, 2), ["12", "10"]);
}

/// A formula with nothing for a legacy reading to intersect keeps the plain `<f>`, as does an
/// explicit `@` the legacy reading would add anyway.
#[test]
fn a_formula_with_nothing_to_intersect_stays_a_plain_formula() {
    let model = model_with(&["=SUM(B1:B6)", "=B1+1", "=ABS(@B1:B6)"]);
    let bytes = save(&model);
    let xml = sheet1_xml(&bytes);
    for (cell, formula) in [
        ("D10", "SUM(B1:B6)"),
        ("D11", "B1+1"),
        ("D12", "ABS(B1:B6)"),
    ] {
        let c = cell_xml(&xml, cell);
        assert!(!c.contains("cm="), "{c}");
        assert!(c.contains(&format!("<f>{formula}</f>")), "{c}");
    }
    assert_eq!(reloaded_values(&bytes, 3), ["12", "2", "#VALUE!"]);
}
