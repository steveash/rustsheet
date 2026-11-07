//! Pivot caches and pivot tables of a loaded workbook (SPEC-0015 "Data model", T-2081).
//!
//! A cache stores its records by column, as indices into each field's interned items, so a record
//! costs 4 bytes per field and no allocation of its own. [`CacheBuilder`] is the one way to build a
//! cache: the xlsx importer (T-2082) and `Txn::add_pivot_table` (T-2084) both use it, and it counts
//! the SPEC-0015 limits before every push, so nothing past them is allocated (CLAUDE.md
//! directive 5). Pivots follow structural edits (T-2086): a row or column insert or delete shifts a
//! pivot's location, drops the pivot or is refused when it cuts through one
//! ([`Model::displace_pivots`]), a sheet rename rewrites the
//! cache sources naming it, and a sheet deletion takes the sheet's pivots with it so an undo puts
//! them back. A row or column move shifts a pivot with its cells, or is refused when it would
//! split one ([`Model::move_pivots`]; T-2401; docs/excel-semantics/web.md "Pivot tables and
//! structural edits").

use std::collections::{HashMap, HashSet};

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use crate::actions::{displace_interval_for_move, interval_survives_move};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::Error;
use crate::model::Model;
use crate::types::sheet_names_match;

mod accumulate;
mod added_groups;
mod calculated;
mod date_groups;
mod filter;
mod grid;
pub(crate) mod layout;
mod model;
mod number_groups;
#[cfg(test)]
mod probe_cases;
mod refresh;
mod result;
mod show_as;

pub use added_groups::{group_by_discrete, AddedGroup};
pub use calculated::{
    check_calculated_fields, CalculatedFieldError, MAX_CALCULATED_FIELDS,
    MAX_CALCULATED_FORMULA_CHARS,
};
pub use date_groups::{group_by_dates, ungroup_dates, DateGroupBy, DateGroupError, DateGroups};
pub use filter::{
    AutoShow, DateGroupItem, DynamicDate, FilterCondition, FilterCriteria, FilterOperator,
    PivotFilter, PivotFilterKind, RelativeUnit,
};
pub use grid::{PivotBudget, PivotPlacement, PivotRefresh, PivotRefreshError, PivotSwap};
pub use layout::{
    lay_out, records_behind, render, AxisLine, CellFormat, PivotCell, PivotLayout, RenderedCell,
    ALL_ITEMS, BLANK_ITEM, COLUMN_LABELS, GRAND_TOTAL, MAX_PIVOT_CELLS, MAX_PIVOT_TEXT_BYTES,
    MULTIPLE_ITEMS, ROW_LABELS, VALUES,
};
pub use model::{
    element_name, Aggregate, Axis, AxisField, CellArea, DataField, ItemType, OpaqueEntry,
    OpaqueNode, OpaqueOwner, PageField, PivotField, PivotGeometry, PivotItem, PivotOptions,
    PivotTable, ShowDataAs, SortType, NO_BASE_ITEM,
};
pub use number_groups::{group_by_numbers, NumberGroupError, NumberGroups};
pub use refresh::{
    read_source, refresh, regroup, RangeGrouping, RegroupError, SharedItemsFlags, SourceArea,
    SourceCells,
};
pub use result::{
    compute, compute_on, AxisNode, AxisTree, PivotComputeError, PivotResult, MAX_PIVOT_NODES,
    MAX_PIVOT_WORK,
};

/// Most records in one cache (SPEC-0015 "Untrusted-input limits").
pub const MAX_RECORDS: usize = 1_048_576;
/// Most record values (records x fields) in one cache, and over the whole workbook. A field
/// grouped in place also keeps its own records ([`CacheField::ungrouped`]), which this count
/// leaves out: at most one such copy per field, so a cache holds at most twice the record values
/// counted here. That factor of two is an accepted bound on memory (directive 5), not an Excel
/// limit (T-4734).
pub const MAX_RECORD_VALUES: usize = 10_000_000;
/// Most items over all fields of one cache.
pub const MAX_ITEMS: usize = 1_000_000;
/// Most fields in one cache.
pub const MAX_FIELDS: usize = 16_384;

/// A pivot cache: the records a pivot table summarizes.
#[derive(Encode, Decode, Debug, PartialEq, Clone, Default)]
pub struct PivotCache {
    /// The `worksheetSource`, kept for a later refresh; `None` for other sources.
    pub source: Option<CacheSource>,
    /// `worksheetSource@name`: the defined name or Table the cache was read from, kept so the
    /// export writes the source back (T-4176). A refresh does not resolve it yet (T-4506).
    pub source_name: Option<String>,
    /// The cache's fields, in cache order.
    pub fields: Vec<CacheField>,
    /// How many records; every [`CacheField::records`] but a calculated field's has this length.
    pub record_count: u32,
    /// `refreshOnLoad`: the facade refreshes the cache after a loaded workbook's first
    /// recalculation (SPEC-0032 "Who refreshes, and when"; pivots.md, probes/t4128-pivot-spike R).
    pub refresh_on_load: bool,
}

