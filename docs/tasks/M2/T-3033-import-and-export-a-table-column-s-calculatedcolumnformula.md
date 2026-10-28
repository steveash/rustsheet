---
id: T-3033
title: "Import and export a table column's calculatedColumnFormula"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2969]
---

## Goal
A workbook round-trips a table column's calculated-column state: xlsx import reads `<calculatedColumnFormula>`
into `TableColumn::calculated_column_formula` as far as T-2969's probe shows Excel honors it on load, and export
writes it back for a column that has one.

## Context
T-3024 added `TableColumn::calculated_column_formula` (crates/rustsheet-engine/src/types.rs). A typed formula sets it
when it fills a column, and only a column that has it is re-filled (docs/excel-semantics/references.md, "Calculated
columns", rule 8 and its Ours paragraph). Import (crates/rustsheet-xlsx/src/import/tables.rs) sets it to `None`, and
export (crates/rustsheet-xlsx/src/export/tables.rs) does not write it. So a column Excel saved as calculated is, after
import, no calculated column for a typed re-fill. Excel writes it in file form, the first data row's text: `A2*2` or
`T[[#This Row],[Col]]*2` (docs/excel-semantics/xlsx-storage.md, "A calculated column is saved as
`calculatedColumnFormula` plus one formula per row"). T-2969 probes whether Excel re-fills such a column on load and
after a typed formula.

## Acceptance criteria
- [ ] Import reads `<calculatedColumnFormula>` per T-2969's recorded rule. Export writes it for a column that has one,
      in file form. An xlsx round-trip test pins both.
- [ ] references.md's Ours paragraph no longer says the column formula is not read or written.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fill rules themselves (T-2958, T-3024, T-3025).
