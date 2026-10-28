//! The budgets of one formula evaluation: how many pairs of areas its intersections may compare
//! and how many steps the walks of its union areas may take, in all (DEVIATION, decision 0014:
//! docs/excel-semantics/references.md "Union area cap"), how many rectangles it may put on record
//! as read empty (references.md "Seen-range cap"), and how many work units it may spend in all
//! (SPEC-0024; arrays.md "One formula evaluation does at most 268,435,456 work units").

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    limits::too_many_visits,
    model::Model,
    recalc_budget::RunBound,
};

/// Most pairs of areas one formula evaluation may compare ([`AreaBudget`]). Each intersection
/// compares at most `MAX_AREAS` pairs, but a chain of cuts (`(U1 U2) A1 A1 A1 ...`) repeats that
/// once per three characters of formula, and a LET-bound union repeats it per argument
/// (`SUM(IF(1,u u A1),IF(1,u u A1),...)`), so the whole formula shares this budget.
pub(crate) const MAX_AREA_PAIRS: usize = 1 << 20;

/// Message of the `#VALUE!` for an expression comparing more than [`MAX_AREA_PAIRS`] pairs.
pub(crate) const TOO_MANY_AREA_PAIRS: &str = "Too many areas to intersect";

/// Most steps the walks of the union areas of one formula evaluation may take in all
/// ([`AreaBudget`]). The area cap bounds the length of an area list, not the work of walking it:
/// `MAX_AREAS` copies of `A:A` fit in a short formula, a LET-bound union can be passed again and
/// again (`SUM(x,x,...)`), and a LAMBDA can walk one per row of a MAP. A union area is walked
/// over its existing cells only ([`Model::for_each_position_in_area`]), and each step is one area
/// walked, one cell handed to the caller, one search of the sheet's column index (an area with
/// cells in few columns walks them through it), one row found, one unallocated chunk of rows
/// passed over, or [`SLOTS_PER_WALK_STEP`] row slots tested, so the work follows the data
/// visited, not the size of the areas. Past the bound the argument is `#VALUE!` (DEVIATION,
/// decision 0014: references.md "Union area cap"). The message keeps its words
/// ([`TOO_MANY_AREA_CELLS`]).
pub(crate) const MAX_AREA_WALK: u64 = 1 << 24;

/// Row slots of an allocated chunk tested per step of [`MAX_AREA_WALK`]: testing a slot is one
/// comparison over a contiguous array, while a step handed to the caller reads and evaluates a
/// cell, so a slot is charged a fraction of a step (measured in T-1497, references.md "Union area
/// cap").
pub(crate) const SLOTS_PER_WALK_STEP: u64 = 16;

/// Message of the `#VALUE!` for union walks past [`MAX_AREA_WALK`] steps.
pub(crate) const TOO_MANY_AREA_CELLS: &str = "Too many cells in the areas of a reference";

/// Most rectangles one formula evaluation may put on record as read empty in a pass
/// ([`Model::record_seen_empty_range`]). A rectangle already on record for the same root is
/// free, so a walk repeated by a LAMBDA costs once (T-1497); a new one needs a reference no
/// earlier read of the pass produced, such as one per row of `MAKEARRAY(..,LAMBDA(r,c,
/// SUM(OFFSET(1:1,r-1,0,c))))`, and nothing else bounds how many a formula makes. Past the bound
/// the formula is `#VALUE!` (DEVIATION, decision 0014: references.md "Seen-range cap").
pub(crate) const MAX_SEEN_RANGES: usize = 1 << 16;

/// Message of the `#VALUE!` for a formula past [`MAX_SEEN_RANGES`].
pub(crate) const TOO_MANY_SEEN_RANGES: &str = "Too many distinct ranges read";

/// Most work units one formula evaluation may spend (SPEC-0024): [`NODE_UNITS`] per node
/// evaluated, [`CELL_UNITS`] per array cell built, one per [`TEXT_BYTES_PER_UNIT`] bytes of each
/// text value, [`LAMBDA_UNITS`] per LAMBDA value made and [`NODE_UNITS`] per node of each LAMBDA
/// body cloned, plus the per-call caps' own charges ([`VISIT_UNITS`] per visit). Each LAMBDA
/// call draws on it again, so it bounds the product of a helper's iterations and the work of
/// each call, which no per-call cap does. Past it the formula is
/// `#VALUE!` ([`TOO_MUCH_WORK`]), which cannot be caught (DEVIATION, decision 0014: arrays.md
/// "One formula evaluation does at most 268,435,456 work units").
pub(crate) const MAX_FORMULA_WORK: u64 = 1 << 28;

