---
id: T-2595
title: "Arrays wider or taller than the grid: MAKEARRAY(1,20000,..), SEQUENCE(1,16385) and HSTACK build them; a cell is #SPILL!"
milestone: M7
lane: engine
status: todo
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
- [ ] The owned cells pass and leave eval/baseline (`make eval-baseline`), with no new known-failure lines.
- [ ] A spill wider than the grid is `#SPILL!` without allocating past the cell bound (unit test).
- [ ] arrays.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising the 1,048,576-cell bound.
