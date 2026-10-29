use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::external_reference::ExternalArea;
use crate::{calc_result::CalcResult, model::Model};

use super::array_node_to_calc_result;
use crate::functions::binary_search::Sequence;
use crate::functions::criteria::{folded_units, WildcardPattern};
use crate::functions::util::collation::{collation_key, CollationKey};
use crate::functions::util::compare_values;
use crate::number_format::excel_precision_key;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use std::sync::{Arc, OnceLock};

/// A two dimensional table for the LOOKUP family of functions. It abstracts over
/// its possible sources, a range reference, an in-formula array literal or a range of a
/// linked workbook's cache, so the lookup logic can be written once for all.
pub(super) enum LookupTable {
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
    Array(Vec<Vec<ArrayNode>>),
    /// A range of a linked workbook's cached sheet, read in place: a whole column or row keeps
    /// its full length, its cells past the cache empty (`Model::cached_external_lookup_area`).
    External(ExternalArea),
}

impl LookupTable {
    pub(super) fn rows(&self) -> i32 {
        match self {
            LookupTable::Range { left, right } => right.row - left.row + 1,
            LookupTable::Array(array) => array.len() as i32,
            LookupTable::External(area) => area.last_row - area.first_row + 1,
        }
    }

    pub(super) fn columns(&self) -> i32 {
        match self {
            LookupTable::Range { left, right } => right.column - left.column + 1,
            LookupTable::Array(array) => array.first().map_or(0, |row| row.len()) as i32,
            LookupTable::External(area) => area.last_column - area.first_column + 1,
        }
    }

    /// The cell at the 0-based `(row, column)` offset as a 1x1 reference when the table is a
    /// range and `as_reference` is set (an aggregate argument sees it so when the lookup's index
    /// argument is a cell reference, T-0295), else its value.
    /// Callers must ensure the offsets are within bounds.
    pub(super) fn pick(
        &self,
        model: &mut Model,
        row: i32,
        column: i32,
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        match self {
            LookupTable::Range { left, .. } if as_reference => {
                let picked = offset_cell(left, row, column);
                CalcResult::Range {
                    left: picked,
                    right: picked,
                }
            }
            _ => self.get(model, row, column, cell),
        }
    }

    /// Returns the value at the 0-based `(row, column)` offset within the table.
    /// Callers must ensure the offsets are within bounds.
    pub(super) fn get(
        &self,
        model: &mut Model,
        row: i32,
        column: i32,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self {
            LookupTable::Range { left, .. } => model.evaluate_cell(offset_cell(left, row, column)),
            LookupTable::Array(array) => {
                array_node_to_calc_result(&array[row as usize][column as usize], cell)
            }
            LookupTable::External(area) => model.external_area_cell(*area, row, column, cell),
        }
    }

    /// The first row of the table: the search vector for HLOOKUP.
    pub(super) fn first_row(&self, model: &mut Model, cell: CellReferenceIndex) -> SearchVector {
        match self {
            LookupTable::Range { left, right } => {
                let end = CellReferenceIndex {
                    row: left.row,
                    ..*right
                };
                model.range_search_vector(left, &end, false)
            }
            LookupTable::External(area) => model.external_search_vector(*area, false, cell),
            LookupTable::Array(_) => SearchVector::new(
                (0..self.columns())
                    .map(|column| self.get(model, 0, column, cell))
                    .collect(),
            ),
        }
    }

    /// The first column of the table: the search vector for VLOOKUP.
    pub(super) fn first_column(&self, model: &mut Model, cell: CellReferenceIndex) -> SearchVector {
        match self {
            LookupTable::Range { left, right } => {
                let end = CellReferenceIndex {
                    column: left.column,
                    ..*right
                };
                model.range_search_vector(left, &end, true)
            }
            LookupTable::External(area) => model.external_search_vector(*area, true, cell),
            LookupTable::Array(_) => SearchVector::new(
                (0..self.rows())
                    .map(|row| self.get(model, row, 0, cell))
                    .collect(),
            ),
        }
    }
}

/// What every cell past the used area of a sheet evaluates to.
static EMPTY_CELL: CalcResult = CalcResult::EmptyCell;

