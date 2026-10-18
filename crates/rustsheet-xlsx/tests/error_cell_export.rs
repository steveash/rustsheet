#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::expressions::token::Error;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;
use std::io::Read;

/// The `<v>` of every `t="e"` cell in sheet 1, in file order.
fn stored_errors(path: &str) -> Vec<String> {
    let xml = part(path, "xl/worksheets/sheet1.xml").unwrap();
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

/// SPEC-0003 "Stored `#BUSY!`" (T-1222): a `#BUSY!` constant is saved as the plain
/// `<c t="e"><v>#BUSY!</v></c>` Excel loads (T-0999), and importing that cell gives the kind back,
/// with ERROR.TYPE 0 (probes/error-stored-constants S!B1).
#[test]
fn stored_busy_round_trips_with_error_type_zero() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.update_cell_with_error(0, 1, 1, Error::BUSY).unwrap();
    model.evaluate();
    let path = "temp_stored_busy_round_trip.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let stored = stored_errors(path);
    let has_vm = part(path, "xl/worksheets/sheet1.xml")
        .unwrap()
        .contains(" vm=");
    let mut reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(stored, ["#BUSY!"]);
    assert!(!has_vm, "#BUSY! needs no rich value");
    reloaded
        .set_user_input(0, 1, 2, "=ERROR.TYPE(A1)".to_string())
        .unwrap();
    reloaded
        .set_user_input(0, 1, 3, "=ISNA(A1)".to_string())
        .unwrap();
    reloaded.evaluate();
    let text = |column| reloaded.get_formatted_cell_value(0, 1, column).unwrap();
    assert_eq!(text(1), "#BUSY!");
    assert_eq!(text(2), "0");
    assert_eq!(text(3), "FALSE");
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

/// One part of the xlsx package at `path`, or `None` if it is absent.
fn part(path: &str, name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    archive
        .by_name(name)
        .ok()?
        .read_to_string(&mut xml)
        .unwrap();
    Some(xml)
}

/// The `<Override .../>` elements of `[Content_Types].xml` that name a rich-value or metadata
/// part, sorted.
fn rich_overrides(path: &str) -> Vec<String> {
    let types = part(path, "[Content_Types].xml").unwrap();
    let mut overrides: Vec<String> = types
        .split("<Override ")
        .skip(1)
        .map(|o| o.split("/>").next().unwrap().to_string())
        .filter(|o| o.contains("/xl/richData/") || o.contains("/xl/metadata.xml"))
        .collect();
    overrides.sort();
    overrides
}

/// Every cell of sheet 1 that carries `vm`, as `(r, cm, vm)`.
fn vm_cells(path: &str) -> Vec<(String, Option<String>, String)> {
    let sheet = part(path, "xl/worksheets/sheet1.xml").unwrap();
    let attribute = |tag: &str, name: &str| {
        tag.split(&format!(" {name}=\""))
            .nth(1)
            .map(|rest| rest.split('"').next().unwrap().to_string())
    };
    sheet
        .split("<c ")
        .skip(1)
        .filter_map(|cell| {
            let tag = format!(" {}", cell.split('>').next().unwrap());
            let vm = attribute(&tag, "vm")?;
            Some((attribute(&tag, "r").unwrap(), attribute(&tag, "cm"), vm))
        })
        .collect()
}

/// The committed fixtures that Excel loaded as evidence for the exported rich-error form
/// (eval/suites/fixtures/README.md) stay what the exporter writes: loading each one and saving
/// it again gives the same `vm` cells, `xl/metadata.xml`, `xl/richData/*` and content-type
/// overrides. A later change to the exporter's parts must then re-run the oracle on new fixtures
/// instead of leaving stale evidence (T-1148).
#[test]
fn rich_error_fixtures_match_the_exporter() {
    for name in ["rich-error-export", "rich-error-export-formulas"] {
        let fixture = format!("../../eval/suites/fixtures/cases/{name}.xlsx");
        let model = load_from_xlsx(&fixture, "en", "UTC", "en").unwrap();
        let path = format!("temp_{name}.xlsx");
        save_to_xlsx(&model, &path).unwrap();
        let parts = [
            "xl/metadata.xml",
            "xl/richData/rdrichvalue.xml",
            "xl/richData/rdrichvaluestructure.xml",
            "xl/richData/rdRichValueTypes.xml",
        ];
        // Read everything before asserting, so a failure leaves no temp file behind.
        let exported: Vec<_> = parts.iter().map(|name| part(&path, name)).collect();
        let (exported_vm, exported_overrides) = (vm_cells(&path), rich_overrides(&path));
        std::fs::remove_file(&path).unwrap();
        for (part_name, exported) in parts.iter().zip(exported) {
            assert!(exported.is_some(), "{name}: {part_name} not exported");
            assert_eq!(exported, part(&fixture, part_name), "{name}: {part_name}");
        }
        assert_eq!(exported_vm, vm_cells(&fixture), "{name}: vm cells");
        assert!(!exported_vm.is_empty(), "{name}: no vm cells");
        assert_eq!(
            exported_overrides,
            rich_overrides(&fixture),
            "{name}: content types"
        );
    }
}

/// SPEC-0010 step 3 (T-1577): a cached formula result holding a lone surrogate is saved as the
/// `_xHHHH_` escape Excel writes (probes/text-surrogate-positions A6 holds `a😀_xDE00_b`), and a
/// `_xHHHH_` look-alike in the well-formed part is still written `_x005F_xHHHH_`.
#[test]
fn cached_lone_surrogate_is_saved_as_an_escape() {
    use rustsheet_engine::types::{Cell, FormulaValue};
    use rustsheet_engine::Text;
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "=\"x\"".to_string()).unwrap();
    model.evaluate();
    let sheet = model.workbook.worksheet_mut(0).unwrap();
    let Some(Cell::CellFormula { v, .. }) = sheet.sheet_data.cell_mut(1, 1) else {
        panic!("A1 is not a formula cell");
    };
    let units: Vec<u16> = "a😀_x0001_"
        .encode_utf16()
        .chain([0xDE00])
        .chain("b".encode_utf16())
        .collect();
    *v = FormulaValue::Text(Text::from_utf16(units));
    let path = "temp_cached_lone_surrogate.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let xml = part(path, "xl/worksheets/sheet1.xml").unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(xml.contains("<v>a😀_x005F_x0001__xDE00_b</v>"), "{xml}");
}
