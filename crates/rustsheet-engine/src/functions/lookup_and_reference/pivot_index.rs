//! The per-pivot index GETPIVOTDATA answers from (SPEC-0015 "Untrusted-input limits", T-2083).
//!
//! Built lazily, on a pivot's first GETPIVOTDATA call, in one pass over the cache's records: each
//! distinct tuple of axis-item indices keeps its records in cache order, so a call collects the
//! records of the tuples its pairs pick and folds them in cache order (one rule however the
//! records are grouped; SPEC-0015 unknown 11 leaves the summation order to T-2085). A call's
//! answer is memoized per (data field, items). The index and the memo live in the evaluation
//! state, keyed by the pivot's position in `Workbook::pivot_tables`, not on the pivot itself, so
//! `Workbook`'s `PartialEq` and `Clone` see no derived state. Both are dropped at each
//! `Model::evaluate` and whenever pivots are dropped: pivots and caches change only between
//! evaluations, so an index never outlives the pivot it was built from, and the memo holds at
//! most one entry per call made in one evaluation.

use std::collections::HashMap;

use crate::expressions::token::Error;
use crate::functions::statistical::variance::{excel_variance, VarianceKind};
use crate::model::Model;
use crate::pivot::{Aggregate, AxisField, CacheItem, PivotCache, PivotTable};

/// The indices of every pivot used in the current evaluation.
#[derive(Default)]
pub(crate) struct PivotIndexes {
    /// By position in `Workbook::pivot_tables`: code that removes or reorders pivots between
    /// evaluations must call [`PivotIndexes::clear`] (`Model::drop_pivots_on` does).
    pivots: HashMap<usize, PivotIndex>,
    /// The `Evaluation::evaluations` count the indices were built in: a new evaluation drops
    /// them (`Model::pivot_axis`).
    evaluation: u64,
    /// How many indices were built since the model was made: a test counts the passes over the
    /// records.
    builds: u64,
}

impl PivotIndexes {
    /// Drops every index and memo.
    pub(crate) fn clear(&mut self) {
        self.pivots.clear();
    }

    /// How many indices were built so far.
    #[cfg(test)]
    pub(crate) fn builds(&self) -> u64 {
        self.builds
    }
}

/// What a call asks one pivot: the data field, and per axis field (in [`PivotIndex::axis`]
/// order) the item it is restricted to, `None` for every item.
#[derive(Hash, PartialEq, Eq, Clone)]
pub(crate) struct PivotQuery {
    pub(crate) data_field: usize,
    pub(crate) items: Box<[Option<u32>]>,
}

/// How an item name matches a field's items, ignoring case.
#[derive(Clone, Copy)]
enum ItemMatch {
    One(u32),
    /// Two text items differ only in case (`east`, `East`): Excel's pivot holds one item for both;
    /// what GETPIVOTDATA does when the cache keeps two is unprobed, so it is `#REF!`.
    Several,
}

struct PivotIndex {
    /// The cache fields on the row axis, then the column axis.
    axis: Vec<u32>,
    /// Per distinct tuple of axis-item indices, its records in cache order.
    tuples: HashMap<Box<[u32]>, Vec<u32>>,
    /// Per axis position, its text items by lowercased name, built on first use.
    item_names: Vec<Option<HashMap<String, ItemMatch>>>,
    memo: HashMap<PivotQuery, Result<f64, Error>>,
}

/// A pivot GETPIVOTDATA can answer from: one data field, at most one field on each of the row
/// and column axes, no page field and no hidden item. Every other shape is one of SPEC-0015's
/// unknowns (1, 4 and 10), `#REF!` until T-2085 probes it.
fn supported(pivot: &PivotTable) -> Option<Vec<u32>> {
    if pivot.data_fields.len() != 1
        || pivot.row_fields.len() > 1
        || pivot.column_fields.len() > 1
        || pivot.fields.iter().any(|f| {
            f.axis == Some(crate::pivot::Axis::Page) || f.hidden.iter().any(|word| *word != 0)
        })
    {
        return None;
    }
    pivot
        .row_fields
        .iter()
        .chain(&pivot.column_fields)
        .map(|field| match field {
            AxisField::Field(index) => Some(*index),
            AxisField::Values => None,
        })
        .collect()
}

/// One pass over the records: `None` when an axis field or a record's index is not in the cache.
fn build(cache: &PivotCache, axis: Vec<u32>) -> Option<PivotIndex> {
    let columns = axis
        .iter()
        .map(|&f| cache.fields.get(f as usize).map(|f| &f.records))
        .collect::<Option<Vec<_>>>()?;
    let mut tuples: HashMap<Box<[u32]>, Vec<u32>> = HashMap::new();
    let mut key = Vec::with_capacity(axis.len());
    for record in 0..cache.record_count {
        key.clear();
        for column in &columns {
            key.push(*column.get(record as usize)?);
        }
        match tuples.get_mut(key.as_slice()) {
            Some(records) => records.push(record),
            None => {
                tuples.insert(key.as_slice().into(), vec![record]);
            }
        }
    }
    Some(PivotIndex {
        item_names: vec![None; axis.len()],
        axis,
        tuples,
        memo: HashMap::new(),
    })
}

