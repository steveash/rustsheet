//! Why a pass is abandoned (`Restart`) and what the driver learns from it (`RestartLog`); see
//! the module documentation of `evaluation`.

use super::{key, CellKey};
use crate::expressions::types::CellReferenceIndex;

/// Why the current pass is abandoned. In every case the pass starts again,
/// after the driver has learned what the restart says about the order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Restart {
    /// A spill cell of this anchor was read, on behalf of `reader`, before the
    /// anchor had run. The word "stale" means is a leftover from a previous evaluation.
    /// Example:
    /// A1 = SEQUENCE(1,1,5)
    /// B1 = SEQUENCE(3)
    /// The evaluation order in A1, B1, no restarts
    /// Then we switch A1 = SEQUENCE(1,1,B3)
    /// Then A1 reads B3 which is a leftover from the previous evaluation of B1.
    StaleRead {
        anchor: CellReferenceIndex,
        reader: CellReferenceIndex,
    },
    /// What this anchor is about to write contradicts something read on behalf
    /// of `readers`, other cells: the anchor should have run before them.
    /// The same two formulas on a fresh sheet, with nothing spilled yet:
    /// A1 = SEQUENCE(1,1,B3)
    /// B1 = SEQUENCE(3)
    /// While evaluating natural order A1 reads B3 and takes note of that. When B1 evaluates it wants to write
    /// B3 and finds a conflict
    Conflict {
        anchor: CellReferenceIndex,
        readers: Vec<CellReferenceIndex>,
    },
    /// What this anchor is about to write contradicts only reads made on its
    /// own behalf: its inputs depend on its own output. It is circular.
    SelfContradiction(CellReferenceIndex),
    /// The spill cells this anchor is giving up, left by a previous
    /// evaluation, blocked an array evaluated on its own behalf and nothing
    /// else. The anchor's inputs depended on its history, not on its output:
    /// the cells are dropped for the rest of the evaluation.
    /// An example will be:
    ///   A1 = 1
    ///   B2 = IF(A1=1, SEQUENCE(2,2), IF(ISERROR(C1), 5, 7))
    ///   C1 = SEQUENCE(3)
    /// The evaluation order in A1, B2, C1, no restarts. C1 is #SPILL!
    /// Now we set A1 = 0. A2 will read C1 and force it's evaluation that will still be blocked by the stale spill from the previous C1 evaluation.
    /// The evaluation of B2 will want to delete C2 and C3 which are the stale cells.
    StaleCells(CellReferenceIndex, Vec<CellKey>),
}

impl Restart {
    fn anchor(&self) -> CellReferenceIndex {
        match self {
            Restart::StaleRead { anchor, .. }
            | Restart::Conflict { anchor, .. }
            | Restart::SelfContradiction(anchor)
            | Restart::StaleCells(anchor, _) => *anchor,
        }
    }
}

/// The driver's memory of one evaluation: the facts learned about the order,
/// and the anchors marked circular.
///
/// Every pass starts from the same sheet, so a pass is a function of the
/// anchor order and of the circular set. A restart says that some readers
/// read an anchor's area before it ran: a fact, "the anchor runs before each
/// of them", that the current order breaks. The driver records the fact and
/// repairs the order so that every fact holds, moving the anchor and what
/// must precede it to just before the reader (`learn`). Since the order
/// always respects the known facts, every restart adds a fact that was not
/// known; there are at most `n(n-1)` of them. When a fact would close a
/// cycle, the anchors on the cycle read each other's areas and no order can
/// serve them: they are marked circular, at most `n` times.
/// A marked anchor runs no formula and, its stale cells dropped, writes and
/// blocks nothing, so it never restarts again. Stale-cell restarts drop at
/// least one cell each. Hence the loop ends, with no cap needed.
pub(super) struct RestartLog {
    /// Anchors marked circular so far; passed to every pass.
    pub(super) circular: Vec<CellKey>,
    /// `(a, r)`: `a` runs before `r`. Learned from restarts; never broken by
    /// the order; acyclic.
    facts: Vec<(CellReferenceIndex, CellReferenceIndex)>,
    /// Every restart of this evaluation.
    pub(super) restarts: u32,
}

