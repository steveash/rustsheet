//! Evaluation of every formula in the workbook.
//!
//! The algorithm in short:
//!
//! * A **pass** is the plain top-down recursion: to evaluate a cell, evaluate
//!   the cells it reads as it meets them. A cell met while it is already being
//!   evaluated closes a cycle. Dynamic array anchors are evaluated first, in a
//!   remembered order, then every cell of every sheet.
//! * A dynamic array writes cells other than its own, and which ones is only
//!   known once it has run. Anything that read one of those positions earlier
//!   in the pass read the wrong thing. So the pass records what formulas saw
//!   at the positions they read, and when an anchor is about to contradict a
//!   record, the pass is **abandoned and started again with that anchor
//!   before the cells that read it**. Each restart is a fact learned about
//!   the sheet, "this anchor runs before that reader"; the driver keeps the
//!   facts and orders the anchors by them. A restart can only happen when
//!   the order breaks a fact not yet known, so there are at most `n²` of
//!   them, and a loop of anchors reading each other's areas shows up as a
//!   cycle among the facts. An anchor whose own inputs read its area is
//!   contradicted by itself wherever it sits: it is circular.
//! * Spill cells left by a previous evaluation are never read as values
//!   before their anchor has run in the current pass; they still block other
//!   arrays, which is how the array that spilled first keeps its cells. The
//!   one thing they cannot do is stand in for the anchor's output: when an
//!   anchor gives them up and only its own inputs were blocked by them, they
//!   were history, not a dependency. They are dropped for the rest of the
//!   evaluation and the pass starts again without them.
//! * The recursion is bounded, in formulas and in stack. When it would go
//!   deeper it **unwinds**: the formulas in progress end without storing
//!   anything but stay on the stack, still `Evaluating`. The outermost one
//!   then **replays** the stack from the top, the deepest cell first, as often
//!   as it takes. A long chain of formulas therefore costs memory and about
//!   one extra run per cell instead of overflowing the stack of the thread.
//!
//! The order of the anchors survives across evaluations, so a sheet that
//! needed restarts once evaluates in a single pass afterwards.
//!
//! Nothing here depends on the order in which cells are visited, only the
//! amount of work does, but for the state Excel's calls leave in its calculation order
//! (`OrderState`): a pass in which a call read it is followed by one in Excel's order
//! (`excel_order.rs`).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::calc_result::{range_dims, CalcResult};
use crate::circular::circular_reference;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::date_and_time::weeknum::WeeknumStart;
use crate::limits::{spill_past_grid_error, spills_past_grid};
use crate::model::Model;
use crate::restart_log::RestartLog;
use crate::seen_ranges::SeenRange;
use crate::types::{ArrayKind, Cell};

/// What a function call leaves behind for the calls Excel makes after it in the same recalc,
/// read by calls whose value therefore depends on Excel's calculation order (`excel_order.rs`). A
/// pass starts it empty, and the Excel-order chain puts it back when a run stops.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct OrderState {
    /// WEEKNUM's stale `return_type` state (SPEC-0017).
    pub(crate) weeknum_start: Option<WeeknumStart>,
    /// The value AMORDEGRC with a period strictly between 0 and 1 returns: what the last
    /// AMORDEGRC or AMORLINC left (financial.md "AMORDEGRC below period 1", decision 0131).
    pub(crate) amor_left_over: f64,
}

/// The hasher of the maps the evaluation (and the facade's diff) keep about positions. Their
/// keys are three small integers we produce ourselves, and every formula looks several up, which
/// made the default hasher, built to resist chosen keys, a fifth of the cost of an evaluation.
/// This one multiplies and rotates, a word at a time.
#[derive(Default, Clone, Copy)]
pub struct KeyHasher(u64);

impl std::hash::Hasher for KeyHasher {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u32(*byte as u32);
        }
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = (self.0.rotate_left(5) ^ i as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
    }

    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub type KeyBuild = std::hash::BuildHasherDefault<KeyHasher>;

/// `(sheet, row, column)`: a position in the workbook.
pub(crate) type CellKey = (u32, i32, i32);

pub(crate) fn key(cell: CellReferenceIndex) -> CellKey {
    (cell.sheet, cell.row, cell.column)
}

/// What the evaluation needs to know about a formula cell in order to run it
/// and to store its result: everything but its value. A few numbers, copied
/// out of the cell, so that the sheet is not borrowed while the formula runs
/// and the cell does not have to be cloned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct FormulaCell {
    /// Index of the formula in the parsed formulas of the sheet.
    pub(crate) f: i32,
    /// Style index.
    pub(crate) s: i32,
    /// For the anchor of an array: whether it is dynamic, and the area
    /// `(width, height)` it occupies now.
    pub(crate) array: Option<(bool, (i32, i32))>,
}

