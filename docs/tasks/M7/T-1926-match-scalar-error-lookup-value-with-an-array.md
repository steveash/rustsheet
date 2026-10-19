---
id: T-1926
title: "MATCH scalar error lookup_value with an array match_type: probe and align runtime with static analysis"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`MATCH(NA(),A1:A3,{0;1})` gives the result Excel gives: probe it, then make the runtime and `match_static_result` agree.

## Context
Found by the T-1917 fidelity review. `fn_match` returns a scalar error lookup_value before it looks at match_type, so a scalar #N/A; static analysis gives it the 2x1 shape of the array match_type. XMATCH (xmatch.rs:185) lifts it instead. No oracle evidence either way. Also probe `MATCH(25,A1:A3,{0;"x"})` (the #VALUE! slot in test_match_lifts_over_array_match_type is hand-written). Likely fix: guard the early return with `!LiftedArg::is_grid(&match_type)`, as XMATCH does.

## Acceptance criteria
- [x] A probe with `ROWS(MATCH(NA(),A1:A3,{0;1}))`, `INDEX(MATCH(NA(),A1:A3,{0;1}),2)` and `MATCH(25,A1:A3,{0;"x"})` has oracle values; runtime and static analysis match them; lookup.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `match-error-lookup-value-array-match-type` (excel-graph): a scalar error lookup_value lifts over an array match_type (`ROWS(MATCH(NA(),A1:A3,{0;1}))` is 2, slot 2 `#N/A`); `MATCH(25,A1:A3,{0;"x"})` is `#N/A` then `#VALUE!`. `fn_match` now only returns the scalar error early when match_type is not an array; static analysis already spilled. lookup.md updated. Eval delta: 8 new probe cells, all passing, no regressions.
