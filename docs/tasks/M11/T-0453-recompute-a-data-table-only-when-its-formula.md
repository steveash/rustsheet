---
id: T-0453
title: "Recompute a data table only when its formula's precedents changed"
milestone: M11
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: [T-0441]
---

## Goal
An edit that cannot change a data table's results does not recompute the table. Today every
`evaluate()` recomputes every table: one on-demand evaluation of the table's formula per result
cell, then a second full pass.

## Context
T-0441 (crates/rustsheet-engine/src/data_tables.rs). A workbook with four 9x9 two-variable tables
over a financial model (calipers pv_goldens/pv23_046) re-evaluates the model 324 times on every
evaluation, edits included.

The results depend on three things:
- the table's formula and the cells it reads, transitively;
- the header values;
- the input cells.

Without the dependency graph (T-0401, Phase 2), a conservative test is enough: skip the
recomputation when no cell the last recomputation read has changed. Record the positions read with
`seen` during the substituted evaluations, and compare their values. Directive 5 still bounds the
recorded set.

## Acceptance criteria
- [x] Release-build timings in the Outcome: an edit to a cell no table reads, in pv23_046, before
      and after, at least 10x faster.
- [x] Results unchanged (calipers: the 5 pv23_056 cells fail on main too, T-0463): `make eval` shows no new known failure, and `rustsheet-eval check --suite
      calipers` passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- The dependency graph itself (T-0401).

## Outcome
Every read of a cell goes through `SheetData`, so while a table is recomputed each sheet records
the positions and areas read and whether its extent was (`sheet_reads.rs`; one flag test per read
otherwise). Before the results are written, the contents of those reads (constants, formula texts,
styles, CSE arrays, dynamic spills), the sheets' extents and the workbook context (names, tables,
settings, locale, language, sheets, hidden rows and columns) are kept with the table
(`data_table_memo.rs`). The next evaluation recomputes a table only if any of that, its definition
or its result cells changed. A formula reaching RAND, RANDBETWEEN, RANDARRAY, NOW, TODAY, CELL,
INFO or GETPIVOTDATA (in a cell or a name), a dynamic spill that the recomputation itself moved,
a recomputation that ran out of budget or an evaluation with cycles to iterate always recomputes.
A skipped table is charged its last cost, so the tables after it see the budget unchanged; with no
table recomputed, the second evaluation is skipped too. `seen` (the task's suggestion) only records
empty and occupied positions in a pass, so it could not serve; the `SheetData` record covers every
read path by construction.

Timings, release build, pv23_046 (4 tables, `iterate="1"` with no cycles), edit to a text label
then `evaluate()`, 400 labels: before 71-153 ms, after 3.0-9.6 ms (fastest 71 ms → 3.0 ms, 23x;
an evaluation with no edit 137 ms → 3.8 ms). Results: in the 21 calipers workbooks with data
tables, 25 edits each (numeric constants at a stride), every cell after every edit is identical to
main's (525 evaluations compared). Unit tests: `test_data_table_memo.rs` (unread edits skip; read
cells, empty positions of a range, whole columns past the used area, INDIRECT targets, names,
hidden rows, volatile calls, overwritten results and structural edits recompute).

Eval delta: none (`make eval-check`: 0 regressions). `rustsheet-eval check --suite calipers`
reports 5 regressions, all in pv23_056 (a workbook with no data table, SUM cells a few ulp off);
the same cells fail on the base commit without this change: the stale baseline T-0463 owns.
Follow-up: T-0495 (an edited workbook diverges from a fresh load in 4 calipers workbooks, on main
too).
