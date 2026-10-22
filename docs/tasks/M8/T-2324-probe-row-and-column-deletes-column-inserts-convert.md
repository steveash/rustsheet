---
id: T-2324
title: "Probe row and column deletes, column inserts, convert-to-range, table delete and sheet delete on a table (T-2197 part 2)"
milestone: M8
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2323]
---

## Goal
docs/excel-semantics/references.md "Structured references when the table changes" covers row deletes,
column inserts and deletes, convert to range, table delete and sheet delete with excel-graph evidence.

## Context
T-2197 settled whole-row inserts (probes/table-edits). The rest of its questions need the sheet ops of
T-2323. Reuse the table-edits layout (Sales at S!B3:C7, formulas on F, with and without a totals row): row
deletes above, at the header, inside, at the last data row, at the totals row, of all data rows;
column inserts left of, inside and right of the table and deletes of a table column; `convert_table`
(do the structured references become plain ranges, and in which form?); `delete_table` (`#REF!`?);
`delete_sheet` of S with the formulas on F (SPEC-0016 assumes `#REF!`). Cut and paste, row and column
moves, undo after a sheet delete and duplicated sheets have no Graph endpoint: put them in M14 Phase 4
(desktop, decision 0039) as T-2197 recorded.

## Acceptance criteria
- [x] Probe cases with oracle values for each listed edit.
- [x] references.md updated; an engine task filed for each finding the engine misses.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementation.

## Outcome
19 probe cases in probes/table-edits (excel-graph, 2026-10-22): row deletes before, at the first data row,
inside, at the last data row (with and without a totals row), below, of the totals row and of every data
row; column inserts at A, B, C, D; column deletes of A, B (Region), C (Qty); convert_table, delete_table,
delete_sheet and the sheet rename. Excel refuses a delete of the header row (409 InsertDeleteConflict), so
that question has no case; references.md records it. references.md's UNVERIFIED entry is replaced by
"Structured references under row deletes, column edits, convert, table and sheet delete" (VERIFIED).
Findings the engine misses are filed: T-2338 (edge row deletes shrink the table), T-2339 (column grow and
table-column delete), T-2340 (`#REF!` for a deleted table or sheet), T-2341 (adapter delete_table clears
the cells, as Graph does), T-2342 (convert to range). The move and inside classes stay with T-2297.
Eval delta: probes +19 cases, 92 new-coverage known failures, 0 regressions; table-delete-row-below,
-row-totals, -insert-column-right-edge and -rename-sheet pass outright.
