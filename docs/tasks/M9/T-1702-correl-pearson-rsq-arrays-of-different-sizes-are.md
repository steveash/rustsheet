---
id: T-1702
title: "CORREL, PEARSON, RSQ: arrays of different sizes are #N/A, and the last digit of their results"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CORREL, PEARSON and RSQ match Excel for arrays of different sizes (`#N/A`, not `#VALUE!`) and in the last digit.

## Context
Found by T-1325: `GROUPBY(Data!A2:A11,Data!D2:D11,_xleta.CORREL)` calls CORREL with each group's values and all
values, which differ in size: Excel gives `#N/A` per group, the engine `#VALUE!` (probes/groupby-pivotby-functions
G!B421:B425, -functions2 G!B281:B285 RSQ, G!B301:B305 PEARSON; PIVOTBY functions2 P!A121). The total row, the same
array twice, is 1.0000000000000002 (CORREL, PEARSON) and 0.9999999999999998 (RSQ) in Excel, exactly 1 in the engine
(G!B426, functions2 G!B286, G!B306): Excel's formula rounds differently. COVARIANCE.P passes. Probe the plain
`CORREL({1;2},{1;2;3})` too.

## Acceptance criteria
- [ ] Those cells pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