/// The worksheet range a cache was read from.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CacheSource {
    /// The source sheet's name.
    pub sheet_name: String,
    /// The source range, as written in the file (`A1:D20`).
    pub range: String,
}

/// One field (source column) of a cache.
#[derive(Encode, Decode, Debug, PartialEq, Clone, Default)]
pub struct CacheField {
    /// The field's name (the source column header).
    pub name: String,
    /// The field's distinct values: its `<sharedItems>`, then the inline record values interned
    /// at import, so records only ever hold indices.
    pub items: Vec<CacheItem>,
    /// Per record, an index into `items`; empty for a calculated field ([`CacheField::formula`]).
    pub records: Vec<u32>,
    /// For a field grouped by numeric ranges, its grouping: `items` are then the group labels
    /// and GETPIVOTDATA also picks a group by its lower bound (T-4235).
    pub number_groups: Option<NumberGroups>,
    /// For a field grouped by dates, its grouping: `items` are then the group labels and
    /// GETPIVOTDATA also picks a group by its calendar value (T-4246).
    pub date_groups: Option<DateGroups>,
    /// For a field Excel added to group another (`databaseField="0"`: a date field's `Quarters`
    /// and `Years`, a manual group field), the base field it derives from, so a refresh builds it
    /// again from the source (T-4171).
    pub added_group: Option<AddedGroup>,
    /// For a calculated field (`cacheField@formula`, `databaseField="0"`), its formula as the
    /// file writes it (`Units*Price`, no `=`). Such a field has no items and no records, follows
    /// every other field of the cache, and is only a data field: its value in each cell is the
    /// formula over that cell's sums (`calculated`, T-4174).
    pub formula: Option<String>,
    /// For a field grouped in place by a range ([`CacheField::number_groups`],
    /// [`CacheField::date_groups`]), its own values before the grouping: the file keeps them as
    /// the field's shared items and record values and derives the groups from them, so the export
    /// writes them back (T-4734). Set by [`keep_ungrouped`] where a field is grouped in place.
    /// `None` for an ungrouped field and for a field Excel added, whose values are its base
    /// field's.
    pub ungrouped: Option<UngroupedValues>,
}

/// A range-grouped field's own items and records ([`CacheField::ungrouped`]): what the field
/// held before [`group_by_numbers`] or [`group_by_dates`] grouped it. At most one copy per
/// field, so a cache holds at most twice the record values it counts against
/// [`MAX_RECORD_VALUES`] (directive 5).
#[derive(Encode, Decode, Debug, PartialEq, Clone, Default)]
pub struct UngroupedValues {
    /// The field's distinct values, as [`CacheField::items`] held them.
    pub items: Vec<CacheItem>,
    /// Per record, an index into `items`.
    pub records: Vec<u32>,
}

/// One value of a cache field.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum CacheItem {
    /// No value (`<m/>`).
    Missing,
    /// A number (`<n v>`).
    Number(f64),
    /// Text (`<s v>`).
    Text(String),
    /// A boolean (`<b v>`).
    Boolean(bool),
    /// An error (`<e v>`).
    Error(Error),
    /// A date (`<d v>`), as a serial number.
    Date(f64),
}

/// Why a cache could not be built. Past a limit the caller drops the whole cache (and every pivot
/// on it), never a truncated one, so the answer is `#REF!` rather than a wrong total.
#[derive(ThisError, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PivotCacheError {
    /// More than [`MAX_FIELDS`] fields.
    #[error("a pivot cache has more than {MAX_FIELDS} fields")]
    TooManyFields,
    /// More than [`MAX_RECORDS`] records.
    #[error("a pivot cache has more than {MAX_RECORDS} records")]
    TooManyRecords,
    /// More than [`MAX_ITEMS`] items over all fields.
    #[error("a pivot cache has more than {MAX_ITEMS} items")]
    TooManyItems,
    /// More record values than the cache's or the workbook's budget allows.
    #[error("pivot caches hold more than {MAX_RECORD_VALUES} record values")]
    TooManyRecordValues,
    /// A field index, a shared-item index or a record's length does not match the cache.
    #[error("a pivot cache record does not match its fields")]
    BadRecord,
    /// A refresh found no source: the cache has none, or it names a sheet or range the workbook
    /// does not hold (`refresh`; a defined name or a Table is T-4506's).
    #[error("a pivot cache's source is not a worksheet range of this workbook")]
    Source,
    /// A refresh found a source header (0-based `column` within the source) that is a number, a
    /// boolean or an error: how Excel names such a field is unprobed (`refresh`).
    #[error("pivot source header {column} is not text")]
    Header {
        /// The header's 0-based column within the source.
        column: u32,
    },
    /// A refresh found values a field's saved range grouping cannot group (`refresh`).
    #[error("a refreshed pivot field no longer fits its grouping")]
    Group,
    /// A manual group (`<discretePr>`) gives an item of its base field no group item
    /// ([`group_by_discrete`]).
    #[error("a manual pivot group does not cover its base field's items")]
    Discrete,
}

