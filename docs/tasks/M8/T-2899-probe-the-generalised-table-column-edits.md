---
id: T-2899
title: "Probe the table column edits T-2339 generalised: ColumnN naming with gaps and case clashes, a multi-column insert in one op, deleting every column, a delete straddling an edge, a column range losing one end"
milestone: M8
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel evidence for the table column-edit shapes T-2339 implemented by generalisation, so each is either confirmed or
filed as an engine fix.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P3).
T-2339 (excel-graph probes/table-edits table-insert-column-inside, -inside-header, table-insert-two-columns-inside-header,
table-delete-column-qty, -region) grows `Sales` on a column insert inside it, names the new column `Column1` (then
`Column2` on a second insert at C) in its header cell, and shrinks it on a delete inside, printing a reference to the
deleted column as `#REF!`. Unprobed, implemented by hand (table_edits.rs `edit_columns`, `column_span`;
stringify/structured.rs `deleted_column`):
- naming: the least `ColumnN` no column has, with a gap (`Column2` existing, no `Column1`) and a case clash (`column1`);
- a single insert of two or more columns inside (the adapter's `insert_column` has no count today: Graph
  `range('C:D')/insert` would need a `count` key in tools/src/rustsheet_tools/sheetop.py);
- deleting every column of a table, and a delete straddling its left or right edge (both fall back today);
- a column range `Sales[[Qty]:[Val]]` whose one end is deleted (printed as its displaced plain area today, whose
  column-delete corners are T-2851's), and one whose both ends are deleted (printed `#REF!` today, as a single
  deleted column is).
Probe each on excel-graph (excel-probe skill), record the results in references.md "Structured references under row
deletes, column edits, convert, table and sheet delete", and file engine tasks for any mismatch.

## Acceptance criteria
- [ ] Oracle-filled probe cases cover every shape above; their failing cells are baselined.
- [ ] references.md records the results and names the engine tasks filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing fixes (filed as engine tasks).