impl FormulaCell {
    /// `None` if the cell does not hold a formula.
    pub(crate) fn of(cell: &Cell) -> Option<FormulaCell> {
        match cell {
            Cell::CellFormula { f, s, .. } => Some(FormulaCell {
                f: *f,
                s: *s,
                array: None,
            }),
            Cell::ArrayFormula { f, s, r, kind, .. } => Some(FormulaCell {
                f: *f,
                s: *s,
                array: Some((matches!(kind, ArrayKind::Dynamic), *r)),
            }),
            _ => None,
        }
    }
}

/// The state of a formula cell within the current pass; absent means not evaluated yet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CellState {
    Evaluating,
    Evaluated,
}

/// What a formula found at a position when it read it, as far as spills are concerned. A spill
/// written into a position seen `Empty`, or a spill cell removed from one seen `Occupied`,
/// contradicts the reader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Seen {
    Empty,
    Occupied,
}

/// What was seen at one position in the current pass: the root of the recursion that first read
/// it as empty, and the root of the recursion on whose behalf an array was first blocked by it.
/// Both can be set for the same position: a leftover spill cell of the anchor being evaluated is
/// read as empty on its behalf, and then blocks an array evaluated on demand inside it. Keeping
/// only the first record would let the anchor remove the cell without contradicting the blocked
/// array.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(crate) struct SeenRecord {
    pub(crate) empty: Option<CellReferenceIndex>,
    pub(crate) occupied: Option<CellReferenceIndex>,
}

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
    pub(crate) fn anchor(&self) -> CellReferenceIndex {
        match self {
            Restart::StaleRead { anchor, .. }
            | Restart::Conflict { anchor, .. }
            | Restart::SelfContradiction(anchor)
            | Restart::StaleCells(anchor, _) => *anchor,
        }
    }
}

