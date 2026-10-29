---
id: T-1867
title: "SLOPE and INTERCEPT: arrays of different sizes are #N/A, not #VALUE!"
milestone: M9
lane: functions-stat
status: done
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
- [x] Those cells pass and leave eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Already fixed on `origin/main` before this task was picked: `probes/groupby-eta-arity` passes 101/101 with no
`F` lines in `eval/baseline/probes.txt`, and `fn_get_two_matrices` (sum.rs) returns `#N/A` for unequal cell
counts (T-1292, T-1620, T-2431), which covers plain `SLOPE({1;2},{1;2;3})`, INTERCEPT, SUMX2MY2 and FORECAST
callers. No code change; eval delta 0.
