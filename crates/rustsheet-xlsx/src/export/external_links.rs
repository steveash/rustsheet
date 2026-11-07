//! The workbooks this one links to: one `xl/externalLinks/externalLinkN.xml` part per
//! [`ExternalBook`], in `[n]` order, with its `externalLinkPath` relationship to the book's target,
//! its content-type override, the workbook's relationship to it and its `<externalReference>`
//! (T-4143). Without them a saved `[1]Sheet!A1` names no book, and Excel reads it as `#REF!`.
//!
//! The part is written the way Excel writes it (fixtures/external-link-cached-lookups.xlsx): the
//! sheet names, the defined names, then one `sheetData` per sheet of the cache, in sheet order,
//! its cells by row and then column.

use std::fmt::Write as _;
use std::io::{Seek, Write};

use rustsheet_engine::expressions::utils::number_to_column;
use rustsheet_engine::types::{ExternalBook, SpillValue, Workbook};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::escape::{escape_text, escape_xml};
use super::worksheets::exported_error_literal;
use super::xml_constants::XML_DECLARATION;
use crate::error::XlsxError;

/// The content type of an externalLink part.
const EXTERNAL_LINK_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.externalLink+xml";
/// The workbook's relationship to an externalLink part.
const EXTERNAL_LINK_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLink";
/// An externalLink part's relationship to the linked workbook.
const EXTERNAL_LINK_PATH_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLinkPath";

/// The content-type overrides of the externalLink parts.
pub(super) fn content_types(workbook: &Workbook) -> Vec<String> {
    (1..=workbook.external_books.len())
        .map(|part| {
            format!(
                r#"<Override PartName="/xl/externalLinks/externalLink{part}.xml" ContentType="{EXTERNAL_LINK_TYPE}"/>"#
            )
        })
        .collect()
}

/// The workbook's relationships to the externalLink parts, `rId{first_id}` on.
pub(super) fn workbook_relationships(workbook: &Workbook, first_id: usize) -> Vec<String> {
    (1..=workbook.external_books.len())
        .map(|part| {
            format!(
                r#"<Relationship Id="rId{}" Type="{EXTERNAL_LINK_RELATIONSHIP}" Target="externalLinks/externalLink{part}.xml"/>"#,
                first_id + part - 1
            )
        })
        .collect()
}

/// The workbook's `<externalReferences>`, its relationships `rId{first_id}` on, in `[n]` order;
/// empty without a linked book.
pub(super) fn workbook_external_references(workbook: &Workbook, first_id: usize) -> String {
    if workbook.external_books.is_empty() {
        return String::new();
    }
    let mut xml = String::from("<externalReferences>");
    for part in 1..=workbook.external_books.len() {
        let _ = write!(
            xml,
            r#"<externalReference r:id="rId{}"/>"#,
            first_id + part - 1
        );
    }
    xml.push_str("</externalReferences>");
    xml
}

/// Writes each externalLink part and its relationships part.
pub(super) fn write<W: Write + Seek>(
    workbook: &Workbook,
    zip: &mut ZipWriter<W>,
    options: SimpleFileOptions,
) -> Result<(), XlsxError> {
    if workbook.external_books.is_empty() {
        return Ok(());
    }
    zip.add_directory("xl/externalLinks", options)?;
    zip.add_directory("xl/externalLinks/_rels", options)?;
    for (part, book) in (1..).zip(&workbook.external_books) {
        zip.start_file(format!("xl/externalLinks/externalLink{part}.xml"), options)?;
        zip.write_all(external_link_xml(book).as_bytes())?;
        zip.start_file(
            format!("xl/externalLinks/_rels/externalLink{part}.xml.rels"),
            options,
        )?;
        zip.write_all(external_link_rels(book).as_bytes())?;
    }
    Ok(())
}

/// The relationships part of a book's externalLink part: `rId1` to its target, outside the
/// package. A book with no target ([`ExternalBook::target`]) keeps the empty one, so it keeps its
/// `[n]` and the books after it keep theirs.
fn external_link_rels(book: &ExternalBook) -> String {
    format!(
        r#"{XML_DECLARATION}
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{EXTERNAL_LINK_PATH_RELATIONSHIP}" Target="{}" TargetMode="External"/></Relationships>"#,
        escape_xml(&book.target)
    )
}