/// The data field's aggregate over `values` (cache order). A Sum over nothing is 0
/// (probes/getpivotdata-real-pivot H13); any other aggregate with no defined value is `#REF!`
/// (SPEC-0015 unknown 2), and a non-finite result `#NUM!` (unknown 16). Var and `StdDev` reuse
/// the worksheet functions' algorithm (`excel_variance`); that the pivot computes them alike is
/// UNVERIFIED (unknown 15).
fn aggregate(function: Aggregate, values: &[f64]) -> Result<f64, Error> {
    let n = values.len();
    let minimum = match function {
        Aggregate::Sum => 0,
        Aggregate::StdDev | Aggregate::Var => 2,
        _ => 1,
    };
    if n < minimum {
        return Err(Error::REF);
    }
    let sum = || values.iter().fold(0.0, |acc, x| acc + x);
    let variance = |kind| excel_variance(values, kind);
    let value = match function {
        Aggregate::Sum => sum(),
        Aggregate::Count | Aggregate::CountNums => n as f64,
        Aggregate::Average => sum() / n as f64,
        Aggregate::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        Aggregate::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        Aggregate::Product => values.iter().fold(1.0, |acc, x| acc * x),
        Aggregate::Var => variance(VarianceKind::Sample),
        Aggregate::VarP => variance(VarianceKind::Population),
        Aggregate::StdDev => variance(VarianceKind::Sample).sqrt(),
        Aggregate::StdDevP => variance(VarianceKind::Population).sqrt(),
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NUM)
    }
}

impl Model {
    /// The axis fields of pivot `pivot` (row axis first), building its index on first use; `None`
    /// when the pivot is not one GETPIVOTDATA answers from ([`supported`]) or its cache is
    /// malformed, both `#REF!`.
    pub(crate) fn pivot_axis(&mut self, pivot: usize) -> Option<&[u32]> {
        let Model {
            workbook,
            evaluation,
            ..
        } = self;
        let indexes = &mut evaluation.pivot_indexes;
        if indexes.evaluation != evaluation.evaluations {
            indexes.pivots.clear();
            indexes.evaluation = evaluation.evaluations;
        }
        if !indexes.pivots.contains_key(&pivot) {
            let table = workbook.pivot_tables.get(pivot)?;
            let cache = workbook.pivot_caches.get(table.cache as usize)?;
            let index = build(cache, supported(table)?)?;
            indexes.builds += 1;
            indexes.pivots.insert(pivot, index);
        }
        indexes
            .pivots
            .get(&pivot)
            .map(|index| index.axis.as_slice())
    }

    /// The item of the field at axis position `position` of pivot `pivot` whose text equals
    /// `name` ignoring case; numbers, dates, booleans and blanks never match (SPEC-0015 unknown
    /// 3). Call [`Model::pivot_axis`] first.
    pub(crate) fn pivot_item(&mut self, pivot: usize, position: usize, name: &str) -> Option<u32> {
        let Model {
            workbook,
            evaluation,
            ..
        } = self;
        let index = evaluation.pivot_indexes.pivots.get_mut(&pivot)?;
        let field = *index.axis.get(position)?;
        let names = index.item_names.get_mut(position)?;
        if names.is_none() {
            let table = workbook.pivot_tables.get(pivot)?;
            let items = &workbook
                .pivot_caches
                .get(table.cache as usize)?
                .fields
                .get(field as usize)?
                .items;
            let mut map = HashMap::new();
            for (item, i) in items.iter().zip(0u32..) {
                if let CacheItem::Text(text) = item {
                    map.entry(text.to_lowercase())
                        .and_modify(|m| *m = ItemMatch::Several)
                        .or_insert(ItemMatch::One(i));
                }
            }
            *names = Some(map);
        }
        match names.as_ref()?.get(&name.to_lowercase())? {
            ItemMatch::One(item) => Some(*item),
            ItemMatch::Several => None,
        }
    }

    /// The answer of pivot `pivot` to `query`, memoized. Every record the query picks must hold
    /// a number in the data field: an error, blank, text, boolean or date there is `#REF!`
    /// (SPEC-0015 unknown 6). Call [`Model::pivot_axis`] first.
    pub(crate) fn pivot_answer(&mut self, pivot: usize, query: &PivotQuery) -> Result<f64, Error> {
        let Model {
            workbook,
            evaluation,
            ..
        } = self;
        let index = evaluation
            .pivot_indexes
            .pivots
            .get_mut(&pivot)
            .ok_or(Error::REF)?;
        if let Some(answer) = index.memo.get(query) {
            return answer.clone();
        }
        let answer = (|| {
            let table = workbook.pivot_tables.get(pivot).ok_or(Error::REF)?;
            let data = table.data_fields.get(query.data_field).ok_or(Error::REF)?;
            let field = workbook
                .pivot_caches
                .get(table.cache as usize)
                .and_then(|cache| cache.fields.get(data.field as usize))
                .ok_or(Error::REF)?;
            let mut records: Vec<u32> = Vec::new();
            for (tuple, tuple_records) in &index.tuples {
                if tuple
                    .iter()
                    .zip(query.items.iter())
                    .all(|(item, wanted)| wanted.is_none_or(|w| w == *item))
                {
                    records.extend_from_slice(tuple_records);
                }
            }
            records.sort_unstable();
            let mut values = Vec::with_capacity(records.len());
            for record in records {
                let item = field
                    .records
                    .get(record as usize)
                    .and_then(|&i| field.items.get(i as usize));
                match item {
                    Some(CacheItem::Number(n)) => values.push(*n),
                    _ => return Err(Error::REF),
                }
            }
            aggregate(data.function, &values)
        })();
        index.memo.insert(query.clone(), answer.clone());
        answer
    }
}
