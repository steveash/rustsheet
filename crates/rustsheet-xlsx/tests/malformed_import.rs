//! Malformed workbooks are untrusted input: import must return an error, never panic (T-0118).

use std::error::Error;
use std::io::{Cursor, Read, Write};

use rustsheet_xlsx::import::load_from_xlsx_bytes;

type TestResult = Result<(), Box<dyn Error>>;

/// Re-zips `tests/basic_text.xlsx`, passing every part through `edit(name, content)`.
fn rewrite(edit: impl Fn(&str, String) -> String) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = std::fs::read("tests/basic_text.xlsx")?;
    let mut src = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i)?;
        let name = file.name().to_string();
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        out.start_file(&name, zip::write::FileOptions::default())?;
        out.write_all(edit(&name, content).as_bytes())?;
    }
    Ok(out.finish()?.into_inner())
}

/// Import must fail with a message containing `needle`.
fn assert_import_error(bytes: &[u8], needle: &str) {
    let err = load_from_xlsx_bytes(bytes, "case", "en", "UTC").err();
    assert!(
        err.as_ref().is_some_and(|e| e.to_string().contains(needle)),
        "expected an import error containing {needle:?}, got {err:?}"
    );
}

#[test]
fn styles_without_fonts_section_is_an_error() -> TestResult {
    let bytes = rewrite(|name, content| {
        let span = (name == "xl/styles.xml")
            .then(|| content.find("<fonts").zip(content.find("</fonts>")))
            .flatten();
        match span {
            Some((start, end)) => format!(
                "{}{}",
                &content[..start],
                &content[end + "</fonts>".len()..]
            ),
            None => content,
        }
    })?;
    assert_import_error(&bytes, "<fonts>");
    Ok(())
}

#[test]
fn sheet_without_relationship_is_an_error() -> TestResult {
    let bytes = rewrite(|name, content| {
        if name == "xl/workbook.xml" {
            content.replace("r:id=\"rId1\"", "r:id=\"rId99\"")
        } else {
            content
        }
    })?;
    assert_import_error(&bytes, "rId99");
    Ok(())
}