/// Everything the evaluation needs besides the workbook itself.
pub(crate) struct Evaluation {
    /// The dynamic anchors, in the order in which they are evaluated. Kept
    /// across evaluations: the order the restarts arrived at stays, so the
    /// next evaluation needs no restart. New anchors are appended in natural
    /// `(sheet, row, column)` order.
    pub(crate) anchor_order: Vec<CellReferenceIndex>,
    /// Formula cells touched in the current pass, and their state. Whatever clears this map or
    /// removes an entry must also call `clear_pass_caches`: the caches hold values read under
    /// these states (T-0448, T-0449).
    pub(crate) cells: HashMap<CellKey, CellState, KeyBuild>,
    /// Cells being evaluated, innermost last.
    pub(crate) stack: Vec<CellReferenceIndex>,
    /// The cell the driver is evaluating: the root of the current recursion,
    /// on whose behalf every read of the pass is recorded.
    pub(crate) root: Option<CellReferenceIndex>,
    /// Cells known to be circular, or to read a circular cell: each keeps its
    /// value from before the recalculation (`Model::circular_value`; recalc.md "A
    /// circular reference without iteration keeps the cells' values"). Marked on the
    /// stack when a read closes a loop or reads a circular cell, or by the
    /// driver when an anchor's spill contradicts its own inputs; the latter
    /// survive restarts within one evaluation.
    pub(crate) circular: HashSet<CellKey, KeyBuild>,
    /// The anchors the driver marked circular (`RestartLog`): they run no
    /// formula. A cell marked in the pass still runs, so that its reads are
    /// made, and recorded, whatever the order the pass reaches it in.
    pub(crate) circular_anchors: HashSet<CellKey, KeyBuild>,
    /// How many cells at the bottom of `stack` are already in `circular`, so
    /// that marking the stack costs only the cells pushed since.
    pub(crate) circular_below: usize,
    /// What formulas saw at the positions they read in the current pass, and
    /// the root of the recursion the read was made in (the cell the driver was
    /// evaluating at the time), one record per kind.
    pub(crate) seen: HashMap<CellKey, SeenRecord, KeyBuild>,
    /// Rectangles read as empty in one go, with their root (`SeenRange`).
    pub(crate) seen_ranges: crate::seen_ranges::SeenRanges,
    /// What the data tables read when they were last recomputed (`data_table_memo.rs`).
    pub(crate) data_table_memo: crate::data_table_memo::DataTableMemo,
    /// Set when the current pass must be abandoned. Every evaluation in
    /// progress then returns without storing anything.
    pub(crate) restart: Option<Restart>,
    /// True while `evaluate()` is running a pass. Outside a pass (conditional
    /// formatting at load time, formula helpers) there is no driver to restart
    /// anything, so cells are simply evaluated on demand.
    pub(crate) in_pass: bool,
    /// Number of restarts the last `evaluate()` needed. For tests and tuning.
    pub(crate) restarts_in_last_evaluation: u32,
    /// Number of `evaluate()` calls so far.
    pub(crate) evaluations: u64,
    /// Number of cells read through `Model::evaluate_cell` so far: the work counter of the
    /// tests that bound how many cells a walk visits (T-2711).
    pub(crate) cell_reads: u64,
    /// Number of formulas running right now, that is the depth of the
    /// recursion. It can be smaller than `stack.len()`: see `unwinding`.
    pub(crate) depth: usize,
    /// The deepest the recursion is allowed to go, in formulas.
    pub(crate) max_depth: usize,
    /// The most stack the recursion is allowed to use, in bytes, measured from
    /// the outermost formula. How much one formula needs varies a lot, with
    /// the formula and with the build, so a number of formulas alone does not
    /// bound the stack.
    pub(crate) max_stack: usize,
    /// Where the stack was when the outermost formula started.
    pub(crate) stack_base: usize,
    /// Set when the recursion got too deep. Like a pending restart, every
    /// evaluation in progress then returns without storing anything; unlike a
    /// restart, the cells stay on `stack`, still `Evaluating`, to be run again
    /// from the top once the recursion has unwound.
    pub(crate) unwinding: bool,
    /// The lookup search vectors built in this pass (`search_cache.rs`, T-0448). Dropped by
    /// `clear_pass_caches`.
    pub(crate) search_cache: crate::functions::lookup_and_reference::search_cache::SearchCache,
    /// The criteria aggregates' range values and match sets of this pass (`ifs_cache.rs`, T-0449),
    /// dropped by `clear_pass_caches`.
    pub(crate) ifs_cache: crate::functions::statistical::ifs_cache::IfsCache,
    /// Reads of a formula cell that was still `Evaluating` (a cycle), or of the spill area of
    /// a running anchor, so far: a search vector whose build made one is not cached.
    pub(crate) evaluating_reads: u64,
    /// GETPIVOTDATA's per-pivot indices and memo, dropped at each `evaluate()` (`pivot_index.rs`).
    pub(crate) pivot_indexes: crate::functions::lookup_and_reference::pivot_index::PivotIndexes,
    /// What Excel's function calls leave for later ones in its calculation order (`OrderState`).
    pub(crate) order_state: OrderState,
    /// Whether a call read that state in this pass, so the pass's values depend on its order.
    pub(crate) order_state_read: bool,
    /// Passes run in Excel's fresh-load order (`excel_order.rs`): set once a pass had a reader.
    pub(crate) excel_order: bool,
    /// In that order's chain, a read of a formula cell not yet evaluated stops the root's run and
    /// is recorded in `blocked_on`, instead of evaluating the cell.
    pub(crate) blocking: bool,
    pub(crate) blocked_on: Option<CellReferenceIndex>,
    /// Number of passes the last `evaluate()` ran, restarted ones included. For tests.
    pub(crate) passes_in_last_evaluation: u32,
    /// Number of data-table recomputations so far (`data_tables.rs`). For tests.
    pub(crate) data_tables_recomputed: u64,
    /// Circular references under iterative calculation (`iteration.rs`).
    pub(crate) iteration: crate::iteration::IterationState,
    /// Turns the pass caches off, so a test can compare cached values with the walk's.
    #[cfg(test)]
    pub(crate) no_pass_caches: bool,
}

/// How deep the recursion may go when nobody has said otherwise, in formulas
/// and in bytes of stack. Measured on a chain of `=A2+1`: a formula takes about
/// 3 KB of stack in a release build and 16 KB in a debug build, and three to
/// four times that when it nests a few functions. The limits are far below
/// what the smallest common stacks hold (1 MB in WebAssembly, 2 MB for a
/// thread) and cost nothing to workbooks that never reach them.
const DEFAULT_MAX_DEPTH: usize = 64;
const DEFAULT_MAX_STACK: usize = 256 * 1024;

/// Roughly where the stack is: the address of a local.
#[inline(never)]
pub(crate) fn stack_address() -> usize {
    let marker = 0u8;
    std::hint::black_box(&marker) as *const u8 as usize
}

/// The limit a new model starts with. Under test it can be set from outside,
/// `IRONCALC_MAX_DEPTH=1 cargo test`, so that every test of the crate runs
/// through the unwind and the replay.
fn default_max_depth() -> usize {
    #[cfg(test)]
    if let Some(max_depth) = std::env::var("IRONCALC_MAX_DEPTH")
        .ok()
        .and_then(|value| value.parse().ok())
    {
        return max_depth;
    }
    DEFAULT_MAX_DEPTH
}

impl Evaluation {
    /// Drops what the caches of this pass hold: called wherever `cells` is cleared or loses an
    /// entry.
    pub(crate) fn clear_pass_caches(&mut self) {
        self.search_cache.clear();
        self.ifs_cache.clear();
    }
}

