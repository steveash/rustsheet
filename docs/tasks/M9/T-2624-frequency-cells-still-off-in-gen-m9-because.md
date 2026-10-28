---
id: T-2624
title: "FREQUENCY cells still off in gen-m9 because another argument function is wrong (ODDFYIELD, FORECAST.ETS, EXPAND, DB, NPER, WRAPCOLS, FILTER, INDIRECT/MAP)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The 19 gen-m9 cells that call FREQUENCY and stay in the baseline after T-2427 match Excel once the other function in the formula does (ODDFYIELD/ODDLYIELD, FORECAST.ETS*, EXPAND, DB, NPER, WRAPCOLS, FILTER, INDIRECT/MAP).

## Context
Left over by T-2427; find them with `rg FREQUENCY eval/baseline/gen-m9.txt` cross-checked against the case formulas (a612e89321a6df9c A155, e6cf3b20a014e6a3 A293, a5342a7e24fd803e A229, caa1eed48b1883fe A313, 06b4e8f5de409d5e A239, ...).

## Acceptance criteria
- [ ] Each of those cells passes or is owned by the task of its other function; their baseline lines disappear.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
