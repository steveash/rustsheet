---
id: T-2086
title: "Pivot tables follow structural edits: row/column insert and delete, sheet rename and delete (SPEC-0015)"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-2081]
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
Pivot tables move with the cells around them: row/column inserts and deletes shift or shrink a pivot's location, deleting its top-left cell or its sheet drops it, and a sheet rename updates its cache source.

## Context
SPEC-0015 "Data model", structural edits. Tables have no such handling to copy, so this is new code in the row/column insert/delete actions (crates/rustsheet-engine/src/actions.rs) and the sheet rename/delete paths. Replaces T-2081's interim rule (any structural edit on the sheet drops its pivots); remove that gap from the web.md section.

- T-2081 (2026-10-21) drops a sheet's pivots in `Model::take_sheet`, which the undo of a sheet deletion also uses, so `restore_sheet`
  brings the sheet back without its pivots; moving pivots instead of dropping them should cover that path too.

## Acceptance criteria
- [x] Unit tests: inserts above/left shift the location, ~~inserts inside grow it, deletes shrink it, deleting the top-left cell~~ (see Outcome) or the sheet drops the pivot, renaming the source sheet keeps GETPIVOTDATA answering.
- [x] web.md records ~~that Excel refuses deleting a pivot's top-left cell while the engine drops the pivot~~ the engine's rule (see Outcome), and the T-2081 interim gap is removed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Refreshing the cache after an edit of its source range (SPEC-0015 non-goal).

## Outcome
Pivots follow row/column inserts and deletes, sheet rename and sheet deletion (`Model::displace_pivots`,
`take_pivots_on`, `restore_pivots`, `rename_pivot_sources` in pivot.rs). An insert at or before a pivot's first line, or a
delete wholly before it, shifts it; an edit after its last line leaves it; a delete of every one of its lines drops it. Changed
from the criteria after the Excel-fidelity review: an insert or delete that cuts through a pivot (inside it, or its first,
inner or last line) is refused with an error before anything changes, instead of growing, shrinking or dropping the pivot.
Nothing evidences grow/shrink, and SPEC-0015 says Excel refuses such edits, which the engine can do as it does for array
formulas. A sheet deletion keeps its pivots in `TakenSheet`, so undo restores them; a rename keeps them and rewrites the
cache sources naming the sheet. New web.md entry "Pivot tables and structural edits" (UNVERIFIED); SPEC-0015 updated.
Eval delta: none (0 fixed, 0 regressions; the harness cannot drive structural edits). Follow-ups: T-2401 (pivots follow
row/column moves, which still drop them), T-2402 (record which inserts/deletes Excel refuses around a pivot).
