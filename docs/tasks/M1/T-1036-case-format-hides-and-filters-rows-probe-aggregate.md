---
id: T-1036
title: "Case format hides and filters rows; probe AGGREGATE and SUBTOTAL over them"
milestone: M1
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1168]
---

## Goal
Probe cases can hide rows and filter tables, and AGGREGATE/SUBTOTAL behavior over them is oracle-backed.

## Context
Raised by the Excel-fidelity reviewer on T-0993. The engine's AGGREGATE skips hidden and filter-hidden rows under odd options and keeps filter-hidden rows under even ones; SUBTOTAL always skips filtered rows. Neither is probed because `rustsheet-case/v1` cannot hide rows or apply a table filter (spec2xlsx in tools/, `evaluate_spec` in crates/rustsheet-eval). Also unprobed: whether options 0-3 skip a cell whose formula contains SUBTOTAL/AGGREGATE inside a larger expression (`=1+SUBTOTAL(9,A1:A2)`); `cell_formula_is_one_of` matches only the outermost call. docs/excel-semantics/numbers.md "AGGREGATE" (UNVERIFIED parts).

## Acceptance criteria
- [x] The case format can mark rows hidden and tables filtered, in both spec2xlsx and the engine loader.
- [x] A probe covers AGGREGATE options 0-7 and SUBTOTAL 1-11/101-111 over hidden and filtered rows, and nested calls inside larger formulas; the engine matches or the gaps are filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`rustsheet-case/v1` sheets take `hidden_rows` (1-based) and `tables` (`name`, `ref`, `filter`). spec2xlsx writes the rows
`hidden="1"` and, for a filtered table, patches a values filter on its first column that keeps exactly the visible rows;
the engine loader sets values, adds tables (new `Txn::add_table` over `Model::add_table`, crates/rustsheet-engine/src/tables.rs),
sets formulas, then hides rows. Probe probes/subtotal-hidden-filtered (150 cells, excel-graph): manual hides are read by
SUBTOTAL 1-11 and skipped by 101-111; filtered rows are skipped by both; on a sheet with a filtered table every hidden row is
filtered; AGGREGATE odd options skip both kinds; any formula containing SUBTOTAL or AGGREGATE is skipped. Recorded in
docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows".
Eval delta: probes +1 case, 119/150 cells pass, 31 new-coverage known failures, 0 regressions. Gaps filed: T-1185 (SUBTOTAL
110/111 read hidden rows), T-1182 (sheet-wide filter rule), T-1183 (nested SUBTOTAL/AGGREGATE detection).