impl Default for Evaluation {
    fn default() -> Self {
        Evaluation {
            anchor_order: Vec::new(),
            cells: HashMap::default(),
            stack: Vec::new(),
            root: None,
            circular: HashSet::default(),
            circular_anchors: HashSet::default(),
            circular_below: 0,
            seen: HashMap::default(),
            seen_ranges: crate::seen_ranges::SeenRanges::default(),
            data_table_memo: Default::default(),
            restart: None,
            in_pass: false,
            restarts_in_last_evaluation: 0,
            evaluations: 0,
            cell_reads: 0,
            depth: 0,
            max_depth: default_max_depth(),
            max_stack: DEFAULT_MAX_STACK,
            stack_base: 0,
            unwinding: false,
            search_cache: Default::default(),
            ifs_cache: Default::default(),
            evaluating_reads: 0,
            pivot_indexes: Default::default(),
            order_state: OrderState::default(),
            order_state_read: false,
            excel_order: false,
            blocking: false,
            blocked_on: None,
            passes_in_last_evaluation: 0,
            data_tables_recomputed: 0,
            iteration: Default::default(),
            #[cfg(test)]
            no_pass_caches: false,
        }
    }
}

impl Model {
    /// Whether the caches of this pass (`search_cache.rs`, `ifs_cache.rs`) may be read or filled
    /// now: inside a pass that is neither abandoned, unwinding, blocked nor in Excel order, so a
    /// value read now stays the pass's value. Under iterative calculation too: a cached range holds
    /// no cycle member (`Model::join_stack`, T-0485). The sweeps run outside a pass.
    pub(crate) fn pass_cache_safe(&self) -> bool {
        let evaluation = &self.evaluation;
        #[cfg(test)]
        if evaluation.no_pass_caches {
            return false;
        }
        evaluation.in_pass
            && evaluation.restart.is_none()
            && !evaluation.unwinding
            && !evaluation.excel_order
            && !evaluation.blocking
    }

    /// Evaluates every formula in the workbook, then its data tables and, if there are any,
    /// everything again so that their readers see the results (`data_tables.rs`). With
    /// iterative calculation on, each round's circular references are then iterated
    /// (`iteration.rs`).
    pub fn evaluate(&mut self) {
        self.evaluation.evaluations += 1;
        self.begin_iteration();
        self.evaluate_passes();
        self.iterate_cycles(true);
        if self.evaluate_data_tables() {
            self.restart_iteration_round();
            self.evaluate_passes();
            self.iterate_cycles(false);
        }
        self.end_iteration();
        // Conditional formats are evaluated on the next style read (SPEC-0021).
        self.cf.stale = true;
    }

    /// Runs passes until one completes without a restart. `RestartLog` learns
    /// from each restart, reorders the anchors and marks what it proves
    /// circular; see there for why the loop ends. A marked anchor stores
    /// `#CIRC!` and keeps no spill cells: its stale cells are dropped when it
    /// is marked, so there is nothing of it to read or to contradict.
    fn evaluate_passes(&mut self) {
        self.sync_anchor_order();
        // Every pass starts from the same sheet: what an abandoned pass wrote
        // is undone. This is what makes a pass a function of the anchor order.
        // The only changes to that sheet within an evaluation are the dropping
        // of stale spill cells: an anchor's own when it gave them up, a marked
        // anchor's when it is marked.
        let mut spills_before = self.dynamic_spills();
        let mut log = RestartLog::new();
        self.evaluation.passes_in_last_evaluation = 0;
        loop {
            self.evaluation.passes_in_last_evaluation += 1;
            let Some(restart) = self.run_pass(&log.circular) else {
                // A call read the state earlier calls left (`OrderState`): its value depends on
                // the order, so the pass that counts is one in Excel's order (SPEC-0017). It starts
                // from the same sheet as any other pass. Without a reader the order is not observable.
                let read = self.evaluation.order_state_read;
                if read && !self.evaluation.excel_order {
                    self.evaluation.excel_order = true;
                    self.restore_dynamic_spills(&spills_before);
                    continue;
                }
                self.evaluation.excel_order = read;
                break;
            };
            let marked = log.record(&restart, &mut self.evaluation.anchor_order);
            spills_before.retain(|(position, cell)| {
                let dropped = match &restart {
                    Restart::StaleCells(_, cells) => cells.contains(&key(*position)),
                    _ => false,
                };
                let of_marked = match cell {
                    Cell::SpillCell { a, .. } => marked
                        .iter()
                        .any(|m| m.sheet == position.sheet && (m.row, m.column) == *a),
                    _ => false,
                };
                !dropped && !of_marked
            });
            self.restore_dynamic_spills(&spills_before);
        }
        self.evaluation.restarts_in_last_evaluation = log.restarts;
        self.evaluation.order_state = OrderState::default();
    }

