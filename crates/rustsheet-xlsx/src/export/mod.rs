#![allow(clippy::unwrap_used)]

mod _rels;
mod conditional_formatting;
mod doc_props;
mod dxfs_styles;
mod escape;
mod rich_errors;
mod shared_strings;
mod styles;
mod styles_util;
mod theme;
mod workbook;
mod workbook_xml_rels;
mod worksheets;
pub(crate) use rich_errors::rich_error_vm;
pub(crate) use worksheets::exported_error_literal;
mod xml_constants;

use std::io::BufWriter;
use std::{
    fs,
    io::{Seek, Write},
};

use rustsheet_engine::expressions::utils::number_to_column;
use rustsheet_engine::types::Workbook;
use rustsheet_engine::Model;

use self::xml_constants::XML_DECLARATION;

use crate::error::XlsxError;

#[cfg(test)]
mod test;

fn get_content_types_xml(workbook: &Workbook, rich: bool) -> String {
    // A list of all files in the zip
    let mut content = vec![
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#.to_string(),
        r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#.to_string(),
        r#"<Default Extension="xml" ContentType="application/xml"/>"#.to_string(),
        r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#.to_string(),
    ];
    for worksheet in 0..workbook.worksheets.len() {
        let sheet = format!(
            r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
            worksheet + 1
        );
        content.push(sheet);
    }
    content.push(
        r#"<Override PartName="/xl/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>"#.to_string(),
    );
    content.extend([
        r#"<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#.to_string(),
        r#"<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>"#.to_string(),
        r#"<Override PartName="/xl/metadata.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheetMetadata+xml"/>"#.to_string(),
    ]);
    if rich {
        for (part, content_type, _, _) in rich_errors::rich_value_parts() {
            content.push(format!(
                r#"<Override PartName="/{part}" ContentType="{content_type}"/>"#
            ));
        }
    }
    content.extend([
        r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#.to_string(),
        r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#.to_string(),
        r#"</Types>"#.to_string(),
    ]);
    format!("{XML_DECLARATION}\n{}", content.join(""))
}

/// Exports a model to an xlsx file
pub fn save_to_xlsx(model: &Model, file_name: &str) -> Result<(), XlsxError> {
    let file_path = std::path::Path::new(&file_name);
    if file_path.exists() {
        return Err(XlsxError::IO(format!("file {file_name} already exists")));
    }
    let file = fs::File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    save_xlsx_to_writer(model, writer)?;

    Ok(())
}

pub fn save_xlsx_to_writer<W: Write + Seek>(model: &Model, writer: W) -> Result<W, XlsxError> {
    let workbook = &model.workbook;
    let selected_sheet = match workbook.views.get(&0) {
        Some(view) => view.sheet,
        _ => 0,
    };
    // Cells holding `#SPILL!`, `#CALC!`, ... point (`vm`) at rich values (T-1014).
    let rich = rich_errors::has_rich_errors(workbook);
    let mut zip = zip::ZipWriter::new(writer);

    let options = zip::write::FileOptions::default();

    // root folder
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(get_content_types_xml(workbook, rich).as_bytes())?;

    zip.add_directory("docProps", options)?;
    zip.start_file("docProps/app.xml", options)?;
    zip.write_all(doc_props::get_app_xml(workbook).as_bytes())?;
    zip.start_file("docProps/core.xml", options)?;
    let milliseconds = model.clock().now_unix_millis();
    zip.write_all(doc_props::get_core_xml(workbook, milliseconds)?.as_bytes())?;

    // Package-relationship item
    zip.add_directory("_rels", options)?;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(_rels::get_dot_rels(workbook).as_bytes())?;

    zip.add_directory("xl", options)?;
    zip.start_file("xl/sharedStrings.xml", options)?;
    zip.write_all(shared_strings::get_shared_strings_xml(workbook).as_bytes())?;
    zip.start_file("xl/styles.xml", options)?;
    zip.write_all(styles::get_styles_xml(workbook).as_bytes())?;
    zip.start_file("xl/workbook.xml", options)?;
    zip.write_all(workbook::get_workbook_xml(workbook, selected_sheet)?.as_bytes())?;
    zip.start_file(crate::import::rich_errors::METADATA_PART, options)?;
    zip.write_all(rich_errors::get_metadata_xml(rich).as_bytes())?;
    if rich {
        zip.add_directory("xl/richData", options)?;
        for (part, _, _, xml) in rich_errors::rich_value_parts() {
            zip.start_file(part, options)?;
            zip.write_all(xml.as_bytes())?;
        }
    }

    zip.add_directory("xl/theme", options)?;
    zip.start_file("xl/theme/theme1.xml", options)?;
    zip.write_all(theme::get_theme_xml(&workbook.theme).as_bytes())?;

    zip.add_directory("xl/_rels", options)?;
    zip.start_file("xl/_rels/workbook.xml.rels", options)?;
    zip.write_all(workbook_xml_rels::get_workbook_xml_rels(workbook, rich).as_bytes())?;

    zip.add_directory("xl/worksheets", options)?;
    // sheet rels parts (one per sheet with external hyperlinks)
    let mut has_worksheet_rels = false;
    for (sheet_index, worksheet) in workbook.worksheets.iter().enumerate() {
        if let Some(rels_xml) = worksheets::get_worksheet_xml_rels(worksheet) {
            if !has_worksheet_rels {
                zip.add_directory("xl/worksheets/_rels", options)?;
                has_worksheet_rels = true;
            }
            zip.start_file(
                format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_index + 1),
                options,
            )?;
            zip.write_all(rels_xml.as_bytes())?;
        }
    }
    for (sheet_index, worksheet) in workbook.worksheets.iter().enumerate() {
        let id = sheet_index + 1;
        zip.start_file(format!("xl/worksheets/sheet{id}.xml"), options)?;
        let dimension = model
            .workbook
            .worksheet(sheet_index as u32)
            .unwrap()
            .dimension();
        let column_min_str = number_to_column(dimension.min_column).unwrap();
        let column_max_str = number_to_column(dimension.max_column).unwrap();
        let min_row = dimension.min_row;
        let max_row = dimension.max_row;
        let sheet_dimension_str = &format!("{column_min_str}{min_row}:{column_max_str}{max_row}");
        let is_sheet_selected = selected_sheet as usize == sheet_index;
        zip.write_all(
            worksheets::get_worksheet_xml(
                worksheet,
                &model.parsed_formulas[sheet_index],
                sheet_dimension_str,
                is_sheet_selected,
            )
            .as_bytes(),
        )?;
    }

    let writer = zip.finish()?;
    Ok(writer)
}

/// Exports a model to an icalc file
pub fn save_to_icalc(model: &Model, file_name: &str) -> Result<(), XlsxError> {
    let file_path = std::path::Path::new(&file_name);
    if file_path.exists() {
        return Err(XlsxError::IO(format!("file {file_name} already exists")));
    }
    let s = bitcode::encode(&model.workbook);
    let mut file = fs::File::create(file_path)?;
    file.write_all(&s)?;

    Ok(())
}
