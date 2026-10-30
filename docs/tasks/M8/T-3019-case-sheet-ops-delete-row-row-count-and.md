---
id: T-3019
title: "Case sheet ops: delete_row row count and header-less tables, for the T-2848 residual probes"
milestone: M8
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Probe cases can delete several rows in one op and describe a table without a header row, so the residual
T-2848 shapes can be probed on Excel and the engine.

## Context
`delete_row` (tools/src/rustsheet_tools/sheetop.py, oracle/backends.py `_RANGE_EDITS`, rustsheet-eval
engine.rs) deletes one row; Graph can delete `range('4:7')/delete`. Spec tables have no header-less form
(spec2xlsx.py). Unprobed after T-2848 (references.md, row deletes on a table's data rows): one delete of
every data row at once (rows 4-7 of `Sales` on S!B3:C7), a two-row delete reaching a data edge (rows 6-7),
and a delete of the first row of a table without a header row.

The probes baseline also holds 7 harness false failures from T-2848: input cells `S!B5`, `S!C5`, `S!E5`
(table-delete-row-only-data-outside), `S!E7`, `S!F7` (-second-table), `S!B5`, `S!C7` (-totals) score as
"expected empty" against the engine's unmoved input value, as `S!C7` of table-delete-row-all-data-below does
(T-2855).

## Acceptance criteria
- [ ] Those 7 cells leave eval/baseline/probes.txt (the harness scores the oracle's unmoved input cells).
- [ ] `delete_row` takes an optional row count on the oracle and engine sides; spec tables take a
      header-less flag; validated in sheetop.py with tests.
- [ ] The three shapes above are probes/table-edits cases, oracle-filled, recorded in references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the table-edit engine behavior (file follow-ups for disagreements).