    /// Sets how many formulas deep the evaluation may recurse. A formula that
    /// reads a formula that reads a formula, and so on, uses the stack of the
    /// thread for every step. Beyond this limit the evaluation unwinds and
    /// carries on from where it stopped instead of going deeper, so the limit
    /// bounds the stack used and never changes a result. Lower it where the
    /// stack is small. The least it can be is 1. See also
    /// `set_max_evaluation_stack`: whichever limit is reached first applies.
    pub fn set_max_evaluation_depth(&mut self, max_depth: usize) {
        self.evaluation.max_depth = max_depth.max(1);
    }

    /// The limit set by `set_max_evaluation_depth`.
    pub fn get_max_evaluation_depth(&self) -> usize {
        self.evaluation.max_depth
    }

    /// Sets how many bytes of stack the evaluation may use before it unwinds,
    /// counted from the first formula of the recursion. The evaluation can
    /// overshoot by what one formula needs, so leave room: a quarter of the
    /// stack is a reasonable value.
    pub fn set_max_evaluation_stack(&mut self, bytes: usize) {
        self.evaluation.max_stack = bytes;
    }

    /// The limit set by `set_max_evaluation_stack`.
    pub fn get_max_evaluation_stack(&self) -> usize {
        self.evaluation.max_stack
    }

    /// One pass over the workbook: anchors first, then every cell. Returns the
    /// reason the pass had to be abandoned, if any.
    fn run_pass(&mut self, circular_anchors: &[CellKey]) -> Option<Restart> {
        let state = &mut self.evaluation;
        state.cells.clear();
        state.clear_pass_caches();
        state.stack.clear();
        state.seen.clear();
        state.seen_ranges.clear();
        state.restart = None;
        state.depth = 0;
        state.unwinding = false;
        state.circular = circular_anchors.iter().copied().collect();
        state.circular_anchors = circular_anchors.iter().copied().collect();
        state.circular_below = 0;
        state.in_pass = true;
        state.order_state = OrderState::default();
        state.order_state_read = false;
        // dynamic links (HYPERLINK) are rebuilt on every evaluation
        self.links.clear();
        self.clear_variable_stack();
        self.clear_lambdas();

        let anchors = self.evaluation.anchor_order.clone();
        let mut restart = None;
        let everything = if self.evaluation.excel_order {
            Vec::new()
        } else {
            self.all_positions()
        };
        for cell in anchors.into_iter().chain(everything) {
            self.evaluation.root = Some(cell);
            self.evaluate_root_cell(cell);
            if self.evaluation.restart.is_some() {
                restart = self.evaluation.restart.take();
                break;
            }
        }
        if restart.is_none() && self.evaluation.excel_order {
            restart = self.run_excel_order();
        }
        self.evaluation.in_pass = false;
        self.evaluation.root = None;
        restart
    }

    /// The value of a cell, evaluating it first if it is a formula that has
    /// not been evaluated in this pass. Reads are performed on behalf of the
    /// cell on top of the stack.
    pub(crate) fn evaluate_cell(&mut self, cell_reference: CellReferenceIndex) -> CalcResult {
        self.evaluation.cell_reads += 1;
        if self.evaluation.restart.is_some() || self.evaluation.unwinding {
            // The pass is being abandoned, or the recursion is unwinding;
            // nothing computed now is kept. This comes before anything is put
            // on record: a read that was not made is nobody's dependency.
            return CalcResult::EmptyCell;
        }
        // The cell is looked at, not cloned: what is needed of it is copied
        // out, and the sheet is free again before anything else is evaluated.
        let cell = match self.fetch_cell(cell_reference) {
            None | Some(Cell::EmptyCell { .. }) => {
                self.record_seen(cell_reference, Seen::Empty);
                return CalcResult::EmptyCell;
            }
            Some(cell) => cell,
        };
        if let Cell::SpillCell { a, .. } = cell {
            let anchor = *a;
            return self.evaluate_spill_cell(cell_reference, anchor);
        }
        match FormulaCell::of(cell) {
            Some(formula_cell) => self.evaluate_formula_cell(cell_reference, formula_cell),
            // A constant.
            None => self.get_cell_value(cell, cell_reference),
        }
    }

