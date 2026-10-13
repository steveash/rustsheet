//! The values Excel cached for the workbooks this one links to (`xl/externalLinks/*.xml`).

use std::{collections::HashMap, io::Read};

use rustsheet_engine::{
    expressions::token::{get_error_by_english_name, Error},
    types::{DefinedName, ExternalBook, SpillValue},
};

use super::{
    package::{resolve_target, Package},
    util::parse_cell_number,
    worksheets::{parse_cell_reference, Relationship},
    xml::XmlNode,
};

/// One [`ExternalBook`] per `externalReference` of the workbook, in order, so `[n]` in a formula
/// is entry `n - 1`. A link that is not to a workbook (DDE, OLE) or whose part is missing or
/// unreadable is an empty book: it caches nothing, and the books after it keep their index.
pub(super) fn load_external_books<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
    rels: &HashMap<String, Relationship>,
    external_references: &[String],
) -> Vec<ExternalBook> {
    external_references
        .iter()
        .map(|id| {
            rels.get(id)
                .and_then(|rel| load_external_book(archive, &resolve_target("xl", &rel.target)))
                .unwrap_or_default()
        })
        .collect()
}

fn load_external_book<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
    path: &str,
) -> Option<ExternalBook> {
    let part = archive.open_optional(path).ok()??;
    let doc = XmlNode::parse(part).ok()?;
    let book = doc.descendants().find(|n| n.has_tag_name("externalBook"))?;
    Some(read_external_book(book))
}

fn read_external_book(book: &XmlNode) -> ExternalBook {
    let sheet_names = book
        .descendants()
        .filter(|n| n.has_tag_name("sheetName"))
        .map(|n| n.attribute("val").unwrap_or_default().to_string())
        .collect();
    let defined_names = book
        .descendants()
        .filter(|n| n.has_tag_name("definedName"))
        .filter_map(|n| {
            Some(DefinedName {
                name: n.attribute("name")?.to_string(),
                formula: n.attribute("refersTo").unwrap_or_default().to_string(),
                sheet_id: n.attribute("sheetId").and_then(|id| id.parse().ok()),
            })
        })
        .collect();
    let mut cells = HashMap::new();
    for sheet_data in book.descendants().filter(|n| n.has_tag_name("sheetData")) {
        let Some(sheet) = sheet_data
            .attribute("sheetId")
            .and_then(|id| id.parse::<u32>().ok())
        else {
            continue;
        };
        for cell in sheet_data.descendants().filter(|n| n.has_tag_name("cell")) {
            let Some(Ok((row, column))) = cell.attribute("r").map(parse_cell_reference) else {
                continue;
            };
            // A cell with no `<v>` caches nothing: the reference reads an empty cell.
            let Some(value) = cell.child("v").map(|v| v.text().unwrap_or_default()) else {
                continue;
            };
            cells.insert(
                (sheet, row, column),
                cached_value(cell.attribute("t"), value),
            );
        }
    }
    ExternalBook {
        sheet_names,
        defined_names,
        cells,
    }
}

/// The value a cache `<cell t=…><v>…</v></cell>` holds: `t` absent or `n` for a number, `str`
/// for text as Excel writes it (poi/ref-56737.xlsx), `b` with `1`/`0`, `e`. `t="s"` would be a
/// shared-string index, which an externalLink part has no table for, so it is read as a number
/// like any type this does not know.
fn cached_value(cell_type: Option<&str>, value: &str) -> SpillValue {
    match cell_type {
        Some("str") => SpillValue::Text(value.to_string()),
        Some("b") => SpillValue::Boolean(value == "1"),
        Some("e") => SpillValue::Error(get_error_by_english_name(value).unwrap_or(Error::ERROR)),
        _ => SpillValue::Number(parse_cell_number(Some(value))),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    // The cache of poi/ref-56737.xlsx externalLink1.xml (Excel-written), trimmed.
    #[test]
    fn reads_sheet_names_names_and_cells() {
        let xml = r#"<externalLink><externalBook><sheetNames><sheetName val="Uses"/><sheetName val="Defines"/></sheetNames><definedNames><definedName name="NR_Global_B2" refersTo="='Defines'!$B$2"/><definedName name="NR_To_A1" refersTo="='Defines'!$A$1" sheetId="1"/></definedNames><sheetDataSet><sheetData sheetId="0"><row r="1"><cell r="A1" t="str"><v>Hello!</v></cell><cell r="B1"/></row></sheetData><sheetData sheetId="1"><row r="2"><cell r="B2"><v>142</v></cell><cell r="C2" t="b"><v>1</v></cell><cell r="D2" t="e"><v>#N/A</v></cell></row></sheetData><sheetData sheetId="2" refreshError="1"/></sheetDataSet></externalBook></externalLink>"#;
        let doc = XmlNode::parse_str(xml).unwrap();
        let book = read_external_book(doc.child("externalBook").unwrap());
        assert_eq!(book.sheet_names, ["Uses", "Defines"]);
        assert_eq!(book.defined_names.len(), 2);
        assert_eq!(book.defined_names[0].sheet_id, None);
        assert_eq!(book.defined_names[1].sheet_id, Some(1));
        assert_eq!(book.defined_names[1].formula, "='Defines'!$A$1");
        assert_eq!(
            book.cells.get(&(0, 1, 1)),
            Some(&SpillValue::Text("Hello!".to_string()))
        );
        assert_eq!(book.cells.get(&(0, 1, 2)), None);
        assert_eq!(book.cells.get(&(1, 2, 2)), Some(&SpillValue::Number(142.0)));
        assert_eq!(book.cells.get(&(1, 2, 3)), Some(&SpillValue::Boolean(true)));
        assert_eq!(
            book.cells.get(&(1, 2, 4)),
            Some(&SpillValue::Error(Error::NA))
        );
        assert_eq!(book.cells.len(), 4);
    }
}
