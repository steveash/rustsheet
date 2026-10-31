---
id: T-3426
title: "Bound the spill area a workbook's array anchors claim in total, not only per anchor"
milestone: M12
lane: engine
status: todo
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
2^20 cells. T-2896 refuses one, a DEVIATION in docs/excel-semantics/xlsx-storage.md.

## Acceptance criteria
- [ ] A test with many anchors each claiming 2^20 cells loads (or is refused) and evaluates in bounded time and memory.
- [ ] The xlsx-storage.md entry cites a probe of a CSE range above 2^20 cells, or a probe task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