    /// A spill cell holds what its anchor wrote. Only a value written by the
    /// anchor in the current pass is a value; anything else is either the
    /// anchor's business (a CSE area is fixed and forwards to its anchor) or a
    /// leftover of a previous evaluation, which is read as empty or triggers a
    /// restart so that the anchor runs first.
    fn evaluate_spill_cell(
        &mut self,
        cell_reference: CellReferenceIndex,
        anchor: (i32, i32),
    ) -> CalcResult {
        let anchor_reference = CellReferenceIndex {
            sheet: cell_reference.sheet,
            row: anchor.0,
            column: anchor.1,
        };
        let kind = match self.fetch_cell(anchor_reference) {
            Some(Cell::ArrayFormula { kind, .. }) => kind.clone(),
            _ => {
                // An orphan: its anchor is gone. Nothing will ever write it again.
                self.record_seen(cell_reference, Seen::Empty);
                return CalcResult::EmptyCell;
            }
        };
        let anchor_state = self.evaluation.cells.get(&key(anchor_reference)).copied();
        match (kind, anchor_state) {
            // Written by the anchor in this pass.
            (_, Some(CellState::Evaluated)) => self.stored_value(cell_reference),
            // A CSE area is fixed: its anchor is evaluated first and the cell
            // holds what it wrote. If the anchor is running, this read closes a
            // cycle, which `evaluate_cell` reports.
            (ArrayKind::Cse, Some(CellState::Evaluating)) => self.evaluate_cell(anchor_reference),
            (ArrayKind::Cse, None) => {
                self.evaluate_cell(anchor_reference);
                self.stored_value(cell_reference)
            }
            // The anchor is running and this read is on its behalf: the area
            // counts as empty until the anchor commits. If the anchor then
            // writes this position, its inputs depended on its own output.
            (ArrayKind::Dynamic, Some(CellState::Evaluating)) => {
                self.evaluation.evaluating_reads += 1;
                self.record_seen(cell_reference, Seen::Empty);
                CalcResult::EmptyCell
            }
            // Left over from a previous evaluation: the anchor should have run
            // before anyone read its cells. Restart with it before the reader.
            // Outside a pass there is no driver, so the anchor is simply
            // evaluated now.
            (ArrayKind::Dynamic, None) => {
                if self.evaluation.in_pass {
                    // Inside a pass the driver has always set the root.
                    let reader = self.evaluation.root.unwrap_or(anchor_reference);
                    self.evaluation.restart = Some(Restart::StaleRead {
                        anchor: anchor_reference,
                        reader,
                    });
                    CalcResult::EmptyCell
                } else {
                    self.evaluate_cell(anchor_reference);
                    self.stored_value(cell_reference)
                }
            }
        }
    }

    fn evaluate_formula_cell(
        &mut self,
        cell_reference: CellReferenceIndex,
        cell: FormulaCell,
    ) -> CalcResult {
        let key = key(cell_reference);
        match self.evaluation.cells.get(&key) {
            Some(CellState::Evaluating) if self.iterating() => {
                self.evaluation.evaluating_reads += 1;
                return self.read_into_cycle(cell_reference);
            }
            Some(CellState::Evaluating) => {
                self.evaluation.evaluating_reads += 1;
                self.mark_stack_circular();
                return circular_reference();
            }
            Some(CellState::Evaluated) => {
                self.mark_readers_of(key);
                self.read_evaluated(cell_reference);
                return self.stored_value(cell_reference);
            }
            None if self.evaluation.blocking && !self.evaluation.stack.is_empty() => {
                // Excel order: the root stops at this read, with no effect (`excel_order.rs`).
                self.evaluation.blocked_on = Some(cell_reference);
                self.evaluation.unwinding = true;
                return CalcResult::EmptyCell;
            }
            None => {}
        }
        self.evaluation.cells.insert(key, CellState::Evaluating);
        // Everything above this length is this cell and what it needs.
        let base = self.evaluation.stack.len();
        self.evaluation.stack.push(cell_reference);
        if self.too_deep() {
            // Too deep. The cell is not run: it stays on the stack, still
            // `Evaluating`, and so will every cell above it as the recursion
            // unwinds. They are run again from the top once it has.
            self.evaluation.unwinding = true;
            return CalcResult::EmptyCell;
        }
        let result = self.run_top(cell_reference, cell);
        if self.evaluation.depth == 0 && self.evaluation.blocked_on.is_some() {
            // The root stopped at a read (Excel order): it is not evaluated in this pass yet.
            self.evaluation.unwinding = false;
            self.evaluation.stack.truncate(base);
            self.evaluation.circular_below = self.evaluation.circular_below.min(base);
            self.evaluation.cells.remove(&key);
            self.evaluation.clear_pass_caches();
            return CalcResult::EmptyCell;
        }
        if self.evaluation.depth > 0 || !self.evaluation.unwinding {
            self.mark_readers_of(key);
            return result;
        }
        // This is the outermost formula and the recursion below it got too
        // deep. It has unwound: nothing is running, and the stack holds this
        // cell and the cells it is waiting for, the deepest on top, none of
        // them evaluated. Run them from the top. A cell that completes leaves
        // the stack and the one below finds it evaluated. A cell that reads
        // further down than the limit allows unwinds again, with more cells on
        // the stack, and the replay carries on from the new top. Every cell is
        // pushed once in a pass, so this ends.
        self.replay(base);
        if self.evaluation.restart.is_some() {
            return CalcResult::EmptyCell;
        }
        self.mark_readers_of(key);
        self.stored_value(cell_reference)
    }

