---
id: T-3003
title: "Probe typing beside a table: constants, totals row, occupied or overlapped new column, and below the last row (auto-expansion)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/references.md "Calculated columns" rule 7 knows what Excel does when a user types beside a
table in the cases T-2968 left unprobed, each backed by excel-graph probe cases with `entered` cells.

## Context
T-2968 grows a table only for a formula typed just right of its header row or a data row, with the new column
otherwise blank, no totals row and no other table in the way (calculated_columns.rs `Model::table_expansion`). Every
other case keeps the old behavior (the cell is typed and the table does not grow), and none of them is probed. Model
the new cases on probes/t2957-expand-right-data and -header (table `T` A1:C4; readouts `COLUMNS(T[#All])`,
`ROWS(T[#All])`, `D1&""`, `ISFORMULA(D1)`, FORMULATEXT of the new column). Use the `excel-probe` skill. Cases to
probe:
- a constant (number, text) typed into D2 and into D1;
- a formula typed beside a table with a totals row (beside a data row, and beside the totals row);
- a formula typed into D2 when D3 or D1 already holds a value;
- a formula typed into the cell just below the last data row (row auto-expansion);
- an unqualified structured formula (`=[@Col]*2`) typed into D2, which the engine refuses there as a parse error;
- whether the header `0` of a formula typed into D1 is text or the number 0 (`ISTEXT(D1)`), and what `=2+3` or
  `="x"` typed there gives.

## Acceptance criteria
- [ ] Probe cases under eval/suites/probes/cases with excel-graph sidecars, one per bullet above.
- [ ] references.md rule 7 states each finding with its evidence. Engine follow-ups are filed as tasks for the
      cases where the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings (follow-up tasks).
