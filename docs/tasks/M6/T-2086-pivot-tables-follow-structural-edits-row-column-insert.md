---
id: T-2086
title: "Pivot tables follow structural edits: row/column insert and delete, sheet rename and delete (SPEC-0015)"
milestone: M6
lane: engine
status: todo
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

## Acceptance criteria
- [ ] Unit tests: inserts above/left shift the location, inserts inside grow it, deletes shrink it, deleting the top-left cell or the sheet drops the pivot, renaming the source sheet keeps GETPIVOTDATA answering.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refreshing the cache after an edit of its source range (SPEC-0015 non-goal).
