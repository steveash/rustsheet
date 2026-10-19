---
id: T-1917
title: "MATCH lifts over an array match_type"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH with an array-valued match_type (`{0;1}`, a multi-cell range) lifts, one position per element, as Excel does, instead of `#N/IMPL!`.

## Context
Found by T-1866. probes/match-lookup-value-lift E14:E16 (excel-graph): `ROWS(MATCH(2,A1:A3,{0;1}))` is 2, `INDEX(MATCH(20,A1:A3,{0;1}),2)` is 2, `INDEX(MATCH(B1:B2,A1:A3,{0;1}),1)` is 3 (lookup_value and match_type broadcast). Ours is `#N/IMPL!`. Code: `Model::fn_match` and `match_type_arg` (functions/lookup_and_reference/mod.rs); lift it with `lift_values` as XMATCH lifts match_mode. Note probes/match-type-nested-passthrough: a multi-cell match_type *reference* in a plain call still reads its top-left cell; check which shapes lift before changing it.

## Acceptance criteria
- [ ] probes/match-lookup-value-lift E14:E16 pass and leave the baseline; no probes/match-type-nested-passthrough cell regresses; static analysis spills by match_type too.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
MATCH's match_type is lifted with lookup_value through `lift_values`; a multi-cell range match_type still reads its top-left cell, and `match_static_result` spills only for an array one. probes/match-lookup-value-lift E14:E16 pass (known failures -3), no other cell changed. lookup.md updated.
