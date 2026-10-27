---
id: T-2338
title: "Tables shrink on row deletes at their data edges and keep one blank data row when every data row goes (T-2324 evidence)"
milestone: M8
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2297]
---

## Goal
A whole-sheet row delete at a table's first or last data row, at the last data row above a totals row,
and of every data row changes the table and the references onto it exactly as Excel does on every cell of
the probes/table-edits delete-row cases.

## Context
T-2324 probed Excel (excel-graph, `delete_row` = Graph `range('r:r')/delete` shift Up) on `Sales` at
S!B3:C7 (header 3, data 4-7, Qty 1,2,4,8; and with a totals row 8); findings in docs/excel-semantics/references.md "Structured references under row deletes, column edits, convert, table and sheet delete".
The engine treats every table as SPEC-0016's fallback (the reference becomes the displaced plain range), so
an edge delete leaves `#REF!` corners: table-delete-row-first-data (row 4: Excel `SUM(Sales[Qty])` 14,
`ROWS(Sales)` 3, FORMULATEXT kept, plain `S!C4:C7` becomes `S!C4:C6`; ours `#REF!`, `=SUM(#REF!:$C$6)`),
table-delete-row-last-data (row 7: 7, 3; ours `#ERROR!`), table-delete-row-last-data-totals (row 7 with
totals: 7, `#All` 5, totals row moves to 7 and `Sales[[#Totals],[Qty]]` is 7), table-delete-row-all-data
(row 4 four times: the table keeps its header and one blank data row, `ROWS(Sales)` 1, `#All` 2,
`SUM` 0, `COUNTA(Sales[Region])` 0, the plain range becomes `S!C4:C4`, not `#REF!`: the last delete keeps the
one data row rather than deleting the sheet row; whether cells below the table shift then is open, so probe
it, excel-probe skill, before implementing that case). T-2297 implements the move and
the shrink strictly inside the data (table-delete-row-before, -inside-data); this task adds the edges,
as T-2321 does for inserts. Excel refuses a delete of the header row (409 InsertDeleteConflict, "it would
move cells in a table"); the engine's answer to that edit is an engine decision (refuse or fall back).

## Acceptance criteria
- [x] Every cell of table-delete-row-first-data, -last-data, -last-data-totals and -all-data passes and
      leaves eval/baseline/probes.txt (`make eval-baseline`), with no new known failure.
- [x] Deleting the totals row removes it from the table (decision 0128, unmasked by T-1671's totals rows):
      table-delete-row-totals F!A3 (`ROWS(Sales[#All])` 5, ours `#ERROR!`) and F!A15 (FORMULATEXT keeps
      `=Sales[[#Totals],[Qty]]`, ours `=#REF!`), and table-delete-row-last-data-totals F!A15 (keeps the text,
      ours `=S!$C$7`) pass and leave eval/baseline/probes.txt.
- [x] references.md's "Implemented in" line updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).

## Outcome
A row delete inside a table's data rows now shrinks it at the edges too, a delete of its totals row
drops that row (`Span::DropTotals`), and a delete of its only data row keeps the row and clears the
table's cells (`Model::last_data_row`; probed first: table-delete-row-all-data-below shows nothing
below moves). A plain range shrinks when a row delete takes its edge rows (`Displacement::range_corners`;
probed with no table: plain-range-delete-row-first, -last). Eval: probes known failures 2865 -> 2821
(46 fixed: every cell of table-delete-row-first-data, -last-data, -last-data-totals, -all-data,
-totals), 0 regressions; 2 new-coverage failures in the new probe cases (T-2855). Follow-ups: T-2848
(probe the row's cells outside the table, a table beside it and a multi-row all-data delete), T-2855
(both new-coverage failures), T-2851 (probe the hand range-shrink shapes). Undo of the kept last data row
is recorded as the clear it is (`UserModel::delete_rows`).
