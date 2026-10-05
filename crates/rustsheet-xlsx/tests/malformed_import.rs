//! Malformed workbooks are untrusted input: import must return an error, never panic (T-0118).

use std::error::Error;
use std::io::{Cursor, Read, Write};

use rustsheet_xlsx::base::Model;
use rustsheet_xlsx::error::{ImportLimit, XlsxError};
use rustsheet_xlsx::import::{
    load_from_xlsx_bytes, load_from_xlsx_bytes_with_limits, ImportLimits,
};

type TestResult = Result<(), Box<dyn Error>>;

/// The parts of `tests/basic_text.xlsx`, as (name, content).
fn parts() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let bytes = std::fs::read("tests/basic_text.xlsx")?;
    let mut src = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut parts = Vec::new();
    for i in 0..src.len() {
        let mut file = src.by_index(i)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        parts.push((file.name().to_string(), content));
    }
    Ok(parts)
}

fn zip_of(parts: &[(String, String)]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in parts {
        out.start_file(name, zip::write::FileOptions::default())?;
        out.write_all(content.as_bytes())?;
    }
    Ok(out.finish()?.into_inner())
}

/// Re-zips `tests/basic_text.xlsx`, passing every part through `edit(name, content)`.
fn rewrite(edit: impl Fn(&str, String) -> String) -> Result<Vec<u8>, Box<dyn Error>> {
    let parts = parts()?
        .into_iter()
        .map(|(name, content)| {
            let content = edit(&name, content);
            (name, content)
        })
        .collect::<Vec<_>>();
    zip_of(&parts)
}

/// Import must succeed, and the workbook must load into a model and evaluate.
fn assert_imports(bytes: &[u8]) -> TestResult {
    let workbook = load_from_xlsx_bytes(bytes, "case", "en", "UTC")?;
    let mut model = Model::from_workbook(workbook, "en")?;
    model.evaluate();
    Ok(())
}