    /// Whether the recursion has gone as deep as it is allowed to, in formulas
    /// or in stack. Never true for the outermost formula, which has to run.
    fn too_deep(&mut self) -> bool {
        let here = stack_address();
        if self.evaluation.depth == 0 {
            self.evaluation.stack_base = here;
            return false;
        }
        self.evaluation.depth >= self.evaluation.max_depth
            || self.evaluation.stack_base.abs_diff(here) >= self.evaluation.max_stack
    }

    /// Runs the cells left on the stack above `base` by an unwind, from the
    /// top, until they are all evaluated or the pass is abandoned.
    fn replay(&mut self, base: usize) {
        while self.evaluation.unwinding {
            self.evaluation.unwinding = false;
            while self.evaluation.stack.len() > base
                && !self.evaluation.unwinding
                && self.evaluation.restart.is_none()
            {
                let Some(&top) = self.evaluation.stack.last() else {
                    break;
                };
                match self.fetch_cell(top).and_then(FormulaCell::of) {
                    Some(cell) => {
                        self.run_top(top, cell);
                    }
                    None => {
                        // Not expected: nothing replaces a formula during a
                        // pass. Let go of it rather than loop on it.
                        self.pop_stack();
                        self.evaluation.cells.insert(key(top), CellState::Evaluated);
                    }
                }
            }
        }
    }

    /// Runs the formula of the cell on top of the stack, stores the result and
    /// takes the cell off the stack. If the pass is being abandoned nothing is
    /// stored. If the recursion is unwinding nothing is stored either, and the
    /// cell stays on the stack, `Evaluating`: it has not been evaluated yet.
    /// A read that closes a loop through it is then still a cycle.
    fn run_top(&mut self, cell_reference: CellReferenceIndex, cell: FormulaCell) -> CalcResult {
        let key = key(cell_reference);
        // A formula can attach a link to its cell as it runs (HYPERLINK). A
        // run that was abandoned to an unwind may have attached one on the
        // strength of values it never really read; this run decides afresh.
        if !self.links.is_empty() {
            self.links.remove(&key);
        }
        self.evaluation.depth += 1;
        let result = if self.evaluation.circular_anchors.contains(&key) {
            circular_reference()
        } else {
            self.compute_formula(
                cell_reference,
                cell.f,
                matches!(cell.array, Some((true, _))),
            )
        };
        self.evaluation.depth -= 1;
        if self.evaluation.unwinding {
            return CalcResult::EmptyCell;
        }
        // The cell may have been found on a cycle while it ran.
        let result = if self.evaluation.circular.contains(&key) {
            self.circular_value(cell_reference)
        } else {
            result
        };
        let stored = if self.evaluation.restart.is_some() {
            // Abandoned pass: store nothing.
            Ok(())
        } else {
            self.set_cells_with_result(cell_reference, cell, &result)
        };
        self.pop_stack();
        if let Err(message) = stored {
            // Not expected to happen; keep the cell from being evaluated again.
            self.evaluation.cells.insert(key, CellState::Evaluated);
            return CalcResult::new_error(Error::ERROR, cell_reference, message);
        }
        if self.evaluation.restart.is_some() {
            return CalcResult::EmptyCell;
        }
        self.evaluation.cells.insert(key, CellState::Evaluated);
        // Return what was stored, so that a dependent evaluated in this pass
        // sees the same value it would read from the sheet later: the first
        // element of a spilled array, or the error the result was turned into.
        self.stored_value(cell_reference)
    }

    /// Runs the parsed formula and turns a reference into a value or an array. `spills` is set
    /// for a dynamic-array anchor ([`Model::evaluate_formula_root`]).
    fn compute_formula(
        &mut self,
        cell_reference: CellReferenceIndex,
        formula: i32,
        spills: bool,
    ) -> CalcResult {
        // Shares the formula, it does not copy it.
        let node = Arc::clone(
            &self.parsed_formulas[cell_reference.sheet as usize][formula as usize].evaluated,
        );
        match self.run_cell_formula(&node, cell_reference, spills) {
            CalcResult::Range { left, right } => {
                let (rows, columns) = range_dims(left, right);
                if left == right {
                    self.evaluate_cell(left)
                } else if spills_past_grid(cell_reference, rows, columns) {
                    spill_past_grid_error(cell_reference)
                } else {
                    match self.evaluate_range(left, right, cell_reference) {
                        Ok(array) => CalcResult::Array(array),
                        Err(error) => error,
                    }
                }
            }
            CalcResult::Lambda(_) => CalcResult::new_error(
                Error::CALC,
                cell_reference,
                "A LAMBDA was returned but not called".to_string(),
            ),
            result => result,
        }
    }

