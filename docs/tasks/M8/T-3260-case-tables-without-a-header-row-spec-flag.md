---
id: T-3260
title: "Case tables without a header row: spec flag, engine and facade support, and the header-less row-delete probe"
milestone: M8
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Probe cases can describe a table without a header row, so the last T-2848 residual shape (a delete of the first
row of a table without a header row) is probed on Excel and the engine.

## Context
- Split from T-3019, which added the multi-row `delete_row` (`count`) and probed the other two shapes.
- spec2xlsx.py `_add_table` requires a literal-text header row; XlsxWriter's `add_table` takes `header_row:
  False`. The engine adapter adds spec tables with `Txn::add_table_with_totals` (crates/rustsheet/src/edit.rs),
  and `Model::add_table_with_totals` (crates/rustsheet-engine/src/tables.rs) always builds the table with one header
  row (`Table::new(.., (1, totals))`) and names columns from it; a header-less table needs Excel's default column
  names (`Column1`, `Column2`, ...: evidence needed via the excel-probe skill) and a facade API change
  (architecture review).
- `TableSpec` in crates/rustsheet-eval/src/case.rs and `moved_input` in spill_extent.rs (data rows start at the
  first row of `ref` without a header).

## Acceptance criteria
- [x] Spec tables take `"header_row": false` in spec2xlsx and the engine adapter, validated with tests.
- [x] A probes/table-edits case deletes the first row of a header-less table, oracle-filled, recorded in
      references.md; disagreements get engine follow-ups.
- [x] `make check` green; reviewers approve.

## Out of scope
- Header-less tables in the xlsx import beyond what the case needs.

## Outcome
- spec2xlsx takes `"header_row": false` on a table (columns `Column1`, `Column2`, ...; one data row is
  enough; no filter or calculated column); tests in tools/tests/test_spec2xlsx_tables.py.
- Engine and facade: `Model::add_table_with_rows` / `Txn::add_table_with_rows` take `(header_row,
  totals_row)`; `add_table_with_totals` delegates. `TableSpec.header_row` (default true) reaches the engine
  adapter, and `spill_extent::moved_input` starts a header-less table's data on its first row. Test
  `table_without_a_header_row_names_its_columns_and_has_no_headers`.
- Probes (excel-graph 2026-10-31): probes/table-edits/table-no-header-baseline and
  table-no-header-delete-first-row, recorded in references.md. Deleting the first row of a header-less table
  shrinks it as a delete of a first data row does. The one engine disagreement, `Table[#Headers]` on a
  header-less table (Excel `#REF!`, the engine read the first data row), is fixed here in the structured
  reference parser as `#Totals` already was. Both cases pass: eval delta +30 passing cells in 2 new cases,
  no known failure added.
