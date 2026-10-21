//! Pivot caches and pivot tables of a loaded workbook (SPEC-0015 "Data model", T-2081).
//!
//! A cache stores its records by column, as indices into each field's interned items, so a record
//! costs 4 bytes per field and no allocation of its own. [`CacheBuilder`] is the one way to build a
//! cache: the xlsx importer (T-2082) and `Txn::add_pivot_table` (T-2084) both use it, and it counts
//! the SPEC-0015 limits before every push, so nothing past them is allocated (CLAUDE.md
//! directive 5). Until T-2086, a structural edit of a sheet drops that sheet's pivots
//! ([`Workbook::drop_pivots_on_sheet`]; docs/excel-semantics/web.md "GETPIVOTDATA against a pivot
//! table").

use std::collections::HashMap;

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use crate::expressions::token::Error;
use crate::model::Model;
use crate::types::Workbook;

/// Most records in one cache (SPEC-0015 "Untrusted-input limits").
pub const MAX_RECORDS: usize = 1_048_576;
/// Most record values (records x fields) in one cache, and over the whole workbook.
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
    /// The cache's fields, in cache order.
    pub fields: Vec<CacheField>,
    /// How many records; every [`CacheField::records`] has this length.
    pub record_count: u32,
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

/// A pivot table on a worksheet.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PivotTable {
    /// The pivot's name.
    pub name: String,
    /// The worksheet's stable `sheet_id`, not its index (api-design.md principle 3).
    pub sheet_id: u32,
    /// `<location ref>`, 0-based.
    pub location: CellArea,
    /// An index into [`Workbook::pivot_caches`].
    pub cache: u32,
    /// One per cache field.
    pub fields: Vec<PivotField>,
    /// `<rowFields>`.
    pub row_fields: Vec<AxisField>,
    /// `<colFields>`.
    pub column_fields: Vec<AxisField>,
    /// `<dataFields>`.
    pub data_fields: Vec<DataField>,
}

/// How a pivot shows one cache field.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct PivotField {
    /// The axis the field is on, if any.
    pub axis: Option<Axis>,
    /// A bitset over the cache field's `items`: bit `i` set means item `i` is hidden.
    pub hidden: Vec<u64>,
}

impl PivotField {
    /// A field on `axis` with a hidden bitset sized for `item_count` items, none hidden.
    #[must_use]
    pub fn new(axis: Option<Axis>, item_count: usize) -> Self {
        PivotField {
            axis,
            hidden: vec![0; item_count.div_ceil(64)],
        }
    }

    /// Hides item `item`; an item past the bitset's size (sized at import) is ignored.
    pub fn hide(&mut self, item: u32) {
        if let Some(word) = self.hidden.get_mut(item as usize / 64) {
            *word |= 1 << (item % 64);
        }
    }

    /// Whether item `item` is hidden; an item past the bitset reads as not hidden.
    #[must_use]
    pub fn is_hidden(&self, item: u32) -> bool {
        self.hidden
            .get(item as usize / 64)
            .is_some_and(|word| word & (1 << (item % 64)) != 0)
    }
}

/// An inclusive, 0-based cell rectangle on one sheet.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct CellArea {
    /// First row.
    pub top: u32,
    /// First column.
    pub left: u32,
    /// Last row.
    pub bottom: u32,
    /// Last column.
    pub right: u32,
}

/// A pivot axis.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Axis {
    /// Rows.
    Row,
    /// Columns.
    Column,
    /// The report filter (page) area.
    Page,
}

/// A `<field x>` of `<rowFields>`/`<colFields>`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AxisField {
    /// A cache field, by index.
    Field(u32),
    /// `x="-2"`: the Values field.
    Values,
}

/// A value field of a pivot.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct DataField {
    /// The caption (`Sum of Sales`).
    pub caption: String,
    /// The cache field it summarizes.
    pub field: u32,
    /// How it summarizes.
    pub function: Aggregate,
}

/// A data field's summary function.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Aggregate {
    /// `sum`.
    Sum,
    /// `count`.
    Count,
    /// `average`.
    Average,
    /// `max`.
    Max,
    /// `min`.
    Min,
    /// `product`.
    Product,
    /// `countNums`.
    CountNums,
    /// `stdDev`.
    StdDev,
    /// `stdDevp`.
    StdDevP,
    /// `var`.
    Var,
    /// `varp`.
    VarP,
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

impl Workbook {
    /// Drops every pivot table on the sheet with stable id `sheet_id`. Structural edits (row and
    /// column insertion and deletion, sheet rename and deletion) call it until T-2086 moves pivots
    /// with the cells: a dropped pivot makes GETPIVOTDATA `#REF!` rather than a wrong answer.
    /// Caches stay, as other pivots may use them.
    pub fn drop_pivots_on_sheet(&mut self, sheet_id: u32) {
        self.pivot_tables.retain(|p| p.sheet_id != sheet_id);
    }
}

impl Model {
    /// [`Workbook::drop_pivots_on_sheet`] for the sheet at index `sheet`.
    pub(crate) fn drop_pivots_on(&mut self, sheet: u32) -> Result<(), String> {
        let sheet_id = self.workbook.worksheet(sheet)?.sheet_id;
        self.workbook.drop_pivots_on_sheet(sheet_id);
        Ok(())
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
        PivotTable {
            name: format!("P{sheet_id}"),
            sheet_id,
            location: CellArea {
                top: 2,
                left: 0,
                bottom: 6,
                right: 4,
            },
            cache: 0,
            fields: Vec::new(),
            row_fields: Vec::new(),
            column_fields: Vec::new(),
            data_fields: Vec::new(),
        }
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

    #[test]
    fn row_and_column_edits_drop_only_that_sheets_pivots() {
        type Edit = fn(&mut Model) -> Result<(), String>;
        let edits: [Edit; 6] = [
            |m| m.insert_rows(0, 1, 1),
            |m| m.delete_rows(0, 1, 1),
            |m| m.insert_columns(0, 1, 1),
            |m| m.delete_columns(0, 1, 1),
            |m| m.move_rows_action(0, 1, 1, 1),
            |m| m.move_columns_action(0, 1, 1, 1),
        ];
        for edit in edits {
            let (mut model, _, second) = model_with_pivots();
            edit(&mut model).unwrap();
            assert_eq!(pivot_sheets(&model), vec![second]);
            assert_eq!(model.workbook.pivot_caches.len(), 1);
        }
    }

    #[test]
    fn sheet_rename_and_delete_drop_that_sheets_pivots() {
        let (mut model, first, second) = model_with_pivots();
        model.rename_sheet_by_index(1, "Renamed").unwrap();
        assert_eq!(pivot_sheets(&model), vec![first]);
        let (mut model, _, second2) = model_with_pivots();
        model.delete_sheet(0).unwrap();
        assert_eq!(pivot_sheets(&model), vec![second2]);
        assert_eq!(second, second2);
    }
}
