//! The rectangles a pass read as empty in one go (`SeenRange`), each recorded once per root.

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, RangeInclusive};

use crate::area_budget::AreaBudget;
use crate::expressions::types::CellReferenceIndex;

/// A rectangle read as empty on behalf of `root`, in one go. A function that walks a whole
/// column or row visits only the sheet's used area; the rest holds nothing, and the formula
/// depends on it all the same. That remainder is one record, not one per cell.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct SeenRange {
    pub(crate) root: CellReferenceIndex,
    pub(crate) sheet: u32,
    pub(crate) row1: i32,
    pub(crate) column1: i32,
    pub(crate) row2: i32,
    pub(crate) column2: i32,
}

impl SeenRange {
    /// The first cell inside the rectangle, in row-major order, that `anchor` writes when it
    /// spills over `area` (width, height): every cell of the area but the anchor's own. A spill is
    /// tested against a rectangle once, by intersecting the two, not cell by cell (T-1512).
    pub(crate) fn first_cell_written(
        &self,
        anchor: CellReferenceIndex,
        (width, height): (i32, i32),
    ) -> Option<(i32, i32)> {
        if self.sheet != anchor.sheet {
            return None;
        }
        let row1 = self.row1.max(anchor.row);
        let row2 = self.row2.min(anchor.row + height - 1);
        let column1 = self.column1.max(anchor.column);
        let column2 = self.column2.min(anchor.column + width - 1);
        if row1 > row2 || column1 > column2 {
            None
        } else if (row1, column1) != (anchor.row, anchor.column) {
            Some((row1, column1))
        } else if column1 < column2 {
            Some((row1, column1 + 1))
        } else if row1 < row2 {
            Some((row1 + 1, column1))
        } else {
            None
        }
    }
}

/// The rectangles read as empty in the current pass, in the order first read, without repeats:
/// a union walked once per row of a MAP reads the same rectangles again and again (T-1497), and
/// every spill commit is checked against them, so the check stays proportional to the
/// distinct rectangles read, not to how often they were read. The set and the buckets use the
/// default hasher, not `KeyBuild`: the rectangles come from formulas (`OFFSET`), so their keys
/// are chosen by the workbook (directive 5).
///
/// Each rectangle is also filed once under a band of rows and columns (`Band`), so that a spill
/// commit visits only the rectangles near the cells it writes, not every rectangle of the pass
/// (T-1545), nor every whole-column rectangle in other columns (T-1646).
#[derive(Default, Debug)]
pub(crate) struct SeenRanges {
    ranges: Vec<SeenRange>,
    members: HashSet<SeenRange>,
    /// The positions in `ranges` of the rectangles filed under each band, in the order read.
    bands: HashMap<Band, Vec<usize>>,
    /// Bit `column_level` of entry `row_level` is set when some band of those levels holds a
    /// rectangle.
    levels: [u32; 32],
}

/// A band of rows and columns of `sheet`: rows `row_index << row_level` to
/// `((row_index + 1) << row_level) - 1`, and likewise for columns. Along each axis a rectangle of
/// extent `n` is filed at the smallest level with `n <= 1 << level`, under the band holding its
/// first row (column), so it lies within that band and the next one: each rectangle is filed once,
/// and a spill finds every rectangle that can meet it in the bands `bands_over` its rows and
/// columns of each pair of levels in use.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Band {
    sheet: u32,
    row_level: u32,
    row_index: i32,
    column_level: u32,
    column_index: i32,
}

/// The level and band index, along one axis, of a rectangle over `first..=last`.
fn level_and_index(first: i32, last: i32) -> (u32, i32) {
    // `first <= last` (`record_seen_empty_range`), so the extent is at least 1. Level 31 holds
    // every index of an `i32` in its bands -1 and 0, whatever the extent.
    let extent = last.abs_diff(first).saturating_add(1);
    let level = extent
        .checked_next_power_of_two()
        .map_or(31, |power| power.trailing_zeros().min(31));
    (level, first >> level)
}

