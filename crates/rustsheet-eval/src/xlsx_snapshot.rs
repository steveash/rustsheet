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

mod tables;
use tables::read_sheet_tables;
pub use tables::TableDef;

use crate::cellref::{parse_range, Addr};
use crate::value::Value;
use rustsheet_engine::expressions::utils::shift::shift_formula_text;
use rustsheet_engine::types::DateSystem;
use rustsheet_xlsx::import::{
    date_system_from_attribute, decode_xlsx_escapes, iso8601_to_serial, rich_errors, ImportLimits,
    PartNames,
};

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
    /// Formula text without leading `=`; for shared-formula members, the master's text shifted to the member's cell.
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

    /// For an array-member cell of `sheet`, the formula of its array anchor (T-1877).
    #[must_use]
    pub fn anchor_formula(&self, sheets: &[SheetSnapshot], sheet: &str) -> Option<String> {
        let Some(FormulaKind::ArrayMember { anchor }) = &self.formula_kind else {
            return None;
        };
        sheets
            .iter()
            .find(|s| s.name == sheet)?
            .cells
            .get(&Addr::parse(anchor)?)?
            .formula
            .clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSnapshot {
    pub name: String,
    #[serde(with = "crate::cellref::a1_map")]
    pub cells: BTreeMap<Addr, CellSnapshot>,
}

/// A `<definedName>` of `xl/workbook.xml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinedName {
    /// The name as written in the workbook (matched case-insensitively).
    pub name: String,
    /// Name of the sheet the name is scoped to (`localSheetId`); `None` for a global name.
    pub sheet: Option<String>,
    /// The name's formula text, without `=`.
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkbookSnapshot {
    /// `docProps/app.xml` `<Application>` (e.g. "Microsoft Excel").
    pub application: Option<String>,
    pub app_version: Option<String>,
    /// `true` when the workbook uses the 1904 date system.
    pub date1904: bool,
    pub sheets: Vec<SheetSnapshot>,
    /// Defined names; absent in snapshots written before T-0045.
    #[serde(default)]
    pub defined_names: Vec<DefinedName>,
    /// Tables; absent in snapshots written before T-0434.
    #[serde(default)]
    pub tables: Vec<TableDef>,
}

impl WorkbookSnapshot {
    /// Every stored cell value (formula results and constants), keyed by (sheet, address).
    #[must_use]
    pub fn values(&self) -> BTreeMap<(String, Addr), Value> {
        let mut map = BTreeMap::new();
        for sheet in &self.sheets {
            for (addr, cell) in &sheet.cells {
                map.insert((sheet.name.clone(), *addr), cell.value.clone());
            }
        }
        map
    }

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

/// The package, with its parts found the way the importer finds them (case-insensitive names,
/// backslash separators; see `PartNames`).
struct Zip {
    archive: zip::ZipArchive<Cursor<Vec<u8>>>,
    names: PartNames,
}

impl Zip {
    fn new(bytes: &[u8]) -> Result<Self, SnapshotError> {
        let archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec()))?;
        let names = PartNames::from_archive(&archive)?;
        Ok(Zip { archive, names })
    }
}

/// Zip-bomb guard (CLAUDE.md directive 5): one part may not inflate past the
/// public API's default whole-package limit, so the harness reads what the API would.
fn read_part(zip: &mut Zip, name: &str) -> Result<Option<String>, SnapshotError> {
    read_part_capped(
        zip,
        name,
        rustsheet::LoadOptions::default().max_uncompressed_bytes,
    )
}

