---
id: T-3033
title: "Import and export a table column's calculatedColumnFormula"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
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
- [x] Import reads `<calculatedColumnFormula>` per T-2969's recorded rule. Export writes it for a column that has one,
      in file form. An xlsx round-trip test pins both.
- [x] references.md's Ours paragraph no longer says the column formula is not read or written.
- [x] `make check` green; reviewers approve.

## Out of scope
- The fill rules themselves (T-2958, T-3024, T-3025).

## Outcome
Import keeps `<calculatedColumnFormula>` text on `TableColumn::calculated_column_formula` (no fill on load, per
T-2969). A typed fill now records the column formula in file form (first data row, `to_excel_string`), and export
writes the stored text, so an exception typed into a data row does not change it (probes/t2957-constant-then-refill).
Tests: `a_calculated_column_keeps_its_column_formula` (xlsx round trip, data row differs from the column formula) and
`the_column_formula_is_file_form_and_survives_an_exception` (engine). xlsx-storage.md and references.md updated; that
Excel opens our exported column formula without repair is unprobed. Eval delta: none.

## Handoff
Tried: import keeps `<calculatedColumnFormula>`, a typed fill records it in file form, export writes the stored text
(PR #2670, branch agent/engine/T-3033-calculated-column-formula, `make check` green on 6d363dd; security,
architecture and quality approve). Excel-fidelity requested changes twice. Round 1 (fixed): export read the first data
row instead of the fill-time formula, and the fallback was not file form. Round 2 (open, MAJOR): importing the column
formula makes a column Excel saved as calculated re-fill on a typed formula (references.md "Ours" paragraph), and no
evidence shows Excel does that after load. t2983-identical-stored-refill only covers the case without a column formula;
t2969's blank column F cannot tell the two apart. Next: add probes under eval/suites/probes/cases/ with spec2xlsx's
`calculated` option (t2983-identical-stored-refill plus `"calculated": {"C": "A2*2"}`, and a structured variant), fill
them on excel-graph (excel-probe skill), then cite the cells in references.md, or if Excel does not re-fill, make
import keep only what the evidence shows. Also address the MINORs: a note in xlsx-storage.md that the stored text is
not updated on row/column inserts, that an empty element reads as none and `array="1"` is dropped, and have the
exception test copy the probe's cells (type into C4).
