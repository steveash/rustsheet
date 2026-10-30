//! The per-pass cache of the criteria aggregates (T-0449). SUMIFS, COUNTIFS and their family
//! walked every cell of every criteria range on every call: 2,000 SUMIFS and 2,000 COUNTIFS over
//! 50,000 rows tested 4 * 10^8 criteria. Within one pass a range's cells, once all evaluated,
//! cannot change, so their values are read once, and each criterion's matching cells are found
//! once per range, as a bit set; a call then intersects its criteria's sets and walks the matches.
//!
//! The walks this replaces read lazily: a criteria range's cell is only read when the criteria
//! before it matched, and a sum range's cell only when all of them did, and reading a formula cell
//! evaluates it. A cache that read every cell could evaluate a cell the walk never reaches, closing
//! a cycle or marking a reader circular where the walk would not. So a range is cached only when
//! reading it evaluates nothing: every formula cell of it is already evaluated in this pass
//! (spill cells: their anchor), and none is circular. Until then the walk runs as before, and the
//! check resumes where it stopped (an evaluated cell stays evaluated for the rest of the pass), so
//! it costs each range one scan per pass. A range of formulas the walk only reads where the
//! criteria before it match may stay uncacheable for the whole pass. So a SUMIFS-shaped call whose
//! leading criteria ranges are cached filters by their match sets, and reads the rest, and the sum
//! cells, at the matching positions in row order, as the walk does (T-3227): the reads that can
//! evaluate anything are the walk's own, in its order.
//!
//! The rest of the argument is `search_cache.rs`'s (T-0448): the first read of a cached range
//! records what it saw, so a spill that later writes into it restarts the pass, and every pass
//! starts with an empty cache; nothing is cached in Excel-order passes or outside a pass. Under
//! iterative calculation a range holding a cycle member is not cached (its reader joins the cycle,
//! where the lazy walk may not read it; `Model::join_stack`, T-0485). Reading the whole range
//! records reads the walk may not have made, so a later spill into one of those cells restarts the
//! pass where the walk would not: slower, never a different value. Values read through the cache
//! are the walk's values, and the matches are taken in the walk's order (row by row), so sums round
//! alike and the first error wins alike.
//!
//! A criterion new to a range is still tested on every cell of it, which costs rank and per-row
//! idioms, a new criterion per call, the range's size each call (calipers `pv26_002`: 272,000
//! COUNTIFS over 1,440 rows). From the second criterion a range is tested with, a comparison with
//! a number or a plain text equality finds its matches in an index of the range's values instead
//! ([`ValueIndex`], T-0465): the numbers sorted, binary-searched, and the texts hashed by their
//! collation key. The cells the index does not order (text, booleans, errors and blanks under a
//! number criterion) are still tested by the predicate, so the set is the walk's. A block inside a
//! cached range with the same columns (a growing prefix `I$11:I11`) is read from that range, and
//! its match set is cut from the range's.
//!
//! SUMPRODUCT's `block*mask*...` argument reads a cached block's cells as numbers, the way `*`
//! reads them ([`BlockNumbers`], T-3230), once per block and pass.
//!
//! Directive 5: ranges between [`MIN_CACHED_CELLS`] and [`MAX_CACHED_CELLS`] cells are cached;
//! all cached values of a pass hold at most [`MAX_CACHED_CELLS`] cells and all match sets at most
//! [`MAX_CACHED_BITS`] bits; past that the walks run as before.

use std::collections::HashMap;
use std::mem::Discriminant;
use std::sync::Arc;

use crate::calc_result::{CalcResult, Range};
use crate::evaluation::{key, CellState, FormulaCell, KeyBuild};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::criteria::{
    indexed_criterion, CriteriaFlavor, CriterionOp, IndexedCriterion,
};
use crate::functions::util::collation::{collation_key, CollationKey};
use crate::model::Model;
use crate::number_format::cmp_excel_precision;
use crate::types::Cell;

