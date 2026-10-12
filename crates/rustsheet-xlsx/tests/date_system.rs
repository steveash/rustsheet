#![allow(clippy::unwrap_used, reason = "tests")]

use rustsheet_engine::types::DateSystem;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::{date_system_from_attribute, load_from_xlsx_bytes};
use std::io::{Cursor, Read};

/// SPEC-0005: `1` and `true` are 1904; absent, `0`, `false` and garbage are 1900, never an error.
#[test]
fn date1904_attribute_maps_to_a_date_system() {
    for (value, expected) in [
        (Some("1"), DateSystem::Excel1904),
        (Some("true"), DateSystem::Excel1904),
        (None, DateSystem::Excel1900),
        (Some("0"), DateSystem::Excel1900),
        (Some("false"), DateSystem::Excel1900),
        (Some("TRUE"), DateSystem::Excel1900),
        (Some("yes"), DateSystem::Excel1900),
        (Some(""), DateSystem::Excel1900),
    ] {
        assert_eq!(date_system_from_attribute(value), expected, "{value:?}");
    }
}

fn export(system: DateSystem) -> Vec<u8> {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    model.workbook.settings.date_system = system;
    save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
}

fn workbook_xml(bytes: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    zip.by_name("xl/workbook.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

#[test]
fn a_1904_workbook_round_trips_and_a_1900_export_writes_no_workbook_pr() {
    let bytes = export(DateSystem::Excel1904);
    assert!(workbook_xml(&bytes).contains("<workbookPr date1904=\"1\"/>"));
    let loaded = load_from_xlsx_bytes(&bytes, "m", "en", "UTC").unwrap();
    assert_eq!(loaded.settings.date_system, DateSystem::Excel1904);

    let bytes = export(DateSystem::Excel1900);
    assert!(!workbook_xml(&bytes).contains("workbookPr"));
    let loaded = load_from_xlsx_bytes(&bytes, "m", "en", "UTC").unwrap();
    assert_eq!(loaded.settings.date_system, DateSystem::Excel1900);
}
