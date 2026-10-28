---
id: T-2838
title: "Keep a table's autoFilter criteria through import and export, and probe an exported filter"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2196]
---

## Goal
An exported table with an applied filter carries its filter criteria, so Excel reads the file's filter (and the
hidden rows SUBTOTAL calls filtered) exactly as it read the imported one.

## Context
T-2196 exports table parts. The import keeps only `Table::has_filters` (whether the `<autoFilter>` has children),
not the `<filterColumn>` criteria, so the export writes an applied filter as `<filterColumn colId="0"/>` with no
criterion (`auto_filter`, crates/rustsheet-xlsx/src/export/tables.rs). That is schema-valid and keeps our own round
trip (crates/rustsheet-xlsx/tests/calc_test_tables/tables.xlsx sheets `filters` and `Functions Filters`), but how
Excel reads it (repair prompt? does SUBTOTAL still treat the sheet's hidden rows as filtered?) is unprobed:
xlsx-storage.md "Table parts in the export".

## Acceptance criteria
- [ ] The import keeps each table's filter columns (values filters, custom filters, at least the kinds tables.xlsx
      uses) and the export writes them back; `has_filters` derives from them.
- [ ] A probe (excel-probe skill) of an exported workbook with a filtered table records whether Excel opens it without
      repair and what SUBTOTAL(109, ...) over its hidden rows gives, and whether a totals row whose cells hold
      constants (no label or formula metadata) loads; xlsx-storage.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter (computing which rows it hides).
- Sort state.
