---
id: T-3626
title: "Probe which row's A1 text Excel saves as calculatedColumnFormula"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know which row's A1 text Excel saves as a table column's `<calculatedColumnFormula>` when the filling formula is typed
into a data row below the first one (for example `=A5*3` typed into C5 of a table whose data rows start at row 2).

## Context
docs/excel-semantics/xlsx-storage.md, "A calculated column is saved as `calculatedColumnFormula` plus one formula per
row", marks this unprobed. T-3033 (PR #2670) records the first data row's file-form text
(`crates/rustsheet-engine/src/calculated_columns.rs`, the fill path) and export writes it unchanged, so an A1 fill typed
below row 2 may save different text than Excel. Values are unaffected (the engine only checks whether a column formula
is set); only the exported text is at stake. The excel-graph download behind probes/t3033-calculated-refill-after-load
(typed `=A5*3` into C5) may already answer it: read `xl/tables/table1.xml` from the cached recalculated workbook if it is
still in `eval/.cache`, otherwise write a probe per the `excel-probe` skill (A1 formula typed into C4 of a fresh table,
download the saved file).

## Acceptance criteria
- [ ] The saved `calculatedColumnFormula` text for an A1 formula typed below the first data row is cited from an
      excel-graph download in the xlsx-storage.md entry (replacing the "unprobed" clause).
- [ ] If it differs from the engine's first-data-row text, a follow-up engine task is filed (or the fix is made here if
      it is size S).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rewriting the stored text on row or column inserts (separate unprobed point in the same entry).
