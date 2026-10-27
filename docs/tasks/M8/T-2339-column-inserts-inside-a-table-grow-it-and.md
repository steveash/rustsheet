---
id: T-2339
title: "Column inserts inside a table grow it, and deleting a table column drops it with its references as #REF! (T-2324 evidence)"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2297]
---

## Goal
A whole column inserted inside a table grows it, and a deleted table column leaves the table and its
references as Excel does on every cell of the probes/table-edits column cases.

## Context
T-2324 (excel-graph, `insert_column` = Graph `range('C:C')/insert` shift Right, `delete_column` = delete shift
Left) on `Sales` at S!B3:C7; findings in docs/excel-semantics/references.md "Structured references under row deletes, column edits, convert, table and sheet delete". Insert at C (between the two columns): `COLUMNS(Sales)` 3,
`COLUMN(Sales[Qty])` 4, FORMULATEXT kept (table-insert-column-inside; ours prints the plain range).
Delete column C (Qty): `SUM(Sales[Qty])` is `#REF!` and prints `=SUM(#REF!)`, `COLUMNS(Sales)` 1,
`ROWS(Sales)` 4, a plain `S!C4:C7` prints `=SUM(S!#REF!)` (table-delete-column-qty; ours `#ERROR!` and
`#REF!:#REF!`). Delete column B (Region): `COUNTA(Sales[Region])` is 1 (COUNTA counts the `#REF!`) and
prints `=COUNTA(#REF!)`, `SUM(Sales[Qty])` 15 kept as text (table-delete-column-region). Inserts at A or B
and the delete of A move the table (T-2297's move class: table-insert-column-before, -left-edge,
-delete-column-before); an insert at D, right of the table, leaves it (passes).

## Acceptance criteria
- [ ] Every cell of table-insert-column-inside, table-delete-column-qty and -region passes and leaves
      eval/baseline/probes.txt, with no new known failure.
- [ ] references.md's "Implemented in" line updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).