/// One value of a record handed to [`CacheBuilder::push_record`].
#[derive(Debug, Clone, PartialEq)]
pub enum RecordValue {
    /// An index into the field's shared items (`<x v>`).
    Shared(u32),
    /// An inline value, interned into the field's items.
    Inline(CacheItem),
}

/// The interning key of a [`CacheItem`]: numbers by bits, so it can be hashed.
#[derive(Hash, PartialEq, Eq)]
enum ItemKey {
    Missing,
    Number(u64),
    Text(String),
    Boolean(bool),
    Error(String),
    Date(u64),
}

impl ItemKey {
    fn of(item: &CacheItem) -> ItemKey {
        match item {
            CacheItem::Missing => ItemKey::Missing,
            CacheItem::Number(n) => ItemKey::Number(n.to_bits()),
            CacheItem::Text(s) => ItemKey::Text(s.clone()),
            CacheItem::Boolean(b) => ItemKey::Boolean(*b),
            CacheItem::Error(e) => ItemKey::Error(e.to_string()),
            CacheItem::Date(d) => ItemKey::Date(d.to_bits()),
        }
    }

    /// The key under which a refresh finds an item (`refresh`): exact for numbers, and for the
    /// type (the number 1 and the text `1` are two items, a date-formatted value never matches a
    /// cached number), but text ignoring case (docs/excel-semantics/pivots.md "Item identity:
    /// exact for numbers and types, case-insensitive for text"; probes/t4134-pivot-items I1, I4,
    /// I5; t4582-pivot-item-edges J3, J4). Case folds as the text functions fold it.
    fn identity(item: &CacheItem) -> ItemKey {
        match item {
            CacheItem::Text(s) => ItemKey::Text(crate::functions::util::collation::fold_case(s)),
            other => ItemKey::of(other),
        }
    }
}

impl PivotCache {
    /// How many of the cache's fields Excel added to group another ([`AddedGroup`]): each is
    /// derived again over every record on a refresh (T-4171).
    #[must_use]
    pub fn added_group_count(&self) -> usize {
        (self.fields.iter())
            .filter(|f| f.added_group.is_some())
            .count()
    }
}

/// `grouped`, the range grouping of `base` that replaces it in place ([`group_by_numbers`],
/// [`group_by_dates`], [`regroup`]), keeping `base`'s items and records as its
/// [`CacheField::ungrouped`] values, moved rather than copied; `grouped` as it is when it holds no
/// range grouping (a refresh that dropped one). The one step every in-place grouping takes (the
/// xlsx importer, [`refresh`], `Txn::add_pivot_table`), so the export can write the field's own
/// values back (T-4734). A field Excel added is not grouped in place and keeps none.
#[must_use]
pub fn keep_ungrouped(base: CacheField, mut grouped: CacheField) -> CacheField {
    if grouped.number_groups.is_some() || grouped.date_groups.is_some() {
        grouped.ungrouped = Some(UngroupedValues {
            items: base.items,
            records: base.records,
        });
    }
    grouped
}

/// Whether two of `items`' texts differ only in case (folded as a refresh finds an item,
/// [`ItemKey::identity`]): [`CacheField::has_case_variant_texts`] over any list of items, such
/// as a grouped field's own values (T-4734).
#[must_use]
pub fn has_case_variant_texts(items: &[CacheItem]) -> bool {
    let mut seen = HashSet::new();
    (items.iter())
        .filter(|item| matches!(item, CacheItem::Text(_)))
        .any(|item| !seen.insert(ItemKey::identity(item)))
}

impl CacheField {
    /// Whether the field's items are group items a refresh builds again: a range grouping, or a
    /// field Excel added to group another ([`AddedGroup`]).
    #[must_use]
    pub fn is_grouped(&self) -> bool {
        self.number_groups.is_some() || self.date_groups.is_some() || self.added_group.is_some()
    }

    /// Whether two of the field's text items differ only in case (folded as a refresh finds an
    /// item, [`ItemKey::identity`]). The import interns inline values exactly, so a file can
    /// yield such a field, but Excel for the web calls a file whose shared items hold one corrupt
    /// (docs/excel-semantics/pivots.md "Case-variant text items make Excel for the web call the
    /// file corrupt"), so the export (T-4176) does not write one.
    #[must_use]
    pub fn has_case_variant_texts(&self) -> bool {
        has_case_variant_texts(&self.items)
    }
}

