---
id: T-1182
title: "On a sheet with a filtered table every hidden row counts as filtered for SUBTOTAL"
milestone: M1
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1036]
---

## Goal
SUBTOTAL 1-11 treat every hidden row on a sheet that has a filtered table as filtered (skipped), not only rows inside the table's range and columns.

## Context
probes/subtotal-hidden-filtered (excel-graph, T-1036): Filter!H1 `=SUBTOTAL(9,B2:B12)` is 237 (the hand-hidden row 12 below the table is skipped too; engine 493) and Filter2!H1 `=SUBTOTAL(9,F1:F12)` is 2030 (rows 1 above, 5 inside and 12 below a table at C3:D10, read in column F outside the table, are all skipped; engine 4095). `cell_hidden_status`/`get_table_for_cell` in functions/subtotal.rs only call a row filtered when the read cell lies inside a table with filters. On a sheet without a filtered table (Hide, Tbl) hidden rows stay manual. docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows".

## Acceptance criteria
- [ ] Filter!H1 and Filter2!H1 pass and no other probe cell regresses; `make eval-baseline` records them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
