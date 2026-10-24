---
id: T-2340
title: "References to a deleted table or a deleted sheet's table are #REF! and FORMULATEXT prints #REF! as Excel does (T-2324)"
milestone: M8
lane: engine
status: todo
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
- [ ] Every cell of table-delete-table and table-delete-sheet passes and leaves eval/baseline/probes.txt,
      with no new known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).

## Handoff
Tried (sonnet run). Table-delete half works on the branch (pushed, `make check` NOT yet run, baseline NOT updated,
no unit test yet):
- `Model::delete_table` (tables.rs) reparses with `Parser::set_loaded(true)` plus the new
  `Parser::set_deleted_table(Some(name))` (parser/mod.rs, operands.rs `name_operand`), so structured references and
  the bare name (`ROWS(Sales)`) become `#REF!`. `rustsheet-eval show probes/table-edits/table-delete-table.case.json`
  now passes 12/12.
Left: table-delete-sheet F!A5, A7, A9, A12 (FORMULATEXT only; values match). Expected `=SUM(#REF!)`,
`=SUM(#REF!C4:C7)`, `=COUNTA(#REF!)`, `=COLUMN(#REF!)`; ours prints the lowered `S!$C$4:$C$7` and `S!C4:C7`.
Why I stopped: `take_sheet` (sheet_undo.rs) lowers the other sheets' structured references to plain ranges
(`take_tables_of_sheet`, tables.rs) so `restore_sheet` can put the tables back; the fix needs a rewrite on a real
delete (`Model::delete_sheet`, not the undo path) that turns structured references into `#REF!` and plain
`S!C4:C7` into `#REF!C4:C7` without breaking undo. That is a design call (WrongRangeKind printing/`stringify.rs`,
or a rewrite pass after `take_sheet`). Next: decide that, then `make eval-baseline`, a unit test for both deletes,
excel-semantics entry in references.md "Implemented in", `make check`, review, PR.
