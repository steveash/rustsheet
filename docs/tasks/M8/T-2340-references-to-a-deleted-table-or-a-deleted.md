---
id: T-2340
title: "References to a deleted table or a deleted sheet's table are #REF! and FORMULATEXT prints #REF! as Excel does (T-2324)"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
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
- [x] Every cell of table-delete-table and table-delete-sheet passes and leaves eval/baseline/probes.txt,
      with no new known failure.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).

## Outcome
A table delete and a sheet delete now rewrite the stored text of every structured reference to the removed
tables (bare `Sales` included) to `#REF!` (`Model::rewrite_table_references`, tables.rs, through a new
`Rewrite::RefError` printer), instead of the handoff's parse-time flag, which a later reparse would have
lost. The sheet delete records what it rewrote (`TakenSheet`, `Diff::DeleteSheet`), so `restore_sheet` and
`UserModel` undo put the structured text back (they used to leave lowered plain ranges). A reference to a
missing sheet is displayed as `#REF!C4:C7` (`is_missing_sheet`, stringify/wrong_reference.rs; stored text and export keep
the name), and the lexer reads `#REF!C4:C7` back as `#REF!`, so autofill of such a formula is not a parse
error. Three upstream move-formula expectations now print `#REF!AB31` for a missing sheet.
Follow-up: T-2612 (the stored text and export of a plain reference to a deleted sheet).
Eval: probes known failures -14 (table-delete-table and table-delete-sheet now pass 12/12 each); 0
regressions.
