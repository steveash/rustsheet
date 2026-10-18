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
        parts.push((file.name()?.into_owned(), content));
    }
    Ok(parts)
}

fn zip_of(parts: &[(String, String)]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in parts {
        out.start_file(name, zip::write::SimpleFileOptions::default())?;
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

/// `tests/basic_text.xlsx` as imported, to compare a rewritten package against.
fn basic_text() -> Result<rustsheet_xlsx::base::types::Workbook, Box<dyn Error>> {
    Ok(load_from_xlsx_bytes(
        &zip_of(&parts()?)?,
        "case",
        "en",
        "UTC",
    )?)
}

#[test]
fn backslash_and_lowercase_entry_names_import_like_excel() -> TestResult {
    // Excel for the web opens poi/49609.xlsx, whose zip entries look like these (T-0112).
    let parts = parts()?
        .into_iter()
        .map(|(name, content)| (name.replace('/', "\\").to_ascii_lowercase(), content))
        .collect::<Vec<_>>();
    let workbook = load_from_xlsx_bytes(&zip_of(&parts)?, "case", "en", "UTC")?;
    assert_eq!(workbook, basic_text()?);
    Ok(())
}

#[test]
fn strict_relationships_namespace_imports() -> TestResult {
    // Excel writes Strict OOXML r:id attributes in this namespace (poi/SimpleStrict.xlsx).
    let bytes = rewrite(|name, content| {
        if name == "xl/workbook.xml" {
            content.replace(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships\"",
                "http://purl.oclc.org/ooxml/officeDocument/relationships\"",
            )
        } else {
            content
        }
    })?;
    assert_eq!(
        load_from_xlsx_bytes(&bytes, "case", "en", "UTC")?,
        basic_text()?
    );
    Ok(())
}

#[test]
fn defined_name_with_out_of_range_local_sheet_id_is_an_error() -> TestResult {
    let bytes = rewrite(|name, content| {
        if name == "xl/workbook.xml" {
            content.replace(
                "</sheets>",
                "</sheets><definedNames><definedName name=\"_xlnm.Print_Area\" \
                 localSheetId=\"9\">Sheet1!$A$1</definedName></definedNames>",
            )
        } else {
            content
        }
    })?;
    assert_import_error(&bytes, "localSheetId 9");
    Ok(())
}

/// A sheet whose A100 holds the formula `1+1+...` stored with exactly `chars` characters.
fn sheet_with_formula_chars(chars: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    // 4,095 `1+` links, then a last term padded to the length.
    let head = "1+".repeat(4_095);
    let formula = format!("{head}{}", "1".repeat(chars - head.len()));
    let cell = format!("<sheetData><row r=\"100\"><c r=\"A100\"><f>{formula}</f></c></row>");
    rewrite(|name, content| {
        if name == "xl/worksheets/sheet1.xml" {
            content.replacen("<sheetData>", &cell, 1)
        } else {
            content
        }
    })
}

// Excel for the web loads a formula of 8,192 stored characters and refuses the workbook at
// 8,193 (T-1060, T-1091; docs/excel-semantics/syntax.md; T-1093).
#[test]
fn formula_longer_than_8192_characters_refuses_the_workbook() -> TestResult {
    assert_imports(&sheet_with_formula_chars(8_192)?)?;
    assert_import_error(
        &sheet_with_formula_chars(8_193)?,
        "Formula in Sheet1!A100 is longer than 8192 UTF-16 code units",
    );
    Ok(())
}

/// A workbook whose only defined name `nm` has `body` as its stored text.
fn workbook_with_name_body(body: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let names = format!(
        "</sheets><definedNames><definedName name=\"nm\">{body}</definedName></definedNames>"
    );
    rewrite(|name, content| {
        if name == "xl/workbook.xml" {
            content.replace("</sheets>", &names)
        } else {
            content
        }
    })
}

/// A sheet whose A100 holds `-LEN("<emoji_count emoji>")`, `8 + 2 * emoji_count` UTF-16 units.
fn sheet_with_emoji_formula(emoji_count: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let f = format!("-LEN(\"{}\")", "\u{1F600}".repeat(emoji_count));
    let cell = format!("<sheetData><row r=\"100\"><c r=\"A100\"><f>{f}</f></c></row>");
    rewrite(|name, content| {
        if name == "xl/worksheets/sheet1.xml" {
            content.replacen("<sheetData>", &cell, 1)
        } else {
            content
        }
    })
}

// Excel for the web loads a name body of 8,191 characters and refuses 8,192 (T-1386).
#[test]
fn name_body_longer_than_8191_characters_refuses_the_workbook() -> TestResult {
    let ascii = "1".repeat(8_191);
    assert_imports(&workbook_with_name_body(&ascii)?)?;
    assert_import_error(
        &workbook_with_name_body(&format!("{ascii}1"))?,
        "definedName nm body is longer than 8191 UTF-16 code units",
    );
    Ok(())
}

// Excel for the web counts UTF-16 code units, emoji being two (T-1397 probes
// `name-body-nonbmp-*`, `formula-length-nonbmp-*`): these bodies are 8,190 and 8,192 units but
// 4,097 and 4,099 scalars, so a scalar count would accept the second.
#[test]
fn non_bmp_text_counts_two_utf16_units_toward_the_limits() -> TestResult {
    let emoji = "\u{1F600}".repeat(4_093);
    assert_imports(&workbook_with_name_body(&format!("\"{emoji}\"&amp;1"))?)?;
    assert_import_error(
        &workbook_with_name_body(&format!("\"{emoji}\"&amp;1&amp;1"))?,
        "definedName nm body is longer than 8191 UTF-16 code units",
    );
    // 4,092 emoji + 8 = 8,192 units loads; one more emoji is 8,194 and is refused.
    assert_imports(&sheet_with_emoji_formula(4_092)?)?;
    assert_import_error(
        &sheet_with_emoji_formula(4_093)?,
        "Formula in Sheet1!A100 is longer than 8192 UTF-16 code units",
    );
    Ok(())
}
