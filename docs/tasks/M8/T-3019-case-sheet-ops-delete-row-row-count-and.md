---
id: T-3019
title: "Case sheet ops: delete_row row count and header-less tables, for the T-2848 residual probes"
milestone: M8
lane: harness
status: done
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
- [x] Those 7 cells leave eval/baseline/probes.txt (the harness scores the oracle's unmoved input cells).
- [x] `delete_row` takes an optional row count on the oracle and engine sides; spec tables take a
      header-less flag; validated in sheetop.py with tests.
- [x] The three shapes above are probes/table-edits cases, oracle-filled, recorded in references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the table-edit engine behavior (file follow-ups for disagreements).

## Outcome
- The 7 T-2848 harness false failures had already left eval/baseline/probes.txt (T-2855's `moved_input`); nothing to do.
- `delete_row` takes an optional `count` (spec `{"op":"delete_row","sheet":"S","row":4,"count":4}`): sheetop.py
  validates it and records a span `4:7` (a count of 1 keeps the plain row, so existing workbooks are byte-identical),
  the Graph replay deletes `range('4:7')`, and the engine adapter calls `delete_rows(.., count)`;
  `SheetOp::moved` and `moved_input` (spill_extent.rs) follow it, including the kept data row below.
- Probes (excel-graph 2026-10-30), recorded in references.md: table-delete-rows-all-data-once (Excel keeps one
  cleared data row; the engine fails 8 cells, owned by T-3259), table-delete-rows-last-two (passes) and
  plain-range-delete-rows-across-edge (passes; it replaces the hand generalisation of multi-row shrinking).
- Header-less tables (spec flag and the third shape) need an engine and facade change: split to T-3260.
- Eval delta: probes +36 checked cells, 28 passing, 8 new-coverage known failures (T-3259); 0 regressions.
