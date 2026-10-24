//! Pivot caches and pivot tables (SPEC-0015 "xlsx import", T-2082).
//!
//! Every part is streamed with a pull parser, never built into a tree: a records part can hold
//! millions of values, and [`CacheBuilder`] counts the SPEC-0015 limits before each push, so
//! nothing past them is allocated (CLAUDE.md directive 5). Anything the model cannot hold
//! faithfully (an OLAP or external source, a grouped or calculated field, a cache saved without
//! records, a bad index, a limit) skips that cache, with every pivot on it, or that pivot: it is
//! logged and dropped whole, never truncated or guessed, so GETPIVOTDATA on it is `#REF!` and the
//! rest of the workbook loads as usual (docs/excel-semantics/web.md "GETPIVOTDATA against a pivot
//! table").

use std::collections::HashMap;
use std::io::{BufRead, Read, Seek};

use quick_xml::events::{BytesStart, Event};
use rustsheet_engine::expressions::token::get_error_by_english_name;
use rustsheet_engine::expressions::utils::{is_valid_column_number, is_valid_row};
use rustsheet_engine::pivot::{
    Aggregate, Axis, AxisField, CacheBuilder, CacheItem, CacheSource, CellArea, DataField,
    PivotCache, PivotCacheError, PivotField, PivotTable, RecordValue, MAX_RECORD_VALUES,
};
use rustsheet_engine::types::DateSystem;

use super::iso_date::iso8601_to_date_system_serial;
use super::package::{part_folder_and_rels, part_key, resolve_relative_target, Package};
use super::worksheets::parse_range;
use super::xml::{find_start_attribute, start_attribute, XmlNode};
use super::{parse_relationships, Relationship};
use crate::error::XlsxError;

/// Most pivot fields on one axis of a pivot (SPEC-0015 "Untrusted-input limits").
const MAX_AXIS_FIELDS: usize = 256;
/// Most data fields of one pivot.
const MAX_DATA_FIELDS: usize = 256;
/// Most pivots on one sheet; the ones past it are dropped.
const MAX_PIVOTS_PER_SHEET: usize = 64;
/// Most pivots in a workbook; the ones past it are dropped. Every accepted pivot allocates a
/// hidden-items bitset per field sized to its cache, so the per-sheet cap alone leaves the total
/// to the file (T-2394, web.md "Pivot caches past the import limits are dropped").
const MAX_PIVOTS: usize = 1_000;
/// Deepest element nesting read in a pivot part: well-formed ones nest about six deep, and the
/// open-element stack must not grow with the file.
const MAX_DEPTH: usize = 32;

/// Why a cache or a pivot was skipped. It is logged, never an import error.
struct Skip(String);

impl From<XlsxError> for Skip {
    fn from(error: XlsxError) -> Self {
        Skip(error.to_string())
    }
}

