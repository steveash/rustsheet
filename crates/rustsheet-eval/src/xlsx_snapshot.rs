//! Minimal, engine-independent xlsx reader that extracts what Excel *stored*:
//! formulas, cached formula results, and constant inputs.
//!
//! This is intentionally separate from `rustsheet_xlsx::import`: if the engine's
//! importer has a bug, the expectations extracted here must not inherit it.
//! Only the parts of SpreadsheetML needed for scoring are parsed.

use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Read};

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use serde::{Deserialize, Serialize};

use crate::cellref::{parse_range, Addr};
use crate::value::Value;
use rustsheet_xlsx::import::decode_xlsx_escapes;

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("xml error in {part}: {message}")]
    Xml { part: String, message: String },
    #[error("malformed workbook: {0}")]
    Malformed(String),
}

/// How a formula is attached to a cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum FormulaKind {
    Normal,
    /// Member of a shared-formula group (`si`). The master carries the text.
    Shared {
        si: u32,
    },
    /// Anchor of a (CSE or dynamic) array formula covering `range`.
    Array {
        range: String,
    },
    /// A cell inside an array formula's range (spilled value, no own formula).
    ArrayMember {
        anchor: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellSnapshot {
    /// Formula text without leading `=`; for shared-formula members, the master's text.
    pub formula: Option<String>,
    pub formula_kind: Option<FormulaKind>,
    /// Cached value (for formulas) or constant value (for inputs).
    pub value: Value,
}

impl CellSnapshot {
    #[must_use]
    pub fn is_formula(&self) -> bool {
        self.formula_kind.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSnapshot {
    pub name: String,
    #[serde(with = "crate::cellref::a1_map")]
    pub cells: BTreeMap<Addr, CellSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkbookSnapshot {
    /// `docProps/app.xml` `<Application>` (e.g. "Microsoft Excel").
    pub application: Option<String>,
    pub app_version: Option<String>,
    /// `true` when the workbook uses the 1904 date system.
    pub date1904: bool,
    pub sheets: Vec<SheetSnapshot>,
}

impl WorkbookSnapshot {
    /// Whether the cached values were written by a Microsoft Excel build (desktop, Mac, web, mobile).
    ///
    /// XlsxWriter and some other libraries also claim `Microsoft Excel` but with the Excel 2007
    /// `AppVersion` `12.0000` and placeholder cached values, so that version is rejected.
    #[must_use]
    pub fn is_excel_authored(&self) -> bool {
        let app_ok = self
            .application
            .as_deref()
            .is_some_and(|a| a.contains("Excel"));
        let version_ok = !self
            .app_version
            .as_deref()
            .is_some_and(|v| v.starts_with("12."));
        app_ok && version_ok
    }
}

type Zip = zip::ZipArchive<Cursor<Vec<u8>>>;

fn read_part(zip: &mut Zip, name: &str) -> Result<Option<String>, SnapshotError> {
    let mut file = match zip.by_name(name) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(Some(s))
}

fn xml_err(part: &str, e: impl std::fmt::Display) -> SnapshotError {
    SnapshotError::Xml {
        part: part.to_string(),
        message: e.to_string(),
    }
}

fn attr(e: &BytesStart<'_>, key: &str) -> Option<String> {
    e.attributes().flatten().find_map(|a| {
        (a.key.local_name().as_ref() == key)
            .then(|| {
                a.normalized_value(XmlVersion::Implicit1_0)
                    .ok()
                    .map(std::borrow::Cow::into_owned)
            })
            .flatten()
    })
}

/// Reads an xlsx file's stored formulas and values.
///
/// # Errors
/// Returns an error if the bytes are not a readable xlsx package.
pub fn read_snapshot(bytes: &[u8]) -> Result<WorkbookSnapshot, SnapshotError> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes.to_vec()))?;
    let (application, app_version) = read_app(&mut zip)?;
    let shared = read_shared_strings(&mut zip)?;
    let rels = read_workbook_rels(&mut zip)?;
    let workbook = read_part(&mut zip, "xl/workbook.xml")?
        .ok_or_else(|| SnapshotError::Malformed("missing xl/workbook.xml".into()))?;
    let (sheet_list, date1904) = parse_workbook(&workbook)?;
    let mut sheets = Vec::new();
    for (name, rid) in sheet_list {
        let Some(target) = rels.get(&rid) else {
            continue;
        };
        let path = normalize_target(target);
        let Some(xml) = read_part(&mut zip, &path)? else {
            continue; // chartsheets / missing parts are not scored
        };
        let cells = parse_sheet(&xml, &path, &shared)?;
        sheets.push(SheetSnapshot { name, cells });
    }
    Ok(WorkbookSnapshot {
        application,
        app_version,
        date1904,
        sheets,
    })
}

fn normalize_target(target: &str) -> String {
    if let Some(stripped) = target.strip_prefix('/') {
        stripped.to_string()
    } else {
        format!("xl/{target}")
    }
}

fn read_app(zip: &mut Zip) -> Result<(Option<String>, Option<String>), SnapshotError> {
    let Some(xml) = read_part(zip, "docProps/app.xml")? else {
        return Ok((None, None));
    };
    let mut reader = Reader::from_str(&xml);
    let (mut app, mut ver, mut current) = (None, None, String::new());
    loop {
        match reader.read_event().map_err(|e| xml_err("app.xml", e))? {
            Event::Start(e) => current = e.local_name().as_ref().to_string(),
            Event::Text(t) => {
                let text = t.xml_content(XmlVersion::Implicit1_0).into_owned();
                match current.as_str() {
                    "Application" => app = Some(text),
                    "AppVersion" => ver = Some(text),
                    _ => {}
                }
            }
            Event::End(_) => current.clear(),
            Event::Eof => break,
            _ => {}
        }
    }
    Ok((app, ver))
}

/// Longest text Excel allows in one cell.
const MAX_CELL_TEXT_CHARS: usize = 32_767;

fn read_shared_strings(zip: &mut Zip) -> Result<Vec<String>, SnapshotError> {
    let Some(xml) = read_part(zip, "xl/sharedStrings.xml")? else {
        return Ok(Vec::new());
    };
    let mut reader = Reader::from_str(&xml);
    let mut out = Vec::new();
    let mut cur = String::new();
    let (mut in_t, mut in_phonetic) = (false, false);
    loop {
        match reader
            .read_event()
            .map_err(|e| xml_err("sharedStrings.xml", e))?
        {
            Event::Start(e) => match e.local_name().as_ref() {
                "si" => cur.clear(),
                "t" => in_t = true,
                "rPh" => in_phonetic = true,
                _ => {}
            },
            Event::Empty(e) if e.local_name().as_ref() == "si" => out.push(String::new()),
            Event::Text(t) if in_t && !in_phonetic => {
                cur.push_str(&t.xml_content(XmlVersion::Implicit1_0));
            }
            Event::GeneralRef(r) if in_t && !in_phonetic => cur.push_str(&resolve_entity(&r)),
            Event::End(e) => match e.local_name().as_ref() {
                "si" => {
                    // Excel caps cell text at 32,767 characters. Snapshot cells copy their shared
                    // string, so an oversized one referenced by many cells would expand without
                    // bound (poi poc-shared-strings: 1 MB x 12,000 cells, T-0118).
                    if cur.chars().count() > MAX_CELL_TEXT_CHARS {
                        return Err(SnapshotError::Malformed(format!(
                            "shared string longer than {MAX_CELL_TEXT_CHARS} characters"
                        )));
                    }
                    out.push(decode_xlsx_escapes(&std::mem::take(&mut cur)));
                }
                "t" => in_t = false,
                "rPh" => in_phonetic = false,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

fn resolve_entity(r: &quick_xml::events::BytesRef<'_>) -> String {
    if let Ok(Some(c)) = r.resolve_char_ref() {
        return c.to_string();
    }
    quick_xml::escape::resolve_predefined_entity(r)
        .map(str::to_string)
        .unwrap_or_default()
}

fn read_workbook_rels(zip: &mut Zip) -> Result<HashMap<String, String>, SnapshotError> {
    let xml = read_part(zip, "xl/_rels/workbook.xml.rels")?
        .ok_or_else(|| SnapshotError::Malformed("missing workbook rels".into()))?;
    let mut reader = Reader::from_str(&xml);
    let mut map = HashMap::new();
    loop {
        match reader
            .read_event()
            .map_err(|e| xml_err("workbook.xml.rels", e))?
        {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "Relationship" => {
                if let (Some(id), Some(target)) = (attr(&e, "Id"), attr(&e, "Target")) {
                    map.insert(id, target);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(map)
}

fn parse_workbook(xml: &str) -> Result<(Vec<(String, String)>, bool), SnapshotError> {
    let mut reader = Reader::from_str(xml);
    let mut sheets = Vec::new();
    let mut date1904 = false;
    loop {
        match reader
            .read_event()
            .map_err(|e| xml_err("workbook.xml", e))?
        {
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                "sheet" => {
                    if let (Some(name), Some(rid)) = (attr(&e, "name"), attr(&e, "id")) {
                        sheets.push((name, rid));
                    }
                }
                "workbookPr" => {
                    date1904 = matches!(attr(&e, "date1904").as_deref(), Some("1" | "true"));
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok((sheets, date1904))
}

#[derive(Default)]
struct PendingCell {
    addr: Option<Addr>,
    t: Option<String>,
    f_text: String,
    f_attrs: Option<(Option<String>, Option<String>, Option<String>)>, // (t, si, ref)
    v_text: Option<String>,
    is_text: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    None,
    F,
    V,
    IsT,
}

fn parse_sheet(
    xml: &str,
    part: &str,
    shared: &[String],
) -> Result<BTreeMap<Addr, CellSnapshot>, SnapshotError> {
    let mut reader = Reader::from_str(xml);
    let mut cells = BTreeMap::new();
    let mut shared_masters: HashMap<u32, String> = HashMap::new();
    let mut array_ranges: Vec<(Addr, Addr, Addr)> = Vec::new(); // (top-left, bottom-right, anchor)
    let mut cur = PendingCell::default();
    let mut field = Field::None;
    let mut in_cell = false;
    // Track implicit cell positions when `r` is omitted.
    let (mut row_num, mut next_col) = (0u32, 1u32);
    loop {
        match reader.read_event().map_err(|e| xml_err(part, e))? {
            Event::Start(e) => match e.local_name().as_ref() {
                "row" => {
                    row_num = attr(&e, "r")
                        .and_then(|r| r.parse().ok())
                        .unwrap_or(row_num + 1);
                    next_col = 1;
                }
                "c" => {
                    cur = start_cell(&e, row_num, &mut next_col);
                    in_cell = true;
                }
                "f" if in_cell => {
                    cur.f_attrs = Some((attr(&e, "t"), attr(&e, "si"), attr(&e, "ref")));
                    field = Field::F;
                }
                "v" if in_cell => field = Field::V,
                "t" if in_cell => field = Field::IsT,
                _ => {}
            },
            Event::Empty(e) => match e.local_name().as_ref() {
                "c" => {
                    let c = start_cell(&e, row_num, &mut next_col);
                    finish_cell(
                        c,
                        shared,
                        &mut shared_masters,
                        &mut array_ranges,
                        &mut cells,
                    );
                }
                "f" if in_cell => {
                    cur.f_attrs = Some((attr(&e, "t"), attr(&e, "si"), attr(&e, "ref")));
                }
                "row" => {
                    row_num = attr(&e, "r")
                        .and_then(|r| r.parse().ok())
                        .unwrap_or(row_num + 1);
                }
                _ => {}
            },
            Event::Text(t) => {
                let text = t.xml_content(XmlVersion::Implicit1_0);
                push_field(&mut cur, field, &text);
            }
            Event::GeneralRef(r) => push_field(&mut cur, field, &resolve_entity(&r)),
            Event::End(e) => match e.local_name().as_ref() {
                "c" => {
                    in_cell = false;
                    let c = std::mem::take(&mut cur);
                    finish_cell(
                        c,
                        shared,
                        &mut shared_masters,
                        &mut array_ranges,
                        &mut cells,
                    );
                }
                "f" | "v" | "t" => field = Field::None,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    // Shared-formula members that appeared before their master get the master text now.
    for cell in cells.values_mut() {
        if let Some(FormulaKind::Shared { si }) = cell.formula_kind {
            if cell.formula.is_none() {
                cell.formula = shared_masters.get(&si).cloned();
            }
        }
    }
    // Cells inside array ranges (other than the anchor) are array members.
    for (tl, br, anchor) in array_ranges {
        for (addr, cell) in cells.range_mut(tl..=br) {
            if addr.col >= tl.col
                && addr.col <= br.col
                && *addr != anchor
                && cell.formula_kind.is_none()
            {
                cell.formula_kind = Some(FormulaKind::ArrayMember {
                    anchor: anchor.to_string(),
                });
            }
        }
    }
    Ok(cells)
}

fn start_cell(e: &BytesStart<'_>, row_num: u32, next_col: &mut u32) -> PendingCell {
    let addr = attr(e, "r").and_then(|r| Addr::parse(&r)).or(Some(Addr {
        row: row_num.max(1),
        col: *next_col,
    }));
    if let Some(a) = addr {
        *next_col = a.col + 1;
    }
    PendingCell {
        addr,
        t: attr(e, "t"),
        ..PendingCell::default()
    }
}

fn push_field(cur: &mut PendingCell, field: Field, text: &str) {
    match field {
        Field::F => cur.f_text.push_str(text),
        Field::V => cur.v_text.get_or_insert_with(String::new).push_str(text),
        Field::IsT => cur.is_text.get_or_insert_with(String::new).push_str(text),
        Field::None => {}
    }
}

fn finish_cell(
    c: PendingCell,
    shared: &[String],
    masters: &mut HashMap<u32, String>,
    arrays: &mut Vec<(Addr, Addr, Addr)>,
    out: &mut BTreeMap<Addr, CellSnapshot>,
) {
    let Some(addr) = c.addr else { return };
    let (formula, formula_kind) = match c.f_attrs {
        None => (None, None),
        Some((ft, si, rf)) => {
            let text = (!c.f_text.is_empty()).then(|| c.f_text.clone());
            match ft.as_deref() {
                Some("shared") => {
                    let si: u32 = si.and_then(|s| s.parse().ok()).unwrap_or(u32::MAX);
                    if let Some(t) = &text {
                        masters.insert(si, t.clone());
                    }
                    let text = text.or_else(|| masters.get(&si).cloned());
                    (text, Some(FormulaKind::Shared { si }))
                }
                Some("array") => {
                    let range = rf.unwrap_or_else(|| addr.to_string());
                    if let Some((tl, br)) = parse_range(&range) {
                        arrays.push((tl, br, addr));
                    }
                    (text, Some(FormulaKind::Array { range }))
                }
                // `dataTable` and plain formulas
                _ => (text, Some(FormulaKind::Normal)),
            }
        }
    };
    let value = decode_value(
        c.t.as_deref(),
        c.v_text.as_deref(),
        c.is_text.as_deref(),
        shared,
    );
    if formula_kind.is_none() && value == Value::Empty {
        return; // styled-but-empty cell: irrelevant for scoring
    }
    out.insert(
        addr,
        CellSnapshot {
            formula,
            formula_kind,
            value,
        },
    );
}

fn decode_value(
    t: Option<&str>,
    v: Option<&str>,
    inline: Option<&str>,
    shared: &[String],
) -> Value {
    match (t.unwrap_or("n"), v) {
        ("inlineStr", _) => Value::Text(decode_xlsx_escapes(inline.unwrap_or_default())),
        ("s", Some(v)) => v
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|i| shared.get(i))
            .map_or(Value::Empty, |s| Value::Text(s.clone())),
        ("str", v) => Value::Text(decode_xlsx_escapes(v.unwrap_or_default())),
        ("b", Some(v)) => Value::Bool(v.trim() == "1" || v.trim().eq_ignore_ascii_case("true")),
        ("e", Some(v)) => Value::Error(v.trim().to_string()),
        (_, Some(v)) if !v.trim().is_empty() => v
            .trim()
            .parse::<f64>()
            .map_or_else(|_| Value::Text(v.to_string()), Value::Number),
        _ => Value::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHEET: &str = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1"><v>1.5</v></c><c r="B1" t="s"><v>0</v></c><c r="C1" t="b"><v>1</v></c></row>
<row r="2"><c r="A2"><f>A1*2</f><v>3</v></c><c r="B2" t="str"><f>B1&amp;"!"</f><v>hi!</v></c><c r="C2" t="e"><f>1/0</f><v>#DIV/0!</v></c></row>
<row r="3"><c r="A3"><f t="shared" ref="A3:A4" si="0">A2+1</f><v>4</v></c><c r="B3"><f t="array" ref="B3:B4">SEQUENCE(2)</f><v>1</v></c></row>
<row r="4"><c r="A4"><f t="shared" si="0"/><v>5</v></c><c r="B4"><v>2</v></c><c r="C4" s="3"/></row>
</sheetData></worksheet>"#;

    #[test]
    fn parses_values_formulas_shared_and_arrays() {
        let cells = parse_sheet(SHEET, "sheet1.xml", &["hi".to_string()]).unwrap_or_default();
        let get = |s: &str| Addr::parse(s).and_then(|a| cells.get(&a)).cloned();
        assert_eq!(get("A1").map(|c| c.value), Some(Value::Number(1.5)));
        assert_eq!(get("B1").map(|c| c.value), Some(Value::Text("hi".into())));
        assert_eq!(get("C1").map(|c| c.value), Some(Value::Bool(true)));
        let a2 = get("A2");
        assert_eq!(
            a2.as_ref().and_then(|c| c.formula.clone()).as_deref(),
            Some("A1*2")
        );
        assert_eq!(get("B2").map(|c| c.value), Some(Value::Text("hi!".into())));
        assert_eq!(
            get("C2").map(|c| c.value),
            Some(Value::Error("#DIV/0!".into()))
        );
        assert_eq!(
            get("A4").and_then(|c| c.formula).as_deref(),
            Some("A2+1"),
            "shared member inherits master text"
        );
        assert_eq!(
            get("B4").and_then(|c| c.formula_kind),
            Some(FormulaKind::ArrayMember {
                anchor: "B3".into()
            })
        );
        assert!(get("C4").is_none(), "styled empty cells are dropped");
    }

    #[test]
    fn decodes_xlsx_control_char_escapes_in_text_values() {
        // CHAR.xlsx stores CHAR(1) as `_x0001_`.
        let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="str"><f>CHAR(1)</f><v>_x0001_</v></c></row></sheetData></worksheet>"#;
        let cells = parse_sheet(sheet, "sheet1.xml", &[]).unwrap_or_default();
        let get = |s: &str| {
            Addr::parse(s)
                .and_then(|a| cells.get(&a))
                .map(|c| c.value.clone())
        };
        assert_eq!(get("A1"), Some(Value::Text("\u{1}".into())));
    }
}
