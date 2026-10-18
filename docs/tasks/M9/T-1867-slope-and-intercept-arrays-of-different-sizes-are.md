---
id: T-1867
title: "SLOPE and INTERCEPT: arrays of different sizes are #N/A, not #VALUE!"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SLOPE and INTERCEPT over arrays of different sizes are `#N/A`, as in Excel.

## Context
Found by T-1706: `GROUPBY(Data!A2:A11,Data!D2:D11,_xleta.SLOPE)` (and INTERCEPT) calls the function with each
group's values and all values, which differ in size: Excel gives `#N/A` per group, the engine `#VALUE!`
(probes/groupby-eta-arity G!B21:B25 SLOPE, G!B61:B65 INTERCEPT). Same family as T-1707. Probe the plain
`SLOPE({1;2},{1;2;3})` too; check SUMX2MY2 (passes through GROUPBY) and FORECAST.

## Acceptance criteria
- [ ] Those cells pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
