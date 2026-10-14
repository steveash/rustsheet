---
id: T-1558
title: "PERCENTRANK and PERCENTRANK.INC of a single value equal to x is 1, not #N/A"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`PERCENTRANK(B1:B6*1,2)` and `PERCENTRANK.INC(B1:B6*1,2)` in a legacy formula, where the array is the one value 2, are 1 in Excel; the engine gives `#N/A`.

## Context
probes/legacy-value-argument-intersection-c T2 and U2 (excel-graph, T-1556). Probe the single-value shapes directly (x equal to, below and above the value; the .EXC form is 0.5, statistics.md) before changing `PercentrankInc`/`PercentrankCompat`.

## Acceptance criteria
- [x] T2 and U2 of that case pass, backed by a probe of the single-value shapes; the rule is in statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
PERCENTRANK/PERCENTRANK.INC of one value equal to x is 1 (was #N/A); other x on a single value stays #N/A via the range check. probes known failures 5618 -> 5616 (T2, U2). The out-of-range single-value shapes are unprobed (noted in statistics.md).