    /// The value currently stored at a position, empty if there is no cell.
    fn stored_value(&self, cell_reference: CellReferenceIndex) -> CalcResult {
        match self.fetch_cell(cell_reference) {
            Some(cell) => self.get_cell_value(cell, cell_reference),
            None => CalcResult::EmptyCell,
        }
    }

    /// Records what the formula being evaluated found at a position, together
    /// with the root of the current recursion, the cell the driver is
    /// evaluating. The first record of each kind is kept. Reads made outside
    /// a pass are nobody's dependency.
    pub(crate) fn record_seen(&mut self, position: CellReferenceIndex, seen: Seen) {
        if !self.evaluation.in_pass {
            return;
        }
        if let Some(root) = self.evaluation.root {
            let record = self.evaluation.seen.entry(key(position)).or_default();
            let slot = match seen {
                Seen::Empty => &mut record.empty,
                Seen::Occupied => &mut record.occupied,
            };
            if slot.is_none() {
                *slot = Some(root);
            }
        }
    }

    /// Records that the formula being evaluated depends on every position of a
    /// rectangle and found them all empty, on behalf of the root. Used for the
    /// part of a whole-column or whole-row range that a walk skips because it
    /// lies outside the sheet's used area (`clip_to_used_area`).
    pub(crate) fn record_seen_empty_range(
        &mut self,
        sheet: u32,
        row1: i32,
        column1: i32,
        row2: i32,
        column2: i32,
    ) {
        if !self.evaluation.in_pass || row1 > row2 || column1 > column2 {
            return;
        }
        if let Some(root) = self.evaluation.root {
            let range = SeenRange {
                root,
                sheet,
                row1,
                column1,
                row2,
                column2,
            };
            self.evaluation
                .seen_ranges
                .push(range, &mut self.area_budget);
        }
    }

    /// Checks, on behalf of the dynamic anchor about to commit, whether spilling
    /// over `area` (width, height; every cell of it but the anchor's own is
    /// written, so `(1, 1)` writes nothing) and removing its own spill cells at
    /// `clears` would contradict what a formula read earlier in this pass. If
    /// so, the pass is abandoned so that it can start again with the anchor
    /// first, and `true` is returned: the anchor must not write anything. When
    /// every contradicted read was made on the anchor's own behalf, the anchor
    /// is circular, unless only the removals contradict: then the cells were
    /// stale, and they go.
    ///
    /// The readers are listed in the order of the writes that contradict them,
    /// cell records first, then rectangles; the rectangles are each met once,
    /// at their first cell written, so the check costs the cells plus the
    /// rectangles, not their product (T-1512); only the rectangles filed in row and column bands
    /// over the spill are met at all (T-1545, T-1646).
    pub(crate) fn spill_contradicts_a_read(
        &mut self,
        anchor: CellReferenceIndex,
        area: (i32, i32),
        clears: &[CellKey],
    ) -> bool {
        if !self.evaluation.in_pass {
            return false;
        }
        let CellReferenceIndex { sheet, row, column } = anchor;
        let (width, height) = area;
        let seen = &self.evaluation.seen;
        // An area of one cell writes nothing: the clear-only check skips the rectangles.
        let by_ranges = if width <= 1 && height <= 1 {
            Vec::new()
        } else {
            self.evaluation.seen_ranges.written_by_spill(anchor, area)
        };
        let contradicted_by_writes: Vec<CellReferenceIndex> = (row..row + height)
            .flat_map(|r| (column..column + width).map(move |c| (sheet, r, c)))
            .filter(|&(_, r, c)| (r, c) != (row, column))
            .filter_map(|p| seen.get(&p).and_then(|record| record.empty))
            .chain(by_ranges.into_iter().map(|(_, root)| root))
            .collect();
        let contradicted_by_clears: Vec<CellReferenceIndex> = clears
            .iter()
            .filter_map(|p| seen.get(p).and_then(|record| record.occupied))
            .collect();
        if contradicted_by_writes.is_empty() && contradicted_by_clears.is_empty() {
            return false;
        }
        let mut listed: HashSet<CellReferenceIndex> = HashSet::new();
        let readers: Vec<CellReferenceIndex> = contradicted_by_writes
            .iter()
            .chain(&contradicted_by_clears)
            .copied()
            .filter(|root| *root != anchor && listed.insert(*root))
            .collect();
        self.evaluation.restart = Some(if !readers.is_empty() {
            Restart::Conflict { anchor, readers }
        } else if contradicted_by_writes.is_empty() {
            Restart::StaleCells(anchor, clears.to_vec())
        } else {
            Restart::SelfContradiction(anchor)
        });
        true
    }
}
