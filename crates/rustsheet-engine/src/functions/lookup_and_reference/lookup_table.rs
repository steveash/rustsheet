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
            LookupTable::Range { left, .. } => model.evaluate_cell(CellReferenceIndex {
                sheet: left.sheet,
                row: left.row + row,
                column: left.column + column,
            }),
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
pub(super) struct SearchVector {
    /// The evaluated prefix.
    values: Vec<CalcResult>,
    /// The full length: `values` followed by `len - values.len()` empty cells.
    len: usize,
}

impl SearchVector {
    /// A vector with no virtual tail.
    pub(super) fn new(values: Vec<CalcResult>) -> SearchVector {
        let len = values.len();
        SearchVector { values, len }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// The value at `index`, which must be below `len`.
    pub(super) fn get(&self, index: usize) -> &CalcResult {
        self.values.get(index).unwrap_or(&EMPTY_CELL)
    }

    /// The value at `index`, or `None` past the end.
    pub(super) fn into_value(mut self, index: usize) -> Option<CalcResult> {
        if index >= self.len {
            return None;
        }
        if index < self.values.len() {
            return Some(self.values.swap_remove(index));
        }
        Some(CalcResult::EmptyCell)
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
    pub(super) fn linear_candidates(&self, forward: bool) -> Vec<usize> {
        let evaluated = self.values.len();
        let tail = (self.len > evaluated).then_some(if forward { evaluated } else { self.len - 1 });
        if forward {
            (0..evaluated).chain(tail).collect()
        } else {
            tail.into_iter().chain((0..evaluated).rev()).collect()
        }
    }

    /// Old style binary search, assuming ascending order. Returns the index of
    /// a matching element or of the largest element smaller than `target`, and
    /// `None` if `target` is smaller than every element.
    // NOTE: We don't know how Excel exactly implements binary search
    // internally: on unsorted values our results and Excel's may differ.
    pub(super) fn binary_search(&self, target: &CalcResult) -> Option<usize> {
        let mut l = 0;
        let mut r = self.len;
        while l < r {
            let m = (l + r) / 2;
            match compare_values(self.get(m), target) {
                -1 => l = m + 1,
                1 => r = m,
                _ => return Some(m),
            }
        }
        l.checked_sub(1)
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
    pub(super) fn range_search_vector(
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
