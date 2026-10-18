use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, model::Model};

use super::array_node_to_calc_result;
use crate::functions::binary_search::Sequence;
use crate::functions::util::compare_values;

/// A two dimensional table for the LOOKUP family of functions. It abstracts over
/// the two possible sources, a range reference or an in-formula array literal, so
/// the lookup logic can be written once for both.
pub(super) enum LookupTable {
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
    Array(Vec<Vec<ArrayNode>>),
}

impl LookupTable {
    pub(super) fn rows(&self) -> i32 {
        match self {
            LookupTable::Range { left, right } => right.row - left.row + 1,
            LookupTable::Array(array) => array.len() as i32,
        }
    }

    pub(super) fn columns(&self) -> i32 {
        match self {
            LookupTable::Range { left, right } => right.column - left.column + 1,
            LookupTable::Array(array) => array.first().map_or(0, |row| row.len()) as i32,
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
pub(crate) struct SearchVector {
    /// The evaluated prefix.
    values: Vec<CalcResult>,
    /// The full length: `values` followed by `len - values.len()` empty cells.
    len: usize,
}

impl SearchVector {
    /// A vector with no virtual tail.
    pub(crate) fn new(values: Vec<CalcResult>) -> SearchVector {
        let len = values.len();
        SearchVector { values, len }
    }

    /// The full length, including the empty tail.
    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// The cells a search actually tests: the evaluated prefix, plus one for an empty tail
    /// (it is all one value, tested once). The cost of one lifted lookup (T-0270).
    pub(crate) fn search_cost(&self) -> usize {
        self.values.len() + usize::from(self.len > self.values.len())
    }

    /// [`SearchVector::search_cost`] as the cells one lifted element is charged
    /// (`LiftCost::Cells`), at least 1.
    pub(crate) fn scan_cells(&self) -> u64 {
        u64::try_from(self.search_cost()).unwrap_or(u64::MAX).max(1)
    }

    /// The value at `index`, which must be below `len`.
    pub(crate) fn get(&self, index: usize) -> &CalcResult {
        self.values.get(index).unwrap_or(&EMPTY_CELL)
    }

    /// The index of the first value that `matches`. The tail is all one
    /// value, so it is tested once.
    pub(super) fn position<F: Fn(&CalcResult) -> bool>(&self, matches: F) -> Option<usize> {
        self.values.iter().position(&matches).or_else(|| {
            (self.len > self.values.len() && matches(&EMPTY_CELL)).then_some(self.values.len())
        })
    }

    /// The indices a linear search must test, in search order. The empty tail
    /// is all one value, so only its first cell in that order is a candidate
    /// (T-0127): its first cell when walking forward, its last when backward.
    pub(crate) fn linear_candidates(&self, forward: bool) -> Vec<usize> {
        let evaluated = self.values.len();
        let tail = (self.len > evaluated).then_some(if forward { evaluated } else { self.len - 1 });
        if forward {
            (0..evaluated).chain(tail).collect()
        } else {
            tail.into_iter().chain((0..evaluated).rev()).collect()
        }
    }

    /// Excel's approximate-match search (VLOOKUP/HLOOKUP with `range_lookup`
    /// TRUE, LOOKUP, MATCH with a positive or negative `match_type`; T-0212).
    /// A binary search that only ever compares values of `target`'s own type:
    /// from each midpoint it skips past blanks, errors and other types (upwards;
    /// when `descending`, downwards to the live range's low bound and then
    /// upwards if nothing is there), and a midpoint with none left in the live
    /// range narrows the search below it. An equal value ends the search at the
    /// edge of its run of adjacent equal values: the last one, or the first when
    /// `descending` (MATCH -1).
    /// Otherwise the result is the last value probed that is smaller than
    /// `target` (larger when `descending`), `None` if there is none.
    /// Excel: see docs/excel-semantics/lookup.md "Approximate match".
    pub(super) fn approximate_search(
        &self,
        target: &CalcResult,
        descending: bool,
    ) -> Option<usize> {
        let target = lookup_key(target);
        let kind = value_kind(target)?;
        let equal = |index: usize| {
            self.values.get(index).is_some_and(|value| {
                value_kind(value) == Some(kind) && compare_values(value, target) == 0
            })
        };
        // Cells in the virtual tail are empty, so the search never enters it.
        let evaluated = self.values.len();
        let mut best = None;
        let mut lo = 0;
        let mut hi = self.len;
        // The live range is lo..hi (hi exclusive); the midpoint is the floor
        // of the inclusive bounds' mean (the lower of two middles).
        while lo < hi {
            let mid = lo + (hi - 1 - lo) / 2;
            // Every scan stays inside the live range, so a round costs O(n) and
            // the O(log n) rounds total O(n log n): a hit far below `mid` may
            // shrink the range by one while the next round rescans the cells
            // between (bounded by the range, never quadratic).
            let of_kind = |index: usize| {
                self.values
                    .get(index)
                    .is_some_and(|value| value_kind(value) == Some(kind))
            };
            let forward = || (mid..hi.min(evaluated)).find(|&index| of_kind(index));
            // A descending search steps backward from the midpoint, an
            // ascending one forward (docs/excel-semantics/lookup.md "MATCH
            // descending search skips backward"; probes/match-descending-argcheck
            // J1-J8). With nothing of the key's type at or below the midpoint it
            // continues forward (gen-m2 b4392a6514a36ade A277:
            // MATCH(-14,{FALSE,0.5},-1) = 2).
            let found = if descending {
                (lo..=mid)
                    .rev()
                    .find(|&index| of_kind(index))
                    .or_else(forward)
            } else {
                forward()
            };
            let Some(m) = found else {
                hi = mid;
                continue;
            };
            match compare_values(&self.values[m], target) {
                0 => {
                    // Linear in the run's length, as Excel's answer needs it.
                    let mut edge = m;
                    if descending {
                        while edge > 0 && equal(edge - 1) {
                            edge -= 1;
                        }
                    } else {
                        while equal(edge + 1) {
                            edge += 1;
                        }
                    }
                    return Some(edge);
                }
                c if (c < 0) != descending => {
                    best = Some(m);
                    lo = m + 1;
                }
                _ => hi = m,
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
        self.values.iter().position(|value| match (value, target) {
            (CalcResult::Number(a), CalcResult::Number(b)) => a == b,
            _ => value_kind(value) == Some(kind) && compare_values(value, target) == 0,
        })
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
        self.len
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
        let values = self.prepare_array(left, &end, down);
        SearchVector {
            values,
            len: usize::try_from(len).unwrap_or(0),
        }
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