/// Builds a [`PivotCache`] field by field and record by record, counting the limits before each
/// push.
pub struct CacheBuilder {
    cache: PivotCache,
    /// Per field, the index of each interned item.
    interned: Vec<HashMap<ItemKey, u32>>,
    items: usize,
    /// Record values this cache may still hold: the smaller of the per-cache limit and what the
    /// workbook has left.
    values_left: usize,
}

impl CacheBuilder {
    /// A builder for a cache read from `source`. `workbook_values_left` is how many record values
    /// the workbook may still hold over all its caches (start from [`MAX_RECORD_VALUES`]).
    #[must_use]
    pub fn new(source: Option<CacheSource>, workbook_values_left: usize) -> Self {
        CacheBuilder {
            cache: PivotCache {
                source,
                ..PivotCache::default()
            },
            interned: Vec::new(),
            items: 0,
            values_left: workbook_values_left.min(MAX_RECORD_VALUES),
        }
    }

    /// Adds a field; fields must all be added before the first record.
    ///
    /// # Errors
    /// [`PivotCacheError::TooManyFields`] past [`MAX_FIELDS`], and
    /// [`PivotCacheError::BadRecord`] once a record has been pushed.
    pub fn add_field(&mut self, name: String) -> Result<u32, PivotCacheError> {
        if self.cache.record_count > 0 {
            return Err(PivotCacheError::BadRecord);
        }
        if self.cache.fields.len() >= MAX_FIELDS {
            return Err(PivotCacheError::TooManyFields);
        }
        self.cache.fields.push(CacheField {
            name,
            ..CacheField::default()
        });
        self.interned.push(HashMap::new());
        // At most MAX_FIELDS fields, so the index fits a u32.
        Ok(self.cache.fields.len() as u32 - 1)
    }

    /// Appends a shared item to `field`, at the next index whatever its value: records refer to
    /// shared items by position, so they are not deduplicated (a later equal inline value finds
    /// the first of them).
    ///
    /// # Errors
    /// [`PivotCacheError::BadRecord`] for an unknown field, [`PivotCacheError::TooManyItems`]
    /// past [`MAX_ITEMS`].
    pub fn add_shared_item(&mut self, field: u32, item: CacheItem) -> Result<u32, PivotCacheError> {
        let index = self.next_item_index(field)?;
        let (Some(f), Some(interned)) = (
            self.cache.fields.get_mut(field as usize),
            self.interned.get_mut(field as usize),
        ) else {
            return Err(PivotCacheError::BadRecord);
        };
        interned.entry(ItemKey::of(&item)).or_insert(index);
        f.items.push(item);
        self.items += 1;
        Ok(index)
    }

    /// Appends one record, one value per field in field order.
    ///
    /// # Errors
    /// [`PivotCacheError::BadRecord`] when the record's length differs from the field count or a
    /// shared index is past the field's items; the record, value and item limits otherwise.
    pub fn push_record(&mut self, values: Vec<RecordValue>) -> Result<(), PivotCacheError> {
        if values.len() != self.cache.fields.len() {
            return Err(PivotCacheError::BadRecord);
        }
        if self.cache.record_count as usize >= MAX_RECORDS {
            return Err(PivotCacheError::TooManyRecords);
        }
        if values.len() > self.values_left {
            return Err(PivotCacheError::TooManyRecordValues);
        }
        // Resolve every value before pushing any, so a bad record leaves the columns aligned.
        let mut indices = Vec::with_capacity(values.len());
        for (field, value) in (0u32..).zip(values) {
            let index = match value {
                RecordValue::Shared(index) => {
                    let len = self
                        .cache
                        .fields
                        .get(field as usize)
                        .map_or(0, |f| f.items.len());
                    if index as usize >= len {
                        return Err(PivotCacheError::BadRecord);
                    }
                    index
                }
                RecordValue::Inline(item) => self.intern(field, item)?,
            };
            indices.push(index);
        }
        for (f, index) in self.cache.fields.iter_mut().zip(indices) {
            f.records.push(index);
        }
        self.values_left -= self.cache.fields.len();
        self.cache.record_count += 1;
        Ok(())
    }

    /// How many record values the cache holds so far (subtract from the workbook's budget).
    #[must_use]
    pub fn record_values(&self) -> usize {
        self.cache.record_count as usize * self.cache.fields.len()
    }

    /// The fields so far, in cache order.
    #[must_use]
    pub fn fields(&self) -> &[CacheField] {
        &self.cache.fields
    }

    /// The finished cache.
    #[must_use]
    pub fn finish(self) -> PivotCache {
        self.cache
    }

    fn next_item_index(&self, field: u32) -> Result<u32, PivotCacheError> {
        let f = self
            .cache
            .fields
            .get(field as usize)
            .ok_or(PivotCacheError::BadRecord)?;
        if self.items >= MAX_ITEMS {
            return Err(PivotCacheError::TooManyItems);
        }
        // At most MAX_ITEMS items, so the index fits a u32.
        Ok(f.items.len() as u32)
    }

