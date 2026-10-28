---
id: T-3081
title: "Probe how Excel opens an exported table filter: kept criteria, a criterion-less filterColumn, SUBTOTAL over its hidden rows"
milestone: M8
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2838]
---

## Goal
Oracle evidence for how Excel reads a table filter our export writes, so xlsx-storage.md "Table parts in the
export" states it instead of calling it unprobed.

## Context
Split from T-2838, which keeps a table's `<filterColumn>` criteria through import and export
(`Table::filter_columns`, `filter_column` in crates/rustsheet-xlsx/src/export/tables.rs). A table that is filtered
with no kept criterion (`Txn::add_table` with `filtered`) is still written as `<filterColumn colId="0"/>`. The probe
cases are built by the spec→xlsx builder, not by our export, so this needs a way to put an exported workbook in
front of the oracle (or a builder path that writes the same table part); find or add one first.

## Acceptance criteria
- [ ] For an exported workbook with (a) a table with a kept values filter and hidden rows, (b) one with a kept
      custom filter, (c) one with a criterion-less `<filterColumn colId="0"/>`, (d) a re-exported top 10 or date-grouped filter
      (written criterion-less): whether Excel opens it without
      repair, and what `SUBTOTAL(109, ...)` and `SUBTOTAL(9, ...)` over the hidden rows give.
- [ ] Whether a totals row whose cells hold constants (no label or formula metadata) loads.
- [ ] docs/excel-semantics/xlsx-storage.md "Table parts in the export" updated with the evidence; follow-up
      engine/xlsx tasks filed for any difference.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter (computing which rows it hides). Sort state.
