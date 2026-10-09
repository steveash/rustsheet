---
id: T-1196
title: "FREQUENCY data_array: a LET variable or IF bound to an array is counted, not returned"
milestone: M1
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FREQUENCY counts a data_array that is a LET variable, IF or CHOOSE bound to an array or number, instead of returning that value as its result.

## Context
Found by the Excel-fidelity review of T-1065. `fn_frequency` (functions/statistical/frequency.rs)
sends a data_array that `is_multi_area` flags (LET variables, IF, CHOOSE, LET, union-bodied names) to
`evaluate_reference_arg`, and its `RefArg::Value(error) => Err(error)` arm returns any non-reference
value as FREQUENCY's result: `=LET(x,{1,2,3},SUM(FREQUENCY(x,2)))` is 6 on main, where
FREQUENCY({1,2,3},2) is {2;1} (sum 3). Names with an array body are not affected since T-1065 looks
through name bodies. Fix: only an error is returned; an array, number or blank value is collected
like `collect_freq_values` does. Add probe cells (LET, IF and CHOOSE returning an array) and get
oracle values.

## Acceptance criteria
- [x] `=LET(x,{1,2,3},SUM(FREQUENCY(x,2)))` is 3 and the new probe cells match the oracle; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- bins_array (single-reference path, unchanged).

## Outcome
Fixed in the T-1065 PR at the fidelity reviewer's request (a name behind a name reached the same arm):
`fn_frequency` now counts a non-error value from `evaluate_reference_arg` via `freq_values_of`.
probes/union-defined-name-uses H19:H22 (excel-graph, 2026-10-09: `aa` = `=arr` 3 and ROWS 2, the LET
form 3, the IF form 3) pass.
