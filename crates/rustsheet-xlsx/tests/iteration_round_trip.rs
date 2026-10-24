#![allow(clippy::unwrap_used, reason = "tests")]

//! Iterative-calculation settings survive export and import (docs/excel-semantics/recalc.md,
//! T-0447).

use rustsheet_engine::types::Iteration;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_to_xlsx;
use rustsheet_xlsx::import::load_from_xlsx;
use std::io::Read;

fn workbook_xml(path: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    archive
        .by_name("xl/workbook.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// `calcPr` carries `iterate`, `iterateCount` and `iterateDelta`, and a loaded workbook iterates
/// its circular references with them; without iteration `calcPr` stays empty.
#[test]
fn iteration_settings_round_trip() {
    let mut model = Model::new_empty("m", "en", "UTC", "en").unwrap();
    let iteration = Iteration {
        max_iterations: 7,
        max_change: 0.25,
    };
    model.workbook.settings.iteration = Some(iteration);
    model.set_user_input(0, 1, 1, "=A1+1".to_string()).unwrap();
    model.evaluate();
    let path = "temp_iteration_round_trip.xlsx";
    save_to_xlsx(&model, path).unwrap();
    let xml = workbook_xml(path);
    let mut reloaded = load_from_xlsx(path, "en", "UTC", "en").unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        xml.contains(r#"<calcPr iterate="1" iterateCount="7" iterateDelta="0.25"/>"#),
        "{xml}"
    );
    assert_eq!(reloaded.workbook.settings.iteration, Some(iteration));
    assert_eq!(reloaded.get_formatted_cell_value(0, 1, 1).unwrap(), "7");
    reloaded.evaluate();
    assert_eq!(
        reloaded.get_formatted_cell_value(0, 1, 1).unwrap(),
        "14",
        "resumes from 7"
    );

    let plain = Model::new_empty("m", "en", "UTC", "en").unwrap();
    let path = "temp_iteration_off_round_trip.xlsx";
    save_to_xlsx(&plain, path).unwrap();
    let xml = workbook_xml(path);
    std::fs::remove_file(path).unwrap();
    assert!(xml.contains("<calcPr/>"), "{xml}");
}
