---
id: T-2838
title: "Keep a table's autoFilter criteria through import and export, and probe an exported filter"
milestone: M8
lane: engine
status: done
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
- [x] The import keeps each table's filter columns (values filters, custom filters, at least the kinds tables.xlsx
      uses) and the export writes them back; `has_filters` derives from them (it keeps its
      import rule instead; see Outcome).
- [ ] (Split to T-3081.) A probe (excel-probe skill) of an exported workbook with a filtered table records whether Excel opens it without
      repair and what SUBTOTAL(109, ...) over its hidden rows gives, and whether a totals row whose cells hold
      constants (no label or formula metadata) loads; xlsx-storage.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter (computing which rows it hides).
- Sort state.

## Outcome
- `Table::filter_columns` (`FilterColumn`, `FilterCriterion`, `CustomFilter` in rustsheet-engine `types`) keeps each
  `<filterColumn>`: a values filter with its blank choice, a custom filter with `and` and its operator/value
  conditions, and any other kind (top 10, dynamic, color, icon) as its column only. The export writes them back
  (`filter_column` in export/tables.rs); a table filtered through the API with no kept criterion still writes
  `<filterColumn colId="0"/>`.
- `has_filters` keeps its import rule (the autoFilter has any child) instead of deriving from the kept columns: an
  autoFilter holding only a `<sortState>` or `<extLst>` stays flagged as before, so SUBTOTAL's behavior is unchanged.
- Values filters with date groups (`dateGroupItem`, `calendarType`) are kept as the criterion-less kind, not as an
  empty values list; filter values are decoded from `_xHHHH_` escapes; kept criteria move with their columns
  through column inserts and deletes inside the table (`edit_filter_columns`, table_edits.rs).
- Tests: tables.xlsx's two filters (Table4 custom, Table16 values) survive an export byte for byte and reload equal;
  every criterion kind (blank, AND, escaped operand, kind not kept) round-trips.
- The Excel probe of an exported filter (and of a constant totals-row cell) is split to T-3081.
- Eval delta: none (xlsx round trip only; no case scores change).
