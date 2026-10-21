---
id: T-2324
title: "Probe row and column deletes, column inserts, convert-to-range, table delete and sheet delete on a table (T-2197 part 2)"
milestone: M8
lane: research
status: todo
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
- [ ] Probe cases with oracle values for each listed edit.
- [ ] references.md updated; an engine task filed for each finding the engine misses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementation.
