//! The rectangles a pass read as empty in one go (`SeenRange`), each recorded once per root.

use std::collections::HashSet;
use std::ops::Deref;

use crate::area_budget::AreaBudget;
use crate::evaluation::CellKey;
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
    /// Whether the position `(sheet, row, column)` lies inside the rectangle.
    pub(crate) fn contains(&self, (sheet, row, column): CellKey) -> bool {
        sheet == self.sheet
            && (self.row1..=self.row2).contains(&row)
            && (self.column1..=self.column2).contains(&column)
    }
}

/// The rectangles read as empty in the current pass, in the order first read, without repeats:
/// a union walked once per row of a MAP reads the same rectangles again and again (T-1497), and
/// every spill commit is checked against all of them, so the check stays proportional to the
/// distinct rectangles read, not to how often they were read. The set uses the default hasher,
/// not `KeyBuild`: the rectangles come from formulas (`OFFSET`), so their keys are chosen by the
/// workbook (directive 5).
#[derive(Default, Debug)]
pub(crate) struct SeenRanges {
    ranges: Vec<SeenRange>,
    members: HashSet<SeenRange>,
}

impl SeenRanges {
    /// Records `range` unless it was recorded already in this pass. Only a new rectangle is paid
    /// for from the formula's `budget`; past it the rectangle goes unrecorded and the formula is
    /// `#VALUE!` (`Model::with_area_budget`).
    pub(crate) fn push(&mut self, range: SeenRange, budget: &mut AreaBudget) {
        if !self.members.contains(&range) && budget.spend_seen_range() {
            self.members.insert(range);
            self.ranges.push(range);
        }
    }

    /// Forgets every rectangle, for a new pass.
    pub(crate) fn clear(&mut self) {
        self.ranges.clear();
        self.members.clear();
    }
}

impl Deref for SeenRanges {
    type Target = [SeenRange];

    fn deref(&self) -> &[SeenRange] {
        &self.ranges
    }
}