/// A one-dimensional list of cell values searched by the lookup functions
/// (MATCH, LOOKUP, HLOOKUP, VLOOKUP).
///
/// A range keeps its full length, but only its cells up to the sheet's last
/// used row/column are evaluated: every cell after that is empty, so the rest
/// is a virtual run of `EmptyCell`s. A whole-column reference would otherwise
/// evaluate a million cells per call (CLAUDE.md directive 5). Every search sees
/// exactly the values the full walk would (binary searches probe the same
/// positions, an exact match on an empty value finds the first empty tail
/// cell), so the result never depends on where the used area ends.
///
/// Cloning shares the values: the per-pass search cache (`search_cache.rs`, T-0448) hands the
/// same vector to every lookup into one range, and the exact-match indexes built on its second
/// search serve them all.
#[derive(Clone)]
pub(crate) struct SearchVector {
    data: Arc<SearchData>,
}

struct SearchData {
    /// The evaluated prefix.
    values: Vec<CalcResult>,
    /// The full length: `values` followed by `len - values.len()` empty cells.
    len: usize,
    /// Exact searches made so far; the indexes are built on the second (a single search is
    /// cheaper as a walk).
    searches: AtomicU32,
    /// First and last position of each value under MATCH's and VLOOKUP's exact equality.
    exact: OnceLock<ExactIndex>,
    /// First and last position of each value under XMATCH's and XLOOKUP's exact equality.
    collated: OnceLock<CollatedIndex>,
}

/// What an indexed exact match found.
pub(crate) enum Indexed {
    /// The index applied: the match's position, or `None` when nothing matches.
    Searched(Option<usize>),
    /// The index does not apply; walk the candidates.
    Unindexed,
}

/// The fewest evaluated values worth indexing: a shorter walk costs about what a lookup does.
const MIN_INDEXED: usize = 16;

/// A value's key under MATCH, VLOOKUP and HLOOKUP's exact match (`exact_position`, and
/// `WildcardPattern` for text without wildcards): numbers equal exactly (-0 is 0, NaN equals
/// nothing), text by its case-folded units, booleans by value. Blanks and errors have none.
#[derive(PartialEq, Eq, Hash)]
enum ExactKey {
    Number(u64),
    Text(Vec<u16>),
    Boolean(bool),
}

impl ExactKey {
    fn of(value: &CalcResult) -> Option<ExactKey> {
        match value {
            CalcResult::Number(x) if !x.is_nan() => Some(ExactKey::Number((x + 0.0).to_bits())),
            CalcResult::String(s) => Some(ExactKey::Text(folded_units(s))),
            CalcResult::Boolean(b) => Some(ExactKey::Boolean(*b)),
            _ => None,
        }
    }
}

/// A value's key under XMATCH and XLOOKUP's exact match (`compare_values` is 0): numbers on
/// their 15-digit values, text under Excel's collation, booleans by value. Blanks and errors
/// have none: a blank only matches a blank lookup value and an error matches nothing.
#[derive(PartialEq, Eq, Hash)]
enum CollatedKey {
    Number(u64),
    Text(CollationKey),
    Boolean(bool),
}

impl CollatedKey {
    fn of(value: &CalcResult) -> Option<CollatedKey> {
        match value {
            CalcResult::Number(x) => Some(CollatedKey::Number(excel_precision_key(*x))),
            CalcResult::String(s) => Some(CollatedKey::Text(collation_key(s))),
            CalcResult::Boolean(b) => Some(CollatedKey::Boolean(*b)),
            _ => None,
        }
    }
}

/// First and last position of every key in the evaluated prefix. The empty tail never matches a
/// keyed value, so it needs no entry.
struct PositionIndex<K>(HashMap<K, (usize, usize)>);

type ExactIndex = PositionIndex<ExactKey>;
type CollatedIndex = PositionIndex<CollatedKey>;

impl<K: Eq + Hash> PositionIndex<K> {
    fn build(values: &[CalcResult], key: impl Fn(&CalcResult) -> Option<K>) -> Self {
        let mut map: HashMap<K, (usize, usize)> = HashMap::new();
        for (index, value) in values.iter().enumerate() {
            if let Some(k) = key(value) {
                map.entry(k)
                    .and_modify(|(_, last)| *last = index)
                    .or_insert((index, index));
            }
        }
        PositionIndex(map)
    }

    fn find(&self, key: &K, forward: bool) -> Option<usize> {
        self.0
            .get(key)
            .map(|&(first, last)| if forward { first } else { last })
    }
}

impl SearchVector {
    /// A vector with no virtual tail.
    pub(crate) fn new(values: Vec<CalcResult>) -> SearchVector {
        let len = values.len();
        SearchVector::with_len(values, len)
    }

