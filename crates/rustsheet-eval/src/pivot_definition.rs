//! A spec pivot's `definition` (case builder v2, T-4122) as the package parts the case builder
//! writes for it (`tools/src/rustsheet_tools/pivot_definition.py`), so the engine adapter loads it
//! through the xlsx importer and the engine renders it on its load-time refresh, as Excel renders
//! the built file on open (T-4152, SPEC-0032 rollout step 7).
//!
//! It mirrors the builder part for part, attribute for attribute: what Excel opened is what the
//! engine loads. The builder validates a definition (spec2xlsx refuses a bad one, and no case
//! reaches the oracle without it), so a definition it would refuse is only an error here. Only
//! the checks that bound the work or keep the XML well formed are repeated (a spec is untrusted
//! input, directive 5). Keep the two in step: a change to the builder's parts is a change here.
//! The golden parts in `tools/tests/pivot_parts_golden/` guard it both ways: this module's tests
//! and `tools/tests/test_pivot_parts_golden.py` each assert their builder writes them.

use std::collections::HashMap;
use std::fmt::Write as _;

use quick_xml::escape::escape;
use rustsheet_engine::pivot::{Aggregate, ShowDataAs, MAX_RECORD_VALUES};
use serde_json::{Map, Value as Json};

use crate::case::{split_qualified, CaseSpec, CellInput, PivotTableSpec, SheetSpec};
use crate::cellref::{parse_range, Addr};
use crate::value::Value;

const MAIN_NS: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
/// The Σ (data) field on an axis: `<field x="-2"/>`.
const VALUES: &str = "Values";
const VALUES_INDEX: i64 = -2;
/// pivotTableDefinition attributes written unless `table` sets them (the builder's
/// `_TABLE_DEFAULTS`, Excel's own new-pivot values), in its order, each `name=value`.
const TABLE_DEFAULTS: &str = "dataCaption=Values updatedVersion=6 minRefreshableVersion=3 createdVersion=6 applyNumberFormats=0 applyBorderFormats=0 applyFontFormats=0 applyPatternFormats=0 applyAlignmentFormats=0 applyWidthHeightFormats=1 useAutoFormatting=1 itemPrintTitles=1 indent=0 outline=1 outlineData=1";
/// `CT_pivotTableDefinition`'s children after `dataFields`, in schema order: the raw-XML slots
/// (`_SLOTS`), the first four before `pivotTableStyleInfo`.
const SLOTS: [&str; 6] = [
    "formats",
    "conditionalFormats",
    "chartFormats",
    "pivotHierarchies",
    "filters",
    "extLst",
];
/// Most numeric group bins the builder writes (`group_labels`).
const MAX_BINS: usize = 10_000;
/// Most entries of one definition list (`rows`, `columns`, `pages`, `data`, `calculated`,
/// `fields`, a field's `order`, `hidden`, `collapsed` or `subtotals`): the importer's cap on a
/// pivot's axis and data fields (SPEC-0015 "Untrusted-input limits"). A spec is untrusted input
/// too (directive 5), so every list is bounded before it is walked.
const MAX_LIST: usize = 256;
/// A field's custom subtotal functions (`SUBTOTALS`, the `<item t>` names).
const SUBTOTALS: [&str; 11] = [
    "sum", "countA", "avg", "max", "min", "product", "count", "stdDev", "stdDevP", "var", "varP",
];
/// The three parts of a defined pivot: its cache definition and records, and its table.
pub(crate) struct Parts {
    pub(crate) cache_definition: String,
    pub(crate) cache_records: String,
    pub(crate) table: String,
}

/// A value a source cell contributes to the builder's cache (`cached_value`); `None` is a blank
/// or a value the builder does not know (a formula: Excel computes the cache again on open).
#[derive(Clone, Debug, PartialEq)]
enum Cached {
    Text(String),
    Number(f64),
    Bool(bool),
    Error(String),
}

/// A cache item's identity in the builder's shared items (`_key`): texts that differ only in
/// case are one item.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Missing,
    Text(String),
    Number(u64),
    Bool(bool),
    Error(String),
}

fn key(value: Option<&Cached>) -> Key {
    match value {
        None => Key::Missing,
        // The builder's `fold`, Python's `str.lower`: `ß` stays apart from `ss`.
        Some(Cached::Text(text)) => Key::Text(text.to_lowercase()),
        Some(Cached::Number(n)) => Key::Number(n.to_bits()),
        Some(Cached::Bool(b)) => Key::Bool(*b),
        Some(Cached::Error(e)) => Key::Error(e.clone()),
    }
}

