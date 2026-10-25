---
id: T-2636
title: "PROB and FREQUENCY: an error element inside an array argument is that error (T-2430 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An error element inside an array argument of PROB and FREQUENCY is that error, as for LARGE and the other functions of T-2430, with the other argument's error still winning where Excel does.

## Context
T-2430 reverted PROB and FREQUENCY to `#VALUE!` for an array's error element: making them propagate it regressed three gen-m9 cells
(gen-m9/589f42e67f4b4b8d A338 `FREQUENCY((TOCOL(..)/Data!$A8),SLOPE(..))` Excel `#VALUE!`, 7fd0a98183c6babf A331 `PROB(VSTACK(..),..)` Excel `#VALUE!`),
so the order of the arguments' errors (data, bins, limits) needs a probe first (excel-probe skill). Code: `numeric_flat_of` in
functions/statistical/prob.rs (T-2430 added `numeric_flat_with_element_errors` for the others), frequency.rs.
Still failing from T-2430's list and not covered here: F.TEST/FTEST (T-2093), FORECAST.ETS* and XNPV `#N/IMPL!`, `SMALL((SMALL({1,#N/A,3},2)/1.005),..)`
(1b31092a0d79e5f1 A363), `T.TEST(SORT(..))` (0620505280d2ea6e A31).

## Acceptance criteria
- [ ] Probe the argument-error order of PROB and FREQUENCY with an array error element; record it in statistics.md.
- [ ] The PROB and FREQUENCY gen-m9 cells with `{1,#N/A,3}` pass, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
