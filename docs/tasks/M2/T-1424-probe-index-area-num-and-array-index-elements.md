---
id: T-1424
title: "Probe INDEX area_num and array-index elements near an integer (does the 2^-22+2^-33 rounding apply?)"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for whether INDEX's area_num, and a near-integer element of an array row or column index, round up near an integer as row_num and column_num do; the engine follows it.

## Context
T-1407 fitted INDEX's row/column rule (lookup.md "INDEX rounds an index up near an integer"; probes/index-snap-tolerance):
within 2^-22 + 2^-33 below an integer counts as that integer, a negative within half that is 0. area_num is still
floored (`index_numbers` in functions/lookup_and_reference/index.rs), and the lifted path (an array index) goes through the same
`index_number` without a probe. Probe with the excel-probe skill: `INDEX((A1:C3,E1:G3),1,1,2-1E-7)`, `..,1,1,1-1E-7)`,
`..,1,1,2-3E-7)`, `..,1,1,-1E-8)`, and `INDEX(A1:C3,{1;3-1E-7},1)`, `INDEX(A1:C3,{-1E-8;1},1)`.

## Acceptance criteria
- [ ] The probe is filled (excel-graph) and recorded in lookup.md; the engine passes it, reusing `near_integer_count`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`probes/index-area-num-snap` (11 cells, excel-graph): area_num rounds up near an integer like row_num (3 cells failed before), and near-integer array-index elements already matched. `index_numbers` now uses `near_integer_count` for area_num. Recorded in lookup.md.
