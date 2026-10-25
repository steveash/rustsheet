---
id: T-2643
title: "Probe convert-to-range for a defined name and out-of-table or cross-sheet #This Row references (T-2342)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Evidence for what convert-to-range does to a defined name that refers to the table and to `#This Row` references the T-2342 probes did not cover.

## Context
T-2342 lowers defined-name references with the delete path's rewrite (unprobed) and probed only a `#This Row` beside a data row on the table's sheet. Probe a defined name `=Sales[Qty]` read via a cell, a `#This Row` outside the data rows, and one from another sheet (excel-probe skill); fix the engine where it differs.

## Acceptance criteria
- [x] Probes with excel-graph oracle values added, references.md updated, engine matches (values; FORMULATEXT of the off-row cells split to T-2751).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done and pushed: probe `probes/table-edits/table-convert-names-this-row` (excel-graph values filled), references.md entry, and a harness fix (`crates/rustsheet-eval/src/engine.rs` defines names containing `[` after the tables, so `Sales[Qty]` names load). With it the defined-name cells (S!E4..E8, T!B1, T!B2) pass 7/13.
Failing: S!F1, S!F9, T!A1 (+ their FORMULATEXT cells F2, F10, A2): Excel keeps `=Sales[@Qty]` and `#VALUE!` for a `#This Row` off the data rows after convert; the engine's `convert_table_to_range` (tables.rs) rewrites the shared R1C1 formula with `Rewrite::Lower(tables, None)`, which prints the relative form `=$C1` (value 0). Cell formulas are shared, so the rewrite has no row. Next: for formulas holding a `ThisRowKind`, rewrite per cell with `to_rc_lowering_at` (structured.rs) and, off the data rows, keep text that evaluates `#VALUE!` (FORMULATEXT parity needs an unresolved-structured-reference form; a design question). Then `make eval-baseline`, `make check`, review.

## Outcome
- New probe `probes/table-edits/table-convert-names-this-row` (13 cells, excel-graph). A defined name `=Sales[Qty]` or
  `=Sales[#All]` keeps working after the convert. An off-row `#This Row` is `#VALUE!`, and FORMULATEXT keeps
  `=Sales[@Qty]`.
- The harness now defines structured-reference names after the tables.
- Convert to range now lowers each off-row `#This Row` cell at its own row (`Model::lower_this_row_references`,
  tables.rs). Data-row cells keep the shared relative form (`=$C4*2`).
- Eval: the new case passes 10/13. The 3 FORMULATEXT cells (S!F2, S!F10, T!A2) need a form for a structured reference
  to a table that no longer exists; that is T-2751. No regressions.
