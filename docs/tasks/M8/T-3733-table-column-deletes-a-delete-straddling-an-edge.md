---
id: T-3733
title: "Table column deletes: a delete straddling an edge or removing every column match Excel"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine's column deletes that straddle a table edge or remove every column of a table give Excel's values and FORMULATEXT text.

## Context
T-2899 probes (excel-graph 2026-11-01, docs/excel-semantics/references.md "Structured references under row deletes, column edits"):
- table-delete-columns-straddle-left (delete B:C, table on C3:E7): the table survives without the deleted column, `COLUMNS(Sales)` 2,
  `INDEX(Sales[#Headers],1,1)` "Qty", `SUM(Sales[Qty])` kept as `=SUM(Sales[Qty])`, `Sales[Region]` is `#REF!` (`=COUNTA(#REF!)`).
  The engine falls back to a plain range (`#REF!` for `COLUMNS`/`ROWS`).
- table-delete-columns-straddle-right (delete D:E, table on B3:D7): same, the last column is dropped and the table keeps two.
- table-delete-all-columns (delete B:C, the whole table): every reference is `#REF!`, `ISREF(Sales)` FALSE, `COUNTA(Sales[Region])` 1;
  the engine matches in value but FORMULATEXT prints `=SUM(S!#REF!)` where Excel prints `=SUM(#REF!)`.
Code: table_edits.rs `edit_columns`, `column_span`; stringify/structured.rs `deleted_column`.

## Acceptance criteria
- [ ] The three probes above (eval/suites/probes/cases/table-edits/) pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column ranges that lose one end (T-3734).

