---
id: T-1269
title: "VLOOKUP/HLOOKUP with an array col_index or range_lookup use the first element (#N/IMPL! today)"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
VLOOKUP/HLOOKUP with an array col_index/row_index or range_lookup give Excel's result instead of `#N/IMPL!`.

## Context
A first oracle fill of probes/vlookup-array-lookup-value (T-0270) showed `VLOOKUP({2;3},A1:B4,{1,2},FALSE)`
spilling only 2; 3 (the index array looks unlifted, its first element used) and `{FALSE;TRUE}` as
range_lookup giving "two"; "three"; the engine gives `#N/IMPL!` from `lookup_index_arg`/`get_boolean`. Probe a scalar lookup
value with an array index, a range index and a non-first element that differs before changing
code (lookup.md "VLOOKUP / HLOOKUP lift over an array lookup_value", T-0270).

## Acceptance criteria
- [ ] Probe case filled by excel-graph; rule recorded in lookup.md. Also probe an error/text index or range_lookup with an array lookup_value (one error or an array of errors?) and blank lookup elements.
- [ ] Both formulas (moved out of T-0270's case) pass in a new probe case; no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP/MATCH.
