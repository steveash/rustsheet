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
probes/vlookup-array-lookup-value A45 (`VLOOKUP({2;3},A1:B4,{1,2},FALSE)` spills only 2; 3, so the
index array is not lifted and its first element is used) and A49 (`{FALSE;TRUE}` range_lookup,
"two"; "three") fail with `#N/IMPL!` from `lookup_index_arg`/`get_boolean`. Probe a scalar lookup
value with an array index, a range index and a non-first element that differs before changing
code (lookup.md "VLOOKUP / HLOOKUP lift over an array lookup_value", T-0270).

## Acceptance criteria
- [ ] Probe case filled by excel-graph; rule recorded in lookup.md.
- [ ] A45, A46, A49, A50 pass; no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP/MATCH.
