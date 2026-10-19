---
id: T-1754
title: "KURT with a direct TRUE argument differs in the last digits"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
KURT with a direct TRUE argument matches Excel's last digit.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q36 (`KURT(A1:A8,TRUE)`): Excel -1.4331834319526635, ours -1.4331834319526626 (about 9e-16, roughly 4 ulp; likely the accumulation order). Numeric fitting, so opus.

Same accumulation order, no TRUE involved (gap triage 2026-10-19): ironcalc-calc/STATISTICAL/MEADIAN_KURT_SKEW_HARMEAN.xlsx
Sheet1!F2 `KURT(A1:A10, {1,2,3},A3)` Excel (Excel Online cache) -1.2129835263391704, ours ...708; probes/stat-argument-modes
Sheet1!H13 `KURT(A1:A8)` Excel -1.1999999999999997, ours ...993. Fit all three cells with one order.

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623, ironcalc-calc MEADIAN_KURT_SKEW_HARMEAN.xlsx F2 and probes/stat-argument-modes H13 pass and leave eval/baseline (ironcalc-calc.txt, probes.txt) via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