fn cached(input: Option<&CellInput>) -> Result<Option<Cached>, String> {
    if let Some(CellInput::Str(text)) = input {
        if text.is_empty() || text.starts_with('=') {
            return Ok(None);
        }
    }
    Ok(match input {
        None
        | Some(
            CellInput::Entered { .. }
            | CellInput::EnteredR1c1 { .. }
            | CellInput::Typed(Value::Empty),
        ) => None,
        Some(CellInput::Str(text) | CellInput::Typed(Value::Text(text))) => {
            Some(Cached::Text(text.clone()))
        }
        Some(CellInput::Number(n) | CellInput::Typed(Value::Number(n))) => {
            if !n.is_finite() {
                return Err(format!("pivot source value {n} is not finite"));
            }
            Some(Cached::Number(*n))
        }
        Some(CellInput::Bool(b) | CellInput::Typed(Value::Bool(b))) => Some(Cached::Bool(*b)),
        Some(CellInput::Typed(Value::Error(e))) => Some(Cached::Error(e.clone())),
        Some(CellInput::StoredError(e)) => Some(Cached::Error(e.v.clone())),
    })
}

/// [`cached`] of a value written in the definition (an `order`, `hidden` or page item).
fn cached_json(raw: &Json) -> Result<Option<Cached>, String> {
    if raw.is_null() {
        return Ok(None);
    }
    let input: CellInput =
        serde_json::from_value(raw.clone()).map_err(|e| format!("pivot item {raw}: {e}"))?;
    cached(Some(&input))
}

/// A cached number's text (`number_text`): a whole number below 1e15 without a fraction, else
/// the shortest text that reads back as it.
fn number_text(value: f64) -> String {
    if value == 0.0 {
        "0".to_string()
    } else if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        format!("{value:?}")
    }
}