    /// The index of `item` in `field`, adding it when the field has no equal item yet.
    fn intern(&mut self, field: u32, item: CacheItem) -> Result<u32, PivotCacheError> {
        let key = ItemKey::of(&item);
        if let Some(&index) = self.interned.get(field as usize).and_then(|m| m.get(&key)) {
            return Ok(index);
        }
        self.add_shared_item(field, item)
    }
}

/// Which lines a structural edit inserts or deletes ([`Model::displace_pivots`]).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PivotLines {
    /// Whole rows.
    Rows,
    /// Whole columns.
    Columns,
}

/// What a row or column insert or delete does to the 0-based span `[first, last]` of a pivot's
/// rows or columns ([`displace_span`]).
#[derive(Debug, PartialEq, Eq)]
enum SpanEdit {
    /// The span is now `[first, last]`.
    Keep(u32, u32),
    /// The edit deletes every line of the span: the pivot goes with its cells.
    Drop,
    /// The edit cuts through the span, or pushes it off the sheet: Excel refuses an edit that
    /// would change part of a pivot (SPEC-0015 "Data model"; UNVERIFIED which edits exactly,
    /// T-2402), so the edit is refused as one that would break an array formula is.
    Refuse,
}

/// What inserting (`delta > 0`) or deleting (`delta < 0`) `|delta|` lines at the 0-based line `at`
/// does to the span `[first, last]` on a sheet whose last 0-based line is `max`: lines inserted at
/// or before `first`, or deleted wholly before it, shift it like the cells it covers; lines
/// inserted or deleted after `last` leave it; a delete of every line of it drops it; anything else
/// is refused ([`SpanEdit`]).
fn displace_span(first: u32, last: u32, at: i64, delta: i64, max: u32) -> SpanEdit {
    let (from, to) = (i64::from(first), i64::from(last));
    if at > to {
        return SpanEdit::Keep(first, last);
    }
    if delta >= 0 && at > from {
        return SpanEdit::Refuse;
    }
    if delta < 0 {
        let end = at - delta;
        if at <= from && end > to {
            return SpanEdit::Drop;
        }
        if end > from {
            return SpanEdit::Refuse;
        }
    }
    let shift = |line: i64| u32::try_from(line + delta).ok().filter(|&l| l <= max);
    match (shift(from), shift(to)) {
        (Some(first), Some(last)) => SpanEdit::Keep(first, last),
        _ => SpanEdit::Refuse,
    }
}

impl Model {
    /// Moves the pivots of the sheet at index `sheet` for an insert (`delta > 0`) or delete
    /// (`delta < 0`) of `|delta|` rows or columns at the 1-based line `at`, with their cells, or
    /// refuses the edit, leaving every pivot as it was, when it cuts through one
    /// ([`displace_span`]). Called before the edit changes anything, so a refusal leaves the model
    /// unchanged. SPEC-0015 "Data model".
    pub(crate) fn displace_pivots(
        &mut self,
        sheet: u32,
        lines: PivotLines,
        at: i32,
        delta: i32,
    ) -> Result<(), String> {
        let rows = lines == PivotLines::Rows;
        let sheet_id = self.workbook.worksheet(sheet)?.sheet_id;
        let (at, delta) = (i64::from(at) - 1, i64::from(delta));
        // The last 0-based line; `LAST_ROW` and `LAST_COLUMN` are 1-based and positive, so the
        // conversion cannot fail.
        let max = u32::try_from(if rows { LAST_ROW } else { LAST_COLUMN } - 1).unwrap_or(0);
        let span = |area: &CellArea| {
            if rows {
                (area.top, area.bottom)
            } else {
                (area.left, area.right)
            }
        };
        // `None` for a pivot on another sheet, which the edit leaves alone.
        let edits: Vec<Option<SpanEdit>> = self
            .workbook
            .pivot_tables
            .iter()
            .map(|pivot| {
                (pivot.sheet_id == sheet_id).then(|| {
                    let (first, last) = span(&pivot.location);
                    displace_span(first, last, at, delta, max)
                })
            })
            .collect();
        if let Some(i) = edits.iter().position(|e| *e == Some(SpanEdit::Refuse)) {
            let name = self
                .workbook
                .pivot_tables
                .get(i)
                .map_or("", |p| p.name.as_str());
            return Err(format!(
                "Cannot change part of the pivot table '{name}' (insert or delete around it)"
            ));
        }
        let pivots = std::mem::take(&mut self.workbook.pivot_tables);
        self.workbook.pivot_tables = pivots
            .into_iter()
            .zip(edits)
            .filter_map(|(mut pivot, edit)| match edit {
                Some(SpanEdit::Keep(first, last)) => {
                    let area = &mut pivot.location;
                    if rows {
                        (area.top, area.bottom) = (first, last);
                    } else {
                        (area.left, area.right) = (first, last);
                    }
                    Some(pivot)
                }
                Some(SpanEdit::Drop) => None,
                None | Some(SpanEdit::Refuse) => Some(pivot),
            })
            .collect();
        // The indices are keyed by position in `pivot_tables`, which dropping shifts.
        self.evaluation.pivot_indexes.clear();
        Ok(())
    }