fn read_part_capped(
    zip: &mut Zip,
    name: &str,
    limit: u64,
) -> Result<Option<String>, SnapshotError> {
    let Some(entry) = zip.names.entry(name) else {
        return Ok(None);
    };
    let mut file = match zip.archive.by_name(entry) {
        Ok(f) => f,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut s = String::new();
    (&mut file).take(limit + 1).read_to_string(&mut s)?;
    if s.len() as u64 > limit {
        return Err(SnapshotError::Malformed(format!(
            "part {name} exceeds the {limit}-byte decompressed size limit"
        )));
    }
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
    let mut zip = Zip::new(bytes)?;
    let (application, app_version) = read_app(&mut zip)?;
    let shared = read_shared_strings(&mut zip)?;
    let rich_errors = read_rich_errors(&mut zip)?;
    let rels = read_workbook_rels(&mut zip)?;
    let workbook = read_part(&mut zip, "xl/workbook.xml")?
        .ok_or_else(|| SnapshotError::Malformed("missing xl/workbook.xml".into()))?;
    let (sheet_list, date1904, defined_names) = parse_workbook(&workbook)?;
    let sheet_names: Vec<String> = sheet_list.iter().map(|(n, _)| n.clone()).collect();
    let defined_names = defined_names
        .into_iter()
        .map(|(name, local, body)| DefinedName {
            name,
            sheet: local.and_then(|i| sheet_names.get(i).cloned()),
            body,
        })
        .collect();
    let mut sheets = Vec::new();
    let mut tables = Vec::new();
    for (name, rid) in sheet_list {
        let Some(target) = rels.get(&rid) else {
            continue;
        };
        let path = normalize_target(target);
        let Some(xml) = read_part(&mut zip, &path)? else {
            continue; // chartsheets / missing parts are not scored
        };
        let cells = parse_sheet(&xml, &path, &shared, &rich_errors)?;
        read_sheet_tables(&mut zip, &path, &name, &mut tables)?;
        sheets.push(SheetSnapshot { name, cells });
    }
    Ok(WorkbookSnapshot {
        application,
        app_version,
        date1904,
        sheets,
        defined_names,
        tables,
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
                    if let Err(e) = ImportLimits::default().check_escaped_text(&cur) {
                        return Err(SnapshotError::Malformed(format!("shared string: {e}")));
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

/// The error each value-metadata entry (`vm`, 1-based) stands for, if it is a rich-value error;
/// decoded by the importer's own reader (T-0921) so the two never disagree.
fn read_rich_errors(zip: &mut Zip) -> Result<Vec<Option<&'static str>>, SnapshotError> {
    let (Some(meta), Some(rich)) = (
        read_part(zip, rich_errors::METADATA_PART)?,
        read_part(zip, rich_errors::RICH_VALUE_PART)?,
    ) else {
        return Ok(Vec::new());
    };
    let structures = read_part(zip, rich_errors::RICH_VALUE_STRUCTURE_PART)?;
    rich_errors::value_metadata_errors(&meta, &rich, structures.as_deref())
        .map_err(|e| xml_err("rich value metadata", e))
}

/// Stands in for a rich-value error whose `errorType` has no evidence-backed mapping yet
/// ([`rich_errors::rich_error_name`]), or a `vm` that resolves to no error. Cells
/// holding it are never scored (runner) or written as goldens (`rustsheet-oracle fill`).
pub const UNDECODED_RICH_ERROR: &str = "#UNDECODED!";

/// Whether `v` is [`UNDECODED_RICH_ERROR`] (an expectation nobody can score yet).
#[must_use]
pub fn is_undecoded(v: &crate::value::Value) -> bool {
    matches!(v, crate::value::Value::Error(e) if e == UNDECODED_RICH_ERROR)
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

/// `(name, localSheetId, body)` per `<definedName>`.
type RawName = (String, Option<usize>, String);

fn raw_name(e: &BytesStart<'_>) -> Option<RawName> {
    let local = attr(e, "localSheetId").and_then(|v| v.parse().ok());
    Some((attr(e, "name")?, local, String::new()))
}

/// `(sheet name, relationship id)` per `<sheet>`.
type SheetRef = (String, String);

fn parse_workbook(xml: &str) -> Result<(Vec<SheetRef>, bool, Vec<RawName>), SnapshotError> {
    let mut reader = Reader::from_str(xml);
    let mut sheets = Vec::new();
    let mut names: Vec<RawName> = Vec::new();
    let mut date1904 = false;
    let mut in_name = false;
    loop {
        match reader
            .read_event()
            .map_err(|e| xml_err("workbook.xml", e))?
        {
            Event::Start(e) if e.local_name().as_ref() == "definedName" => {
                names.extend(raw_name(&e));
                in_name = true;
            }
            // A self-closing name has no body text and no End event.
            Event::Empty(e) if e.local_name().as_ref() == "definedName" => {
                names.extend(raw_name(&e));
            }
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                "sheet" => {
                    if let (Some(name), Some(rid)) = (attr(&e, "name"), attr(&e, "id")) {
                        sheets.push((name, rid));
                    }
                }
                "workbookPr" => {
                    date1904 = date_system_from_attribute(attr(&e, "date1904").as_deref())
                        == DateSystem::Excel1904;
                }
                _ => {}
            },
            Event::Text(t) if in_name => {
                if let Some(last) = names.last_mut() {
                    last.2.push_str(&t.xml_content(XmlVersion::Implicit1_0));
                }
            }
            Event::GeneralRef(r) if in_name => {
                if let Some(last) = names.last_mut() {
                    last.2.push_str(&resolve_entity(&r));
                }
            }
            Event::End(e) if e.local_name().as_ref() == "definedName" => in_name = false,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok((sheets, date1904, names))
}

#[derive(Default)]
struct PendingCell {
    addr: Option<Addr>,
    t: Option<String>,
    /// The `vm` attribute: 1-based index into the workbook's value metadata.
    vm: Option<usize>,
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
    rich_errors: &[Option<&'static str>],
) -> Result<BTreeMap<Addr, CellSnapshot>, SnapshotError> {
    let mut reader = Reader::from_str(xml);
    let mut cells = BTreeMap::new();
    let mut shared_masters: HashMap<u32, (Addr, String)> = HashMap::new();
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
                        rich_errors,
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
                        rich_errors,
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
    // Shared-formula members show the master's formula as it applies to their own cell.
    for (addr, cell) in &mut cells {
        if let Some(FormulaKind::Shared { si }) = cell.formula_kind {
            if cell.formula.is_none() {
                if let Some((master, text)) = shared_masters.get(&si) {
                    cell.formula = Some(shifted_member_formula(text, *master, *addr));
                }
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
        vm: attr(e, "vm").and_then(|v| v.parse().ok()),
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

/// The master's formula text moved from `master` to `member`, spacing preserved. Falls back to
/// the master's text when it does not lex (nothing better is known).
fn shifted_member_formula(text: &str, master: Addr, member: Addr) -> String {
    let delta = |to: u32, from: u32| i32::try_from(i64::from(to) - i64::from(from)).ok();
    delta(member.row, master.row)
        .zip(delta(member.col, master.col))
        .and_then(|(dr, dc)| shift_formula_text(text, dr, dc))
        .unwrap_or_else(|| text.to_string())
}

fn finish_cell(
    c: PendingCell,
    shared: &[String],
    rich_errors: &[Option<&'static str>],
    masters: &mut HashMap<u32, (Addr, String)>,
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
                        masters.insert(si, (addr, t.clone()));
                    }
                    // A member's text is derived from its master in `parse_sheet`, which also
                    // handles members that precede their master.
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
    let mut value = decode_value(
        c.t.as_deref(),
        c.v_text.as_deref(),
        c.is_text.as_deref(),
        shared,
    );
    // Excel stores rich errors (`#CALC!`, `#SPILL!`, ...) as `#VALUE!` plus a rich-value `vm`;
    // recover the real error. A `vm` that cannot be resolved (no such entry, no errorType key,
    // a missing field) is undecoded, never the cached `#VALUE!` (T-0203 review).
    if let (Value::Error(e), Some(vm)) = (&value, c.vm) {
        if e == "#VALUE!" {
            let real = rich_errors
                .get(vm.wrapping_sub(1))
                .copied()
                .flatten()
                .unwrap_or(UNDECODED_RICH_ERROR);
            value = Value::Error(real.to_string());
        }
    }
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
        ("d", Some(v)) => {
            iso8601_to_serial(v).map_or_else(|| Value::Error("#VALUE!".to_string()), Value::Number)
        }
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
    use std::io::Write;

    fn zip_with(name: &str, body: &[u8]) -> Zip {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        w.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        w.write_all(body).unwrap();
        let bytes = w.finish().unwrap().into_inner();
        Zip::new(&bytes).unwrap()
    }

    #[test]
    fn part_over_the_size_limit_is_malformed() {
        let mut zip = zip_with("xl/big.xml", &[b'a'; 100]);
        assert_eq!(
            read_part_capped(&mut zip, "xl/big.xml", 100)
                .unwrap()
                .unwrap()
                .len(),
            100
        );
        let err = read_part_capped(&mut zip, "xl/big.xml", 99).unwrap_err();
        assert!(matches!(&err, SnapshotError::Malformed(m) if m.contains("xl/big.xml")));
    }

    #[test]
    fn parts_are_found_case_insensitively_across_backslashes() {
        let mut zip = zip_with("xl\\sharedstrings.xml", b"s");
        assert_eq!(
            read_part(&mut zip, "xl/sharedStrings.xml")
                .unwrap()
                .as_deref(),
            Some("s")
        );
        assert_eq!(read_part(&mut zip, "xl/styles.xml").unwrap(), None);
    }

    const SHEET: &str = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1"><v>1.5</v></c><c r="B1" t="s"><v>0</v></c><c r="C1" t="b"><v>1</v></c></row>
<row r="2"><c r="A2"><f>A1*2</f><v>3</v></c><c r="B2" t="str"><f>B1&amp;"!"</f><v>hi!</v></c><c r="C2" t="e"><f>1/0</f><v>#DIV/0!</v></c></row>
<row r="3"><c r="A3"><f t="shared" ref="A3:A4" si="0">A2+1</f><v>4</v></c><c r="B3"><f t="array" ref="B3:B4">SEQUENCE(2)</f><v>1</v></c></row>
<row r="4"><c r="A4"><f t="shared" si="0"/><v>5</v></c><c r="B4"><v>2</v></c><c r="C4" s="3"/></row>
</sheetData></worksheet>"#;

    #[test]
    fn shared_member_before_master_is_shifted_with_absolute_refs() {
        let xml = r#"<worksheet><sheetData>
<row r="2"><c r="A2"><f t="shared" si="3"/><v>0</v></c></row>
<row r="1"><c r="A1"><f t="shared" ref="A1:A2" si="3">$B1+C$1+ SUM(D1:E2)</f><v>0</v></c></row>
</sheetData></worksheet>"#;
        let cells = parse_sheet(xml, "s.xml", &[], &[]).unwrap_or_default();
        let a2 = Addr::parse("A2")
            .and_then(|a| cells.get(&a))
            .and_then(|c| c.formula.clone());
        assert_eq!(a2.as_deref(), Some("$B2+C$1+ SUM(D2:E3)"));
    }

    #[test]
    fn parses_values_formulas_shared_and_arrays() {
        let cells = parse_sheet(SHEET, "sheet1.xml", &["hi".to_string()], &[]).unwrap_or_default();
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
            Some("A3+1"),
            "shared member shows the master's formula shifted to its own cell"
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
    fn rich_value_errors_replace_the_cached_value_error() {
        let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="e" vm="1"><f>1</f><v>#VALUE!</v></c><c r="B1" t="e" vm="2"><f>1</f><v>#VALUE!</v></c><c r="C1" t="e"><f>1</f><v>#VALUE!</v></c></row></sheetData></worksheet>"#;
        // errorType 13 = #CALC!, 8 = #SPILL! (ERROR.TYPE.xlsx)
        let rich = [Some("#CALC!"), Some("#SPILL!")];
        let cells = parse_sheet(sheet, "sheet1.xml", &[], &rich).unwrap_or_default();
        let get = |s: &str| {
            Addr::parse(s)
                .and_then(|a| cells.get(&a))
                .map(|c| c.value.clone())
        };
        assert_eq!(get("A1"), Some(Value::Error("#CALC!".into())));
        assert_eq!(get("B1"), Some(Value::Error("#SPILL!".into())));
        assert_eq!(get("C1"), Some(Value::Error("#VALUE!".into())));
    }

    /// A `vm` that resolves to no error (out of range, no errorType) is undecoded, not `#VALUE!`.
    #[test]
    fn unresolved_rich_value_error_is_undecoded() {
        let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="e" vm="3"><f>1</f><v>#VALUE!</v></c><c r="B1" t="e" vm="1"><f>1</f><v>#VALUE!</v></c><c r="C1" t="e" vm="1"><f>1</f><v>#N/A</v></c></row></sheetData></worksheet>"#;
        let rich = [None, Some("#CALC!")];
        let cells = parse_sheet(sheet, "sheet1.xml", &[], &rich).unwrap_or_default();
        let get = |s: &str| {
            Addr::parse(s)
                .and_then(|a| cells.get(&a))
                .map(|c| c.value.clone())
        };
        assert_eq!(get("A1"), Some(Value::Error(UNDECODED_RICH_ERROR.into())));
        assert_eq!(get("B1"), Some(Value::Error(UNDECODED_RICH_ERROR.into())));
        assert_eq!(get("C1"), Some(Value::Error("#N/A".into())));
    }

    #[test]
    fn decodes_xlsx_control_char_escapes_in_text_values() {
        // CHAR.xlsx stores CHAR(1) as `_x0001_`.
        let sheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="str"><f>CHAR(1)</f><v>_x0001_</v></c></row></sheetData></worksheet>"#;
        let cells = parse_sheet(sheet, "sheet1.xml", &[], &[]).unwrap_or_default();
        let get = |s: &str| {
            Addr::parse(s)
                .and_then(|a| cells.get(&a))
                .map(|c| c.value.clone())
        };
        assert_eq!(get("A1"), Some(Value::Text("\u{1}".into())));
    }
}

#[cfg(test)]
mod defined_name_tests {
    use super::*;

    #[test]
    fn reads_scoped_empty_and_escaped_defined_names() {
        let xml = r#"<workbook><sheets><sheet name="S" r:id="rId1"/></sheets><definedNames>
            <definedName name="G">S!$A$1+1&lt;2</definedName>
            <definedName name="E"/>
            <definedName name="L" localSheetId="0">TODAY()</definedName>
            </definedNames></workbook>"#;
        let (_, _, names) = parse_workbook(xml).expect("parses");
        assert_eq!(
            names,
            [
                ("G".to_string(), None, "S!$A$1+1<2".to_string()),
                ("E".to_string(), None, String::new()),
                ("L".to_string(), Some(0), "TODAY()".to_string()),
            ]
        );
    }
}

#[cfg(test)]
mod anchor_formula_tests {
    use super::*;

    fn cell(formula: Option<&str>, kind: Option<FormulaKind>) -> CellSnapshot {
        CellSnapshot {
            formula: formula.map(str::to_string),
            formula_kind: kind,
            value: Value::Empty,
        }
    }

    #[test]
    fn member_resolves_its_anchor_formula() {
        let anchor = Addr::parse("A1").expect("addr");
        let member = cell(
            None,
            Some(FormulaKind::ArrayMember {
                anchor: "A1".into(),
            }),
        );
        let sheets = [SheetSnapshot {
            name: "S".into(),
            cells: BTreeMap::from([(anchor, cell(Some("SUM(1)"), None))]),
        }];
        assert_eq!(
            member.anchor_formula(&sheets, "S").as_deref(),
            Some("SUM(1)")
        );
        assert_eq!(member.anchor_formula(&sheets, "Other"), None);
        assert_eq!(cell(None, None).anchor_formula(&sheets, "S"), None);
    }
}