// The weights below are fitted so that each kind's worst case reaches [`MAX_FORMULA_WORK`] in
// under 10 s of a release build (SPEC-0024 section 4, T-2803; the timings are in arrays.md "One
// formula evaluation does at most 268,435,456 work units", T-2923's re-fit): a LAMBDA call of a
// one-node body under MAKEARRAY costs about 250 ns, which its node and built cell (9 units) pay
// for, a built cell about 25 ns, a byte a text scanner reads (FIND, SUBSTITUTE over 32,767 bytes)
// about 5 ns, and a LAMBDA made on each call a few microseconds ([`LAMBDA_UNITS`]). A visit cap's
// units are timed by its own cap instead ([`VISIT_UNITS`], T-2813).

/// Work units of one node evaluated ([`MAX_FORMULA_WORK`]).
pub(crate) const NODE_UNITS: u64 = 6;

/// Work units of one array cell built ([`MAX_FORMULA_WORK`]).
pub(crate) const CELL_UNITS: u64 = 3;

/// Bytes of a text value charged as one work unit ([`MAX_FORMULA_WORK`]).
pub(crate) const TEXT_BYTES_PER_UNIT: u64 = 4;

/// Work units of one visit of a per-call cap (`limits::MAX_LIFTED_CELL_VISITS`: a lifted call's
/// cell visits and kernel units, GROUPBY and PIVOTBY's) ([`MAX_FORMULA_WORK`]). One, the cap's own
/// number, everywhere (SPEC-0024 as amended by T-2813): an eval cell may spend most of one cap on
/// one call, so a higher weight would refuse calls the cap allows, and the time a call's units
/// take is its kernel weight's to bound (one call at its cap within 5 s), not this budget's.
pub(crate) const VISIT_UNITS: u64 = 1;

/// Work units of one LAMBDA value stored (`Model::store_lambda`), on top of [`NODE_UNITS`] per
/// node of the body it clones ([`MAX_FORMULA_WORK`]). Every LAMBDA a formula makes stays in
/// `Model::lambdas` until the evaluation ends, and each later call slows with the map's size, so a
/// body that makes a LAMBDA on each call costs far more than its nodes (SPEC-0024 as amended by
/// T-2923; fitted by time on section 4's nested worst cases).
pub(crate) const LAMBDA_UNITS: u64 = 512;

/// Message of the formula's error once it spent [`MAX_FORMULA_WORK`].
pub(crate) const TOO_MUCH_WORK: &str = "Formula needs too much work to calculate";

/// What is left of one formula evaluation's budgets of area pairs ([`MAX_AREA_PAIRS`]), of union
/// walk steps ([`MAX_AREA_WALK`]), of seen-empty rectangles ([`MAX_SEEN_RANGES`]) and of work units
/// ([`MAX_FORMULA_WORK`]).
/// [`Model::with_area_budget`] gives each formula run a fresh one, so every argument and operand of
/// the formula draws on the same budget, and its worst-case work does not grow with the number of
/// arguments. Only that call refills it, so every entry point that starts a formula evaluation
/// (`run_cell_formula`, `evaluate_outside_cell`) must go through it; a new one that did not would
/// draw on whatever budget the model holds.
#[derive(Clone, Copy)]
pub(crate) struct AreaBudget {
    pairs: usize,
    walk: u64,
    seen_ranges: usize,
    /// Whether a rectangle went unrecorded for want of [`MAX_SEEN_RANGES`]: the formula's value
    /// may then depend on a read the pass does not know of, so it is not used.
    seen_ranges_exceeded: bool,
    /// Work units left ([`MAX_FORMULA_WORK`], or less when a pool of the recalculation has less
    /// left, SPEC-0028).
    pub(crate) work: u64,
    /// Set once a charge did not fit; every later charge then fails at once, so the formula
    /// unwinds without doing more work.
    pub(crate) work_exceeded: bool,
    /// Whether the run draws on the pools of the recalculation's pass (SPEC-0028 section 2).
    pub(crate) pooled: bool,
    /// Which bound gave the run its units, and so what running out of them means.
    pub(crate) bound: RunBound,
    /// `work` when the run last settled what it spent ([`Model::start_run`]).
    pub(crate) unsettled_from: u64,
    /// Units the run spent, as of its last settle.
    pub(crate) spent: u64,
    /// The work each formula run started from this one gets, [`MAX_FORMULA_WORK`] unless a test
    /// lowers it on the model's own budget (SPEC-0024 section 2: a security bound, not an API option).
    #[cfg(test)]
    pub(crate) limit: u64,
    /// The work units the last formula run started from this one spent, for the tests of SPEC-0024.
    #[cfg(test)]
    pub(crate) last_spent: u64,
}

impl AreaBudget {
    /// Whether `count` more rectangles fit in what is left of the budget of [`MAX_SEEN_RANGES`]:
    /// a walk that may instead record its cells one by one asks before it clips (T-0405).
    pub(crate) fn has_seen_ranges(&self, count: usize) -> bool {
        self.seen_ranges >= count
    }