    /// `values` followed by an empty tail up to `len`.
    pub(super) fn with_len(values: Vec<CalcResult>, len: usize) -> SearchVector {
        SearchVector {
            data: Arc::new(SearchData {
                values,
                len,
                searches: AtomicU32::new(0),
                exact: OnceLock::new(),
                collated: OnceLock::new(),
            }),
        }
    }

    /// Whether this search should go through an index: the vector has been searched before and
    /// is long enough to make a walk dearer than a hash lookup.
    fn index_worthwhile(&self) -> bool {
        let earlier = self.data.searches.fetch_add(1, AtomicOrdering::Relaxed);
        earlier >= 1 && self.data.values.len() >= MIN_INDEXED
    }

    /// The exact match of MATCH (type 0), VLOOKUP and HLOOKUP (`range_lookup` FALSE): text is
    /// matched as a wildcard pattern (`WildcardPattern`), anything else by `exact_position`.
    /// From the second search on, a pattern without wildcards and a non-text value are looked up
    /// in an index with the same equality instead of walked (T-0448).
    pub(super) fn exact_match(&self, target: &CalcResult) -> Option<usize> {
        let pattern = match target {
            CalcResult::String(s) => Some(WildcardPattern::new(s)),
            _ => None,
        };
        if self.index_worthwhile() {
            let key = match &pattern {
                Some(pattern) => pattern.literal_units().map(ExactKey::Text),
                None => ExactKey::of(lookup_key(target)),
            };
            if let Some(key) = key {
                let index = self
                    .data
                    .exact
                    .get_or_init(|| PositionIndex::build(&self.data.values, ExactKey::of));
                return index.find(&key, true);
            }
        }
        match pattern {
            Some(pattern) => self.position(|value| pattern.matches_result(value)),
            None => self.exact_position(target),
        }
    }

    /// XMATCH's and XLOOKUP's exact match (match mode 0) of a non-blank, non-error `target`
    /// through an index, `forward` for the first match and else the last;
    /// [`Indexed::Unindexed`] when the index does not apply (first search, short vector, or a
    /// target without a key) and the caller walks the candidates instead (T-0448).
    pub(crate) fn collated_match(&self, target: &CalcResult, forward: bool) -> Indexed {
        let Some(key) = CollatedKey::of(target) else {
            return Indexed::Unindexed;
        };
        if !self.index_worthwhile() {
            return Indexed::Unindexed;
        }
        let index = self
            .data
            .collated
            .get_or_init(|| PositionIndex::build(&self.data.values, CollatedKey::of));
        Indexed::Searched(index.find(&key, forward))
    }

    /// The full length, including the empty tail.
    pub(super) fn len(&self) -> usize {
        self.data.len
    }

    /// The cells a search actually tests: the evaluated prefix, plus one for an empty tail
    /// (it is all one value, tested once). The cost of one lifted lookup (T-0270).
    pub(crate) fn search_cost(&self) -> usize {
        self.data.values.len() + usize::from(self.data.len > self.data.values.len())
    }

    /// [`SearchVector::search_cost`] as the cells one lifted element is charged
    /// (`LiftCost::Cells`), at least 1.
    pub(crate) fn scan_cells(&self) -> u64 {
        u64::try_from(self.search_cost()).unwrap_or(u64::MAX).max(1)
    }

    /// The value at `index`, which must be below `len`.
    pub(crate) fn get(&self, index: usize) -> &CalcResult {
        self.data.values.get(index).unwrap_or(&EMPTY_CELL)
    }

    /// The index of the first value that `matches`. The tail is all one
    /// value, so it is tested once.
    pub(super) fn position<F: Fn(&CalcResult) -> bool>(&self, matches: F) -> Option<usize> {
        self.data.values.iter().position(&matches).or_else(|| {
            (self.data.len > self.data.values.len() && matches(&EMPTY_CELL))
                .then_some(self.data.values.len())
        })
    }

    /// The indices a linear search must test, in search order. The empty tail
    /// is all one value, so only its first cell in that order is a candidate
    /// (T-0127): its first cell when walking forward, its last when backward.
    pub(crate) fn linear_candidates(&self, forward: bool) -> Vec<usize> {
        let evaluated = self.data.values.len();
        let tail = (self.data.len > evaluated).then_some(if forward {
            evaluated
        } else {
            self.data.len - 1
        });
        if forward {
            (0..evaluated).chain(tail).collect()
        } else {
            tail.into_iter().chain((0..evaluated).rev()).collect()
        }
    }