use super::if_ifs::Criterion;

/// Ranges with fewer cells are cheaper to walk again than to cache.
const MIN_CACHED_CELLS: usize = 64;

/// Cells (values, not bytes) all cached ranges of one pass may hold together; also the largest
/// range cached.
const MAX_CACHED_CELLS: usize = 4_000_000;

/// Bits all match sets of one pass may hold together, their keys' text included (4 MB).
const MAX_CACHED_BITS: usize = 32_000_000;

/// Cached ranges per column span that a block inside them is read from (a bound on the search).
const MAX_PARENTS: usize = 16;

/// Distinct criteria tested on a range by the walk before its values are indexed: an index costs
/// a sort, which one criterion does not repay.
const INDEX_AFTER_MISSES: u32 = 2;

/// A block of one sheet: sheet, top row, left column, bottom row, right column.
type RectKey = (u32, i32, i32, i32, i32);

/// A criterion, as `build_criteria` reads it: two equal keys build the same predicate.
#[derive(PartialEq, Eq, Hash)]
enum CriterionKey {
    Number(u64),
    Text(Vec<u16>),
    Boolean(bool),
    Error(Discriminant<Error>),
}

impl CriterionKey {
    /// What the key holds beyond its fixed size, in bits, charged to the match-set budget.
    fn extra_bits(&self) -> usize {
        match self {
            CriterionKey::Text(units) => units.len().saturating_mul(16),
            _ => 0,
        }
    }

    /// The key of `criterion`; `None` for anything else (an array element is already a scalar).
    fn of(criterion: &CalcResult) -> Option<CriterionKey> {
        Some(match criterion {
            CalcResult::Number(x) => CriterionKey::Number(x.to_bits()),
            CalcResult::String(s) => CriterionKey::Text(s.units().collect()),
            CalcResult::Boolean(b) => CriterionKey::Boolean(*b),
            CalcResult::Error { error, .. } => CriterionKey::Error(std::mem::discriminant(error)),
            // An empty criterion is the criterion 0 (`indexed_criterion`), so it shares 0's set.
            CalcResult::EmptyCell | CalcResult::EmptyArg => CriterionKey::Number(0.0f64.to_bits()),
            _ => return None,
        })
    }
}

/// A range's place in the cache.
enum RangeState {
    /// Its values, row by row.
    Ready(Arc<[CalcResult]>),
    /// Not cacheable yet: the cells before `checked` (row by row) can be read without
    /// evaluating anything; the cell at `checked` could not when last looked at.
    Waiting { checked: usize },
    /// Never cacheable in this pass (a circular cell, or over the budget).
    Never,
}

/// The cells of a cached block read as numbers, as an arithmetic operator reads them
/// (`Model::calc_result_to_number`: a blank is 0, a boolean 1 or 0, a text its number), for
/// SUMPRODUCT's `block*mask*...` argument (T-3230).
pub(crate) struct BlockNumbers {
    /// Row by row; 0 where the cell reads as no number.
    pub(crate) values: Box<[f64]>,
    /// The first cell, row by row, that reads as no number, and its error: the cell's own, or
    /// `#VALUE!` for a text that is no number.
    pub(crate) first_error: Option<(usize, Error)>,
}

/// The values of a cached range ordered for the criteria [`IndexedCriterion`] describes.
struct ValueIndex {
    /// The finite numbers with their positions, by value (`total_cmp`), then position.
    numbers: Vec<(f64, u32)>,
    /// The positions of every other cell, ascending: a number criterion tests them.
    others: Vec<u32>,
    /// The positions of the texts, ascending, by collation key.
    texts: HashMap<CollationKey, Vec<u32>>,
}

/// A range's index: the walk's misses so far, or the index once built.
enum IndexState {
    Counting(u32),
    Ready(Arc<ValueIndex>),
    /// Over the budget.
    Never,
}

