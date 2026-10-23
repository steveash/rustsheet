---
id: T-2519
title: "One-cell TAKE/DROP of a range under +0 or N() is a scalar, not a lifting 1x1 array (OFFSET height/width/rows)"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ROWS(OFFSET(B9:C12,5,1,DROP(H2:H4,-2,0)+0,2))` and the same with `N(..)` or TAKE are 3 as in Excel (ours: 1, a lifted 1x1).

## Context
Evidence: probes/t2454-offset-one-cell-array-args A5:D5, A9:D9 and probes/t2454-offset-array-height A12:D12 (excel-graph, 2026-10-23). Excel's DROP/TAKE of a range is a reference, so a one-cell result under `+0` or `N()` is a scalar, while `{3}+0` and SEQUENCE(1,1,3)+0 stay lifting 1x1 arrays. Ours evaluates DROP/TAKE in value context to a 1x1 `CalcResult::Array` (drop_take.rs `take_drop_reference`, value path), which OFFSET then lifts. T-2454 fixed the bare form by reading OFFSET's numbers as references (`offset_values`).

## Acceptance criteria
- [ ] The cells listed above pass and leave eval/baseline/probes.txt; `{3}+0` and SEQUENCE rows keep passing.
- [ ] `make check` green; reviewers approve.

Also probe `CHOOSE(1,DROP(H2:H4,-2,0))` and `LET(x,DROP(H2:H4,-2,0),x)` as OFFSET's height, and find the mechanism of the scalar height 1E-15 giving 2 rows (lookup.md, probes/t2454-offset-array-height A1).

## Out of scope
- Arguments the probes did not cover.