    /// Takes one rectangle from what is left of the budget of [`MAX_SEEN_RANGES`], or, past it,
    /// marks the formula's run as exceeded ([`Model::with_area_budget`]) and returns `false`: the
    /// rectangle must not be recorded.
    pub(crate) fn spend_seen_range(&mut self) -> bool {
        match self.seen_ranges.checked_sub(1) {
            Some(left) => {
                self.seen_ranges = left;
                true
            }
            None => {
                self.seen_ranges_exceeded = true;
                false
            }
        }
    }
}

impl Default for AreaBudget {
    fn default() -> AreaBudget {
        AreaBudget::for_run(MAX_FORMULA_WORK, MAX_FORMULA_WORK, RunBound::Own, false)
    }
}

impl AreaBudget {
    /// A fresh budget for a run whose own limit is `limit` ([`MAX_FORMULA_WORK`], or a test's lower
    /// one) and which may spend `work` units, given by `bound` ([`Model::start_run`]).
    pub(crate) fn for_run(limit: u64, work: u64, bound: RunBound, pooled: bool) -> AreaBudget {
        #[cfg(not(test))]
        let _ = limit;
        AreaBudget {
            pairs: MAX_AREA_PAIRS,
            walk: MAX_AREA_WALK,
            seen_ranges: MAX_SEEN_RANGES,
            seen_ranges_exceeded: false,
            work,
            work_exceeded: false,
            pooled,
            bound,
            unsettled_from: work,
            spent: 0,
            #[cfg(test)]
            limit,
            #[cfg(test)]
            last_spent: 0,
        }
    }

    /// Adds what the run spent since its last settle to `spent` and returns it.
    pub(crate) fn settle_spent(&mut self) -> u64 {
        let units = self.unsettled_from.saturating_sub(self.work);
        self.unsettled_from = self.work;
        self.spent = self.spent.saturating_add(units);
        units
    }
}

/// The error of a formula past [`MAX_FORMULA_WORK`]. Out of line, so the inlined charges stay a
/// subtraction and a branch, and the evaluator's frames do not grow (T-0444).
#[cold]
#[inline(never)]
pub(crate) fn too_much_work(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, TOO_MUCH_WORK.to_string())
}

/// The units of a text value of `bytes` UTF-8 bytes: one per [`TEXT_BYTES_PER_UNIT`], rounded up.
pub(crate) fn text_units(bytes: usize) -> u64 {
    u64::try_from(bytes)
        .unwrap_or(u64::MAX)
        .div_ceil(TEXT_BYTES_PER_UNIT)
}

/// The units of building a `rows` x `columns` array: [`CELL_UNITS`] per cell, saturating, so an
/// oversized shape fails its charge instead of wrapping into a small one.
pub(crate) fn cell_units(rows: usize, columns: usize) -> u64 {
    u64::try_from(rows.saturating_mul(columns))
        .unwrap_or(u64::MAX)
        .saturating_mul(CELL_UNITS)
}

/// The units of cloning `body`: [`NODE_UNITS`] per node, saturating.
pub(crate) fn clone_units(body: &Node) -> u64 {
    body.node_count().saturating_mul(NODE_UNITS)
}

/// The units of `visits` visits of a per-call cap: [`VISIT_UNITS`] each, saturating.
pub(crate) fn visit_units(visits: u64) -> u64 {
    visits.saturating_mul(VISIT_UNITS)
}

impl Model {
    /// Runs `f`, one formula's evaluation in `cell`, with a fresh [`AreaBudget`], and gives the
    /// budget it interrupted back afterwards: a cell evaluated while another formula runs has its
    /// own budget and leaves the other's as it was, but for what the pools of the recalculation
    /// have left (SPEC-0028: the runs' spend is settled into them here, [`Model::start_run`] and
    /// [`Model::end_run`]). A run that read more rectangles than
    /// [`MAX_SEEN_RANGES`] lets it record is `#VALUE!`, whatever `f` returned: a spill into a
    /// rectangle it could not record would not restart the pass, so its value cannot be trusted,
    /// and an error here, unlike one at the read, cannot be caught (`IFERROR`) and replaced by a
    /// value that still depends on the read.
    pub(crate) fn with_area_budget(
        &mut self,
        cell: CellReferenceIndex,
        f: impl FnOnce(&mut Self) -> CalcResult,
    ) -> CalcResult {
        #[cfg(not(test))]
        let fresh = self.start_run(MAX_FORMULA_WORK);
        #[cfg(test)]
        let fresh = self.start_run(self.area_budget.limit);
        let outer = std::mem::replace(&mut self.area_budget, fresh);
        let result = f(self);
        let mut inner = std::mem::replace(&mut self.area_budget, outer);
        // Past the work budget, the run was cut short wherever the charge failed, and `f` may have
        // turned that into a value (`IFERROR`), so the formula's value is the error (SPEC-0024).
        let exceeded = self.end_run(&mut inner, cell);
        #[cfg(test)]
        {
            self.area_budget.last_spent = inner.spent;
        }
        if let Some(error) = exceeded {
            return error;
        }
        if inner.seen_ranges_exceeded {
            return CalcResult::new_error(Error::VALUE, cell, TOO_MANY_SEEN_RANGES.to_string());
        }
        result
    }

