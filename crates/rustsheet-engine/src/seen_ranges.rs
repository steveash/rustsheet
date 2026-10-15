//! The rectangles a pass read as empty in one go (`SeenRange`), each recorded once per root.

use std::collections::{HashMap, HashSet};
use std::ops::Deref;

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
/// Each rectangle is also filed once under a row band (`Band`), so that a spill commit visits
/// only the rectangles in the rows it writes, not every rectangle of the pass (T-1545).
#[derive(Default, Debug)]
pub(crate) struct SeenRanges {
    ranges: Vec<SeenRange>,
    members: HashSet<SeenRange>,
    /// The positions in `ranges` of the rectangles filed under each band, in the order read.
    bands: HashMap<Band, Vec<usize>>,
    /// Bit `level` is set when some band of that level holds a rectangle.
    levels: u32,
}

/// A row band: rows `index << level` to `((index + 1) << level) - 1` of `sheet`. A rectangle of
/// height `h` is filed at the smallest level with `h <= 1 << level`, under the band holding its
/// first row, so it lies within that band and the next one: each rectangle is filed once, and a
/// spill over rows `a..=b` finds every rectangle that can meet it in the bands
/// `(a >> level) - 1 ..= b >> level` of each level, about twice its height plus the levels.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Band {
    sheet: u32,
    level: u32,
    index: i32,
}

impl Band {
    fn of(range: &SeenRange) -> Band {
        // `row1 <= row2` (`record_seen_empty_range`), so the height is at least 1. Level 31 holds
        // every row of an `i32` in its bands -1 and 0, whatever the height.
        let height = range.row2.abs_diff(range.row1).saturating_add(1);
        let level = height
            .checked_next_power_of_two()
            .map_or(31, |power| power.trailing_zeros().min(31));
        Band {
            sheet: range.sheet,
            level,
            index: range.row1 >> level,
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
            self.levels |= 1 << band.level;
            self.members.insert(range);
            self.ranges.push(range);
        }
    }

    /// The rectangles `anchor` writes into when it spills over `area` (width, height), each with
    /// the first cell written in it (`SeenRange::first_cell_written`), ordered by that cell and,
    /// at the same cell, in the order read. Only the bands over the spill's rows are visited.
    pub(crate) fn written_by_spill(
        &self,
        anchor: CellReferenceIndex,
        area: (i32, i32),
    ) -> Vec<((i32, i32), CellReferenceIndex)> {
        let first_row = anchor.row;
        let last_row = anchor.row.saturating_add(area.1.max(1) - 1);
        let levels = || (0..32).filter(|level| self.levels & (1 << level) != 0);
        let bands_over = |level: u32| (first_row >> level).saturating_sub(1)..=(last_row >> level);
        let probes = levels().fold(0_u64, |probes, level| {
            let bands = bands_over(level);
            probes + u64::from(bands.end().abs_diff(*bands.start())) + 1
        });
        let mut met: Vec<((i32, i32), usize)> = Vec::new();
        let mut meet = |position: usize| {
            if let Some(cell) = self.ranges[position].first_cell_written(anchor, area) {
                met.push((cell, position));
            }
        };
        // A spill so tall that probing its bands costs more than testing every rectangle on
        // record tests them all instead, so a commit never costs more than before the bands.
        if probes >= self.ranges.len() as u64 {
            (0..self.ranges.len()).for_each(&mut meet);
        } else {
            for level in levels() {
                for index in bands_over(level) {
                    let band = Band {
                        sheet: anchor.sheet,
                        level,
                        index,
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
        self.levels = 0;
    }
}

impl Deref for SeenRanges {
    type Target = [SeenRange];

    fn deref(&self) -> &[SeenRange] {
        &self.ranges
    }
}
