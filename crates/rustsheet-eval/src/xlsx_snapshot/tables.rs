//! Table parts (`xl/tables/tableN.xml`) of a workbook snapshot.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};

use super::{attr, read_part, xml_err, SnapshotError, Zip};

/// A table part (`xl/tables/tableN.xml`), as far as volatile-cell detection needs it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableDef {
    /// The table's `displayName` (what structured references name; matched case-insensitively).
    pub name: String,
    /// Name of the sheet that holds the table.
    pub sheet: String,
    /// The table's `ref` including header and totals rows, e.g. `J15:L23`.
    pub range: String,
    /// `headerRowCount` (1 unless the table has no header row).
    pub header_rows: u32,
    /// `totalsRowCount`.
    pub totals_rows: u32,
    /// Column names, left to right.
    pub columns: Vec<String>,
}

/// Appends the tables of the sheet part at `sheet_path` (found through its relationships).
pub(super) fn read_sheet_tables(
    zip: &mut Zip,
    sheet_path: &str,
    sheet: &str,
    out: &mut Vec<TableDef>,
) -> Result<(), SnapshotError> {
    let (dir, file) = sheet_path.rsplit_once('/').unwrap_or(("", sheet_path));
    let Some(rels) = read_part(zip, &format!("{dir}/_rels/{file}.rels"))? else {
        return Ok(());
    };
    let mut reader = Reader::from_str(&rels);
    let mut targets = Vec::new();
    loop {
        match reader.read_event().map_err(|e| xml_err(file, e))? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "Relationship" => {
                if let (Some(kind), Some(target)) = (attr(&e, "Type"), attr(&e, "Target")) {
                    if kind.ends_with("/table") {
                        targets.push(target);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    for target in targets {
        let path = match target.strip_prefix('/') {
            Some(abs) => abs.to_string(),
            None => resolve_relative(dir, &target),
        };
        if let Some(xml) = read_part(zip, &path)? {
            out.extend(parse_table(&xml, sheet, &path)?);
        }
    }
    Ok(())
}

/// `target` (a relationship target, possibly with `../` segments) resolved against `dir`.
fn resolve_relative(dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    for seg in target.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(seg),
        }
    }
    parts.join("/")
}

fn parse_table(xml: &str, sheet: &str, part: &str) -> Result<Option<TableDef>, SnapshotError> {
    let mut reader = Reader::from_str(xml);
    let mut table: Option<TableDef> = None;
    loop {
        match reader.read_event().map_err(|e| xml_err(part, e))? {
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                "table" => {
                    let count = |key| attr(&e, key).and_then(|v| v.parse().ok());
                    table = Some(TableDef {
                        name: attr(&e, "displayName")
                            .or_else(|| attr(&e, "name"))
                            .unwrap_or_default(),
                        sheet: sheet.to_string(),
                        range: attr(&e, "ref").unwrap_or_default(),
                        header_rows: count("headerRowCount").unwrap_or(1),
                        totals_rows: count("totalsRowCount").unwrap_or(0),
                        columns: Vec::new(),
                    });
                }
                "tableColumn" => {
                    if let (Some(t), Some(name)) = (table.as_mut(), attr(&e, "name")) {
                        t.columns.push(name);
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(table)
}
