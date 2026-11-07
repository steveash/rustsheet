#![allow(clippy::unwrap_used, reason = "tests")]
//! A workbook that links to other workbooks keeps the links when saved: one externalLink part per
//! book, with its target, so `[n]` references read the same cached values after a reload (T-4143;
//! the round-trip census docs/research/xlsx-round-trip-2026-11-04.md §2.2).

use std::io::{Cursor, Read};

use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::{load_from_xlsx, load_from_xlsx_bytes};

const CASES: [&str; 7] = [
    "external-link-cached-ranges",
    "external-link-cached-text",
    "external-link-cached-index-array",
    "external-link-cached-lookups",
    "external-link-cached-3d-span",
    "external-link-uncached-lookups",
    "external-link-unsorted-approx-lookups",
];

fn path(case: &str) -> String {
    format!("../../eval/suites/fixtures/cases/{case}.xlsx")
}

fn saved(model: &Model) -> Vec<u8> {
    save_xlsx_to_writer(model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut file = archive.by_name(name).unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    xml
}

/// Every cell's formatted value, sheet by sheet, after a recalculation.
fn values(model: &mut Model) -> Vec<(u32, i32, i32, String)> {
    model.evaluate();
    let mut values = Vec::new();
    for sheet in 0..u32::try_from(model.workbook.worksheets.len()).unwrap() {
        let dimension = model.workbook.worksheet(sheet).unwrap().dimension();
        for row in dimension.min_row..=dimension.max_row {
            for column in dimension.min_column..=dimension.max_column {
                let value = model.get_formatted_cell_value(sheet, row, column).unwrap();
                values.push((sheet, row, column, value));
            }
        }
    }
    values
}

#[test]
fn a_reload_reads_the_same_external_books_and_values() {
    for case in CASES {
        let mut model = load_from_xlsx(&path(case), "en", "UTC", "en").unwrap();
        assert!(!model.workbook.external_books.is_empty(), "{case}: no link");
        assert!(
            model
                .workbook
                .external_books
                .iter()
                .all(|book| !book.target.is_empty()),
            "{case}: a book without its target"
        );
        let bytes = saved(&model);
        let workbook = load_from_xlsx_bytes(&bytes, "saved", "en", "UTC").unwrap();
        let mut reloaded = Model::from_workbook(workbook, "en").unwrap();
        assert_eq!(
            reloaded.workbook.external_books, model.workbook.external_books,
            "{case}: external books"
        );
        assert_eq!(values(&mut reloaded), values(&mut model), "{case}: values");
        // A second save writes the same parts: the export is a fixed point.
        assert_eq!(
            part(&saved(&reloaded), "xl/externalLinks/externalLink1.xml"),
            part(&bytes, "xl/externalLinks/externalLink1.xml"),
            "{case}: second save"
        );
    }
}

#[test]
fn the_package_relates_each_link_to_its_original_target() {
    let source = std::fs::read(path("external-link-cached-lookups")).unwrap();
    let model = load_from_xlsx(&path("external-link-cached-lookups"), "en", "UTC", "en").unwrap();
    let bytes = saved(&model);
    assert!(part(&bytes, "[Content_Types].xml").contains(
        r#"<Override PartName="/xl/externalLinks/externalLink1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.externalLink+xml"/>"#
    ));
    // The source and the saved file name the same book, outside the package.
    for bytes in [&source, &bytes] {
        assert!(
            part(bytes, "xl/externalLinks/_rels/externalLink1.xml.rels").contains(
                r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLinkPath" Target="56737.xlsx" TargetMode="External""#
            )
        );
    }
    let workbook = part(&bytes, "xl/workbook.xml");
    let rels = part(&bytes, "xl/_rels/workbook.xml.rels");
    let id = workbook
        .split(r#"<externalReference r:id=""#)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap();
    assert!(rels.contains(&format!(
        r#"<Relationship Id="{id}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLink" Target="externalLinks/externalLink1.xml"/>"#
    )));
    // `<externalReferences>` sits between `<sheets>` and `<definedNames>` (CT_Workbook order).
    let at = |tag: &str| workbook.find(tag).unwrap();
    assert!(at("</sheets>") < at("<externalReferences>"));
    assert!(at("</externalReferences>") < at("<definedNames>"));
}

#[test]
fn a_workbook_without_links_writes_no_external_link_part() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    assert!(model.workbook.external_books.is_empty());
    let bytes = saved(&model);
    let archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    assert!(archive
        .file_names()
        .all(|name| !name.unwrap().contains("externalLink")));
    assert!(!part(&bytes, "xl/workbook.xml").contains("externalReferences"));
}
