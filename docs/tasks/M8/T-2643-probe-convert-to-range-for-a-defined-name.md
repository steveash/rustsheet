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
- [x] Probes with excel-graph oracle values added, references.md updated, engine matches (values; FORMULATEXT of the off-row cells split to T-2763).
- [x] `make check` green; reviewers approve.

## Out of scope
-


## Outcome
- New probe `probes/table-edits/table-convert-names-this-row` (13 cells, excel-graph). A defined name `=Sales[Qty]` or
  `=Sales[#All]` keeps working after the convert. An off-row `#This Row` is `#VALUE!`, and FORMULATEXT keeps
  `=Sales[@Qty]`.
- The harness now defines structured-reference names after the tables.
- Convert to range now lowers each off-row `#This Row` cell at its own row (`Model::lower_this_row_references`,
  tables.rs). Data-row cells keep the shared relative form (`=$C4*2`).
- Eval: the new case passes 10/13. The 3 FORMULATEXT cells (S!F2, S!F10, T!A2) need a form for a structured reference
  to a table that no longer exists; that is T-2763. No regressions.
