use rustsheet_engine::types::Workbook;

use super::rich_errors::rich_value_parts;
use super::xml_constants::{XML_DECLARATION, XML_WORKSHEET};

/// With `rich`, the workbook also relates to the rich-value parts (T-1014).
pub(crate) fn get_workbook_xml_rels(workbook: &Workbook, rich: bool) -> String {
    let mut relationships_str: Vec<String> = vec![];
    let worksheet_count = workbook.worksheets.len() + 1;
    for id in 1..worksheet_count {
        relationships_str.push(format!(
            "<Relationship Id=\"rId{id}\" Type=\"{XML_WORKSHEET}\" Target=\"worksheets/sheet{id}.xml\"/>"
        ));
    }
    let mut id = worksheet_count;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sheetMetadata\" Target=\"metadata.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>")
    );
    if rich {
        for (part, _, relationship, _) in rich_value_parts() {
            id += 1;
            let target = part.trim_start_matches("xl/");
            relationships_str.push(format!(
                "<Relationship Id=\"rId{id}\" Type=\"{relationship}\" Target=\"{target}\"/>"
            ));
        }
    }
    format!(
        "{XML_DECLARATION}\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{}</Relationships>",
        relationships_str.join("")
    )
}