fn item_xml(value: Option<&Cached>) -> String {
    match value {
        None => "<m/>".to_string(),
        Some(Cached::Bool(b)) => format!(r#"<b v="{}"/>"#, u8::from(*b)),
        Some(Cached::Number(n)) => format!(r#"<n v="{}"/>"#, number_text(*n)),
        Some(Cached::Error(e)) => format!(r#"<e v="{}"/>"#, escape(e)),
        Some(Cached::Text(t)) => format!(r#"<s v="{}"/>"#, escape(t)),
    }
}

/// An author attribute's text (`_attr_value`).
fn attr_value(value: &Json) -> Result<String, String> {
    match value {
        Json::Bool(b) => Ok(if *b { "1" } else { "0" }.to_string()),
        Json::Number(n) => n
            .as_f64()
            .map(number_text)
            .ok_or_else(|| format!("attribute value {n}")),
        Json::String(s) => Ok(s.clone()),
        other => Err(format!(
            "attribute value {other} is not a string, number or boolean"
        )),
    }
}

/// ` k="v"` for each author attribute of `raw` (`_attrs`).
fn attrs(raw: Option<&Json>) -> Result<String, String> {
    let Some(raw) = raw.filter(|r| !r.is_null()) else {
        return Ok(String::new());
    };
    let object = raw.as_object().ok_or("attributes are an object")?;
    let mut out = String::new();
    for (name, value) in object {
        // `_ATTR_NAME`: a letter, then letters and digits, so a name cannot break out of the tag.
        let mut chars = name.chars();
        let valid = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric());
        if !valid {
            return Err(format!("attribute name {name:?}"));
        }
        let _ = write!(out, r#" {name}="{}""#, escape(&attr_value(value)?));
    }
    Ok(out)
}

fn number(value: Option<&Json>) -> f64 {
    value.and_then(Json::as_f64).unwrap_or(0.0)
}

/// One cache field and the pivot's use of it (`_Field`).
struct Field<'a> {
    name: String,
    /// Per record; empty for a calculated field.
    values: Vec<Option<Cached>>,
    formula: Option<String>,
    /// Distinct values, first-seen order (or `order`'s).
    items: Vec<Option<Cached>>,
    /// Its `fields` entry.
    spec: Option<&'a Map<String, Json>>,
    axis: Option<&'static str>,
    data: bool,
    /// Keys of its calculated items (`calculated`, T-4130).
    calculated: std::collections::HashSet<Key>,
    /// Its part in a discrete or date group (`groups.rs`, T-4170).
    role: Role,
    /// The group items its pivotField indexes, when grouped by a discrete or date group.
    labels: Option<Vec<Option<Cached>>>,
    /// Its pivotField's item order (indices into `labels`), when not their own.
    item_order: Option<Vec<usize>>,
    /// Its `<fieldGroup>`, written after its shared items.
    group_xml: String,
}

/// A field's part in a discrete or date group (`pivot_groups.py`'s `added`, `dates` and
/// `discrete_base`, which exclude each other).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// Not grouped by one.
    Plain,
    /// A field a group adds (`databaseField="0"`).
    Added,
    /// Grouped by date in place: cached as dates.
    Dates,
    /// The base of a discrete group: its shared items are listed.
    DiscreteBase,
}

impl<'a> Field<'a> {
    fn new(name: String, values: Vec<Option<Cached>>, formula: Option<String>) -> Self {
        let mut seen = std::collections::HashSet::new();
        let items = (values.iter())
            .filter(|v| seen.insert(key(v.as_ref())))
            .cloned()
            .collect();
        Field {
            name,
            values,
            formula,
            items,
            spec: None,
            axis: None,
            data: false,
            calculated: std::collections::HashSet::new(),
            role: Role::Plain,
            labels: None,
            item_order: None,
            group_xml: String::new(),
        }
    }

    fn entry(&self, name: &str) -> Option<&'a Json> {
        self.spec.and_then(|s| s.get(name))
    }

    /// Puts the items `order` lists first, in that order (`reorder`).
    fn reorder(&mut self, order: &[Json]) -> Result<(), String> {
        let keys = (bounded(order, "order")?.iter())
            .map(|raw| cached_json(raw).map(|v| key(v.as_ref())))
            .collect::<Result<Vec<_>, _>>()?;
        let by_key: HashMap<Key, &Option<Cached>> =
            self.items.iter().map(|v| (key(v.as_ref()), v)).collect();
        let mut first = Vec::with_capacity(keys.len());
        for k in &keys {
            let item = by_key
                .get(k)
                .ok_or_else(|| format!("an order item is not in field {:?}", self.name))?;
            first.push((*item).clone());
        }
        let listed: std::collections::HashSet<&Key> = keys.iter().collect();
        let rest = (self.items.iter()).filter(|v| !listed.contains(&key(v.as_ref())));
        self.items = first.into_iter().chain(rest.cloned()).collect();
        Ok(())
    }

    fn numeric_only(&self) -> bool {
        (self.values.iter()).all(|v| matches!(v, None | Some(Cached::Number(_))))
    }

    fn group(&self) -> Option<&'a Map<String, Json>> {
        self.entry("group").and_then(Json::as_object)
    }

    /// Whether records refer to the shared items by index (`listed`).
    fn listed(&self) -> bool {
        self.role == Role::DiscreteBase
            || !(self.numeric_only()
                && (self.axis.is_none() || self.group().is_some() || self.role == Role::Dates))
    }

    /// `<sharedItems>` (`shared_items`): the flags always, the list when records refer to it; a
    /// date-grouped field's date flags (`date_shared_items`).
    fn shared_items(&self) -> Result<String, String> {
        if self.role == Role::Dates {
            return groups::date_shared_items(&self.values);
        }
        Ok(self.plain_shared_items())
    }

    fn plain_shared_items(&self) -> String {
        let strings = (self.values.iter()).any(|v| {
            matches!(
                v,
                Some(Cached::Text(_) | Cached::Bool(_) | Cached::Error(_))
            )
        });
        let numbers: Vec<f64> = (self.values.iter())
            .filter_map(|v| match v {
                Some(Cached::Number(n)) => Some(*n),
                _ => None,
            })
            .collect();
        let blank = self.values.iter().any(Option::is_none);
        let mut flags = String::new();
        if !strings && !blank {
            flags.push_str(r#" containsSemiMixedTypes="0""#);
        }
        if !strings {
            flags.push_str(r#" containsString="0""#);
        }
        if strings && !numbers.is_empty() {
            flags.push_str(r#" containsMixedTypes="1""#);
        }
        if !numbers.is_empty() {
            flags.push_str(r#" containsNumber="1""#);
            if numbers.iter().all(|n| n.fract() == 0.0) {
                flags.push_str(r#" containsInteger="1""#);
            }
            let min = numbers.iter().copied().fold(f64::INFINITY, f64::min);
            let max = numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let _ = write!(
                flags,
                r#" minValue="{}" maxValue="{}""#,
                number_text(min),
                number_text(max)
            );
        }
        if blank {
            flags.push_str(r#" containsBlank="1""#);
        }
        if !self.listed() {
            return format!("<sharedItems{flags}/>");
        }
        let items: String = self.items.iter().map(|v| item_xml(v.as_ref())).collect();
        format!(
            r#"<sharedItems{flags} count="{}">{items}</sharedItems>"#,
            self.items.len()
        )
    }

    /// Numeric group items as the builder writes them (`group_labels`); Excel relabels them.
    fn group_labels(&self) -> Result<Vec<String>, String> {
        let group = self.group();
        let get = |name: &str| number(group.and_then(|g| g.get(name)));
        let (start, end, step) = (get("start"), get("end"), get("interval"));
        let refused = || {
            format!(
                "pivot field {:?} group needs 0 < interval and at most 10,000 bins",
                self.name
            )
        };
        let finite = start.is_finite() && end.is_finite() && step.is_finite();
        if !finite || step <= 0.0 || end < start {
            return Err(refused());
        }
        // The builder's `while lo <= end: lo += step` loop, bounded: a step too small to move
        // `low` (1e20 + 2) or past the bin cap is refused instead of looping. It writes one bin
        // past `end` (t4128 G); Excel rebuilds the items on open (pivots.md "Numeric and date
        // group labels are rewritten by Excel").
        let mut labels = vec![format!("<{}", number_text(start))];
        let mut low = start;
        while low <= end {
            let high = low + step;
            if labels.len() > MAX_BINS || high <= low || !high.is_finite() {
                return Err(refused());
            }
            labels.push(format!("{}-{}", number_text(low), number_text(high)));
            low = high;
        }
        labels.push(format!(">{}", number_text(end)));
        Ok(labels)
    }
}

/// A data field (`_data`'s entries).
struct DataEntry<'a> {
    field: usize,
    subtotal: String,
    name: String,
    raw: &'a Map<String, Json>,
}

/// A pivot built from a `definition` (`DefinedPivot`).
struct DefinedPivot<'a> {
    name: &'a str,
    d: &'a Map<String, Json>,
    fields: Vec<Field<'a>>,
    records: usize,
    rows: Vec<i64>,
    columns: Vec<i64>,
    /// Each page field, the key of its selected item (none: `(All)`) and its other attributes.
    pages: Vec<(usize, Option<Key>, String)>,
    /// Each calculated item's entry, field and shared-item index (`calculated_items`, T-4130).
    calculated_items: Vec<(&'a Map<String, Json>, usize, usize)>,
    data: Vec<DataEntry<'a>>,
    refresh_on_load: bool,
}

impl<'a> DefinedPivot<'a> {
    fn new(
        name: &'a str,
        d: &'a Map<String, Json>,
        header: Vec<String>,
        rows: &[Vec<Option<Cached>>],
    ) -> Result<Self, String> {
        let mut fields: Vec<Field<'a>> = (header.into_iter().enumerate())
            .map(|(i, h)| {
                let values = rows.iter().map(|r| r.get(i).cloned().flatten()).collect();
                Field::new(h, values, None)
            })
            .collect();
        let calculated = d.get("calculated").and_then(Json::as_array);
        for calc in bounded(calculated.map_or(&[], Vec::as_slice), "calculated")? {
            let text = |k: &str| calc.get(k).and_then(Json::as_str).map(str::to_string);
            let (Some(field), Some(formula)) = (text("name"), text("formula")) else {
                return Err(format!(
                    "pivot {name}: a calculated field is {{name, formula}}"
                ));
            };
            fields.push(Field::new(field, Vec::new(), Some(formula)));
        }
        let mut pivot = DefinedPivot {
            name,
            d,
            fields,
            records: rows.len(),
            rows: Vec::new(),
            columns: Vec::new(),
            pages: Vec::new(),
            calculated_items: Vec::new(),
            data: Vec::new(),
            refresh_on_load: d
                .get("refresh_on_load")
                .and_then(Json::as_bool)
                .unwrap_or(true),
        };
        pivot.read_fields()?;
        pivot.read_calculated_items()?;
        pivot.rows = pivot.axis("rows", "axisRow")?;
        pivot.columns = pivot.axis("columns", "axisCol")?;
        pivot.read_pages()?;
        pivot.read_data()?;
        // Several data fields need the Values field: on the columns unless placed (`_values_field`).
        let placed = pivot.rows.contains(&VALUES_INDEX) || pivot.columns.contains(&VALUES_INDEX);
        if pivot.data.len() > 1 && !placed {
            pivot.columns.push(VALUES_INDEX);
        }
        Ok(pivot)
    }

    fn find(&self, name: &str) -> Option<usize> {
        let folded = name.to_lowercase();
        (self.fields.iter()).position(|f| f.name.to_lowercase() == folded)
    }

    fn index(&self, name: &str) -> Result<usize, String> {
        self.find(name)
            .ok_or_else(|| format!("pivot {}: no field {name:?}", self.name))
    }

    fn list(&self, key: &str) -> Result<&'a [Json], String> {
        bounded(
            self.d
                .get(key)
                .and_then(Json::as_array)
                .map_or(&[], Vec::as_slice),
            key,
        )
    }

    fn axis(&mut self, key: &str, axis: &'static str) -> Result<Vec<i64>, String> {
        let mut out = Vec::new();
        for name in self.list(key)? {
            let name = name.as_str().ok_or("an axis lists field names")?;
            if name == VALUES {
                out.push(VALUES_INDEX);
                continue;
            }
            let i = self.index(name)?;
            self.fields[i].axis = Some(axis);
            out.push(i64::try_from(i).map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    fn read_pages(&mut self) -> Result<(), String> {
        for page in self.list("pages")? {
            let field = page
                .get("field")
                .and_then(Json::as_str)
                .ok_or("a page names a field")?;
            let i = self.index(field)?;
            if self.fields[i].role == Role::Dates {
                return Err(format!(
                    "pivot {}: page field {field:?} is date-grouped",
                    self.name
                ));
            }
            self.fields[i].axis = Some("axisPage");
            let item = (page.get("item"))
                .map(|raw| cached_json(raw).map(|v| key(v.as_ref())))
                .transpose()?;
            let extra = checks::managed_attrs(page.get("attrs"), &["fld", "item", "hier"])?;
            self.pages.push((i, item, extra));
        }
        Ok(())
    }

    fn read_data(&mut self) -> Result<(), String> {
        for entry in self.list("data")? {
            let raw = entry.as_object().ok_or("a data field is an object")?;
            let field = raw
                .get("field")
                .and_then(Json::as_str)
                .ok_or("a data field names a field")?;
            let i = self.index(field)?;
            if self.fields[i].role == Role::Added {
                return Err(format!(
                    "pivot {}: group field {field:?} is not a data field",
                    self.name
                ));
            }
            let subtotal = raw.get("subtotal").and_then(Json::as_str).unwrap_or("sum");
            let verb = Aggregate::from_consolidate_name(subtotal)
                .ok_or_else(|| format!("subtotal {subtotal:?}"))?
                .word();
            self.fields[i].data = true;
            let name = match raw.get("name").and_then(Json::as_str) {
                Some(name) if !name.is_empty() => name.to_string(),
                _ => format!("{verb} of {}", self.fields[i].name),
            };
            self.data.push(DataEntry {
                field: i,
                subtotal: subtotal.to_string(),
                name,
                raw,
            });
        }
        Ok(())
    }

    fn cache_definition(&self, source_sheet: &str, source_ref: &str) -> Result<String, String> {
        let mut fields = String::new();
        for (index, f) in self.fields.iter().enumerate() {
            let name = escape(&f.name);
            if let Some(formula) = &f.formula {
                let _ = write!(
                    fields,
                    r#"<cacheField name="{name}" numFmtId="0" formula="{}" databaseField="0"/>"#,
                    escape(formula)
                );
                continue;
            }
            if f.role == Role::Added {
                let _ = write!(
                    fields,
                    r#"<cacheField name="{name}" numFmtId="0" databaseField="0">{}</cacheField>"#,
                    raw_cache::group_items(f, &f.group_xml)?
                );
                continue;
            }
            let mut group = f.group_xml.clone();
            if let Some(g) = f.group() {
                let labels = f.group_labels()?;
                let items = labels.iter().fold(String::new(), |mut out, label| {
                    let _ = write!(out, r#"<s v="{}"/>"#, escape(label));
                    out
                });
                group = format!(
                    r#"<fieldGroup base="{index}"><rangePr autoStart="0" autoEnd="0" startNum="{}" endNum="{}" groupInterval="{}"/><groupItems count="{}">{items}</groupItems></fieldGroup>"#,
                    number_text(number(g.get("start"))),
                    number_text(number(g.get("end"))),
                    number_text(number(g.get("interval"))),
                    labels.len()
                );
            }
            let _ = write!(
                fields,
                r#"<cacheField name="{name}" numFmtId="0">{}{}</cacheField>"#,
                f.shared_items()?,
                raw_cache::group_items(f, &group)?
            );
        }
        let extra = attrs(self.d.get("cache"))?;
        let mut calculated = String::new();
        if !self.calculated_items.is_empty() {
            let entries = (self.calculated_items.iter())
                .map(|(raw, field, x)| Self::calculated_item(raw, *field, *x))
                .collect::<Result<String, String>>()?;
            calculated = format!(
                r#"<calculatedItems count="{}">{entries}</calculatedItems>"#,
                self.calculated_items.len()
            );
        }
        let refresh = if self.refresh_on_load {
            r#" refreshOnLoad="1""#
        } else {
            ""
        };
        Ok(format!(
            r#"<pivotCacheDefinition xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" r:id="rId1"{refresh} createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{}"{extra}><cacheSource type="worksheet"><worksheetSource ref="{source_ref}" sheet="{}"/></cacheSource><cacheFields count="{}">{fields}</cacheFields>{calculated}</pivotCacheDefinition>"#,
            self.records,
            escape(source_sheet),
            self.fields.len()
        ))
    }

    fn cache_records(&self) -> Result<String, String> {
        let database: Vec<&Field<'_>> = (self.fields.iter())
            .filter(|f| f.formula.is_none() && f.role != Role::Added)
            .collect();
        // A listed field's records refer to its items by index; the others hold values inline.
        let index: Vec<Option<HashMap<Key, usize>>> = (database.iter())
            .map(|f| {
                let keys = f
                    .items
                    .iter()
                    .enumerate()
                    .map(|(k, v)| (key(v.as_ref()), k));
                f.listed().then(|| keys.collect())
            })
            .collect();
        let names: Vec<&str> = database.iter().map(|f| f.name.as_str()).collect();
        let raw = self.records_xml(&names)?;
        let mut rows = String::new();
        for r in 0..self.records {
            rows.push_str("<r>");
            for (k, (f, at)) in database.iter().zip(&index).enumerate() {
                let value = f.values.get(r).and_then(Option::as_ref);
                if let Some(xml) = raw.get(&(r, k)) {
                    rows.push_str(xml);
                } else if let Some(k) = at.as_ref().and_then(|at| at.get(&key(value))) {
                    let _ = write!(rows, r#"<x v="{k}"/>"#);
                } else if let (true, Some(Cached::Number(n))) = (f.role == Role::Dates, value) {
                    let _ = write!(rows, r#"<d v="{}"/>"#, groups::iso_date(*n)?);
                } else {
                    rows.push_str(&item_xml(value));
                }
            }
            rows.push_str("</r>");
        }
        Ok(format!(
            r#"<pivotCacheRecords xmlns="{MAIN_NS}" count="{}">{rows}</pivotCacheRecords>"#,
            self.records
        ))
    }

    fn page_field(&self, field: usize, item: Option<&Key>, extra: &str) -> Result<String, String> {
        let Some(wanted) = item else {
            return Ok(format!(r#"<pageField fld="{field}" hier="-1"{extra}/>"#));
        };
        let k = (self.fields[field].items.iter())
            .position(|v| key(v.as_ref()) == *wanted)
            .ok_or("a page item is not in its field")?;
        Ok(format!(
            r#"<pageField fld="{field}" item="{k}" hier="-1"{extra}/>"#
        ))
    }

    fn data_field(&self, e: &DataEntry<'_>) -> Result<String, String> {
        let mut attrs = format!(r#" name="{}" fld="{}""#, escape(&e.name), e.field);
        if e.subtotal != "sum" {
            let _ = write!(attrs, r#" subtotal="{}""#, e.subtotal);
        }
        let mut ext = String::new();
        // The builder's `SHOW_DATA_AS` is the engine's one name table: an x14 mode
        // (`SHOW_AS_X14`) is the x14 dataField extension's `pivotShowAs`, with no `showDataAs`.
        if let Some(show) = e.raw.get("showDataAs").and_then(Json::as_str) {
            let mode = ShowDataAs::from_name(show).ok_or_else(|| format!("showDataAs {show:?}"))?;
            match mode.x14_data_field_extension("") {
                Some(extension) => ext = format!("<extLst>{extension}</extLst>"),
                None => {
                    let _ = write!(attrs, r#" showDataAs="{}""#, mode.name());
                }
            }
        }
        let base = match e.raw.get("baseField") {
            Some(Json::String(name)) => self.index(name)?.to_string(),
            Some(other) => other.as_i64().unwrap_or(0).to_string(),
            None => "0".to_string(),
        };
        let base_item = e.raw.get("baseItem").and_then(Json::as_i64).unwrap_or(0);
        let _ = write!(attrs, r#" baseField="{base}" baseItem="{base_item}""#);
        if let Some(format) = e.raw.get("numFmtId").and_then(Json::as_i64) {
            let _ = write!(attrs, r#" numFmtId="{format}""#);
        }
        Ok(if ext.is_empty() {
            format!("<dataField{attrs}/>")
        } else {
            format!("<dataField{attrs}>{ext}</dataField>")
        })
    }

    /// The author's XML for `slot`, as written (`_raw`).
    fn raw(&self, slot: &str) -> String {
        (self.d.get("xml").and_then(|x| x.get(slot)))
            .and_then(Json::as_str)
            .map_or_else(String::new, |text| text.trim().to_string())
    }

    fn table_definition(&self, cache_id: u32, anchor: Addr) -> Result<String, String> {
        let table = self.d.get("table").and_then(Json::as_object);
        let extra = attrs(self.d.get("table"))?;
        let mut head = (TABLE_DEFAULTS.split(' '))
            .filter_map(|pair| pair.split_once('='))
            .filter(|(k, _)| !table.is_some_and(|t| t.contains_key(*k)))
            .fold(String::new(), |mut out, (k, v)| {
                let _ = write!(out, r#" {k}="{v}""#);
                out
            });
        if self.rows.contains(&VALUES_INDEX) {
            head.push_str(r#" dataOnRows="1""#);
        }
        let corner = Addr {
            row: anchor.row + 1,
            col: anchor.col + 1,
        };
        // `_area`: a range from the anchor, written from its parsed corners.
        let area = match self.d.get("area").and_then(Json::as_str) {
            None => format!("{anchor}:{corner}"),
            Some(area) => {
                let (from, to) = (area.split_once(':').and_then(|_| parse_range(area)))
                    .filter(|(from, to)| {
                        *from == anchor && to.row >= from.row && to.col >= from.col
                    })
                    .ok_or_else(|| format!("pivot {}: area {area:?}", self.name))?;
                format!("{from}:{to}")
            }
        };
        let pages = if self.pages.is_empty() {
            String::new()
        } else {
            format!(r#" rowPageCount="{}" colPageCount="1""#, self.pages.len())
        };
        let mut parts = format!(
            r#"<location ref="{area}" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"{pages}/>"#
        );
        let fields = (self.fields.iter())
            .map(Self::pivot_field)
            .collect::<Result<String, String>>()?;
        let _ = write!(
            parts,
            r#"<pivotFields count="{}">{fields}</pivotFields>"#,
            self.fields.len()
        );
        for (tag, axis, lines) in [
            ("rowFields", &self.rows, "rowItems"),
            ("colFields", &self.columns, "colItems"),
        ] {
            if !axis.is_empty() {
                let fields = axis.iter().fold(String::new(), |mut out, x| {
                    let _ = write!(out, r#"<field x="{x}"/>"#);
                    out
                });
                let _ = write!(parts, r#"<{tag} count="{}">{fields}</{tag}>"#, axis.len());
            }
            parts.push_str(&self.raw(lines));
        }
        if !self.pages.is_empty() {
            let pages = (self.pages.iter())
                .map(|(field, item, extra)| self.page_field(*field, item.as_ref(), extra))
                .collect::<Result<String, String>>()?;
            let _ = write!(
                parts,
                r#"<pageFields count="{}">{pages}</pageFields>"#,
                self.pages.len()
            );
        }
        let data = (self.data.iter())
            .map(|e| self.data_field(e))
            .collect::<Result<String, String>>()?;
        let _ = write!(
            parts,
            r#"<dataFields count="{}">{data}</dataFields>"#,
            self.data.len()
        );
        for slot in &SLOTS[..4] {
            parts.push_str(&self.raw(slot));
        }
        parts.push_str(r#"<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1" showRowStripes="0" showColStripes="0" showLastColumn="1"/>"#);
        for slot in &SLOTS[4..] {
            parts.push_str(&self.raw(slot));
        }
        Ok(format!(
            r#"<pivotTableDefinition xmlns="{MAIN_NS}" name="{}" cacheId="{cache_id}"{head}{extra}>{parts}</pivotTableDefinition>"#,
            escape(self.name)
        ))
    }
}

/// `list` when it holds at most [`MAX_LIST`] entries.
fn bounded<'j>(list: &'j [Json], what: &str) -> Result<&'j [Json], String> {
    if list.len() > MAX_LIST {
        return Err(format!("more than {MAX_LIST} entries in {what}"));
    }
    Ok(list)
}

/// The parts the case builder writes for `pivot` of `sheet` (its `definition`, `definition`),
/// with cache id `cache_id` (`pivot.py` `_defined`).
///
/// # Errors
/// A definition or source the builder would refuse.
pub(crate) fn parts(
    spec: &CaseSpec,
    sheet: &SheetSpec,
    pivot: &PivotTableSpec,
    definition: &Json,
    cache_id: u32,
) -> Result<Parts, String> {
    let name = pivot.name.as_str();
    let d = definition
        .as_object()
        .ok_or_else(|| format!("pivot {name}: a definition is an object"))?;
    let (source_sheet, range) = split_qualified(&pivot.source)
        .unwrap_or_else(|| (sheet.name.clone(), pivot.source.clone()));
    let cells = &(spec.sheets.iter())
        .find(|s| s.name == source_sheet)
        .ok_or_else(|| format!("pivot source sheet {source_sheet:?} is not in the spec"))?
        .cells;
    let (from, to) = parse_range(&range)
        .filter(|(from, to)| from.row < to.row && from.col <= to.col)
        .ok_or_else(|| format!("pivot {name}: source {range} has no data rows"))?;
    // Every cell of the source is read: bound it as a refresh bounds its records.
    let size = u64::from(to.row - from.row + 1) * u64::from(to.col - from.col + 1);
    if size > MAX_RECORD_VALUES as u64 {
        return Err(format!(
            "pivot {name}: source {range} holds more than {MAX_RECORD_VALUES} values"
        ));
    }
    let mut rows = Vec::new();
    let mut header = Vec::new();
    for row in from.row..=to.row {
        let mut values = Vec::new();
        for col in from.col..=to.col {
            let cell = cells.get(&Addr { row, col }.to_string());
            if row == from.row {
                match cell {
                    Some(CellInput::Str(text)) if !text.is_empty() => header.push(text.clone()),
                    _ => header.push(String::new()),
                }
            } else {
                values.push(cached(cell)?);
            }
        }
        if row > from.row {
            rows.push(values);
        }
    }
    if let Some(names) = d.get("cache_names").and_then(Json::as_array) {
        if names.len() != header.len() {
            return Err(format!(
                "pivot {name}: cache_names needs one name per source column"
            ));
        }
        header = (names.iter())
            .map(|n| n.as_str().map(str::to_string))
            .collect::<Option<_>>()
            .ok_or_else(|| format!("pivot {name}: cache_names are texts"))?;
    }
    if header.iter().any(String::is_empty) {
        return Err(format!(
            "pivot {name}: the source header has a blank or non-text cell"
        ));
    }
    let anchor = Addr::parse(&pivot.location)
        .ok_or_else(|| format!("pivot {name}: bad location {}", pivot.location))?;
    let built = DefinedPivot::new(name, d, header, &rows)?;
    let source_ref = format!("{from}:{to}");
    Ok(Parts {
        cache_definition: built.cache_definition(&source_sheet, &source_ref)?,
        cache_records: built.cache_records()?,
        table: built.table_definition(cache_id, anchor)?,
    })
}

mod calculated_items;
mod checks;
#[cfg(test)]
mod export_tests;
mod groups;
mod item_order;
mod pivot_field;
mod raw_cache;
#[cfg(test)]
mod tests;
