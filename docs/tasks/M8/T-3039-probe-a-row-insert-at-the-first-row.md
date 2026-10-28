---
id: T-3039
title: "Probe a row insert at the first row of a table without a header row (spec2xlsx header_row: false)"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/references.md says, with excel-graph evidence, what a whole-row insert at the first row of a
table *without* a header row does: does the table move, or grow by one row at its top?

## Context
- Left open by T-2845. Its other items were probed on 2026-10-28. With a header row, an insert at the first data row
  grows the table (T-2197). For a headerless table the engine falls back (the reference prints the displaced area)
  until this is known. See references.md "Structured references when the table changes".
- The spec builder cannot write a headerless table: `tools/src/rustsheet_tools/spec2xlsx.py` always makes the first
  row of `ref` the header row. Add `"header_row": false` (`headerRowCount="0"` in the table part, with unit tests in
  tools/), then a probe case in eval/suites/probes/cases/table-edits. Model it on table-insert-row-above-first-data:
  `Sales` on S!B3:C6 with data rows 3-6, an `insert_row` at row 3 and at row 4, and F reading `SUM(Sales[Qty])`,
  `ROWS(Sales)`, `ROW(Sales)` and FORMULATEXT. Fill it with `rustsheet-oracle fill -b excel-graph --suite probes
  --case <id>`. The eval adapter reads `header_rows` (crates/rustsheet-eval xlsx_snapshot/tables.rs). Check that the
  engine registers a spec table with no header (it may need the same flag).
- File an engine task for any mismatch.

## Acceptance criteria
- [ ] spec2xlsx supports `"header_row": false`, with tests (tools: ruff, mypy strict).
- [ ] The probe case is oracled (excel-graph) and in the probes baseline.
- [ ] references.md records the finding; implementation tasks are filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column inserts at a headerless table's first column.
