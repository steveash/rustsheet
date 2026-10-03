//! Malformed workbooks are untrusted input: import must return an error, never panic (T-0118).

use std::io::{Cursor, Read, Write};

use rustsheet_xlsx::import::load_from_xlsx_bytes;

/// Re-zips `tests/basic_text.xlsx`, passing every part through `edit(name, content)`.
fn rewrite(edit: impl Fn(&str, String) -> String) -> Vec<u8> {
    let bytes = std::fs::read("tests/basic_text.xlsx").unwrap();
    let mut src = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i).unwrap();
        let name = file.name().to_string();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        out.start_file(&name, zip::write::FileOptions::default())
            .unwrap();
        out.write_all(edit(&name, content).as_bytes()).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn styles_without_fonts_section_is_an_error() {
    let bytes = rewrite(|name, content| {
        if name == "xl/styles.xml" {
            let start = content.find("<fonts").unwrap();
            let end = content.find("</fonts>").unwrap() + "</fonts>".len();
            format!("{}{}", &content[..start], &content[end..])
        } else {
            content
        }
    });
    let err = load_from_xlsx_bytes(&bytes, "case", "en", "UTC").unwrap_err();
    assert!(err.to_string().contains("<fonts>"), "{err}");
}

#[test]
fn sheet_without_relationship_is_an_error() {
    let bytes = rewrite(|name, content| {
        if name == "xl/workbook.xml" {
            content.replace("r:id=\"rId1\"", "r:id=\"rId99\"")
        } else {
            content
        }
    });
    let err = load_from_xlsx_bytes(&bytes, "case", "en", "UTC").unwrap_err();
    assert!(err.to_string().contains("rId99"), "{err}");
}