/// Removes the first `<tag ...>...</tag>` (or `<tag/>`) element from `xml`.
fn strip_element(xml: &str, tag: &str) -> String {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let Some(start) = xml.find(&open) else {
        return xml.to_string();
    };
    let end = match xml[start..].find(&close) {
        Some(offset) => start + offset + close.len(),
        None => start + xml[start..].find("/>").map_or(0, |o| o + 2),
    };
    format!("{}{}", &xml[..start], &xml[end..])
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
fn styles_without_cell_xfs_section_is_an_error() -> TestResult {
    let bytes = rewrite(|name, content| {
        let span = (name == "xl/styles.xml")
            .then(|| content.find("<cellXfs").zip(content.find("</cellXfs>")))
            .flatten();
        match span {
            Some((start, end)) => format!(
                "{}{}",
                &content[..start],
                &content[end + "</cellXfs>".len()..]
            ),
            None => content,
        }
    })?;
    assert_import_error(&bytes, "<cellXfs>");
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

#[test]
fn empty_fonts_with_an_xf_pointing_at_a_font_imports() -> TestResult {
    let bytes = rewrite(|name, content| {
        if name != "xl/styles.xml" {
            return content;
        }
        let start = content.find("<fonts").unwrap_or(0);
        let end = content.find("</fonts>").map_or(0, |e| e + "</fonts>".len());
        format!(
            "{}<fonts count=\"0\"/>{}",
            &content[..start],
            &content[end..]
        )
    })?;
    assert_imports(&bytes)
}

#[test]
fn styles_without_cell_styles_section_imports() -> TestResult {
    let bytes = rewrite(|name, content| {
        if name == "xl/styles.xml" {
            strip_element(&content, "cellStyles")
        } else {
            content
        }
    })?;
    assert_imports(&bytes)
}

#[test]
fn comment_with_an_empty_text_run_imports() -> TestResult {
    let mut parts = parts()?;
    parts.push((
        "xl/worksheets/_rels/sheet1.xml.rels".to_string(),
        concat!(
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
            "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments\" Target=\"../comments1.xml\"/>",
            "</Relationships>"
        )
        .to_string(),
    ));
    parts.push((
        "xl/comments1.xml".to_string(),
        concat!(
            "<comments xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">",
            "<authors><author>a</author></authors><commentList>",
            "<comment ref=\"A1\" authorId=\"0\"><text><r><t/></r><r><t>note</t></r></text></comment>",
            "</commentList></comments>"
        )
        .to_string(),
    ));
    assert_imports(&zip_of(&parts)?)
}

#[test]
fn worksheet_outside_the_worksheets_folder_imports() -> TestResult {
    let parts = parts()?
        .into_iter()
        .map(|(name, content)| match name.as_str() {
            "xl/worksheets/sheet1.xml" => ("xl/sheet1.xml".to_string(), content),
            "xl/_rels/workbook.xml.rels" => (
                name,
                content.replace(
                    "Target=\"worksheets/sheet1.xml\"",
                    "Target=\"/xl/sheet1.xml\"",
                ),
            ),
            _ => (name, content),
        })
        .collect::<Vec<_>>();
    assert_imports(&zip_of(&parts)?)
}

#[test]
fn shared_string_longer_than_the_text_limit_is_a_limit_error() -> TestResult {
    let long = "x".repeat(32_768);
    let bytes = rewrite(|name, content| {
        if name == "xl/sharedStrings.xml" {
            content.replacen("<t>", &format!("<t>{long}"), 1)
        } else {
            content
        }
    })?;
    assert_eq!(
        load_from_xlsx_bytes(&bytes, "case", "en", "UTC").err(),
        Some(XlsxError::Limit {
            limit: ImportLimit::TextLength,
            max: 32_767
        })
    );
    Ok(())
}

#[test]
fn inline_string_longer_than_the_text_limit_is_a_limit_error() -> TestResult {
    let cell = format!(
        "<sheetData><row r=\"100\"><c r=\"A100\" t=\"inlineStr\"><is><t>{}</t></is></c></row>",
        "x".repeat(32_768)
    );
    let bytes = rewrite(|name, content| {
        if name == "xl/worksheets/sheet1.xml" {
            content.replacen("<sheetData>", &cell, 1)
        } else {
            content
        }
    })?;
    assert_eq!(
        load_from_xlsx_bytes(&bytes, "case", "en", "UTC").err(),
        Some(XlsxError::Limit {
            limit: ImportLimit::TextLength,
            max: 32_767
        })
    );
    Ok(())
}

/// A sheet whose A100 is a `t="str"` formula result of `count` carriage returns, stored as
/// `_x000D_` escapes as Excel writes them.
fn sheet_with_escaped_text(count: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let cell = format!(
        "<sheetData><row r=\"100\"><c r=\"A100\" t=\"str\"><f>REPT(CHAR(13),{count})</f><v>{}</v></c></row>",
        "_x000D_".repeat(count)
    );
    rewrite(|name, content| {
        if name == "xl/worksheets/sheet1.xml" {
            content.replacen("<sheetData>", &cell, 1)
        } else {
            content
        }
    })
}

#[test]
fn escaped_cell_text_counts_decoded_characters() -> TestResult {
    assert_imports(&sheet_with_escaped_text(32_767)?)?;
    assert_eq!(
        load_from_xlsx_bytes(&sheet_with_escaped_text(32_768)?, "case", "en", "UTC").err(),
        Some(XlsxError::Limit {
            limit: ImportLimit::TextLength,
            max: 32_767
        })
    );
    Ok(())
}

#[test]
fn inflating_past_the_byte_budget_is_a_limit_error() -> TestResult {
    let bytes = zip_of(&parts()?)?;
    let total: u64 = parts()?.iter().map(|(_, c)| c.len() as u64).sum();
    let fits = ImportLimits {
        max_uncompressed_bytes: total,
        ..ImportLimits::default()
    };
    assert!(load_from_xlsx_bytes_with_limits(&bytes, "case", "en", "UTC", &fits).is_ok());
    let tight = ImportLimits {
        max_uncompressed_bytes: 1_000,
        ..ImportLimits::default()
    };
    assert_eq!(
        load_from_xlsx_bytes_with_limits(&bytes, "case", "en", "UTC", &tight).err(),
        Some(XlsxError::Limit {
            limit: ImportLimit::UncompressedBytes,
            max: 1_000
        })
    );
    Ok(())
}

#[test]
fn entity_bomb_in_a_part_is_a_limit_error() -> TestResult {
    let bytes = rewrite(|name, content| {
        if name == "xl/sharedStrings.xml" {
            let bomb =
                "<!DOCTYPE sst [<!ENTITY a \"lol\"><!ENTITY b \"&a;&a;&a;&a;&a;&a;&a;&a;\">]>";
            content
                .replacen("?>", &format!("?>{bomb}"), 1)
                .replacen("<t>", "<t>&b;", 1)
        } else {
            content
        }
    })?;
    assert_eq!(
        load_from_xlsx_bytes(&bytes, "case", "en", "UTC").err(),
        Some(XlsxError::Limit {
            limit: ImportLimit::EntityExpansion,
            max: 0
        })
    );
    Ok(())
}
