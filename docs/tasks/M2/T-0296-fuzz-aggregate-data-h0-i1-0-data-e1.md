---
id: T-0296
title: "Fuzz: AGGREGATE(Data!H0:I1,0,Data!E1:E6) reaches a scalar-context cell with a 2x2 array (debug_assert panic)"
milestone: M2
lane: functions-math
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Evaluating `=AGGREGATE(Data!H0:I1,0,Data!E1:E6)` in a cell never panics, in debug or release builds, and gives Excel's result.

## Context
Nightly fuzz run 37881397477 (2026-10-09, `main` b5dd7cd), target `formula_eval`, artifact
`crash-580f499317b1f0b76221bfc18c45df967373cfa1`. The input is the 34 bytes
`AGGREGATE(Data!H0:I1,0,Data!E1:E6)`; the target sets it as `=...` in `C3` of its small model
(`fuzz/fuzz_targets/formula_eval.rs`). Reproduced by the steward on 2da0c8c with a debug test
through the facade (`Workbook::set`):

    panicked at crates/rustsheet-engine/src/model.rs:998:25:
    Larger-than-1x1 array reached scalar-context cell (sheet=0, row=3, column=3, 2x2);
    implicit intersection was expected to collapse it.

Stack: `Model::set_cells_with_result` <- `run_top` (evaluation.rs:899) <- `evaluate_formula_cell`.
Same assert as T-0278's xlsx_import crash, reached through a different function: static analysis
treats this AGGREGATE call as scalar while evaluation returns a 2x2 array (note the row-0
corner `H0`, which is not a valid A1 row, and function 0, which is out of range). Find which
part the result shape comes from (`functions/aggregate.rs`, static analysis) and make the two agree.
Excel's value for the formula needs the oracle (excel-probe skill) if it is not already implied by
an existing probe.

## Acceptance criteria
- [ ] The reproducer is a unit test (decision 0035) that passes in a debug build.
- [ ] Static analysis and evaluation agree on the result shape of AGGREGATE with a multi-cell first argument; the value matches excel-graph evidence, or the case is left pending and the task blocked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions that may reach the same assert (file them as found).
