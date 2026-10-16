---
id: T-1671
title: "Txn::add_table takes a totals row; the eval adapter loads spec totals rows"
milestone: M8
lane: api
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A table added through the facade (`Txn::add_table`) can have a totals row, and the eval engine
adapter loads a spec table's `"totals_row": true` with it, so `Table[#Totals]`, `Table[#All]` and
totals-row formulas evaluate as in Excel.

## Context
- T-1607 added `totals_row` to the case format (`TableSpec`, crates/rustsheet-eval/src/case.rs) and
  spec2xlsx writes it, but `Model::add_table` (crates/rustsheet-engine/src/tables.rs) always sets
  `totals_row_count: 0`, so the adapter registers such a table without its last row
  (`table_range` in crates/rustsheet-eval/src/engine.rs, `TODO(T-1671)`). The engine already reads
  `totals_row_count` from imported xlsx tables.
- Evidence: suite gen-m8b (category `totals`, and `#Totals`/`#All` cells in the other categories).

## Acceptance criteria
- [ ] `Txn::add_table` (or a sibling) can set a one-row totals row; a unit test shows `#Totals` and
  `#All` of such a table.
- [ ] The adapter passes `totals_row` through and the `TODO(T-1671)` is gone; gen-m8b totals cells
  that now pass are recorded with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Totals-row functions metadata (`totalsRowFunction`), table resizing.
