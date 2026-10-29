---
id: T-2968
title: "Typing beside a table auto-expands it and fills the new column, as Excel does (probes/t2957-expand-right-*)"
milestone: M2
lane: engine
status: done
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
- [x] The listed known failures of probes/t2957-expand-right-data and probes/t2957-expand-right-header disappear with
      no new ones.
- [x] Engine unit tests for both sequences, and that a stored (imported) cell beside a table does not expand it.
- [x] The references.md entry gets an "Ours:" line; `make check` green; reviewers approve.

## Out of scope
- The calculated-column fill inside an existing column (T-2958).
- Row auto-expansion below a table (unprobed).

## Outcome
- A formula typed (not stored) into the cell just right of a table's header row or a data row grows the table by one
  column: `Model::table_expansion` (calculated_columns.rs) finds it, and `set_typed_user_input` applies it before the
  calculated-column fill runs over the new column. The new column is named as a column insert names one
  (`table_edits::append_column`, reusing `edit_columns`), and the name is written into the header cell.
- A formula typed into the header cell itself is not stored: the header is the text `0` (-header).
- The table does not grow, and the cell is typed as before, in four unprobed cases: a constant typed there, a table
  with a totals row, a new column whose other cells are not all blank, and a new column that another table overlaps.
  Nor does it grow for a formula that does not parse there as typed (a parse error, or a refusal per decision 0103):
  Excel refuses that entry.
- Facade (`Txn::set`): the growth is part of the edit. It logs the header cell and the filled rows, and
  `Undo::TableExpanded` puts the table back on rollback (`Model::restore_table`).
- Tests:
  - Engine (test_calculated_columns.rs): both probe sequences, plus no growth for a stored formula, a constant, a
    totals row and an occupied column.
  - Facade (edit/tests/calculated_column.rs): one edit holds D1:D4, and rollback restores the table.
- probes/t2957-expand-right-data now declares the hand expectation `S!D1` = `Column1`. The sidecar records only
  formula cells, and the oracle's own H3/H4 read that header (as table-insert-column-inside does for its inserted
  header). Without it, the spill-extent check scores the header Excel writes as a cell Excel left empty.
- Eval delta: probes known failures 2167 → 2157 (the 10 listed cells fixed), 0 regressions.
- Follow-up: T-3003 (probe the unprobed cases: constants, totals row, occupied column, below the last row, the
  header `0`'s type).