    /// Takes `units` from the formula's budget of [`MAX_FORMULA_WORK`], or, past it, marks the run
    /// as exceeded and gives its `#VALUE!` (and so does every later charge of the run). `units` is
    /// computed with saturating arithmetic ([`cell_units`], [`text_units`]), so an oversized charge
    /// fails and never wraps.
    #[inline]
    pub(crate) fn spend_work(
        &mut self,
        units: u64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        let budget = &mut self.area_budget;
        match budget.work.checked_sub(units) {
            Some(left) if !budget.work_exceeded => {
                budget.work = left;
                Ok(())
            }
            _ => {
                budget.work_exceeded = true;
                Err(too_much_work(cell))
            }
        }
    }

    /// The per-call cap of `visits` cells (`limits::too_many_visits`), then the same visits charged
    /// to the formula's work budget, so a call within its cap cannot be multiplied by a LAMBDA
    /// helper past the formula's (SPEC-0024 section 1, "per-call capped work").
    pub(crate) fn spend_visits(
        &mut self,
        visits: u64,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        too_many_visits(visits, cell).or_else(|| self.spend_work(visit_units(visits), cell).err())
    }

    /// Charges one node evaluated ([`NODE_UNITS`]), the charge that bounds a LAMBDA body run again
    /// and again (SPEC-0024 section 1).
    #[inline]
    pub(crate) fn spend_node(&mut self, cell: CellReferenceIndex) -> Result<(), CalcResult> {
        self.spend_work(NODE_UNITS, cell)
    }

    /// `value`, a node's, once charged ([`Model::spend_value`]), or the budget's error.
    #[inline]
    pub(crate) fn charged(&mut self, value: CalcResult, cell: CellReferenceIndex) -> CalcResult {
        match self.spend_value(&value, cell) {
            Ok(()) => value,
            Err(error) => error,
        }
    }

    /// Charges a node's value: a text by its length ([`text_units`]) and an array by its cells
    /// ([`cell_units`]), wherever it enters the evaluator, so producers (REPT, `&`, SEQUENCE,
    /// MAKEARRAY, a broadcast) and the scanners reading their results pay for what was built
    /// (SPEC-0024 section 1). Any other value costs nothing more than its node.
    #[inline]
    pub(crate) fn spend_value(
        &mut self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        let units = match value {
            CalcResult::String(text) => text_units(text.len()),
            CalcResult::Array(array) => cell_units(array.len(), array.first().map_or(0, Vec::len)),
            _ => return Ok(()),
        };
        self.spend_work(units, cell)
    }

    /// Charges a LAMBDA value made with body `body`: [`LAMBDA_UNITS`] for storing it and the
    /// nodes of the body it clones ([`clone_units`]).
    pub(crate) fn spend_lambda_made(
        &mut self,
        body: &Node,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.spend_work(LAMBDA_UNITS.saturating_add(clone_units(body)), cell)
    }

    /// [`Model::spend_work`] for building a `rows` x `columns` array ([`cell_units`]).
    pub(crate) fn spend_cells(
        &mut self,
        rows: usize,
        columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.spend_work(cell_units(rows, columns), cell)
    }

    /// Whether the formula has spent its work budget ([`MAX_FORMULA_WORK`]): the LAMBDA helpers
    /// stop their loops on it, so a run past the budget costs at most one more call per helper.
    pub(crate) fn work_exceeded(&self) -> bool {
        self.area_budget.work_exceeded
    }

    /// Takes `steps` from what is left of the formula's budget of [`MAX_AREA_WALK`], or gives
    /// the `#VALUE!` of a walk past it and leaves the budget as it was.
    pub(crate) fn spend_area_walk(
        &mut self,
        steps: u64,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.area_budget.walk = self.area_budget.walk.checked_sub(steps).ok_or_else(|| {
            CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREA_CELLS.to_string())
        })?;
        Ok(())
    }

    /// Takes `pairs` from what is left of the formula's budget of [`MAX_AREA_PAIRS`], or gives
    /// the `#VALUE!` of an intersection past it and leaves the budget as it was.
    pub(crate) fn spend_area_pairs(
        &mut self,
        pairs: usize,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        self.area_budget.pairs = self.area_budget.pairs.checked_sub(pairs).ok_or_else(|| {
            CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREA_PAIRS.to_string())
        })?;
        Ok(())
    }
}