/// The cells of a block read from a cached range: `values[offset..offset + len]`, where the
/// cached range is `rect`.
pub(super) struct RangeView {
    rect: RectKey,
    values: Arc<[CalcResult]>,
    offset: usize,
    len: usize,
}

impl RangeView {
    /// The block's `index`-th value, row by row.
    pub(super) fn get(&self, index: usize) -> Option<&CalcResult> {
        if index < self.len {
            self.values.get(self.offset + index)
        } else {
            None
        }
    }
}

/// What [`Model::ifs_leading_matches`] finds: the cells every criteria range before the first one
/// not cacheable yet matches, and from that range on, each range's cached values or `None`.
pub(super) struct LeadingMatches {
    pub(super) matched: Vec<u64>,
    pub(super) rest: Vec<Option<RangeView>>,
}

/// The position of the `index`-th cell of `range`, counting row by row.
pub(super) fn range_cell(range: &Range, index: usize) -> CellReferenceIndex {
    rect_cell(rect_of(range), index)
}

/// The criteria ranges' values and match sets of the current pass.
#[derive(Default)]
pub(crate) struct IfsCache {
    ranges: HashMap<RectKey, RangeState, KeyBuild>,
    matches: HashMap<(RectKey, CriterionKey), Arc<[u64]>>,
    indexes: HashMap<RectKey, IndexState, KeyBuild>,
    numbers: HashMap<RectKey, Arc<BlockNumbers>, KeyBuild>,
    /// The cached ranges of each (sheet, left column, right column), as (top row, bottom row).
    parents: HashMap<(u32, i32, i32), Vec<(i32, i32)>>,
    cells: usize,
    bits: usize,
    /// Match sets cut from a larger cached range's, for the tests of T-0465.
    #[cfg(test)]
    pub(crate) windows: usize,
    /// SUMIFS-shaped calls that read some of their ranges as the walk does, for the tests of
    /// T-3227.
    #[cfg(test)]
    pub(crate) mixed: usize,
    /// SUMPRODUCT arguments read as masked blocks, for the tests of T-3230.
    #[cfg(test)]
    pub(crate) masked: usize,
}

impl IfsCache {
    /// Forgets everything: a new pass, or cells whose state was reset.
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
        self.matches.clear();
        self.indexes.clear();
        self.numbers.clear();
        self.parents.clear();
        self.cells = 0;
        self.bits = 0;
        #[cfg(test)]
        {
            self.windows = 0;
            self.mixed = 0;
            self.masked = 0;
        }
    }

    /// How many ranges were indexed in this pass, for the tests of T-0465.
    #[cfg(test)]
    pub(crate) fn indexed(&self) -> usize {
        self.indexes
            .values()
            .filter(|state| matches!(state, IndexState::Ready(_)))
            .count()
    }

    /// Match-set bits charged in this pass, for the budget test.
    #[cfg(test)]
    pub(crate) fn match_bits(&self) -> (usize, usize) {
        (self.bits, MAX_CACHED_BITS)
    }
}

/// The block `range` covers.
fn rect_of(range: &Range) -> RectKey {
    (
        range.left.sheet,
        range.left.row,
        range.left.column,
        range.right.row,
        range.right.column,
    )
}

/// The position of the `index`-th cell of `rect`, counting row by row.
fn rect_cell(rect: RectKey, index: usize) -> CellReferenceIndex {
    let (sheet, top, left, _, right) = rect;
    let width = usize::try_from(right - left + 1).unwrap_or(1).max(1);
    CellReferenceIndex {
        sheet,
        row: top + i32::try_from(index / width).unwrap_or(0),
        column: left + i32::try_from(index % width).unwrap_or(0),
    }
}

/// How many cells `rect` holds.
fn rect_size(rect: RectKey) -> usize {
    let (_, top, left, bottom, right) = rect;
    let rows = usize::try_from(bottom - top + 1).unwrap_or(0);
    let columns = usize::try_from(right - left + 1).unwrap_or(0);
    rows.saturating_mul(columns)
}

