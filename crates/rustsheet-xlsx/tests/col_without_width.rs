//! A `<col>` without a `width` attribute imports and takes the sheet default width (T-0418,
//! ECMA-376 18.3.1.13; port of ironcalc@9d5fa0dc).

use std::error::Error;
use std::io::{Cursor, Read, Write};

use rustsheet_engine::{COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH};
use rustsheet_xlsx::import::load_from_xlsx_bytes;

type TestResult = Result<(), Box<dyn Error>>;
/// (width, hidden, style) of each imported `<col>`.
type ColSummary = (f64, bool, Option<i32>);

/// `tests/basic_text.xlsx` with its `<cols>` replaced by `cols` and, when given, its
/// `<sheetFormatPr>` attributes extended with `format_attrs`.
fn workbook_with(cols: &str, format_attrs: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut src = zip::ZipArchive::new(Cursor::new(std::fs::read("tests/basic_text.xlsx")?))?;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut file = src.by_index(i)?;
        let name = file.name().to_string();
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        if name == "xl/worksheets/sheet1.xml" {
            let start = content.find("<cols>").ok_or("no <cols>")?;
            let end = content.find("</cols>").ok_or("no </cols>")? + "</cols>".len();
            content = format!(
                "{}<cols>{cols}</cols>{}",
                &content[..start],
                &content[end..]
            );
            content = content.replace(
                "<sheetFormatPr ",
                &format!("<sheetFormatPr {format_attrs} "),
            );
        }
        out.start_file(name, zip::write::FileOptions::default())?;
        out.write_all(content.as_bytes())?;
    }
    Ok(out.finish()?.into_inner())
}

fn widths(bytes: &[u8]) -> Result<Vec<ColSummary>, Box<dyn Error>> {
    let workbook = load_from_xlsx_bytes(bytes, "case", "en", "UTC")?;
    Ok(workbook.worksheets[0]
        .cols
        .iter()
        .map(|c| (c.width, c.hidden, c.style))
        .collect())
}

#[test]
fn col_without_width_takes_the_engine_default() -> TestResult {
    let bytes = workbook_with(
        r#"<col min="2" max="2" style="1"/><col min="3" max="3" hidden="1"/>"#,
        "",
    )?;
    let default = DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR;
    assert_eq!(
        widths(&bytes)?,
        vec![(default, false, Some(1)), (default, true, None)]
    );
    Ok(())
}

#[test]
fn col_without_width_takes_default_col_width() -> TestResult {
    let bytes = workbook_with(
        r#"<col min="2" max="2" style="1"/><col min="3" max="3" width="5" customWidth="1"/>"#,
        r#"defaultColWidth="12.5""#,
    )?;
    assert_eq!(
        widths(&bytes)?,
        vec![(12.5, false, Some(1)), (5.0, false, None)]
    );
    Ok(())
}
