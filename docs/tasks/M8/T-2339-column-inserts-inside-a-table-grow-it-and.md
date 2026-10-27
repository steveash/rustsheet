---
id: T-2339
title: "Column inserts inside a table grow it, and deleting a table column drops it with its references as #REF! (T-2324 evidence)"
milestone: M8
lane: engine
status: done
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
- [x] Every cell of table-insert-column-inside, table-delete-column-qty and -region passes and leaves
      eval/baseline/probes.txt, with no new known failure.
- [x] references.md's "Implemented in" line updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).

## Outcome
Tables now follow a column edit inside them (table_edits.rs `column_span`, `edit_columns`, `inserted_headers`): an
insert right of the first column grows the table and adds a `ColumnN` column whose name is written in its header cell
(new excel-graph probes table-insert-column-inside-header and table-insert-two-columns-inside-header: Excel names the
first `Column1`, then `Column2` on a second insert at C); a delete inside that leaves a column shrinks it and drops the
column, and a structured reference to it prints `#REF!` (stringify/structured.rs `deleted_column`, carried in
`Displacement::deleted_columns`). The header cells S!C3/S!D3 are scored with Excel's text as read through INDIRECT in
the same probes, because the oracle records formula cells only and the spill-extent check would otherwise flag them.
Eval delta (probes): 15 fixed (table-insert-column-inside F!A5, A12; table-delete-column-qty F!A2-A5, A10, A12;
table-delete-column-region F!A2-A5, A9, A10, A12), 0 regressions, 2 new cases with 3 new-coverage failures (INDIRECT of
a structured reference: T-2888). Not met here: table-delete-column-qty F!A7 (`=SUM(S!#REF!)`, a plain range print) is
T-2855's. Follow-ups: T-2888, T-2899 (probe the generalised shapes).
