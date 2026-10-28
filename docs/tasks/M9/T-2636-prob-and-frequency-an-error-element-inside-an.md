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
T-2430 reverted PROB and FREQUENCY to `#VALUE!` for an array's error element: making them propagate it regressed two gen-m9 cells
(gen-m9/589f42e67f4b4b8d A338 `FREQUENCY((TOCOL(..)/Data!$A8),SLOPE(..))` Excel `#VALUE!`, 7fd0a98183c6babf A331 `PROB(VSTACK(..),..)` Excel `#VALUE!`),
so the order of the arguments' errors (data, bins, limits) needs a probe first (excel-probe skill). Code: `numeric_flat_of` in
functions/statistical/prob.rs (T-2430 added `numeric_flat_with_element_errors` for the others), frequency.rs.
Still failing from T-2430's list and not covered here: F.TEST/FTEST (T-2093), FORECAST.ETS* and XNPV `#N/IMPL!`, `SMALL((SMALL({1,#N/A,3},2)/1.005),..)`
(1b31092a0d79e5f1 A363), `T.TEST(SORT(..))` (0620505280d2ea6e A31).
- Conflicting desktop evidence for the bins argument (M5 planner, 2026-10-25). corpus-bench/53062 (`formamspnc (7)`, AE6:AQ6
  and the cells downstream; the case is a whole-case `F *` line in eval/baseline/corpus-bench.txt) uses the CSE array formula
  `LOOKUP(1,1/FREQUENCY(0,ABS($R$1:$AI$1-R6)),$R$1:$AI$1)`, where AI1 is the empty text `""`. Its bins array therefore has a
  `#VALUE!` element. Excel's cached result is a number (AE6 = 0, AI6 = -2), so desktop FREQUENCY did not return `#VALUE!` for
  that error element in bins. The engine gives `#VALUE!` (`ROWS(FREQUENCY(0,ABS({13,15,17,""}-13)))` is `#VALUE!`). This
  failure is most of the case's 338 failing cells. The probe must cover an error element in bins on its own, in addition
  to the data argument, before error propagation is implemented: `ROWS(FREQUENCY(0,{1,#VALUE!,3}))`,
  `INDEX(FREQUENCY(0,{1,#VALUE!,3}),1)`, `FREQUENCY({1;2},{#N/A})`.

Update (T-2427, 2026-10-27): FREQUENCY's data array now propagates an error element, and a scalar bins error beats it
(probes/frequency-bins B1-B3). Left here: PROB; FREQUENCY's bins array (still `#VALUE!`, the corpus-bench/53062 evidence
above); and an error cell in a data range against a scalar bins error, which keeps `main`'s data-first order unprobed.
Also to probe (T-2427 review, 2026-10-28): `FREQUENCY({1;2},"2")` and a referenced `"2"` bin (numeric text, skipped in ours);
a typed or computed boolean as data (`FREQUENCY(TRUE,{1;2})`, `FREQUENCY(1=1,{1;2})`); a computed boolean bin
(`FREQUENCY({1;2},1=1)`); and a data-array error element against an error cell in a bins range.

## Acceptance criteria
- [ ] Probe the argument-error order of PROB and FREQUENCY with an array error element; record it in statistics.md.
- [ ] Probe `SLOPE`, `INTERCEPT`, `STEYX`, `RSQ`, `FORECAST.LINEAR` and `CHISQ.TEST` with `{1,#N/A,3},{1,2,3}`, and the reverse position (plain first argument, error element in the second); record it in statistics.md (T-2430 changed them unverified).
- [ ] The PROB and FREQUENCY gen-m9 cells with `{1,#N/A,3}` pass, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