/// The bands of `level`, along one axis, that can hold a rectangle reaching into `first..=last`:
/// those over it and the one before, whose rectangles can reach into the next band. A level-0
/// rectangle is one row (column) wide and never does.
fn bands_over(level: u32, (first, last): (i32, i32)) -> RangeInclusive<i32> {
    let start = first >> level;
    let start = if level == 0 {
        start
    } else {
        start.saturating_sub(1)
    };
    start..=(last >> level)
}

impl Band {
    fn of(range: &SeenRange) -> Band {
        let (row_level, row_index) = level_and_index(range.row1, range.row2);
        let (column_level, column_index) = level_and_index(range.column1, range.column2);
        Band {
            sheet: range.sheet,
            row_level,
            row_index,
            column_level,
            column_index,
        }
    }
}

impl SeenRanges {
    /// Records `range` unless it was recorded already in this pass. Only a new rectangle is paid
    /// for from the formula's `budget`; past it the rectangle goes unrecorded and the formula is
    /// `#VALUE!` (`Model::with_area_budget`).
    pub(crate) fn push(&mut self, range: SeenRange, budget: &mut AreaBudget) {
        if !self.members.contains(&range) && budget.spend_seen_range() {
            let band = Band::of(&range);
            self.bands.entry(band).or_default().push(self.ranges.len());
            self.levels[band.row_level as usize] |= 1 << band.column_level;
            self.members.insert(range);
            self.ranges.push(range);
        }
    }

    /// The rectangles `anchor` writes into when it spills over `area` (width, height), each with
    /// the first cell written in it (`SeenRange::first_cell_written`), ordered by that cell and,
    /// at the same cell, in the order read. Only the bands over the spill's cells are visited.
    pub(crate) fn written_by_spill(
        &self,
        anchor: CellReferenceIndex,
        area: (i32, i32),
    ) -> Vec<((i32, i32), CellReferenceIndex)> {
        let rows = (anchor.row, anchor.row.saturating_add(area.1.max(1) - 1));
        let columns = (
            anchor.column,
            anchor.column.saturating_add(area.0.max(1) - 1),
        );
        // The pairs of levels in use, walking only the set bits of each row level's mask.
        let level_pairs = || {
            (0..32_u32).flat_map(move |row_level| {
                let mut column_levels = self.levels[row_level as usize];
                std::iter::from_fn(move || {
                    if column_levels == 0 {
                        return None;
                    }
                    let column_level = column_levels.trailing_zeros();
                    column_levels &= column_levels - 1;
                    Some((row_level, column_level))
                })
            })
        };
        let count =
            |bands: RangeInclusive<i32>| u64::from(bands.end().abs_diff(*bands.start())) + 1;
        let probes = level_pairs().fold(0_u64, |probes, (row_level, column_level)| {
            probes.saturating_add(
                count(bands_over(row_level, rows))
                    .saturating_mul(count(bands_over(column_level, columns))),
            )
        });
        let mut met: Vec<((i32, i32), usize)> = Vec::new();
        let mut meet = |position: usize| {
            if let Some(cell) = self.ranges[position].first_cell_written(anchor, area) {
                met.push((cell, position));
            }
        };
        // A spill so large that probing its bands costs more than testing every rectangle on
        // record tests them all instead, so a commit never costs more than before the bands.
        if probes >= self.ranges.len() as u64 {
            (0..self.ranges.len()).for_each(&mut meet);
        } else {
            for (row_level, column_level) in level_pairs() {
                for row_index in bands_over(row_level, rows) {
                    for column_index in bands_over(column_level, columns) {
                        let band = Band {
                            sheet: anchor.sheet,
                            row_level,
                            row_index,
                            column_level,
                            column_index,
                        };
                        self.bands
                            .get(&band)
                            .into_iter()
                            .flatten()
                            .copied()
                            .for_each(&mut meet);
                    }
                }
            }
        }
        met.sort_unstable();
        met.into_iter()
            .map(|(cell, position)| (cell, self.ranges[position].root))
            .collect()
    }

    /// Forgets every rectangle, for a new pass.
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
        self.members.clear();
        self.bands.clear();
        self.levels = [0; 32];
    }
}

impl Deref for SeenRanges {
    type Target = [SeenRange];

    fn deref(&self) -> &[SeenRange] {
        &self.ranges
    }
}
