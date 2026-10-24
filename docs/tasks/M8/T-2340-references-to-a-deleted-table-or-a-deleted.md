---
id: T-2340
title: "References to a deleted table or a deleted sheet's table are #REF! and FORMULATEXT prints #REF! as Excel does (T-2324)"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-2322, T-2341]
---

## Goal
After a table is deleted, or its sheet is, every structured reference to it is `#REF!` and FORMULATEXT
prints it as `#REF!`, as Excel does.

## Context
T-2324 (excel-graph) in docs/excel-semantics/references.md "Structured references under row deletes, column edits, convert, table and sheet delete". table-delete-table (Graph `DELETE tables('Sales')`): `SUM(Sales[Qty])`,
`ROWS(Sales)`, `ROW(Sales[#Headers])`, `COLUMNS(Sales)`, `COLUMN(Sales[Qty])` are `#REF!`,
`COUNTA(Sales[Region])` is 1, and FORMULATEXT prints `=SUM(#REF!)`, `=COUNTA(#REF!)`, `=COLUMN(#REF!)`; ours
keeps a parse error (`#ERROR!`, `#NAME?` for ROWS/COLUMNS, text unchanged). table-delete-sheet (sheet S
deleted): values already match; FORMULATEXT prints `=SUM(#REF!)` for the structured reference and
`=SUM(#REF!C4:C7)` for a plain `S!C4:C7` (ours prints `S!$C$4:$C$7` and `S!C4:C7`). T-2322 makes a loaded
unknown table `#REF!`; share its rewrite. A6 of table-delete-table (`SUM(S!C4:C7)` 0: Graph clears the
cells) is T-2341's.

## Acceptance criteria
- [ ] Every cell of table-delete-table and table-delete-sheet passes and leaves eval/baseline/probes.txt,
      with no new known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).
