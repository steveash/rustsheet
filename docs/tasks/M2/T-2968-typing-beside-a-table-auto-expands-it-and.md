---
id: T-2968
title: "Typing beside a table auto-expands it and fills the new column, as Excel does (probes/t2957-expand-right-*)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2958]
---

## Goal
A formula or constant typed into the cell just right of a table grows the table by one column, as Excel does; a
formula typed into a data row there fills the new column (T-2958's fill), and one typed into the header row becomes
the header text Excel shows.

## Context
Found by T-2957 (docs/excel-semantics/references.md, "Calculated columns: a formula typed into a table column"). Table
`T` A1:C4; typing beside it (excel-graph, 2026-10-27, two runs each):
- probes/t2957-expand-right-data: D2 typed `=A2*2`. `T` becomes A1:D4 (`COLUMNS(T[#All])` 4 in H1, `ROWS` 4 in H2),
  D1 holds the text `Column1` (H3; `ISFORMULA(D1)` FALSE in H4), and D2:D4 hold `=A2*2`, `=A3*2`, `=A4*2` (F2:F4,
  values 2, 4, 8). Excel saves `calculatedColumnFormula` `A2*2` for the new column.
- probes/t2957-expand-right-header: D1 typed `=1+1`. `T` becomes A1:D4 and the header is the **text** `0` (H3), not a
  formula and not `2`; the new column is named `0` and its data cells stay blank (F2:F4 `#N/A`).
Known failures of these cases in eval/baseline/probes.txt: expand-right-data S!D3, S!D4, S!F3, S!F4, S!H1, S!H3;
expand-right-header S!F1, S!H1, S!H3, S!H4. Column naming on insert already exists (`ColumnN`, T-2339,
table_edits.rs `edit_columns`); reuse it. Constants typed beside the table, typing below the last row (row
auto-expansion), and typing beside a table with a totals row are unprobed: probe them first if this task covers
them.

## Acceptance criteria
- [ ] The listed known failures of probes/t2957-expand-right-data and probes/t2957-expand-right-header disappear with
      no new ones.
- [ ] Engine unit tests for both sequences, and that a stored (imported) cell beside a table does not expand it.
- [ ] The references.md entry gets an "Ours:" line; `make check` green; reviewers approve.

## Out of scope
- The calculated-column fill inside an existing column (T-2958).
- Row auto-expansion below a table (unprobed).