    /// Moves the pivots of the sheet at index `sheet` with the 1-based lines `[group_start,
    /// group_end]` that a row or column move shifts by `delta`, or refuses the move, changing
    /// nothing, when it would split a pivot (as a move that splits a merged cell is refused;
    /// SPEC-0015 "Data model", UNVERIFIED which moves Excel refuses, T-2402). Called before the
    /// move changes anything.
    pub(crate) fn move_pivots(
        &mut self,
        sheet: u32,
        lines: PivotLines,
        group_start: i32,
        group_end: i32,
        delta: i32,
    ) -> Result<(), String> {
        let rows = lines == PivotLines::Rows;
        let sheet_id = self.workbook.worksheet(sheet)?.sheet_id;
        // The 1-based span of a pivot's rows or columns, as a move sees it; spans are within the
        // sheet, so the conversion cannot fail.
        let span = |area: &CellArea| {
            let (first, last) = if rows {
                (area.top, area.bottom)
            } else {
                (area.left, area.right)
            };
            (
                i32::try_from(first).unwrap_or(0) + 1,
                i32::try_from(last).unwrap_or(0) + 1,
            )
        };
        let on_sheet = |p: &&mut PivotTable| p.sheet_id == sheet_id;
        if let Some(pivot) = self
            .workbook
            .pivot_tables
            .iter()
            .filter(|p| p.sheet_id == sheet_id)
            .find(|p| {
                let (first, last) = span(&p.location);
                !interval_survives_move(first, last, group_start, group_end, delta)
            })
        {
            return Err(format!(
                "Cannot move {} because that would split the pivot table '{}'",
                if rows { "rows" } else { "columns" },
                pivot.name
            ));
        }
        let count = group_end - group_start + 1;
        for pivot in self.workbook.pivot_tables.iter_mut().filter(on_sheet) {
            let (first, last) = span(&pivot.location);
            let (first, last) =
                displace_interval_for_move(first, last, group_start, group_end, delta, count);
            // The moved span is within the sheet (the move was bounds-checked), so it is positive.
            let (first, last) = (
                u32::try_from(first - 1).unwrap_or(0),
                u32::try_from(last - 1).unwrap_or(0),
            );
            let area = &mut pivot.location;
            if rows {
                (area.top, area.bottom) = (first, last);
            } else {
                (area.left, area.right) = (first, last);
            }
        }
        self.evaluation.pivot_indexes.clear();
        Ok(())
    }

    /// Takes the pivots on the sheet at index `sheet` out of the workbook, for a sheet deletion
    /// that [`Model::restore_pivots`] may undo.
    pub(crate) fn take_pivots_on(&mut self, sheet: u32) -> Result<Vec<PivotTable>, String> {
        let sheet_id = self.workbook.worksheet(sheet)?.sheet_id;
        let (taken, kept) = std::mem::take(&mut self.workbook.pivot_tables)
            .into_iter()
            .partition(|p| p.sheet_id == sheet_id);
        self.workbook.pivot_tables = kept;
        self.evaluation.pivot_indexes.clear();
        Ok(taken)
    }

    /// Puts back pivots taken by [`Model::take_pivots_on`]. Their caches are never removed, so
    /// their cache indices still hold.
    pub(crate) fn restore_pivots(&mut self, pivots: Vec<PivotTable>) {
        self.workbook.pivot_tables.extend(pivots);
        self.evaluation.pivot_indexes.clear();
    }

