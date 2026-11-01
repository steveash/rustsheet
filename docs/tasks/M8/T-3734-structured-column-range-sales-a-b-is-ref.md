---
id: T-3734
title: "Structured column range Sales[[A]:[B]] is #REF! when either end column is deleted"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A structured column range `Sales[[Qty]:[Val]]` is `#REF!` (value and FORMULATEXT) as soon as either end column is deleted.

## Context
T-2899 probes (excel-graph 2026-11-01): table-column-range-delete-end (Val deleted) and -delete-start (Qty deleted) give
`SUM(Sales[[Qty]:[Val]])` `#REF!`, `COLUMNS(...)` `#REF!` and FORMULATEXT `=SUM(#REF!)`; the engine prints the displaced plain
area instead. Deleting both ends (one edit or two) is `#REF!` too and already matches in the one-edit case
(table-column-range-delete-both-ends passes except the plain-range cells T-2851 owns). The plain `S!C4:D7` after a one-column
delete shrinks to `S!C4:C7` in Excel (probe cells F!A4, F!A5): a column-delete corner of T-2851's.
Code: stringify/structured.rs `deleted_column`.

## Acceptance criteria
- [ ] table-column-range-delete-end, -delete-start and -delete-both-ends-separate: the structured cells (F!A1-A3) pass.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
- The plain-range cells F!A4/F!A5 of those probes (T-2851).

