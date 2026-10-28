---
id: T-2848
title: "Probe what a whole-row delete of a table's last data row does to that row's cells outside the table, and a multi-row delete of every data row"
milestone: M8
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel evidence for two unprobed shapes of a row delete that takes every data row of a table.

## Context
T-2324 and T-2338 (probes/table-edits/table-delete-row-all-data, -all-data-below) show that when
`delete_row` deletes a table's only data row, Excel keeps the sheet row, clears it, and moves
nothing below it. T-2338 implements that (`Model::last_data_row`, table_edits.rs), clearing only the
table's own cells of the row. Two questions are open:
- the cells of that sheet row outside the table (S!E4 = 5 next to `Sales` on S!B3:C4): cleared, kept,
  or does the delete fail?
- a second table beside it in the same sheet row (`Other` on S!E3:F7 next to `Sales` on S!B3:C4):
  the engine keeps and clears `Sales`'s row and leaves `Other` alone, which is a guess;
- the only data row of a table with a totals row (the engine keeps and clears it as without one);
- a two-row delete reaching a data edge (rows 6-7 of `Sales` on S!B3:C7: the engine shrinks it to
  B3:C5) and a delete of the first row of a table without a header row (the engine falls back);
- one delete of every data row at once (rows 4-7 of `Sales` on S!B3:C7, Graph `range('4:7')/delete`):
  does Excel delete three rows and keep one, refuse, or do something else? The engine treats it as a
  fallback (the structured references print the displaced plain range).
references.md "Structured references under row deletes, column edits, convert, table and sheet
delete" records the finding.

## Acceptance criteria
- [ ] probes/table-edits cases for both shapes, oracle-filled (excel-graph), with a formula on
      another sheet reading each moved or kept cell and its FORMULATEXT.
- [ ] references.md updated; a follow-up engine task filed where the engine disagrees.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine.

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns visibly-off probes cells of SUM (2), ROWS (2), COUNTA (1), COLUMN (1) and COLUMNS (1). With T-2339, T-2855 and the T-0087 sweep it lifts SUM, COUNTA, COLUMN, ROWS and COLUMNS.

## Outcome
Probed three of the five shapes on excel-graph (2026-10-28): the cells outside the table in the cleared
sheet row, a second table beside it, and an only-data-row delete with a totals row (probes/table-edits
`table-delete-row-only-data-outside`, `-second-table`, `-totals`). Excel keeps the outside cells and `Other`
untouched and keeps and clears the totals-table row without moving the totals row; the engine agrees on every
checked `F` cell. Recorded in references.md. A multi-row delete and a header-less table cannot be written as
case ops yet: filed T-3019. Eval delta: probes known failures 2151 -> 2158 (7 new-coverage cells on `S` input
cells that the oracle sidecar leaves empty, as in table-delete-row-all-data-below), 3 new cases, 0 regressions.