impl From<PivotCacheError> for Skip {
    fn from(error: PivotCacheError) -> Self {
        Skip(error.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for Skip {
    fn from(error: quick_xml::events::attributes::AttrError) -> Self {
        Skip(error.to_string())
    }
}

fn skip<T>(reason: impl Into<String>) -> Result<T, Skip> {
    Err(Skip(reason.into()))
}

/// A worksheet pivots may sit on.
pub(super) struct PivotSheet {
    /// The sheet's stable `sheetId`.
    pub(super) sheet_id: u32,
    /// The worksheet part (`xl/worksheets/sheet1.xml`).
    pub(super) path: String,
}

/// The pivot caches listed in the workbook (`caches`: each `<pivotCache>`'s `cacheId` and
/// `r:id`, resolved through the workbook relationships `rels`) and the pivot tables of `sheets`.
pub(super) fn load_pivots<R: Read + Seek>(
    archive: &mut Package<R>,
    rels: &HashMap<String, Relationship>,
    caches: &[(Option<String>, Option<String>)],
    sheets: &[PivotSheet],
    date_system: DateSystem,
) -> (Vec<PivotCache>, Vec<PivotTable>) {
    let (pivot_caches, cache_ids) =
        load_caches(archive, rels, caches, MAX_RECORD_VALUES, date_system);
    if pivot_caches.is_empty() {
        return (pivot_caches, Vec::new());
    }
    let mut parts = Vec::new();
    for sheet in sheets {
        match pivot_table_parts(archive, &sheet.path) {
            Ok(found) => {
                // One log line per sheet for the pivots dropped, not one per pivot: a file can
                // list millions.
                let mut dropped = 0_usize;
                for (n, part) in found.into_iter().enumerate() {
                    if n < MAX_PIVOTS_PER_SHEET && parts.len() < MAX_PIVOTS {
                        parts.push((sheet.sheet_id, part));
                    } else {
                        dropped += 1;
                    }
                }
                if dropped > 0 {
                    log_skip(
                        "pivot tables of",
                        &sheet.path,
                        &Skip(format!(
                            "{dropped} dropped past the limits ({MAX_PIVOTS_PER_SHEET} on a sheet, {MAX_PIVOTS} in a workbook)"
                        )),
                    );
                }
            }
            Err(reason) => log_skip("pivot tables of", &sheet.path, &reason),
        }
    }
    // Part names match case-insensitively with `\` as `/` (`part_key`), so count on that key.
    let keys: Vec<String> = parts.iter().map(|(_, part)| part_key(part)).collect();
    let reached = occurrences(keys.iter());
    let mut pivot_tables = Vec::new();
    for ((sheet_id, part), key) in parts.iter().zip(&keys) {
        let result = if reached.get(key).is_some_and(|&n| n > 1) {
            skip("the part is reached twice")
        } else {
            archive
                .open(part)
                .map_err(Skip::from)
                .and_then(|reader| read_pivot_table(reader, *sheet_id, &pivot_caches, &cache_ids))
        };
        match result {
            Ok(pivot) => pivot_tables.push(pivot),
            Err(reason) => log_skip("pivot table", part, &reason),
        }
    }
    (pivot_caches, pivot_tables)
}

/// How many times each value occurs (a file can list millions, so never a pairwise scan).
fn occurrences<'a, T: Eq + std::hash::Hash + 'a>(
    values: impl Iterator<Item = &'a T>,
) -> HashMap<&'a T, usize> {
    let mut counts = HashMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

/// Logs a skipped cache or pivot to stderr, as the theme importer does: the importer has no
/// warnings channel. Part names and reasons come from the file, so they are quoted (no raw
/// newlines or escapes) and cut short.
fn log_skip(what: &str, part: &str, reason: &Skip) {
    const MAX_CHARS: usize = 200;
    let part: String = part.chars().take(MAX_CHARS).collect();
    let reason: String = reason.0.chars().take(MAX_CHARS).collect();
    eprintln!("RustSheet: skipped {what} {part:?}: {reason:?}");
}

/// The caches that load, in workbook order, and each one's index by `cacheId`. A `cacheId`
/// listed twice and a definition part reached twice skip every cache involved. `values_left` is
/// the workbook's record-value budget ([`MAX_RECORD_VALUES`]; smaller in tests).
fn load_caches<R: Read + Seek>(
    archive: &mut Package<R>,
    rels: &HashMap<String, Relationship>,
    caches: &[(Option<String>, Option<String>)],
    mut values_left: usize,
    date_system: DateSystem,
) -> (Vec<PivotCache>, HashMap<u32, u32>) {
    let listed: Vec<(Option<u32>, Option<String>)> = caches
        .iter()
        .map(|(id, rel_id)| {
            let id = id.as_deref().and_then(|id| id.parse::<u32>().ok());
            let part = rel_id
                .as_deref()
                .and_then(|rel_id| rels.get(rel_id))
                .filter(|rel| !rel.external && rel.rel_type.ends_with("/pivotCacheDefinition"))
                .and_then(|rel| resolve_relative_target("xl", &rel.target));
            (id, part)
        })
        .collect();
    let id_counts = occurrences(listed.iter().filter_map(|(id, _)| id.as_ref()));
    let part_keys: Vec<Option<String>> = listed
        .iter()
        .map(|(_, part)| part.as_deref().map(part_key))
        .collect();
    let part_counts = occurrences(part_keys.iter().flatten());
    let mut pivot_caches = Vec::new();
    let mut ids = HashMap::new();
    for ((id, part), key) in listed.iter().zip(&part_keys) {
        let name = part.as_deref().unwrap_or("(no part)");
        let result = match (id, part) {
            (None, _) => skip("no valid cacheId"),
            (_, None) => skip("no internal pivotCacheDefinition relationship"),
            (Some(id), _) if id_counts.get(id).is_some_and(|&n| n > 1) => {
                skip(format!("cacheId {id} is listed twice"))
            }
            (_, Some(_))
                if key
                    .as_ref()
                    .is_some_and(|key| part_counts.get(key).is_some_and(|&n| n > 1)) =>
            {
                skip("the part is reached twice")
            }
            (Some(id), Some(part)) => {
                load_cache(archive, part, values_left, date_system).map(|c| (*id, c))
            }
        };
        match result {
            Ok((id, cache)) => {
                values_left =
                    values_left.saturating_sub(cache.record_count as usize * cache.fields.len());
                // At most one cache per listed `<pivotCache>`, bounded by the byte budget.
                ids.insert(id, pivot_caches.len() as u32);
                pivot_caches.push(cache);
            }
            Err(reason) => log_skip("pivot cache", name, &reason),
        }
    }
    (pivot_caches, ids)
}

/// One cache: its definition part, then the records part the definition's `r:id` points to.
fn load_cache<R: Read + Seek>(
    archive: &mut Package<R>,
    part: &str,
    values_left: usize,
    date_system: DateSystem,
) -> Result<PivotCache, Skip> {
    let limits = *archive.limits();
    let (mut builder, field_count, records_id) =
        read_definition(archive.open(part)?, values_left, &limits, date_system)?;
    let Some(records_id) = records_id else {
        return skip("no records part (saveData=\"0\")");
    };
    let (folder, rels_path) = part_folder_and_rels(part);
    let records = part_relationships(archive, &rels_path)?
        .into_iter()
        .find(|(id, _)| *id == records_id)
        .map(|(_, rel)| rel)
        .filter(|rel| !rel.external && rel.rel_type.ends_with("/pivotCacheRecords"))
        .and_then(|rel| resolve_relative_target(folder, &rel.target));
    let Some(records) = records else {
        return skip("no internal pivotCacheRecords relationship");
    };
    read_records(
        archive.open(&records)?,
        &mut builder,
        field_count,
        &limits,
        date_system,
    )?;
    Ok(builder.finish())
}

/// The relationships of a part in document order; none when the part has no relationships part.
fn part_relationships<R: Read + Seek>(
    archive: &mut Package<R>,
    rels_path: &str,
) -> Result<Vec<(String, Relationship)>, XlsxError> {
    match archive.open_optional(rels_path)? {
        Some(file) => parse_relationships(&XmlNode::parse(file)?),
        None => Ok(Vec::new()),
    }
}

/// The pivot table parts of the worksheet part `sheet`, in relationship order.
fn pivot_table_parts<R: Read + Seek>(
    archive: &mut Package<R>,
    sheet: &str,
) -> Result<Vec<String>, Skip> {
    let (folder, rels_path) = part_folder_and_rels(sheet);
    let mut parts = Vec::new();
    for (_, rel) in part_relationships(archive, &rels_path)? {
        if !rel.rel_type.ends_with("/pivotTable") {
            continue;
        }
        match resolve_relative_target(folder, &rel.target).filter(|_| !rel.external) {
            Some(part) => parts.push(part),
            None => log_skip(
                "pivot table",
                &rel.target,
                &Skip("not inside the package".into()),
            ),
        }
    }
    Ok(parts)
}

/// An element start, or the end of the element named, as [`walk`] reports them.
enum Tag<'a> {
    Open(&'a BytesStart<'a>),
    Close(&'a str),
}

/// Streams the elements of a part to `visit`, with the local names of the enclosing elements,
/// outermost first (an empty element is an `Open` then a `Close`).
fn walk<B: BufRead>(
    part: B,
    mut visit: impl FnMut(&[String], Tag<'_>) -> Result<(), Skip>,
) -> Result<(), Skip> {
    let mut reader = quick_xml::Reader::from_reader(part);
    let mut buffer = Vec::new();
    let mut path: Vec<String> = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(XlsxError::from)?
        {
            Event::Start(element) => {
                visit(&path, Tag::Open(&element))?;
                if path.len() >= MAX_DEPTH {
                    return skip("elements nest too deep");
                }
                path.push(element.local_name().as_ref().to_string());
            }
            Event::Empty(element) => {
                visit(&path, Tag::Open(&element))?;
                visit(&path, Tag::Close(element.local_name().as_ref()))?;
            }
            Event::End(element) => {
                path.pop();
                visit(&path, Tag::Close(element.local_name().as_ref()))?;
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
        buffer.clear();
    }
}

/// Whether the enclosing elements are exactly `names`.
fn at(path: &[String], names: &[&str]) -> bool {
    path.len() == names.len() && path.iter().zip(names).all(|(p, n)| p == n)
}

fn local_name(element: &BytesStart) -> String {
    element.local_name().as_ref().to_string()
}

/// The attribute `name`.
fn attribute(element: &BytesStart, name: &str) -> Result<Option<String>, Skip> {
    Ok(start_attribute(element, name)?)
}

/// The relationship id: an `id` attribute with a prefix (`r:id`).
fn relationship_id(element: &BytesStart) -> Result<Option<String>, Skip> {
    Ok(find_start_attribute(element, |name| {
        name.prefix().is_some() && name.local_name().as_ref() == "id"
    })?)
}

fn index_attribute(element: &BytesStart, name: &str) -> Result<u32, Skip> {
    match attribute(element, name)?.map(|v| v.parse::<u32>()) {
        Some(Ok(index)) => Ok(index),
        _ => skip(format!("no valid {name} index")),
    }
}

/// The cache item an `<s>`, `<n>`, `<b>`, `<e>`, `<d>` or `<m>` element holds; `None` for any
/// other element.
fn cache_item(
    element: &BytesStart,
    limits: &super::package::ImportLimits,
    date_system: DateSystem,
) -> Result<Option<CacheItem>, Skip> {
    let tag = local_name(element);
    if tag == "m" {
        return Ok(Some(CacheItem::Missing));
    }
    if !matches!(tag.as_str(), "s" | "n" | "b" | "e" | "d") {
        return Ok(None);
    }
    let Some(v) = attribute(element, "v")? else {
        return skip("a cache value without v");
    };
    let item = match tag.as_str() {
        "s" => {
            limits.check_text(&v)?;
            CacheItem::Text(v)
        }
        "n" => match v.trim().parse::<f64>() {
            Ok(n) if n.is_finite() => CacheItem::Number(n),
            _ => return skip(format!("bad number {v:?}")),
        },
        "b" => match v.as_str() {
            "1" | "true" => CacheItem::Boolean(true),
            "0" | "false" => CacheItem::Boolean(false),
            _ => return skip(format!("bad boolean {v:?}")),
        },
        "e" => match get_error_by_english_name(&v) {
            Some(error) => CacheItem::Error(error),
            None => return skip(format!("unknown error {v:?}")),
        },
        // `<d>` is an ISO 8601 text; the cell serial it stands for depends on the workbook's date
        // system (1904 serials are 1462 days lower).
        _ => match iso8601_to_date_system_serial(&v, date_system) {
            Some(serial) => CacheItem::Date(serial),
            None => return skip(format!("bad date {v:?}")),
        },
    };
    Ok(Some(item))
}

const DEFINITION: &str = "pivotCacheDefinition";

/// A `pivotCacheDefinition` part: a builder holding its fields and shared items, the field count,
/// and the `r:id` of its records part.
fn read_definition<B: BufRead>(
    part: B,
    values_left: usize,
    limits: &super::package::ImportLimits,
    date_system: DateSystem,
) -> Result<(CacheBuilder, u32, Option<String>), Skip> {
    let mut records_id = None;
    let mut source: Option<Option<CacheSource>> = None;
    let mut builder: Option<CacheBuilder> = None;
    let mut field_count = 0;
    walk(part, |path, tag| {
        let Tag::Open(element) = tag else {
            return Ok(());
        };
        let name = local_name(element);
        if path.is_empty() {
            if name != DEFINITION {
                return skip("not a pivotCacheDefinition");
            }
            records_id = relationship_id(element)?;
        } else if at(path, &[DEFINITION]) {
            match name.as_str() {
                "cacheSource" => {
                    if attribute(element, "type")?.as_deref() != Some("worksheet") {
                        return skip("not a worksheet source");
                    }
                    source = Some(None);
                }
                "cacheHierarchies" => return skip("an OLAP cache"),
                "cacheFields" => {
                    let Some(source) = source.take() else {
                        return skip("no worksheet cacheSource before cacheFields");
                    };
                    builder = Some(CacheBuilder::new(source, values_left));
                }
                _ => {}
            }
        } else if at(path, &[DEFINITION, "cacheSource"]) && name == "worksheetSource" {
            // A named source (`name`: a defined name or a table) is kept as no source.
            if let (Some(sheet_name), Some(range)) =
                (attribute(element, "sheet")?, attribute(element, "ref")?)
            {
                if source == Some(None) {
                    source = Some(Some(CacheSource { sheet_name, range }));
                }
            }
        } else if at(path, &[DEFINITION, "cacheFields"]) && name == "cacheField" {
            let Some(builder) = builder.as_mut() else {
                return skip("a cacheField outside cacheFields");
            };
            if attribute(element, "formula")?.is_some()
                || attribute(element, "databaseField")?.as_deref() == Some("0")
            {
                return skip("a calculated or grouped field");
            }
            let Some(field_name) = attribute(element, "name")? else {
                return skip("a cacheField without a name");
            };
            limits.check_text(&field_name)?;
            builder.add_field(field_name)?;
            field_count += 1;
        } else if at(path, &[DEFINITION, "cacheFields", "cacheField"]) && name == "fieldGroup" {
            return skip("a grouped field");
        } else if at(
            path,
            &[DEFINITION, "cacheFields", "cacheField", "sharedItems"],
        ) {
            if let (Some(builder), Some(item)) =
                (builder.as_mut(), cache_item(element, limits, date_system)?)
            {
                builder.add_shared_item(field_count - 1, item)?;
            }
        }
        Ok(())
    })?;
    match builder {
        Some(builder) => Ok((builder, field_count, records_id)),
        None => skip("no cacheFields"),
    }
}

/// A `pivotCacheRecords` part, appended record by record to `builder`.
fn read_records<B: BufRead>(
    part: B,
    builder: &mut CacheBuilder,
    field_count: u32,
    limits: &super::package::ImportLimits,
    date_system: DateSystem,
) -> Result<(), Skip> {
    let mut record: Vec<RecordValue> = Vec::new();
    walk(part, |path, tag| {
        match tag {
            Tag::Open(element) if path.is_empty() => {
                if local_name(element) != "pivotCacheRecords" {
                    return skip("not a pivotCacheRecords");
                }
            }
            Tag::Open(element) if at(path, &["pivotCacheRecords", "r"]) => {
                // Checked before the push, so a record never grows past its fields.
                if record.len() >= field_count as usize {
                    return Err(PivotCacheError::BadRecord.into());
                }
                let value = if local_name(element) == "x" {
                    RecordValue::Shared(index_attribute(element, "v")?)
                } else if let Some(item) = cache_item(element, limits, date_system)? {
                    RecordValue::Inline(item)
                } else {
                    return skip("an unknown record value");
                };
                record.push(value);
            }
            Tag::Close("r") if at(path, &["pivotCacheRecords"]) => {
                builder.push_record(std::mem::take(&mut record))?;
            }
            _ => {}
        }
        Ok(())
    })
}

const TABLE: &str = "pivotTableDefinition";

/// A `pivotTableDefinition` part on the sheet `sheet_id`, against the loaded `caches`.
fn read_pivot_table<B: BufRead>(
    part: B,
    sheet_id: u32,
    caches: &[PivotCache],
    cache_ids: &HashMap<u32, u32>,
) -> Result<PivotTable, Skip> {
    let mut pivot_name = String::new();
    let mut cache_index = 0;
    let mut area = None;
    let mut fields: Vec<PivotField> = Vec::new();
    let mut row_fields = Vec::new();
    let mut column_fields = Vec::new();
    let mut data_fields = Vec::new();
    let mut cache: Option<&PivotCache> = None;
    let mut page_fields = 0;
    walk(part, |path, tag| {
        let Tag::Open(element) = tag else {
            return Ok(());
        };
        let name = local_name(element);
        if path.is_empty() {
            if name != TABLE {
                return skip("not a pivotTableDefinition");
            }
            pivot_name = attribute(element, "name")?.unwrap_or_default();
            let id = index_attribute(element, "cacheId")?;
            let Some(&index) = cache_ids.get(&id) else {
                return skip(format!("cacheId {id} has no loaded cache"));
            };
            cache_index = index;
            cache = caches.get(index as usize);
            return Ok(());
        }
        let Some(cache) = cache else {
            return skip("no pivotTableDefinition root");
        };
        let field_count = cache.fields.len();
        let field_index = |element: &BytesStart, attr: &str| -> Result<u32, Skip> {
            let index = index_attribute(element, attr)?;
            if index as usize >= field_count {
                return skip(format!("field index {index} is past the cache's fields"));
            }
            Ok(index)
        };
        if at(path, &[TABLE]) && name == "location" {
            area = Some(location(attribute(element, "ref")?.as_deref())?);
        } else if at(path, &[TABLE, "pivotFields"]) && name == "pivotField" {
            let Some(cache_field) = cache.fields.get(fields.len()) else {
                return skip("more pivotFields than cache fields");
            };
            let axis = match attribute(element, "axis")?.as_deref() {
                Some("axisRow") => Some(Axis::Row),
                Some("axisCol") => Some(Axis::Column),
                Some("axisPage") => Some(Axis::Page),
                Some("axisValues") | None => None,
                Some(other) => return skip(format!("unknown axis {other:?}")),
            };
            fields.push(PivotField::new(axis, cache_field.items.len()));
        } else if at(path, &[TABLE, "pivotFields", "pivotField", "items"]) && name == "item" {
            // Subtotal rows (`t="default"`, `t="sum"`, ...) name no cache item.
            if attribute(element, "t")?.is_some_and(|t| t != "data") {
                return Ok(());
            }
            let current = fields.len().saturating_sub(1);
            let (Some(field), Some(cache_field)) = (fields.last_mut(), cache.fields.get(current))
            else {
                return Ok(());
            };
            let item = index_attribute(element, "x")?;
            if item as usize >= cache_field.items.len() {
                return skip(format!("item index {item} is past the field's items"));
            }
            if matches!(attribute(element, "h")?.as_deref(), Some("1" | "true")) {
                field.hide(item);
            }
        } else if at(path, &[TABLE, "rowFields"]) || at(path, &[TABLE, "colFields"]) {
            if name != "field" {
                return Ok(());
            }
            let axis_field = match attribute(element, "x")?.as_deref() {
                Some("-2") => AxisField::Values,
                _ => AxisField::Field(field_index(element, "x")?),
            };
            let axis_fields = if path.get(1).is_some_and(|p| p == "rowFields") {
                &mut row_fields
            } else {
                &mut column_fields
            };
            if axis_fields.len() >= MAX_AXIS_FIELDS {
                return skip("too many fields on an axis");
            }
            axis_fields.push(axis_field);
        } else if at(path, &[TABLE, "pageFields"]) && name == "pageField" {
            field_index(element, "fld")?;
            page_fields += 1;
            if page_fields > MAX_AXIS_FIELDS {
                return skip("too many page fields");
            }
        } else if at(path, &[TABLE, "dataFields"]) && name == "dataField" {
            if data_fields.len() >= MAX_DATA_FIELDS {
                return skip("too many data fields");
            }
            // "Show values as" (% of total, running total, ...) changes every value; not modeled.
            if attribute(element, "showDataAs")?.is_some_and(|s| s != "normal") {
                return skip("a data field shown as a calculation");
            }
            let field = field_index(element, "fld")?;
            let function = aggregate(attribute(element, "subtotal")?.as_deref())?;
            let caption = attribute(element, "name")?.unwrap_or_default();
            data_fields.push(DataField {
                caption,
                field,
                function,
            });
        }
        Ok(())
    })?;
    let Some(cache) = cache else {
        return skip("not a pivotTableDefinition");
    };
    let Some(location) = area else {
        return skip("no location");
    };
    if cache.fields.len() != fields.len() {
        return skip("pivotFields do not match the cache fields");
    }
    Ok(PivotTable {
        name: pivot_name,
        sheet_id,
        location,
        cache: cache_index,
        fields,
        row_fields,
        column_fields,
        data_fields,
    })
}

/// A `<location ref>` as a 0-based area, inside the sheet.
fn location(reference: Option<&str>) -> Result<CellArea, Skip> {
    let Some((top, left, bottom, right)) = reference.and_then(|r| parse_range(r).ok()) else {
        return skip("no valid location");
    };
    let valid = is_valid_row(top)
        && is_valid_row(bottom)
        && is_valid_column_number(left)
        && is_valid_column_number(right)
        && top <= bottom
        && left <= right;
    let to_index = |n: i32| u32::try_from(n - 1).ok();
    match (
        valid,
        to_index(top),
        to_index(left),
        to_index(bottom),
        to_index(right),
    ) {
        (true, Some(top), Some(left), Some(bottom), Some(right)) => Ok(CellArea {
            top,
            left,
            bottom,
            right,
        }),
        _ => skip("a location outside the sheet"),
    }
}

/// A `dataField@subtotal`; absent is `sum`.
fn aggregate(subtotal: Option<&str>) -> Result<Aggregate, Skip> {
    Ok(match subtotal.unwrap_or("sum") {
        "sum" => Aggregate::Sum,
        "count" => Aggregate::Count,
        "average" => Aggregate::Average,
        "max" => Aggregate::Max,
        "min" => Aggregate::Min,
        "product" => Aggregate::Product,
        "countNums" => Aggregate::CountNums,
        "stdDev" => Aggregate::StdDev,
        "stdDevp" => Aggregate::StdDevP,
        "var" => Aggregate::Var,
        "varp" => Aggregate::VarP,
        other => return skip(format!("unknown subtotal {other:?}")),
    })
}

#[cfg(test)]
mod tests;
