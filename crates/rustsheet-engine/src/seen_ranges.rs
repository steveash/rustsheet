//! The rectangles a pass read as empty in one go (`SeenRange`), each recorded once per root.

use std::collections::HashSet;
use std::ops::Deref;

use crate::evaluation::SeenRange;

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
    /// Whether `range` was recorded already in this pass.
    pub(crate) fn contains(&self, range: &SeenRange) -> bool {
        self.members.contains(range)
    }

    /// Records `range` unless it was recorded already in this pass.
    pub(crate) fn push(&mut self, range: SeenRange) {
        if self.members.insert(range) {
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