impl RestartLog {
    pub(super) fn new() -> Self {
        RestartLog {
            circular: Vec::new(),
            facts: Vec::new(),
            restarts: 0,
        }
    }

    /// Records a restart: learns its facts, marks what it proves circular and
    /// reorders `order` to respect the facts. Returns the anchors newly
    /// marked circular.
    pub(super) fn record(
        &mut self,
        restart: &Restart,
        order: &mut Vec<CellReferenceIndex>,
    ) -> Vec<CellReferenceIndex> {
        self.restarts += 1;
        let anchor = restart.anchor();
        let readers: Vec<CellReferenceIndex> = match restart {
            Restart::StaleRead { reader, .. } => vec![*reader],
            Restart::Conflict { readers, .. } => readers.clone(),
            Restart::SelfContradiction(_) => vec![],
            Restart::StaleCells(..) => return Vec::new(),
        };
        let mut newly_circular: Vec<CellReferenceIndex> = Vec::new();
        if let Restart::SelfContradiction(_) = restart {
            self.mark(anchor, &mut newly_circular);
        }
        for reader in readers {
            if self.circular.contains(&key(anchor)) {
                break;
            }
            if self.circular.contains(&key(reader)) {
                // Marked while an earlier reader was learned: on a loop,
                // nothing to order it against.
                continue;
            }
            self.learn(anchor, reader, order, &mut newly_circular);
        }
        newly_circular
    }

    /// Learns that `anchor` runs before `reader`, one restart's fact. If the
    /// facts already place `reader` before `anchor`, the two are on a loop of
    /// anchors reading each other's areas, with everything the facts place
    /// between them: all of it is marked. Otherwise the fact is kept and the
    /// order repaired: `anchor`, and whatever the facts place before it, move
    /// from behind `reader` to just before it, in their present order.
    fn learn(
        &mut self,
        anchor: CellReferenceIndex,
        reader: CellReferenceIndex,
        order: &mut Vec<CellReferenceIndex>,
        newly_circular: &mut Vec<CellReferenceIndex>,
    ) {
        if reader == anchor {
            return;
        }
        let before_anchor = self.reachable(anchor, |a, r| (r, a));
        if before_anchor.contains(&reader) {
            let after_reader = self.reachable(reader, |a, r| (a, r));
            for cell in before_anchor {
                if after_reader.contains(&cell) {
                    self.mark(cell, newly_circular);
                }
            }
            return;
        }
        self.facts.push((anchor, reader));
        let Some(reader_position) = order.iter().position(|c| *c == reader) else {
            return;
        };
        let moved: Vec<CellReferenceIndex> = order[reader_position + 1..]
            .iter()
            .copied()
            .filter(|cell| before_anchor.contains(cell))
            .collect();
        order.retain(|cell| !moved.contains(cell));
        order.splice(reader_position..reader_position, moved);
    }

    /// Marks an anchor circular. A marked anchor neither writes nor reads:
    /// its facts are spent.
    fn mark(&mut self, cell: CellReferenceIndex, newly_circular: &mut Vec<CellReferenceIndex>) {
        if self.circular.contains(&key(cell)) {
            return;
        }
        self.circular.push(key(cell));
        newly_circular.push(cell);
        self.facts.retain(|(a, r)| *a != cell && *r != cell);
    }

    /// Everything reachable from `start` along the facts, `start` included,
    /// with `edge` giving the direction to follow.
    fn reachable(
        &self,
        start: CellReferenceIndex,
        edge: fn(
            CellReferenceIndex,
            CellReferenceIndex,
        ) -> (CellReferenceIndex, CellReferenceIndex),
    ) -> Vec<CellReferenceIndex> {
        let mut seen = vec![start];
        let mut todo = vec![start];
        while let Some(cell) = todo.pop() {
            for (a, r) in &self.facts {
                let (from, to) = edge(*a, *r);
                if from == cell && !seen.contains(&to) {
                    seen.push(to);
                    todo.push(to);
                }
            }
        }
        seen
    }
}
