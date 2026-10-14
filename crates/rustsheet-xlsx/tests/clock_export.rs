#![allow(clippy::unwrap_used)]

//! The exporter stamps `dcterms:modified` from the workbook clock (SPEC-0008).

use std::io::{Cursor, Read};

use rustsheet_engine::{clock::Clock, Model};
use rustsheet_xlsx::export::save_xlsx_to_writer;

fn save(model: &Model) -> Vec<u8> {
    save_xlsx_to_writer(model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
}

fn core_xml(bytes: &[u8]) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    archive
        .by_name("docProps/core.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

#[test]
fn modified_time_is_the_fixed_clock_and_saves_are_reproducible() {
    // 2023-03-20T13:44:25.208Z.
    let clock = Clock::fixed_unix_millis(1_679_319_865_208).unwrap();
    let mut model = Model::new_empty_with_clock("clock", "en", "UTC", "en", clock).unwrap();
    model.set_user_input(0, 1, 1, "=NOW()".to_string()).unwrap();
    model.evaluate();
    let first = save(&model);
    let xml = core_xml(&first);
    assert!(
        xml.contains(">2023-03-20T13:44:25Z</dcterms:modified>"),
        "{xml}"
    );
    assert!(
        xml.contains(">2023-03-20T13:44:25Z</dcterms:created>"),
        "{xml}"
    );
    assert_eq!(first, save(&model));
}