/// The externalLink part of `book`.
fn external_link_xml(book: &ExternalBook) -> String {
    let mut xml = format!(
        r#"{XML_DECLARATION}
<externalLink xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><externalBook xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="rId1">"#
    );
    if !book.sheet_names.is_empty() {
        xml.push_str("<sheetNames>");
        for name in &book.sheet_names {
            let _ = write!(xml, r#"<sheetName val="{}"/>"#, escape_xml(name));
        }
        xml.push_str("</sheetNames>");
    }
    if !book.defined_names.is_empty() {
        xml.push_str("<definedNames>");
        for name in &book.defined_names {
            let sheet_id = name
                .sheet_id
                .map(|id| format!(r#" sheetId="{id}""#))
                .unwrap_or_default();
            let _ = write!(
                xml,
                r#"<definedName name="{}" refersTo="{}"{sheet_id}/>"#,
                escape_xml(&name.name),
                escape_xml(&name.formula)
            );
        }
        xml.push_str("</definedNames>");
    }
    let sheet_data = sheet_data_set(book);
    if !sheet_data.is_empty() {
        let _ = write!(xml, "<sheetDataSet>{sheet_data}</sheetDataSet>");
    }
    xml.push_str("</externalBook></externalLink>");
    xml
}

/// The `sheetData` elements of `book`'s cache, in sheet order: one per sheet it was read with
/// ([`ExternalBook::cached_sheets`]), and one per other sheet it holds cells of or marks
/// `refreshError`. Empty for a book with none.
fn sheet_data_set(book: &ExternalBook) -> String {
    let mut cells: Vec<(&(u32, i32, i32), &SpillValue)> = book.cells.iter().collect();
    cells.sort_unstable_by_key(|(key, _)| **key);
    let mut sheets: Vec<u32> = (book.cached_sheets.iter().copied())
        .chain(cells.iter().map(|((sheet, _, _), _)| *sheet))
        .chain(book.refresh_error_sheets.iter().copied())
        .collect();
    sheets.sort_unstable();
    sheets.dedup();
    let mut xml = String::new();
    let mut cells = cells.into_iter().peekable();
    for sheet in sheets {
        let refresh_error = if book.refresh_error_sheets.contains(&sheet) {
            r#" refreshError="1""#
        } else {
            ""
        };
        let _ = write!(xml, r#"<sheetData sheetId="{sheet}"{refresh_error}>"#);
        let mut open_row = None;
        while let Some(((_, row, column), value)) =
            cells.next_if(|((cell_sheet, _, _), _)| *cell_sheet == sheet)
        {
            if open_row != Some(*row) {
                if open_row.is_some() {
                    xml.push_str("</row>");
                }
                let _ = write!(xml, r#"<row r="{row}">"#);
                open_row = Some(*row);
            }
            let Some(column) = number_to_column(*column) else {
                continue;
            };
            let _ = write!(
                xml,
                r#"<cell r="{column}{row}"{}</cell>"#,
                cell_value(value)
            );
        }
        if open_row.is_some() {
            xml.push_str("</row>");
        }
        xml.push_str("</sheetData>");
    }
    xml
}

/// The rest of a cache `<cell>` after its `r`: its type and `<v>`, as the importer reads them
/// (`t="str"` for text, as Excel writes it; poi/ref-56737.xlsx).
fn cell_value(value: &SpillValue) -> String {
    match value {
        SpillValue::Number(n) => format!("><v>{n}</v>"),
        SpillValue::Text(text) => format!(r#" t="str"><v>{}</v>"#, escape_text(text)),
        SpillValue::Boolean(b) => format!(r#" t="b"><v>{}</v>"#, u8::from(*b)),
        SpillValue::Error(error) => {
            format!(r#" t="e"><v>{}</v>"#, exported_error_literal(error))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use std::collections::HashMap;

    use rustsheet_engine::expressions::token::Error;
    use rustsheet_engine::types::DefinedName;

    use super::*;

    #[test]
    fn writes_names_and_cells_by_sheet_row_and_column() {
        let book = ExternalBook {
            target: "a&b.xlsx".to_string(),
            sheet_names: vec!["Uses".to_string(), "Defines".to_string()],
            defined_names: vec![DefinedName {
                name: "NR_To_A1".to_string(),
                formula: "='Defines'!$A$1".to_string(),
                sheet_id: Some(1),
                hidden: false,
                comment: None,
            }],
            cells: HashMap::from([
                ((1, 2, 2), SpillValue::Number(142.0)),
                ((0, 1, 2), SpillValue::Boolean(true)),
                ((0, 1, 1), SpillValue::Text("<Hi>".into())),
                ((0, 2, 1), SpillValue::Error(Error::NA)),
            ]),
            cached_sheets: Vec::new(),
            refresh_error_sheets: vec![2],
        };
        let xml = external_link_xml(&book);
        assert!(xml.contains(
            r#"<sheetNames><sheetName val="Uses"/><sheetName val="Defines"/></sheetNames><definedNames><definedName name="NR_To_A1" refersTo="=&apos;Defines&apos;!$A$1" sheetId="1"/></definedNames>"#
        ));
        assert!(xml.contains(
            r#"<sheetDataSet><sheetData sheetId="0"><row r="1"><cell r="A1" t="str"><v>&lt;Hi&gt;</v></cell><cell r="B1" t="b"><v>1</v></cell></row><row r="2"><cell r="A2" t="e"><v>#N/A</v></cell></row></sheetData><sheetData sheetId="1"><row r="2"><cell r="B2"><v>142</v></cell></row></sheetData><sheetData sheetId="2" refreshError="1"></sheetData></sheetDataSet>"#
        ));
        assert!(
            external_link_rels(&book).contains(r#"Target="a&amp;b.xlsx" TargetMode="External""#)
        );
    }

    // fixtures/external-link-uncached-lookups.xlsx: no `sheetData` for sheet 1, an empty one for 2.
    #[test]
    fn writes_sheet_data_only_for_the_sheets_it_was_read_with() {
        let book = ExternalBook {
            sheet_names: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            cells: HashMap::from([((0, 1, 1), SpillValue::Number(1.5))]),
            cached_sheets: vec![0, 2],
            ..ExternalBook::default()
        };
        assert_eq!(
            sheet_data_set(&book),
            r#"<sheetData sheetId="0"><row r="1"><cell r="A1"><v>1.5</v></cell></row></sheetData><sheetData sheetId="2"></sheetData>"#
        );
    }
}