    /// Excel's ascending approximate-match search (VLOOKUP/HLOOKUP with
    /// `range_lookup` TRUE, LOOKUP, MATCH with a positive `match_type`; T-0212).
    /// A binary search that only ever compares values of `target`'s own type:
    /// from each midpoint it skips forward past blanks, errors and other types,
    /// and a midpoint with none left in the live range narrows the search below
    /// it. An equal value ends the search at the last of its run of adjacent
    /// equal values. Otherwise the result is the last value probed that is
    /// smaller than `target`, `None` if there is none.
    /// Excel: see docs/excel-semantics/lookup.md "Approximate match".
    pub(super) fn approximate_search(&self, target: &CalcResult) -> Option<usize> {
        let target = lookup_key(target);
        let kind = value_kind(target)?;
        let equal = |index: usize| {
            self.data.values.get(index).is_some_and(|value| {
                value_kind(value) == Some(kind) && compare_searched(value, target) == 0
            })
        };
        // Cells in the virtual tail are empty, so the search never enters it.
        let evaluated = self.data.values.len();
        let mut best = None;
        let mut lo = 0;
        let mut hi = self.data.len;
        // The live range is lo..hi (hi exclusive); the midpoint is the floor
        // of the inclusive bounds' mean (the lower of two middles).
        while lo < hi {
            let mid = lo + (hi - 1 - lo) / 2;
            // Every scan stays inside the live range, so a round costs O(n) and
            // the O(log n) rounds total O(n log n).
            let found = (mid..hi.min(evaluated)).find(|&index| {
                self.data
                    .values
                    .get(index)
                    .is_some_and(|value| value_kind(value) == Some(kind))
            });
            let Some(m) = found else {
                hi = mid;
                continue;
            };
            match compare_searched(&self.data.values[m], target) {
                0 => {
                    // Linear in the run's length, as Excel's answer needs it.
                    let mut edge = m;
                    while equal(edge + 1) {
                        edge += 1;
                    }
                    return Some(edge);
                }
                c if c < 0 => {
                    best = Some(m);
                    lo = m + 1;
                }
                _ => hi = m,
            }
        }
        best
    }

    /// Excel's descending search (MATCH with a negative `match_type`): not a
    /// binary search but a scan from the top over the values of `target`'s own
    /// type (blanks, errors and other types are passed over). It ends at the
    /// first value equal to `target`, or before the first one smaller than it,
    /// giving the last value seen that is larger; `None` if the first value of
    /// the type is already smaller, or there is none.
    /// Excel: docs/excel-semantics/lookup.md "MATCH descending search scans
    /// from the top" (probes/lookup-match-descending-blank: over the unsorted
    /// 0.5,0.3,0,-2.5,2.5,.. a key 0 gives 3 and 0.0001 gives 2).
    pub(super) fn descending_search(&self, target: &CalcResult) -> Option<usize> {
        let target = lookup_key(target);
        let kind = value_kind(target)?;
        let mut best = None;
        for (index, value) in self.data.values.iter().enumerate() {
            if value_kind(value) != Some(kind) {
                continue;
            }
            match compare_searched(value, target) {
                0 => return Some(index),
                c if c > 0 => best = Some(index),
                _ => break,
            }
        }
        best
    }

    /// Exact-match search (`range_lookup` FALSE, MATCH 0) for a non-text
    /// `target`: the first value of its own type equal to it. A blank never
    /// matches, a blank `target` is 0, and numbers must be exactly equal:
    /// `MATCH(0.1+0.2,{0.3},0)` is #N/A (probes/lookup-approximate-search
    /// N9, N32, N37, N42; T-0212).
    pub(super) fn exact_position(&self, target: &CalcResult) -> Option<usize> {
        let target = lookup_key(target);
        let kind = value_kind(target)?;
        self.data
            .values
            .iter()
            .position(|value| match (value, target) {
                (CalcResult::Number(a), CalcResult::Number(b)) => a == b,
                _ => value_kind(value) == Some(kind) && compare_values(value, target) == 0,
            })
    }
}