impl Model {
    /// Whether reading `at` evaluates nothing and marks nothing: a constant or a blank, a formula
    /// cell already evaluated in this pass, or a spill cell whose anchor is. `None` for a
    /// circular cell or a cycle member under iterative calculation (whose reader joins it, where
    /// the lazy walk may not read it), which never becomes readable this way.
    fn read_is_inert(&self, at: CellReferenceIndex) -> Option<bool> {
        if self.evaluation.circular.contains(&key(at)) || self.is_cycle_member(at) {
            return None;
        }
        let evaluated = |at: CellReferenceIndex| {
            self.evaluation.cells.get(&key(at)) == Some(&CellState::Evaluated)
        };
        Some(match self.fetch_cell(at) {
            None => true,
            Some(Cell::SpillCell { a, .. }) => {
                let anchor = CellReferenceIndex {
                    sheet: at.sheet,
                    row: a.0,
                    column: a.1,
                };
                if self.is_cycle_member(anchor) {
                    return None;
                }
                matches!(self.fetch_cell(anchor), Some(Cell::ArrayFormula { .. }))
                    && evaluated(anchor)
            }
            Some(cell) => FormulaCell::of(cell).is_none() || evaluated(at),
        })
    }

    /// The values of `rect` row by row, when they can be read without evaluating anything (module
    /// docs), from a cached range that holds it when there is one; `None` when the caller must
    /// walk.
    fn ifs_range_values(&mut self, rect: RectKey) -> Option<RangeView> {
        if !self.pass_cache_safe() {
            return None;
        }
        let size = rect_size(rect);
        if !(MIN_CACHED_CELLS..=MAX_CACHED_CELLS).contains(&size) {
            return None;
        }
        let whole = |rect: RectKey, values: &Arc<[CalcResult]>| RangeView {
            rect,
            values: values.clone(),
            offset: 0,
            len: size,
        };
        let (sheet, top, left, bottom, right) = rect;
        let cache = &self.evaluation.ifs_cache;
        let parent = cache
            .parents
            .get(&(sheet, left, right))
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find(|&&(first, last)| first <= top && bottom <= last)
            })
            .map(|&(first, last)| (sheet, first, left, last, right));
        if let Some(parent) = parent {
            if let Some(RangeState::Ready(values)) = cache.ranges.get(&parent) {
                let rows_above = usize::try_from(top - parent.1).unwrap_or(0);
                let width = usize::try_from(right - left + 1).unwrap_or(1);
                return Some(RangeView {
                    offset: rows_above * width,
                    ..whole(parent, values)
                });
            }
        }
        let checked = match cache.ranges.get(&rect) {
            Some(RangeState::Ready(values)) => return Some(whole(rect, values)),
            Some(RangeState::Never) => return None,
            Some(RangeState::Waiting { checked }) => *checked,
            None => 0,
        };
        for index in checked..size {
            let state = match self.read_is_inert(rect_cell(rect, index)) {
                Some(true) => continue,
                Some(false) => RangeState::Waiting { checked: index },
                None => RangeState::Never,
            };
            self.evaluation.ifs_cache.ranges.insert(rect, state);
            return None;
        }
        if self.evaluation.ifs_cache.cells + size > MAX_CACHED_CELLS {
            self.evaluation
                .ifs_cache
                .ranges
                .insert(rect, RangeState::Never);
            return None;
        }
        // Every read is inert: it records what it saw, as the walk's first read does.
        let values: Arc<[CalcResult]> = (0..size)
            .map(|index| self.evaluate_cell(rect_cell(rect, index)))
            .collect();
        if !self.pass_cache_safe() {
            return None;
        }
        let cache = &mut self.evaluation.ifs_cache;
        cache.cells += size;
        cache.ranges.insert(rect, RangeState::Ready(values.clone()));
        let blocks = cache.parents.entry((sheet, left, right)).or_default();
        if blocks.len() < MAX_PARENTS {
            blocks.push((top, bottom));
        }
        Some(whole(rect, &values))
    }

    /// The cells of the block `left:right` read as numbers ([`BlockNumbers`]), when its values can
    /// be read without evaluating anything (module docs); `None` when the caller must walk.
    pub(crate) fn block_numbers(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    ) -> Option<Arc<BlockNumbers>> {
        if left.sheet != right.sheet || !self.pass_cache_safe() {
            return None;
        }
        let rect = (left.sheet, left.row, left.column, right.row, right.column);
        if let Some(numbers) = self.evaluation.ifs_cache.numbers.get(&rect) {
            return Some(numbers.clone());
        }
        let view = self.ifs_range_values(rect)?;
        if self.evaluation.ifs_cache.cells.saturating_add(view.len) > MAX_CACHED_CELLS {
            return None;
        }
        let mut first_error = None;
        let values = (0..view.len)
            .map(|index| {
                let number = view
                    .get(index)
                    .map_or(Ok(0.0), |value| self.calc_result_to_number(value));
                number.unwrap_or_else(|error| {
                    first_error.get_or_insert((index, error));
                    0.0
                })
            })
            .collect();
        let numbers = Arc::new(BlockNumbers {
            values,
            first_error,
        });
        let cache = &mut self.evaluation.ifs_cache;
        cache.cells += view.len;
        cache.numbers.insert(rect, numbers.clone());
        Some(numbers)
    }

    /// The index of `values` (the cached values of `rect`), once the walk has tested
    /// [`INDEX_AFTER_MISSES`] criteria on it; this call counts as one.
    fn value_index(&mut self, rect: RectKey, values: &[CalcResult]) -> Option<Arc<ValueIndex>> {
        let cache = &mut self.evaluation.ifs_cache;
        let misses = match cache.indexes.get(&rect) {
            Some(IndexState::Ready(index)) => return Some(index.clone()),
            Some(IndexState::Never) => return None,
            Some(IndexState::Counting(misses)) => misses + 1,
            None => 1,
        };
        if misses < INDEX_AFTER_MISSES {
            cache.indexes.insert(rect, IndexState::Counting(misses));
            return None;
        }
        // A position per cell, and a key unit per text unit.
        let text_units: usize = values
            .iter()
            .map(|value| match value {
                CalcResult::String(s) => s.len_utf16(),
                _ => 0,
            })
            .sum();
        let cost = values.len().saturating_add(text_units);
        if cache.cells.saturating_add(cost) > MAX_CACHED_CELLS {
            cache.indexes.insert(rect, IndexState::Never);
            return None;
        }
        cache.cells += cost;
        let index = Arc::new(ValueIndex::of(values));
        cache.indexes.insert(rect, IndexState::Ready(index.clone()));
        Some(index)
    }

    /// The cells of `view` that `matches`, as a bit set: cut from those of the cached range it
    /// reads ([`Model::range_matches`]).
    fn ifs_matches(
        &mut self,
        view: &RangeView,
        criterion: &CalcResult,
        matches: &dyn Fn(&CalcResult) -> bool,
    ) -> Arc<[u64]> {
        let bits = self.range_matches(view.rect, &view.values, criterion, matches);
        if view.offset == 0 && view.len == view.values.len() {
            bits
        } else {
            #[cfg(test)]
            {
                self.evaluation.ifs_cache.windows += 1;
            }
            window(&bits, view.offset, view.len).into()
        }
    }

    /// The cells of `values` (the cached values of `rect`) that `matches`, as a bit set, from the
    /// cache when `criterion` was tested on `rect` before in this pass, else from the index of
    /// `rect` when it has one and the criterion is one it orders.
    fn range_matches(
        &mut self,
        rect: RectKey,
        values: &[CalcResult],
        criterion: &CalcResult,
        matches: &dyn Fn(&CalcResult) -> bool,
    ) -> Arc<[u64]> {
        let key = CriterionKey::of(criterion).map(|k| (rect, k));
        if let Some(bits) = key
            .as_ref()
            .and_then(|k| self.evaluation.ifs_cache.matches.get(k))
        {
            return bits.clone();
        }
        let indexed = self.indexed(criterion);
        let index = indexed
            .as_ref()
            .and_then(|_| self.value_index(rect, values));
        let bits: Arc<[u64]> = match (indexed, index) {
            (Some(indexed), Some(index)) => index.matches(&indexed, values, matches),
            _ => {
                let mut bits = vec![0u64; values.len().div_ceil(64)];
                for (position, value) in values.iter().enumerate() {
                    if matches(value) {
                        set_bit(&mut bits, position);
                    }
                }
                bits
            }
        }
        .into();
        let cache = &mut self.evaluation.ifs_cache;
        if let Some(key) = key {
            // The set's words, the key's text and a word for the entry itself.
            let cost = (bits.len() + 1)
                .saturating_mul(64)
                .saturating_add(key.1.extra_bits());
            if cache.bits.saturating_add(cost) <= MAX_CACHED_BITS {
                cache.bits += cost;
                cache.matches.insert(key, bits.clone());
            }
        }
        bits
    }

    /// The cells (row by row within `rects[0]`'s shape) that every criterion matches, as a bit set:
    /// `rects[k]` is the block of the k-th criteria range the walk would read, `criteria[k]` its
    /// criterion and `matchers[k]` its predicate. `None` when a range is not cacheable yet.
    pub(super) fn ifs_all_matches(
        &mut self,
        rects: &[Range],
        criteria: &[CalcResult],
        matchers: &[Criterion<'_>],
    ) -> Option<Vec<u64>> {
        self.criteria_matches(rects, criteria, matchers, true)
            .map(|leading| leading.matched)
    }

    /// [`Model::ifs_all_matches`] for the criteria ranges up to the first one not cacheable yet,
    /// and the cached values, if any, of each range from that one on (T-3227). `None` when the
    /// first range is not cacheable yet.
    pub(super) fn ifs_leading_matches(
        &mut self,
        rects: &[Range],
        criteria: &[CalcResult],
        matchers: &[Criterion<'_>],
    ) -> Option<LeadingMatches> {
        self.criteria_matches(rects, criteria, matchers, false)
    }

    /// [`Model::ifs_leading_matches`], or with `whole`, [`Model::ifs_all_matches`]: then `None`
    /// at the first range not cacheable yet. Either way no range after an uncached first one is
    /// looked at, and with `whole` none after any uncached one: the walk runs.
    fn criteria_matches(
        &mut self,
        rects: &[Range],
        criteria: &[CalcResult],
        matchers: &[Criterion<'_>],
        whole: bool,
    ) -> Option<LeadingMatches> {
        let mut matched: Option<Vec<u64>> = None;
        let mut rest = Vec::new();
        for ((range, criterion), matches) in rects.iter().zip(criteria).zip(matchers) {
            let view = self.ifs_range_values(rect_of(range));
            if view.is_none() && (whole || matched.is_none()) {
                return None;
            }
            match view {
                Some(view) if rest.is_empty() => {
                    let bits = self.ifs_matches(&view, criterion, matches.as_ref());
                    match matched.as_mut() {
                        None => matched = Some(bits.to_vec()),
                        Some(all) => all.iter_mut().zip(bits.iter()).for_each(|(a, b)| *a &= b),
                    }
                }
                view => rest.push(view),
            }
        }
        Some(LeadingMatches {
            matched: matched?,
            rest,
        })
    }

    /// How many cells of `range` `criterion` matches (`matches` its predicate): COUNTIFS with one
    /// criterion. Counted from the index of the cached range it reads when it has one and the
    /// criterion is one the index orders, else from the match set (which counts towards building
    /// the index). `None` when the range is not cacheable yet.
    pub(super) fn ifs_count(
        &mut self,
        range: &Range,
        criterion: &CalcResult,
        matches: &Criterion<'_>,
    ) -> Option<usize> {
        let view = self.ifs_range_values(rect_of(range))?;
        if let Some(IndexState::Ready(index)) = self.evaluation.ifs_cache.indexes.get(&view.rect) {
            if let Some(indexed) = self.indexed(criterion) {
                let window = view.offset..view.offset + view.len;
                return Some(index.count(&indexed, &view.values, matches.as_ref(), window));
            }
        }
        Some(count_bits(&self.ifs_matches(
            &view,
            criterion,
            matches.as_ref(),
        )))
    }

    /// `criterion` as an [`IndexedCriterion`], read as the criteria aggregates read it.
    fn indexed(&self, criterion: &CalcResult) -> Option<IndexedCriterion> {
        indexed_criterion(
            criterion,
            self.locale,
            &|| self.current_year(),
            self.workbook.settings.date_system,
            CriteriaFlavor::Standard,
        )
    }

    /// The cached values of `range` (`ifs_range_values`) for the SUMIFS-shaped walks' value range.
    pub(super) fn ifs_values(&mut self, range: &Range) -> Option<RangeView> {
        self.ifs_range_values(rect_of(range))
    }
}

impl ValueIndex {
    /// The index of a range's `values`.
    fn of(values: &[CalcResult]) -> ValueIndex {
        let mut numbers = Vec::new();
        let mut others = Vec::new();
        let mut texts: HashMap<CollationKey, Vec<u32>> = HashMap::new();
        // Cached ranges hold at most `MAX_CACHED_CELLS` cells, so a position fits in a `u32`.
        for (position, value) in (0u32..).zip(values) {
            match value {
                CalcResult::Number(x) if x.is_finite() => numbers.push((*x, position)),
                CalcResult::String(s) => {
                    texts.entry(collation_key(s)).or_default().push(position);
                    others.push(position);
                }
                _ => others.push(position),
            }
        }
        numbers.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        ValueIndex {
            numbers,
            others,
            texts,
        }
    }

    /// The cells of `values` (the indexed range's) that `criterion` matches, as a bit set;
    /// `matches` is its predicate, which tests the cells the index does not order.
    fn matches(
        &self,
        criterion: &IndexedCriterion,
        values: &[CalcResult],
        matches: &dyn Fn(&CalcResult) -> bool,
    ) -> Vec<u64> {
        let mut bits = vec![0u64; values.len().div_ceil(64)];
        self.for_each_match(criterion, values, matches, |position| {
            set_bit(&mut bits, position as usize);
        });
        bits
    }

    /// How many cells of `values[window]` (`values` the indexed range's) `criterion` matches;
    /// over the whole range a number criterion is counted from its run, without visiting it.
    fn count(
        &self,
        criterion: &IndexedCriterion,
        values: &[CalcResult],
        matches: &dyn Fn(&CalcResult) -> bool,
        window: std::ops::Range<usize>,
    ) -> usize {
        if let (IndexedCriterion::Number(op, target), true) =
            (criterion, window == (0..values.len()))
        {
            let (low, high) = number_interval(&self.numbers, *op, *target);
            let numbers = if *op == CriterionOp::Ne {
                self.numbers.len() - (high - low)
            } else {
                high - low
            };
            let others = self
                .others
                .iter()
                .filter(|&&position| values.get(position as usize).is_some_and(matches))
                .count();
            return numbers + others;
        }
        let mut count = 0;
        self.for_each_match(criterion, values, matches, |position| {
            if window.contains(&(position as usize)) {
                count += 1;
            }
        });
        count
    }

    /// Calls `visit` with each position of `values` (the indexed range's) that `criterion`
    /// matches, in no particular order; `matches` tests the cells the index does not order.
    fn for_each_match(
        &self,
        criterion: &IndexedCriterion,
        values: &[CalcResult],
        matches: &dyn Fn(&CalcResult) -> bool,
        mut set: impl FnMut(u32),
    ) {
        match criterion {
            // Only a text equals a non-empty text (`result_is_equal_to_string`). Texts equal under
            // the criteria's `cmp_text_pairwise` share a collation key, but not every text of the
            // key is equal to the target (`"a"` plus 16 U+0302 has the key of `"a"`), so `matches`
            // tests each one.
            IndexedCriterion::Text(target) => {
                for &position in self.texts.get(&collation_key(target)).into_iter().flatten() {
                    if values.get(position as usize).is_some_and(matches) {
                        set(position);
                    }
                }
            }
            IndexedCriterion::Number(op, target) => {
                let (low, high) = number_interval(&self.numbers, *op, *target);
                let matched = if *op == CriterionOp::Ne {
                    [&self.numbers[..low], &self.numbers[high..]]
                } else {
                    [&self.numbers[low..high], &[][..]]
                };
                for &(_, position) in matched.into_iter().flatten() {
                    set(position);
                }
                for &position in &self.others {
                    if values.get(position as usize).is_some_and(matches) {
                        set(position);
                    }
                }
            }
        }
    }
}

/// The positions in `numbers` (finite, ascending) of the numbers `op` `target` (for `Ne`, of
/// those equal to it), as `low..high`: each comparison holds on a run of them, since it is
/// monotone in the number. Every comparison is the criteria's 15-digit one, which is monotone
/// (`cmp_excel_precision`; T-3103). The predicates these runs stand for are
/// `IndexedCriterion::matcher`'s (criteria.rs); the tests of T-0465 hold the two together. Always
/// `low <= high <= len`.
fn number_interval(numbers: &[(f64, u32)], op: CriterionOp, target: f64) -> (usize, usize) {
    let first = |below: &dyn Fn(f64) -> bool| numbers.partition_point(|&(x, _)| below(x));
    let below = first(&|x| cmp_excel_precision(x, target).is_lt());
    let up_to = first(&|x| cmp_excel_precision(x, target).is_le());
    let (low, high) = match op {
        CriterionOp::Lt => (0, below),
        CriterionOp::Le => (0, up_to),
        CriterionOp::Gt => (up_to, numbers.len()),
        CriterionOp::Ge => (below, numbers.len()),
        CriterionOp::Eq | CriterionOp::Ne => (below, up_to),
    };
    // Monotone predicates give `low <= high`; the clamp keeps the slices in bounds regardless.
    (low.min(high), high)
}

/// Sets bit `position` of `bits`.
fn set_bit(bits: &mut [u64], position: usize) {
    if let Some(word) = bits.get_mut(position / 64) {
        *word |= 1u64 << (position % 64);
    }
}

/// Bits `offset..offset + len` of `bits`, as a set of their own.
fn window(bits: &[u64], offset: usize, len: usize) -> Vec<u64> {
    let (first, shift) = (offset / 64, offset % 64);
    let word = |index: usize| bits.get(index).copied().unwrap_or(0);
    let mut cut: Vec<u64> = (0..len.div_ceil(64))
        .map(|k| {
            let low = word(first + k) >> shift;
            if shift == 0 {
                low
            } else {
                low | word(first + k + 1) << (64 - shift)
            }
        })
        .collect();
    if !len.is_multiple_of(64) {
        if let Some(last) = cut.last_mut() {
            *last &= (1u64 << (len % 64)) - 1;
        }
    }
    cut
}

/// The indices of the set bits of `bits`, ascending (row by row).
pub(super) fn set_bits(bits: &[u64]) -> impl Iterator<Item = usize> + '_ {
    bits.iter().enumerate().flat_map(|(word_index, &word)| {
        let mut word = word;
        std::iter::from_fn(move || {
            (word != 0).then(|| {
                let bit = word.trailing_zeros() as usize;
                word &= word - 1;
                word_index * 64 + bit
            })
        })
    })
}

/// How many bits of `bits` are set.
pub(super) fn count_bits(bits: &[u64]) -> usize {
    bits.iter().map(|word| word.count_ones() as usize).sum()
}