    /// Rewrites the cache sources on the sheet `old_name` (sheet names ignore case) to
    /// `new_name`, so they keep naming their sheet after a rename.
    pub(crate) fn rename_pivot_sources(&mut self, old_name: &str, new_name: &str) {
        let sources = self.workbook.pivot_caches.iter_mut();
        for source in sources.filter_map(|cache| cache.source.as_mut()) {
            if sheet_names_match(&source.sheet_name, old_name) {
                new_name.clone_into(&mut source.sheet_name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn text(s: &str) -> RecordValue {
        RecordValue::Inline(CacheItem::Text(s.to_string()))
    }

    #[test]
    fn records_are_columnar_and_interned() {
        let mut b = CacheBuilder::new(None, MAX_RECORD_VALUES);
        let region = b.add_field("Region".to_string()).unwrap();
        b.add_field("Sales".to_string()).unwrap();
        b.add_shared_item(region, CacheItem::Text("East".to_string()))
            .unwrap();
        b.push_record(vec![
            RecordValue::Shared(0),
            RecordValue::Inline(CacheItem::Number(5.0)),
        ])
        .unwrap();
        b.push_record(vec![
            text("West"),
            RecordValue::Inline(CacheItem::Number(5.0)),
        ])
        .unwrap();
        b.push_record(vec![
            text("East"),
            RecordValue::Inline(CacheItem::Number(7.0)),
        ])
        .unwrap();
        assert_eq!(b.record_values(), 6);
        let cache = b.finish();
        assert_eq!(cache.record_count, 3);
        assert_eq!(cache.fields[0].records, vec![0, 1, 0]);
        assert_eq!(cache.fields[0].items.len(), 2);
        assert_eq!(cache.fields[1].records, vec![0, 0, 1]);
    }

    #[test]
    fn bad_records_are_refused_and_leave_the_columns_aligned() {
        let mut b = CacheBuilder::new(None, MAX_RECORD_VALUES);
        b.add_field("A".to_string()).unwrap();
        b.add_field("B".to_string()).unwrap();
        assert_eq!(
            b.push_record(vec![text("x")]),
            Err(PivotCacheError::BadRecord)
        );
        assert_eq!(
            b.push_record(vec![text("x"), RecordValue::Shared(3)]),
            Err(PivotCacheError::BadRecord)
        );
        b.push_record(vec![text("x"), text("y")]).unwrap();
        assert_eq!(
            b.add_field("C".to_string()),
            Err(PivotCacheError::BadRecord)
        );
        assert_eq!(
            b.add_shared_item(9, CacheItem::Missing),
            Err(PivotCacheError::BadRecord)
        );
        let cache = b.finish();
        assert_eq!(cache.fields[0].records.len(), 1);
        assert_eq!(cache.fields[1].records.len(), 1);
    }

    #[test]
    fn the_workbook_budget_limits_record_values() {
        let mut b = CacheBuilder::new(None, 3);
        b.add_field("A".to_string()).unwrap();
        b.add_field("B".to_string()).unwrap();
        b.push_record(vec![text("x"), text("y")]).unwrap();
        assert_eq!(
            b.push_record(vec![text("x"), text("y")]),
            Err(PivotCacheError::TooManyRecordValues)
        );
    }

    #[test]
    fn the_hidden_bitset_reads_past_its_size_as_not_hidden() {
        let mut f = PivotField::new(Some(Axis::Row), 70);
        assert_eq!(f.hidden.len(), 2);
        f.hide(65);
        f.hide(500);
        assert!(f.is_hidden(65));
        assert!(!f.is_hidden(64));
        assert!(!f.is_hidden(500));
        assert!(!PivotField::default().is_hidden(0));
    }

    fn pivot_on(sheet_id: u32) -> PivotTable {
        let location = CellArea {
            top: 2,
            left: 0,
            bottom: 6,
            right: 4,
        };
        PivotTable::new(format!("P{sheet_id}"), sheet_id, location, 0)
    }

    /// A model with sheets Sheet1 and Sheet2 and one pivot on each.
    fn model_with_pivots() -> (Model, u32, u32) {
        let mut model = crate::test::util::new_empty_model();
        model.new_sheet();
        let first = model.workbook.worksheets[0].sheet_id;
        let second = model.workbook.worksheets[1].sheet_id;
        model.workbook.pivot_caches.push(PivotCache::default());
        model.workbook.pivot_tables = vec![pivot_on(first), pivot_on(second)];
        (model, first, second)
    }

    fn pivot_sheets(model: &Model) -> Vec<u32> {
        model
            .workbook
            .pivot_tables
            .iter()
            .map(|p| p.sheet_id)
            .collect()
    }

    #[test]
    fn a_new_workbook_has_no_pivots() {
        let model = crate::test::util::new_empty_model();
        assert!(model.workbook.pivot_caches.is_empty());
        assert!(model.workbook.pivot_tables.is_empty());
    }

    fn locations(model: &Model) -> Vec<(u32, [u32; 4])> {
        let pivots = model.workbook.pivot_tables.iter();
        pivots
            .map(|p| {
                let a = &p.location;
                (p.sheet_id, [a.top, a.left, a.bottom, a.right])
            })
            .collect()
    }

    #[test]
    fn displace_span_shifts_drops_or_refuses() {
        // The span is 0-based rows 2..=6; `at` is the 0-based first inserted or deleted line.
        let max = 1_048_575;
        let keep = SpanEdit::Keep;
        // Inserts at or before the first line shift; after the last leave it; inside refuse.
        assert_eq!(displace_span(2, 6, 0, 3, max), keep(5, 9));
        assert_eq!(displace_span(2, 6, 2, 1, max), keep(3, 7));
        assert_eq!(displace_span(2, 6, 7, 1, max), keep(2, 6));
        assert_eq!(displace_span(2, 6, 3, 2, max), SpanEdit::Refuse);
        assert_eq!(displace_span(2, 6, 6, 1, max), SpanEdit::Refuse);
        // Deletes wholly before shift; after leave it; over all of it drop; through it refuse.
        assert_eq!(displace_span(2, 6, 0, -2, max), keep(0, 4));
        assert_eq!(displace_span(2, 6, 7, -3, max), keep(2, 6));
        assert_eq!(displace_span(2, 6, 2, -5, max), SpanEdit::Drop);
        assert_eq!(displace_span(2, 6, 0, -10, max), SpanEdit::Drop);
        assert_eq!(displace_span(2, 6, 2, -1, max), SpanEdit::Refuse);
        assert_eq!(displace_span(2, 6, 0, -3, max), SpanEdit::Refuse);
        assert_eq!(displace_span(2, 6, 4, -10, max), SpanEdit::Refuse);
        // An insert pushing the span off the sheet is refused.
        assert_eq!(displace_span(2, 6, 0, 1_048_569, max), keep(1_048_571, max));
        assert_eq!(displace_span(2, 6, 0, 1_048_570, max), SpanEdit::Refuse);
    }

    #[test]
    fn row_and_column_edits_move_only_that_sheets_pivots() {
        type Edit = fn(&mut Model) -> Result<(), String>;
        // The pivots sit at A3:E7 (0-based rows 2..=6, columns 0..=4); edits are 1-based.
        let edits: [(Edit, [u32; 4]); 6] = [
            (|m| m.insert_rows(0, 1, 2), [4, 0, 8, 4]),
            (|m| m.insert_rows(0, 8, 1), [2, 0, 6, 4]),
            (|m| m.delete_rows(0, 1, 2), [0, 0, 4, 4]),
            (|m| m.delete_rows(0, 8, 4), [2, 0, 6, 4]),
            (|m| m.insert_columns(0, 1, 3), [2, 3, 6, 7]),
            (|m| m.delete_columns(0, 6, 1), [2, 0, 6, 4]),
        ];
        for (edit, moved) in edits {
            let (mut model, first, second) = model_with_pivots();
            edit(&mut model).unwrap();
            assert_eq!(
                locations(&model),
                vec![(first, moved), (second, [2, 0, 6, 4])]
            );
        }
    }

    #[test]
    fn an_edit_through_a_pivot_is_refused_and_changes_nothing() {
        type Edit = fn(&mut Model) -> Result<(), String>;
        let edits: [Edit; 7] = [
            |m| m.move_rows_action(0, 3, 1, 1),
            |m| m.move_columns_action(0, 2, 1, 1),
            |m| m.insert_rows(0, 4, 1),
            |m| m.delete_rows(0, 3, 1),
            |m| m.delete_rows(0, 7, 1),
            |m| m.insert_columns(0, 2, 1),
            |m| m.delete_columns(0, 1, 1),
        ];
        for edit in edits {
            let (mut model, _, _) = model_with_pivots();
            model.set_user_input(0, 5, 2, "x".to_string()).unwrap();
            let before = locations(&model);
            let error = edit(&mut model).unwrap_err();
            assert!(error.contains("pivot table 'P"), "{error}");
            assert_eq!(locations(&model), before);
            assert_eq!(model.get_formatted_cell_value(0, 5, 2).unwrap(), "x");
        }
    }

    #[test]
    fn deleting_every_line_of_a_pivot_drops_it() {
        type Edit = fn(&mut Model) -> Result<(), String>;
        let edits: [Edit; 2] = [|m| m.delete_rows(0, 2, 6), |m| m.delete_columns(0, 1, 5)];
        for edit in edits {
            let (mut model, _, second) = model_with_pivots();
            edit(&mut model).unwrap();
            assert_eq!(pivot_sheets(&model), vec![second]);
            assert_eq!(model.workbook.pivot_caches.len(), 1);
        }
    }

    #[test]
    fn sheet_rename_keeps_the_pivots_and_renames_the_cache_source() {
        let (mut model, first, second) = model_with_pivots();
        model.workbook.pivot_caches[0].source = Some(CacheSource {
            sheet_name: "sheet2".to_string(),
            range: "A1:C7".to_string(),
        });
        model.rename_sheet_by_index(1, "Renamed").unwrap();
        assert_eq!(pivot_sheets(&model), vec![first, second]);
        let source = model.workbook.pivot_caches[0].source.as_ref().unwrap();
        assert_eq!(source.sheet_name, "Renamed");
        model.rename_sheet_by_index(0, "Other").unwrap();
        let source = model.workbook.pivot_caches[0].source.as_ref().unwrap();
        assert_eq!(source.sheet_name, "Renamed");
    }

    #[test]
    fn sheet_delete_takes_its_pivots_and_undo_restores_them() {
        let (mut model, first, second) = model_with_pivots();
        let taken = model.take_sheet(0).unwrap();
        assert_eq!(pivot_sheets(&model), vec![second]);
        model.restore_sheet(0, taken).unwrap();
        assert_eq!(pivot_sheets(&model), vec![second, first]);
        model.delete_sheet(1).unwrap();
        assert_eq!(pivot_sheets(&model), vec![first]);
    }
}
