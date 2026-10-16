---
id: T-1725
title: "FORECAST and FORECAST.LINEAR: array x, unequal sizes and large-offset data as Excel (T-1620 probes)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORECAST and FORECAST.LINEAR spill over an array x, return Excel's errors for unequal sizes, and stay accurate on data with a large offset.

## Context
Found by T-1620 (probes/regression-forecast, FORECAST in column L, FORECAST.LINEAR in column P; excel-graph 2026-10-16):
- an array x spills one forecast per element: `FORECAST.LINEAR({7,8},A1:A6,B1:B6)` is 7.2 with 7.9714285714285715 spilled right (P7, Q7); ours `#N/IMPL!`. A range x (`B1:B2`) is `#SPILL!` where it collides, ours `#VALUE!`.
- unequal sizes (`FORECAST(7,A1:A5,B1:B6)`, `FORECAST(7,{2,4,5},{1,2})`) are `#N/A`, ours `#VALUE!`.
- large offset: `FORECAST(7,A1:A6,I1:I6)` is -7285714278.784956, ours `#DIV/0!`.
- ULP: `FORECAST(0,A1:A6,B1:B6)` 1.7999999999999998, ours 1.8.

## Acceptance criteria
- [ ] The mismatching non-ULP cells of probes/regression-forecast pass and the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing further inputs (file a probe task instead).
