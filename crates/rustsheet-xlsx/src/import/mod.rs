mod columns;
mod conditional_formatting;
mod iso_date;
mod metadata;
mod package;
pub mod rich_errors;
pub(crate) mod shared_strings;
mod sheet_data;
mod styles;
mod tables;
mod theme;
mod util;
mod workbook;
mod worksheets;
mod xml;

use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Cursor, Read, Seek},
};

use xml::XmlNode;

use rustsheet_engine::{
    expressions::{
        parser::{new_parser_english, stringify::to_english_string},
        types::CellReferenceRC,
    },
    types::{Metadata, Workbook, WorkbookSettings, WorkbookView},
    Model,
};

use crate::error::XlsxError;

pub use iso_date::iso8601_to_serial;
pub use package::{ImportLimits, PartNames};
pub use shared_strings::decode_xlsx_escapes;
use shared_strings::read_shared_strings;
pub use workbook::date_system_from_attribute;

use metadata::load_metadata;
use package::{resolve_target, Package};
use styles::load_styles;
use util::get_attribute;
use workbook::load_workbook;
use worksheets::{load_sheets, Relationship};

fn load_relationships<R: Read + Seek>(
    archive: &mut Package<R>,
) -> Result<HashMap<String, Relationship>, XlsxError> {
    let file = archive.open("xl/_rels/workbook.xml.rels")?;
    let doc = XmlNode::parse(file)?;
    let nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
        .collect();
    let mut rels = HashMap::new();
    for node in nodes {
        rels.insert(
            get_attribute(node, "Id")?.to_string(),
            Relationship {
                rel_type: get_attribute(node, "Type")?.to_string(),
                target: get_attribute(node, "Target")?.to_string(),
            },
        );
    }
    Ok(rels)
}

fn resolve_theme_path(rels: &HashMap<String, Relationship>) -> Option<String> {
    let target = &rels
        .values()
        .find(|r| r.rel_type.ends_with("/theme"))?
        .target;
    Some(resolve_target("xl", target))
}

// FIXME: This is a bit of a HACK. We basically re-parse all the defined names assuming the context is `A1`
// and there is no tables or defined names.
fn reparse_formula_hack(formula: &str, worksheets: &[String]) -> Result<String, XlsxError> {
    let defined_names = Vec::new();
    let tables = HashMap::new();
    let mut parser = new_parser_english(worksheets.to_owned(), defined_names, tables);
    let cell_reference = CellReferenceRC {
        sheet: worksheets[0].clone(),
        column: 1,
        row: 1,
    };
    let t = parser.parse(formula, &cell_reference);

    Ok(to_english_string(&t, &cell_reference))
}

fn load_xlsx_from_reader<R: Read + Seek>(
    name: String,
    reader: R,
    locale: &str,
    tz: &str,
    limits: &ImportLimits,
) -> Result<Workbook, XlsxError> {
    let mut archive = Package::new(reader, *limits)?;
    let workbook = load_package(&mut archive, name, locale, tz);
    // A part that ran over the byte budget may have been skipped as optional or surfaced as an
    // XML error; either way the overrun is what is reported.
    match archive.overrun() {
        Some(overrun) => Err(overrun),
        None => workbook,
    }
}

fn load_package<R: Read + Seek>(
    archive: &mut Package<R>,
    name: String,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    let mut shared_strings = read_shared_strings(archive)?;
    let (mut workbook, date_system) = load_workbook(archive)?;
    let rels = load_relationships(archive)?;
    let theme_path = resolve_theme_path(&rels);
    let theme = theme::load(archive, theme_path.as_deref());
    let mut tables = HashMap::new();
    // Styles must be loaded before the worksheets: conditional-formatting rules
    // stored in x14 `extLst` extensions carry inline `<x14:dxf>` formats that we
    // append to `styles.dxfs`, referencing them back by index from the rule.
    let mut styles = load_styles(archive, &theme)?;
    let (worksheets, selected_sheet) = load_sheets(
        archive,
        &rels,
        &workbook,
        &mut tables,
        &mut shared_strings,
        &theme,
        &mut styles.dxfs,
    )?;
    // reparse formulas in defined names, since they may refer to sheets and tables that have been loaded
    let worksheet_names = worksheets
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    for dn in &mut workbook.defined_names {
        dn.formula = reparse_formula_hack(&dn.formula, &worksheet_names)?;
    }
    let metadata = match load_metadata(archive) {
        Ok(metadata) => metadata,
        Err(_) => {
            // In case there is no metadata, add some
            Metadata {
                application: "Unknown application".to_string(),
                app_version: "".to_string(),
                creator: "".to_string(),
                last_modified_by: "".to_string(),
                created: "".to_string(),
                last_modified: "".to_string(),
            }
        }
    };
    let mut views = HashMap::new();
    views.insert(
        0,
        WorkbookView {
            sheet: selected_sheet,
            window_width: 800,
            window_height: 600,
        },
    );
    Ok(Workbook {
        shared_strings: shared_strings.into_strings(),
        defined_names: workbook.defined_names,
        worksheets,
        styles,
        name,
        settings: WorkbookSettings {
            tz: tz.to_string(),
            locale: locale.to_string(),
            date_system,
        },
        metadata,
        tables,
        views,
        theme,
    })
}

// Imports a file from disk into an internal representation
fn load_from_excel(file_name: &str, locale: &str, tz: &str) -> Result<Workbook, XlsxError> {
    let file_path = std::path::Path::new(file_name);
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let name = file_path
        .file_stem()
        .ok_or_else(|| XlsxError::IO("Could not extract workbook name".to_string()))?
        .to_string_lossy()
        .to_string();
    load_xlsx_from_reader(name, reader, locale, tz, &ImportLimits::default())
}

/// Loads a [Workbook] from the bytes of an xlsx file, under the default [`ImportLimits`].
/// This is useful, for instance, when bytes are transferred over the network
pub fn load_from_xlsx_bytes(
    bytes: &[u8],
    name: &str,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    load_from_xlsx_bytes_with_limits(bytes, name, locale, tz, &ImportLimits::default())
}

/// Loads a [Workbook] from the bytes of an xlsx file, rejecting it with
/// [`XlsxError::Limit`] if it exceeds `limits`.
pub fn load_from_xlsx_bytes_with_limits(
    bytes: &[u8],
    name: &str,
    locale: &str,
    tz: &str,
    limits: &ImportLimits,
) -> Result<Workbook, XlsxError> {
    load_xlsx_from_reader(name.to_string(), Cursor::new(bytes), locale, tz, limits)
}

/// Loads a [Model] from an xlsx file
pub fn load_from_xlsx(
    file_name: &str,
    locale: &str,
    tz: &str,
    language: &str,
) -> Result<Model, XlsxError> {
    let workbook = load_from_excel(file_name, locale, tz)?;
    Model::from_workbook(workbook, language).map_err(XlsxError::Workbook)
}

/// Loads a [Model] from an `ic` file (a file in the IronCalc internal representation)
pub fn load_from_icalc(file_name: &str, language_id: &str) -> Result<Model, XlsxError> {
    let contents = fs::read(file_name)
        .map_err(|e| XlsxError::IO(format!("Could not extract workbook name: {e}")))?;
    let workbook: Workbook = bitcode::decode(&contents)
        .map_err(|e| XlsxError::IO(format!("Failed to decode file: {e}")))?;
    Model::from_workbook(workbook, language_id).map_err(XlsxError::Workbook)
}
