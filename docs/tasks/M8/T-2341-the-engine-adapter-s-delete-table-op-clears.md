---
id: T-2341
title: "The engine adapter's delete_table op clears the table's cells, as Graph's DELETE tables does (T-2324)"
milestone: M8
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The eval engine adapter's `delete_table` sheet op leaves the sheet as Graph's `DELETE tables('T')` does:
the table and its cells are gone.

## Context
T-2324: after `delete_table` on `Sales` (S!B3:C7) Excel-graph gives `SUM(S!C4:C7)` 0
(probes/table-edits/table-delete-table F!A6), so Graph's table delete clears the range; the adapter
(`apply_sheet_ops` in crates/rustsheet-eval/src/engine.rs) calls `remove_table`, which keeps the cells (ours
15). The same case points to cleared cells, not deleted rows: F!A7 still prints `=SUM(S!C4:C7)` (not shifted or `#REF!`). Confirm it: probe a cell below the table (e.g. S!C9 moving
or not) with the excel-probe skill before changing the adapter, and record it in docs/excel-semantics/references.md "Structured references under row deletes, column edits, convert, table and sheet delete". Desktop has no "delete
table" command (M14 if needed).

## Acceptance criteria
- [x] table-delete-table F!A6 passes; a probe shows whether rows are removed or only cleared.
- [x] docs/architecture/oracle.md's op table states it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).

## Outcome
The adapter's `delete_table` now calls the new `Model::delete_table`, which clears the table's range and then removes the table. F!A6 of table-delete-table passes (probes known failures 12 to 11 in the baseline file; 0 regressions). The oracle shows the cells cleared and F!A7's text unshifted, so rows are cleared, not removed; table-delete-table-cells (excel-graph) shows the header, the text column and the Qty column all cleared and the cells below the table unmoved, so rows are cleared, not removed. oracle.md and references.md state it.
