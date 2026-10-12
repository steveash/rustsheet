use std::io::Read;

use super::package::Package;

use super::xml::XmlNode;
use rustsheet_engine::types::{DateSystem, DefinedName, SheetState};

use crate::error::XlsxError;
use crate::util::name_from_xlsx;

use super::{
    util::get_attribute,
    worksheets::{Sheet, WorkbookXML},
};

/// The date system a `workbookPr@date1904` attribute selects: `1` and `true` are 1904; absent,
/// `0`, `false` and anything else are 1900, never an error (workbooks are untrusted).
#[must_use]
pub fn date_system_from_attribute(value: Option<&str>) -> DateSystem {
    if matches!(value, Some("1" | "true")) {
        DateSystem::Excel1904
    } else {
        DateSystem::Excel1900
    }
}

pub(super) fn load_workbook<R: Read + std::io::Seek>(
    archive: &mut Package<R>,
) -> Result<(WorkbookXML, DateSystem), XlsxError> {
    let file = archive.open("xl/workbook.xml")?;
    let doc = XmlNode::parse(file)?;
    let mut defined_names = Vec::new();
    let mut sheets = Vec::new();
    let date_system = date_system_from_attribute(
        doc.descendants()
            .find(|n| n.has_tag_name("workbookPr"))
            .and_then(|n| n.attribute("date1904")),
    );
    // Get the sheets
    let sheet_nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("sheet"))
        .collect();
    for sheet in sheet_nodes {
        let name = get_attribute(sheet, "name")?.to_string();
        let sheet_id = get_attribute(sheet, "sheetId")?.to_string();
        let sheet_id = sheet_id.parse::<u32>()?;
        let id = sheet
            .relationship_id()
            .ok_or_else(|| XlsxError::Xml("Missing \"r:id\" XML attribute".to_string()))?
            .to_string();
        let state = match sheet.attribute("state") {
            Some("visible") | None => SheetState::Visible,
            Some("hidden") => SheetState::Hidden,
            Some("veryHidden") => SheetState::VeryHidden,
            Some(state) => return Err(XlsxError::Xml(format!("Unknown sheet state: {state}"))),
        };
        sheets.push(Sheet {
            name,
            sheet_id,
            id,
            state,
        });
    }
    // Get the defined names
    let name_nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("definedName"))
        .collect();
    for node in name_nodes {
        let name = name_from_xlsx(get_attribute(node, "name")?).to_string();
        let formula = node.text().unwrap_or("").to_string();
        // NOTE: In Excel the `localSheetId` is just the index of the worksheet and unrelated to the sheetId
        let sheet_id = match node.attribute("localSheetId") {
            Some(s) => {
                let index = s.parse::<usize>()?;
                let sheet = sheets.get(index).ok_or_else(|| {
                    XlsxError::Workbook(format!(
                        "definedName {name}: no sheet at localSheetId {index}"
                    ))
                })?;
                Some(sheet.sheet_id)
            }
            None => None,
        };
        defined_names.push(DefinedName {
            name,
            formula,
            sheet_id,
        });
    }
    // read the relationships file
    Ok((
        WorkbookXML {
            worksheets: sheets,
            defined_names,
        },
        date_system,
    ))
}
