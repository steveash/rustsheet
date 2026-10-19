---
id: T-1958
title: "XMATCH lifts search_mode over a range or an array"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XMATCH's search_mode lifts over a multi-cell range or an array, each element the scalar XMATCH's result (a value), as Excel does.

## Context
Split from T-1810. Evidence: docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (probes/reference-lift-elements, excel-graph, 2026-10-19), last bullet: `XMATCH(2,A1:A3,0,{1;-1})` is 2, 2; an invalid mode element is `#VALUE!` in its slot (`{1;3}`); a boolean range element counts as its number (TRUE is 1); `XMATCH(2,{1;2;2},0,{2;-2})` is 2, 3. Today search_mode is a plain scalar (`#N/IMPL!` over an array). Failing cells: probes/reference-lift-elements F13:H14. `fn_xmatch` (functions/lookup_and_reference/xmatch.rs) already lifts lookup_value and match_mode through one `lift_values` call; add search_mode to that call (validated per element), not a new broadcast.

## Acceptance criteria
- [ ] probes/reference-lift-elements F13, G13, H13, F14, G14, H14 pass; eval/baseline updated with no new known failures.
- [ ] lookup.md's XMATCH bullet notes it is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