/// How the approximate and descending searches order a value against the key: numbers as exact
/// doubles, not at 15 digits like `compare_values` and the comparison operators, so a table number
/// a few ULP from the key is above or below it, never equal (`MATCH(0.3,{2;0.1+0.2;0.5},1)` is #N/A;
/// docs/excel-semantics/lookup.md "Approximate match", probes/t3073-lookup-near-equal-approximate;
/// T-3099). Text and booleans compare as `compare_values` does.
fn compare_searched(value: &CalcResult, target: &CalcResult) -> i32 {
    match (value, target) {
        (CalcResult::Number(a), CalcResult::Number(b)) => match a.partial_cmp(b) {
            Some(Ordering::Less) => -1,
            Some(Ordering::Greater) => 1,
            _ => 0,
        },
        _ => compare_values(value, target),
    }
}

/// The value a lookup searches for: a blank lookup value, or an empty value such as
/// `IFERROR(<blank>,..)` (T-0291), is the number 0.
fn lookup_key(target: &CalcResult) -> &CalcResult {
    static ZERO: CalcResult = CalcResult::Number(0.0);
    if matches!(target, CalcResult::EmptyCell | CalcResult::EmptyArg) {
        &ZERO
    } else {
        target
    }
}

/// The type classes the lookup searches compare within: numbers, text and
/// booleans. Blanks and errors belong to none, so they never match.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueKind {
    Number,
    Text,
    Boolean,
}

/// Whether the approximate search can stop on `value`: a number, text or a boolean. It skips
/// every other value (a blank, an error) whatever it searches for.
pub(super) fn is_searched_kind(value: &CalcResult) -> bool {
    value_kind(value).is_some()
}

fn value_kind(value: &CalcResult) -> Option<ValueKind> {
    match value {
        CalcResult::Number(_) => Some(ValueKind::Number),
        CalcResult::String(_) => Some(ValueKind::Text),
        CalcResult::Boolean(_) => Some(ValueKind::Boolean),
        _ => None,
    }
}

impl Sequence<CalcResult> for SearchVector {
    fn seq_len(&self) -> usize {
        self.data.len
    }
    fn at(&self, index: usize) -> &CalcResult {
        self.get(index)
    }
}

impl Model {
    /// The search vector of the single row or column `left:right`, walking
    /// down if `down` is set. Only the cells up to the sheet's last used
    /// row/column are evaluated (see `SearchVector`).
    pub(crate) fn range_search_vector(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        down: bool,
    ) -> SearchVector {
        let (len, end) = if down {
            let row = self.clip_search_rows(left.sheet, left.row, right.row, left.column);
            (
                right.row - left.row + 1,
                CellReferenceIndex { row, ..*right },
            )
        } else {
            let column = self.clip_search_columns(left.sheet, left.column, right.column, left.row);
            (
                right.column - left.column + 1,
                CellReferenceIndex { column, ..*right },
            )
        };
        self.cached_search_vector(left, &end, usize::try_from(len).unwrap_or(0), down)
    }

    /// The last row of `row1..=row2` worth evaluating down `column`: rows
    /// below the sheet's last used row are empty. The skipped cells are
    /// recorded as seen empty, so a later spill into them restarts the pass.
    /// `Model::clip_to_used_area` does the same for 2-D walks that truncate
    /// (XMATCH, XLOOKUP, ...; T-0127 moves XMATCH here).
    fn clip_search_rows(&mut self, sheet: u32, row1: i32, row2: i32, column: i32) -> i32 {
        let last_used = self
            .workbook
            .worksheet(sheet)
            .map_or(LAST_ROW, |ws| ws.last_used_row());
        let last = row2.min(last_used);
        self.record_seen_empty_range(sheet, last.max(row1 - 1) + 1, column, row2, column);
        last
    }

    /// The column counterpart of `clip_search_rows`, walking along `row`. Only
    /// ranges that run to the last column are clipped: finding the last used
    /// column visits every row, which would dominate a short range.
    fn clip_search_columns(&mut self, sheet: u32, column1: i32, column2: i32, row: i32) -> i32 {
        if column2 < LAST_COLUMN {
            return column2;
        }
        let last_used = self
            .workbook
            .worksheet(sheet)
            .map_or(LAST_COLUMN, |ws| ws.last_used_column());
        let last = column2.min(last_used);
        self.record_seen_empty_range(sheet, row, last.max(column1 - 1) + 1, row, column2);
        last
    }
}

/// The cell `(row, column)` cells (0-based) below and right of `left`.
fn offset_cell(left: &CellReferenceIndex, row: i32, column: i32) -> CellReferenceIndex {
    CellReferenceIndex {
        sheet: left.sheet,
        row: left.row + row,
        column: left.column + column,
    }
}
