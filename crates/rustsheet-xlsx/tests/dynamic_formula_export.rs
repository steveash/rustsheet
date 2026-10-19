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

/// A model with `formula` in D10, as a legacy formula when `legacy`.
fn model_with_one(formula: &str, legacy: bool) -> Model {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    if legacy {
        model
            .set_cell_with_legacy_formula(0, 10, 4, formula)
            .unwrap();
    } else {
        model.set_user_input(0, 10, 4, formula.to_string()).unwrap();
    }
    model.evaluate();
    model
}

/// The stored (R1C1) formulas of the first sheet after an xlsx round trip, and D10:D11.
fn reloaded(bytes: &[u8]) -> (Vec<String>, Vec<String>) {
    let workbook = load_from_xlsx_bytes(bytes, "m", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    let stored = model.workbook.worksheet(0).unwrap().shared_formulas.clone();
    let values = (10..12)
        .map(|row| model.get_formatted_cell_value(0, row, 4).unwrap())
        .collect();
    (stored, values)
}

/// SPEC-0012 rule 5 (T-1713): a legacy N or T over an array-valued call is saved as a plain
/// `<f>` that never shows its marker, and the import reads it back marked and `#VALUE!`; its
/// dynamic twin is saved with `cm="1"` and reads back unmarked, spilling or giving its first
/// element (probes/legacy-n-t-reference-args, legacy-array-result A, G).
#[test]
fn a_legacy_n_or_t_over_an_array_round_trips_as_a_plain_formula() {
    for (formula, xml_text, dynamic) in [
        ("=N(IF(1,{2;3}))", "N(IF(1,{2;3}))", ["2", "3"]),
        (
            "=T(IF(1,{\"a\";\"b\"}))",
            "T(IF(1,{&quot;a&quot;;&quot;b&quot;}))",
            ["a", "b"],
        ),
        ("=N(CHOOSE(1,{2;3}))", "N(CHOOSE(1,{2;3}))", ["2", "3"]),
        ("=N(INDEX({2;3},0,1))", "N(INDEX({2;3},0,1))", ["2", "3"]),
        (
            "=N(_xlfn.IFS(1,{2;3}))",
            "N(_xlfn.IFS(1,{2;3}))",
            ["2", "3"],
        ),
    ] {
        let bytes = save(&model_with_one(formula, true));
        let c = cell_xml(&sheet1_xml(&bytes), "D10").to_string();
        assert!(!c.contains("cm="), "{c}");
        assert!(c.contains(&format!("<f>{xml_text}</f>")), "{c}");
        let (stored, values) = reloaded(&bytes);
        assert!(
            stored[0].starts_with("NLG1(") || stored[0].starts_with("TLG1("),
            "{formula}: {stored:?}"
        );
        assert_eq!(values, ["#VALUE!", ""], "{formula}");

        let bytes = save(&model_with_one(formula, false));
        let c = cell_xml(&sheet1_xml(&bytes), "D10").to_string();
        assert!(c.contains(" cm=\"1\""), "{c}");
        let (stored, values) = reloaded(&bytes);
        assert!(
            !stored[0].contains("NLG1(") && !stored[0].contains("TLG1("),
            "{formula}: {stored:?}"
        );
        assert_eq!(values, dynamic, "{formula}");
    }
}

/// A statically scalar N over IF gets no marker, legacy or dynamic, so the dynamic one stays a
/// plain `<f>` (SPEC-0012 rule 2; T-1552).
#[test]
fn a_statically_scalar_n_over_if_stays_a_plain_formula() {
    for legacy in [true, false] {
        let bytes = save(&model_with_one("=N(IF(A1,1,2))", legacy));
        let c = cell_xml(&sheet1_xml(&bytes), "D10").to_string();
        assert!(!c.contains("cm="), "{c}");
        assert!(c.contains("<f>N(IF(A1,1,2))</f>"), "{c}");
        let (stored, values) = reloaded(&bytes);
        assert!(
            !stored[0].contains("NLG1(") && !stored[0].contains("TLG1("),
            "{stored:?}"
        );
        assert_eq!(values, ["2", ""]);
    }
}

/// A legacy and a dynamic `=N(IF(1,{2;3}))` on one sheet are two shared formulas, so each keeps
/// its reading through a save and a reload.
#[test]
fn a_legacy_and_a_dynamic_twin_are_two_shared_formulas() {
    let mut model = model_with_one("=N(IF(1,{2;3}))", true);
    model
        .set_user_input(0, 10, 5, "=N(IF(1,{2;3}))".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(
        model.workbook.worksheet(0).unwrap().shared_formulas.len(),
        2
    );
    let workbook = load_from_xlsx_bytes(&save(&model), "m", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    let values: Vec<String> = [(10, 4), (11, 4), (10, 5), (11, 5)]
        .iter()
        .map(|(row, column)| model.get_formatted_cell_value(0, *row, *column).unwrap())
        .collect();
    assert_eq!(values, ["#VALUE!", "", "2", "3"]);
}
