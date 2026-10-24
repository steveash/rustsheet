---
id: T-2600
title: "Arrays wider or taller than the grid: MAKEARRAY(1,20000,..), SEQUENCE(1,16385) and HSTACK build them; a cell is #SPILL!"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
An array value may have more columns than 16384 or more rows than 1048576 when its cells fit the engine's cell bound, as in Excel; only a spill of it is `#SPILL!`.

## Context
probes/t2579-array-value-edges (excel-graph, 2026-10-24, T-2579): `COLUMNS(MAKEARRAY(1,20000,LAMBDA(r,c,c)))` is 20000
(B20; INDEX of it B21, SUM B22, `+1` B30), `ROWS(MAKEARRAY(1048577,1,..))` 1048577 (B23), `COLUMNS(SEQUENCE(1,16385))` 16385
(B24), `COLUMNS(HSTACK(SEQUENCE(1,10000),SEQUENCE(1,10000)))` and `COLUMNS(TRANSPOSE(SEQUENCE(20000)))` 20000 (B28, B29),
and `ERROR.TYPE(MAKEARRAY(1,16385,..))` as a cell is `#SPILL!` (B26). Ours: `#VALUE!` from `limits::check_array_size`'s
per-dimension checks (B23, B24, B26 fail; the others pass through paths that skip it). arrays.md "Array results above
1,048,576 cells" says only a dimension past the grid is `#VALUE!` (`SEQUENCE(1048577)`): reconcile with B23/B24 (which
functions keep a per-dimension `#VALUE!`), probing more if needed. Keep the cell-count bound (DEVIATION, decision 0014).
Owned: probes/t2579-array-value-edges B23, B24, B26; gen-m9/4250f880ab640c49 Calc!A237 (`BETAINV(..,MAKEARRAY(4,DATE(1952,..),..),..)`,
a 4x51638 array, Excel `#SPILL!`).

## Acceptance criteria
- [x] The owned cells pass and leave eval/baseline (`make eval-baseline`), with no new known-failure lines. (Partly: B24 and B26
  pass; B23 is an accepted deviation, decision 0108; gen-m9 A237 moved to T-2607.)
- [x] A spill wider than the grid is `#SPILL!` without allocating past the cell bound (unit test).
- [x] arrays.md records the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising the 1,048,576-cell bound.

## Outcome
- MAKEARRAY and SEQUENCE share `limits::check_built_array_size`: any shape within the cell bound is built (SEQUENCE still
  refuses rows past the grid); above it, columns past the grid and rows of 2^31 or more are `#VALUE!` as in Excel (gen-m9
  f2bd5f380ec002bf A177, gen-m7 b2078b5d702c691a A2553, 8c4570978f18f4c2 A2736), anything else the cell bound's `#ERROR!`.
  `SIZE_LIMIT_EXCLUSIVE` moved to limits.rs. The lifting driver's broadcast checks the cell count only, so a lift over a
  wider-than-grid array spills to `#SPILL!` (B26).
- probes/t2579-array-value-edges: B24 and B26 fixed; B23 (`ROWS(MAKEARRAY(1048577,1,..))`, one cell past the bound) listed
  in probes/deviations (decision 0108).
- Not met: gen-m9/4250f880ab640c49 Calc!A237 is now `#ERROR!` from BETAINV's lift cost cap (was `#VALUE!`): T-2607.
- The thresholds between the measured sizes are unprobed: T-2608. arrays.md "Arrays past the grid".
