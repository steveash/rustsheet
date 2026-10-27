---
id: T-2848
title: "Probe what a whole-row delete of a table's last data row does to that row's cells outside the table, and a multi-row delete of every data row"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
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
