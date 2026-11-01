---
id: T-3426
title: "Bound the spill area a workbook's array anchors claim in total, not only per anchor"
milestone: M12
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A crafted workbook or diff list cannot make loading, evaluation or a clear do work far beyond its own size through the
spill ranges its array anchors claim.

## Context
Found in T-2896's security review. `workbook_check::check_cell` now caps each anchor's spill range (`r`) at the array
limit, 2^20 cells. But n anchors, each a few bytes, can still claim n x 2^20 cells. Evaluation fills a CSE anchor's
whole range (model.rs, the loop after `retire_own_spill_cells`), and the range clears and `undo_set_cell_value` walk an
anchor's range densely (model.rs and user_model/cell_undo.rs: `for r in row..row + height`). A thousand anchors means
about 10^9 steps, and memory for that many spill cells. Options: walk only the cells that exist in those loops
(`SheetData::places_in_block`), check at load that anchors' ranges do not overlap (their sum is then bounded by the
grid), or cap the total claimed area. Also probe whether Excel opens a file whose CSE or cached spill range exceeds
2^20 cells (T-2896 refuses one, a DEVIATION in docs/excel-semantics/xlsx-storage.md), and whether Excel lets a CSE
formula be entered over more than 2^20 cells (T-2896 refuses it, a DEVIATION in docs/excel-semantics/arrays.md).
P1: until this lands, a small crafted workbook or diff list can still claim n x 2^20 cells.

## Acceptance criteria
- [x] A test with many anchors each claiming 2^20 cells loads (or is refused) and evaluates in bounded time and memory.
- [x] The xlsx-storage.md entry cites a probe of a file whose CSE or cached spill range is above 2^20 cells, and the
      arrays.md entry a probe of entering a CSE formula over more than 2^20 cells (e.g. over A:B); or a probe task is
      filed for each.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The engine now bounds what array anchors may claim against what is stored, not just each anchor on its own.
`workbook_check::SpillTally` counts the cells it is given and the spill cells their anchors claim. Its `check` refuses
anchors that claim more cells than are counted, past one whole array (`MAX_SIZE`, 2^20). That slack applies once per
load, and once per undo, redo or external diff call.

**Why this bound.** Every consumer that walks an anchor's range then costs at most the input size plus 2^20. Those
consumers are: evaluation filling a CSE range, `reset_dynamic_array_spills`, the clears and `undo_set_cell_value`.

**Real files fit.**
- The engine stores every cell of a spill, and resets a blocked spill's range to its anchor (model.rs, `r: (1, 1)`).
- The Excel-fidelity review scanned the repository's xlsx files: 622 array formulas store all 3,447 cells they claim.
- What range Excel writes for a #SPILL! anchor is UNVERIFIED; it is in probe T-3436.
- The eval gate loads every suite workbook with no regression.

**Loads.** `check_indices` tallies every sheet and checks once at the end, which keeps side-by-side column spills legal.
Refusing a file is a DEVIATION, recorded in docs/excel-semantics/xlsx-storage.md.

**Diffs.**
- `UserModel::restored_spills` tallies the cells one undo, redo or external call restores (`restored_cell`, and
  `check_worksheet` for a restored sheet).
- `apply_diff_list` and `apply_undo_diff_list` check the tally before each diff and before evaluating.
- An external call seeds the tally with the model's own cells first. Otherwise calls made while evaluation is paused
  could pile up anchors (security review).
- A restored dynamic anchor comes back claiming only itself, as `reset_dynamic_array_spills` leaves one. Its spill
  cells are never in a diff, since evaluation rebuilds them. Before this fix, undoing the clear of two 600,000-cell
  dynamic spills was refused (Excel-fidelity review). A restored CSE anchor brings its cells with it.
- A list restoring ten whole-column CSE anchors with nothing behind them is refused before anything walks them.
- Local undo and redo share the check, but apply only diffs the model recorded itself, so they pass it.

**Bounded separately: T-3437.** The xlsx importer expands each array `ref` into a map before `Model::from_workbook`
runs, so this check cannot bound that expansion.

**Rejected options.**
- Overlap checks: non-overlapping anchors can still claim the whole grid.
- Walking only existing cells in each consumer: that leaves a CSE fill, which must create its cells.

**Probe.** T-3436 probes Excel for both DEVIATIONs and for a #SPILL! anchor's range.

**Tests.**
- `test_workbook_check.rs`:
  - `anchors_claiming_more_cells_than_are_stored_are_refused`;
  - `spills_the_engine_writes_still_load`.
- `test_external_diffs_untrusted.rs`:
  - `anchors_restored_claiming_more_cells_than_they_bring_are_refused`;
  - `undoing_the_clear_of_large_dynamic_spills_still_works`, local and through a peer.
