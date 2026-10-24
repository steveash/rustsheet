---
id: T-2402
title: "Record which row/column inserts and deletes Excel refuses around a pivot table (SPEC-0015 structural edits)"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2086]
---

## Goal
docs/excel-semantics/web.md "Pivot tables and structural edits" states, from evidence, which whole-row and whole-column inserts and deletes Excel refuses on a sheet holding a pivot table, and what an allowed one does to the pivot, so its UNVERIFIED status goes.

## Context
T-2086 made the engine shift a pivot for an insert at or before its first line or a delete wholly before it, drop it for a
delete of every one of its lines, and refuse (`Err`) any other insert or delete that meets its lines (`displace_span`,
crates/rustsheet-engine/src/pivot.rs). The refusal rests on SPEC-0015's statement that Excel refuses an edit that would change
part of a pivot; no oracle evidence backs it, nor the drop. The formula oracle cannot make structural edits; Microsoft Graph's
workbook range `insert` and `delete` actions (`shift: Down/Up/Right/Left` on an `entireRow`/`entireColumn` range) on a workbook
holding a pivot can, as can a recorded desktop observation (decision 0039: desktop-only evidence never blocks a task).
Cases: insert rows above, at the first row, inside and just below a pivot; delete rows wholly above, its first row, an inner
row, its last row, every row of it, and a block straddling its first row; the same for columns.
Also moves (cut and insert of whole rows or columns; the engine's `Model::move_pivots`, T-2401): lines above or left of a pivot moved past it, its whole span moved, a group straddling an edge, lines moved entirely inside it; the same for columns. With a report filter (page field) above the body, also an insert
between the filter area and the body (the engine only checks `<location>`; page-field pivots are `#REF!` today).

## Acceptance criteria
- [ ] web.md "Pivot tables and structural edits" lists each case above with Excel's outcome (inserts, deletes and moves) and its evidence (Graph run or desktop observation, dated).
- [ ] Where Excel differs from the engine's rule, a follow-up engine task is filed with the case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (follow-up tasks).
